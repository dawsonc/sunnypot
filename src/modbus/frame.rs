//! Modbus/TCP framing: the MBAP header, request parsing, and response encoding.
//!
//! Written by hand rather than taken from a crate. The published Modbus crates are client-shaped,
//! and a honeypot needs to control exactly which byte it puts on the wire in response to input a
//! real client would never send.

use tokio::io::{AsyncRead, AsyncReadExt};

/// Transaction id, protocol id, length, unit id.
pub const MBAP_HEADER_LEN: usize = 7;

/// A PDU is at most 253 bytes; with the unit id that is the 254-byte maximum the length field may
/// legally carry.
pub const MAX_PDU_LEN: usize = 253;

/// FC 3 reads at most 125 registers — 250 bytes, the most a response PDU can hold.
pub const MAX_READ_QUANTITY: u16 = 125;

pub const FC_READ_HOLDING_REGISTERS: u8 = 0x03;

/// Set on the function code of an exception response.
const EXCEPTION_FLAG: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExceptionCode {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
}

impl ExceptionCode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// One Modbus/TCP frame: an MBAP header and the PDU it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub transaction_id: u16,
    pub protocol_id: u16,
    pub unit_id: u8,
    pub pdu: Vec<u8>,
}

impl Frame {
    /// The function code the PDU carries. An empty PDU has none; `0` is not a valid function code,
    /// so it reads as "nothing was asked for" in the corpus.
    pub fn function_code(&self) -> u8 {
        self.pdu.first().copied().unwrap_or(0)
    }

    /// A response frame that echoes the request's routing fields, as the protocol requires.
    pub fn reply_to(request: &Frame, pdu: Vec<u8>) -> Self {
        Self {
            transaction_id: request.transaction_id,
            protocol_id: request.protocol_id,
            unit_id: request.unit_id,
            pdu,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(MBAP_HEADER_LEN + self.pdu.len());
        out.extend_from_slice(&self.transaction_id.to_be_bytes());
        out.extend_from_slice(&self.protocol_id.to_be_bytes());
        let length = (self.pdu.len() + 1) as u16; // unit id plus PDU
        out.extend_from_slice(&length.to_be_bytes());
        out.push(self.unit_id);
        out.extend_from_slice(&self.pdu);
        out
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("transport: {0}")]
    Io(#[from] std::io::Error),
    #[error("MBAP length field is {0}, outside the legal 2..=254")]
    BadLength(u16),
}

/// Read one frame. `Ok(None)` means the peer closed cleanly between frames.
pub async fn read_frame<R>(reader: &mut R) -> Result<Option<Frame>, FrameError>
where
    R: AsyncRead + Unpin,
{
    let mut header = [0u8; MBAP_HEADER_LEN];
    match reader.read_exact(&mut header).await {
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err.into()),
    }

    let length = u16::from_be_bytes([header[4], header[5]]);
    // The length field covers the unit id and the PDU.
    if length < 2 || length as usize > MAX_PDU_LEN + 1 {
        return Err(FrameError::BadLength(length));
    }

    let mut pdu = vec![0u8; length as usize - 1];
    reader.read_exact(&mut pdu).await?;

    Ok(Some(Frame {
        transaction_id: u16::from_be_bytes([header[0], header[1]]),
        protocol_id: u16::from_be_bytes([header[2], header[3]]),
        unit_id: header[6],
        pdu,
    }))
}

/// What a request PDU asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    ReadHoldingRegisters {
        start: u16,
        quantity: u16,
    },
    /// A function code sunnypot does not serve.
    UnsupportedFunction,
    /// A function code sunnypot serves, carrying a payload it cannot read.
    MalformedPayload,
}

impl Request {
    pub fn parse(pdu: &[u8]) -> Self {
        let Some(&function_code) = pdu.first() else {
            return Request::MalformedPayload;
        };
        match function_code {
            FC_READ_HOLDING_REGISTERS => {
                if pdu.len() != 5 {
                    return Request::MalformedPayload;
                }
                Request::ReadHoldingRegisters {
                    start: u16::from_be_bytes([pdu[1], pdu[2]]),
                    quantity: u16::from_be_bytes([pdu[3], pdu[4]]),
                }
            }
            _ => Request::UnsupportedFunction,
        }
    }
}

/// # Panics (debug only)
///
/// The byte count is one byte wide, so a caller must have already rejected quantities above
/// [`MAX_READ_QUANTITY`]. The assertion pins that invariant here, where the cast is.
pub fn encode_read_response(registers: &[u16]) -> Vec<u8> {
    debug_assert!(
        registers.len() <= MAX_READ_QUANTITY as usize,
        "response of {} registers cannot state its own byte count",
        registers.len()
    );
    let mut pdu = Vec::with_capacity(2 + registers.len() * 2);
    pdu.push(FC_READ_HOLDING_REGISTERS);
    pdu.push((registers.len() * 2) as u8);
    for register in registers {
        pdu.extend_from_slice(&register.to_be_bytes());
    }
    pdu
}

pub fn encode_exception(function_code: u8, code: ExceptionCode) -> Vec<u8> {
    vec![function_code | EXCEPTION_FLAG, code.as_u8()]
}
