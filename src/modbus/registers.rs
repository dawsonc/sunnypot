//! The SunSpec register map.
//!
//! Typed and in code, not config-described data: the map has to project a live plant, which a
//! static data description cannot do. Addresses and lengths come from
//! `docs/adr/0002-impersonate-fronius-primo.md`.
//!
//! Addresses here are protocol (0-based) addresses. Vendor register maps, and the ADR, tabulate the
//! same registers 1-based: the identifier this module puts at 40000 is the one those tables call
//! 40001. Standard SunSpec discovery probes the 0-based address.

use crate::config::Identity;

use super::frame::{ExceptionCode, MAX_READ_QUANTITY};

/// Where the SunSpec identifier lives. Discovery logic probes this address (and 50000, and 0)
/// looking for the marker.
pub const BASE_ADDRESS: u16 = 40000;

/// `SunS` as two big-endian registers. The marker that says "a SunSpec map starts here".
const SUNSPEC_IDENTIFIER: [u16; 2] = [0x5375, 0x6E53];

const MODEL_COMMON: u16 = 1;

/// The Common block is 65 registers on a Fronius Datamanager, not the 66 of a textbook SunSpec map.
/// Discovery walks the chain by length, so 65 is what keeps the chain both traversable and
/// Fronius-shaped.
const COMMON_LENGTH: u16 = 65;

/// The chain terminator: model id `0xFFFF` and a zero length.
const END_MARKER: [u16; 2] = [0xFFFF, 0x0000];

const STRING32_REGISTERS: usize = 16;
const STRING16_REGISTERS: usize = 8;

pub struct RegisterMap {
    /// Register values, in address order, starting at [`BASE_ADDRESS`].
    registers: Vec<u16>,
}

impl RegisterMap {
    pub fn new(identity: &Identity) -> Self {
        let mut registers = Vec::new();

        registers.extend_from_slice(&SUNSPEC_IDENTIFIER);

        registers.push(MODEL_COMMON);
        registers.push(COMMON_LENGTH);
        push_string(&mut registers, &identity.manufacturer, STRING32_REGISTERS);
        push_string(&mut registers, &identity.product, STRING32_REGISTERS);
        push_string(&mut registers, &identity.options, STRING16_REGISTERS);
        push_string(&mut registers, &identity.version, STRING16_REGISTERS);
        push_string(&mut registers, &identity.serial, STRING32_REGISTERS);
        registers.push(identity.device_address);

        registers.extend_from_slice(&END_MARKER);

        Self { registers }
    }

    /// The address one past the last mapped register.
    fn end_address(&self) -> u32 {
        BASE_ADDRESS as u32 + self.registers.len() as u32
    }

    /// Read a register range, or say why not.
    ///
    /// Everything outside the mapped block is an illegal address: a real inverter does not answer
    /// for registers it does not have, and a scanner that sweeps the whole space would notice.
    pub fn read(&self, start: u16, quantity: u16) -> Result<&[u16], ExceptionCode> {
        if quantity == 0 || quantity > MAX_READ_QUANTITY {
            return Err(ExceptionCode::IllegalDataValue);
        }

        let start = start as u32;
        let end = start + quantity as u32;
        let base = BASE_ADDRESS as u32;
        if start < base || end > self.end_address() {
            return Err(ExceptionCode::IllegalDataAddress);
        }

        Ok(&self.registers[(start - base) as usize..(end - base) as usize])
    }
}

/// Pack a SunSpec fixed-width string: ASCII, two characters per register, big-endian, zero-padded.
///
/// Config load rejects anything that would not fit, so truncation here would be a bug rather than a
/// silent shortening. `chars` beyond the field width are dropped rather than panicking, because a
/// honeypot that stops serving is worse than one that serves a short string.
fn push_string(registers: &mut Vec<u16>, value: &str, width_registers: usize) {
    let mut bytes = value.as_bytes().to_vec();
    bytes.resize(width_registers * 2, 0);
    let (pairs, _) = bytes.as_chunks::<2>();
    for pair in pairs.iter().take(width_registers) {
        registers.push(u16::from_be_bytes(*pair));
    }
}
