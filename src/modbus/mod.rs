//! The Modbus/TCP surface.
//!
//! Serves the SunSpec register map over Modbus/TCP. Framing is hand-written: the available crates
//! are client-oriented, and this is a server that has to control its own error responses.
//!
//! The surface reads values and never owns them.

pub mod frame;
pub mod registers;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use crate::capture::{
    CaptureSink, ConnectionClosed, ConnectionOpened, Event, ModbusRequest, Surface,
};
use crate::clock::Clock;
use crate::config::Config;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use frame::{ExceptionCode, Frame, Request};
use registers::RegisterMap;

pub struct ModbusSurface {
    registers: RegisterMap,
    clock: Arc<dyn Clock>,
    sink: Arc<dyn CaptureSink>,
}

impl ModbusSurface {
    pub fn new(config: &Config, clock: Arc<dyn Clock>, sink: Arc<dyn CaptureSink>) -> Self {
        Self {
            registers: RegisterMap::new(&config.identity),
            clock,
            sink,
        }
    }

    /// Accept forever. One task per connection; a failing connection never takes the listener down.
    pub async fn serve(self: Arc<Self>, listener: TcpListener) {
        let local_port = match listener.local_addr() {
            Ok(addr) => addr.port(),
            Err(_) => 0,
        };
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let surface = Arc::clone(&self);
                    tokio::spawn(async move {
                        surface.handle_connection(stream, peer, local_port).await;
                    });
                }
                Err(_) => {
                    // Accept errors are per-connection (fd limits, a peer that vanished between
                    // SYN and accept). Keep listening.
                    continue;
                }
            }
        }
    }

    async fn handle_connection(&self, mut stream: TcpStream, peer: SocketAddr, local_port: u16) {
        let connection_id = Uuid::new_v4().to_string();
        let peer = peer.to_string();
        let opened_at = self.clock.now();

        self.sink.record(Event::new(
            &*self.clock,
            &connection_id,
            Surface::Modbus,
            ConnectionOpened {
                peer: peer.clone(),
                local_port,
            },
        ));

        let mut requests = 0u64;
        let error = self
            .converse(&mut stream, &peer, &connection_id, &mut requests)
            .await
            .err();

        // Best-effort: the peer may already be gone.
        let _ = stream.shutdown().await;

        let duration_ms = (self.clock.now() - opened_at).num_milliseconds().max(0) as u64;
        self.sink.record(Event::new(
            &*self.clock,
            &connection_id,
            Surface::Modbus,
            ConnectionClosed {
                peer,
                local_port,
                duration_ms,
                requests,
                error,
            },
        ));
    }

    /// Read requests until the peer goes away. `Err` carries the reason the conversation ended
    /// badly, for the close event.
    async fn converse(
        &self,
        stream: &mut TcpStream,
        peer: &str,
        connection_id: &str,
        requests: &mut u64,
    ) -> Result<(), String> {
        loop {
            let request = match frame::read_frame(stream).await {
                Ok(Some(frame)) => frame,
                Ok(None) => return Ok(()),
                Err(err) => return Err(err.to_string()),
            };
            *requests += 1;

            let response = self.respond(&request, peer, connection_id);

            if let Err(err) = stream.write_all(&response.encode()).await {
                return Err(err.to_string());
            }
        }
    }

    /// Answer one request, and record what was asked and what came back.
    fn respond(&self, request: &Frame, peer: &str, connection_id: &str) -> Frame {
        let parsed = Request::parse(&request.pdu);
        let function_code = request.pdu.first().copied().unwrap_or(0);

        let outcome = match parsed {
            Request::ReadHoldingRegisters { start, quantity } => self
                .registers
                .read(start, quantity)
                .map(frame::encode_read_response),
            Request::UnsupportedFunction => Err(ExceptionCode::IllegalFunction),
            Request::MalformedPayload => Err(ExceptionCode::IllegalDataValue),
        };

        let (start_address, quantity) = parsed.range();
        let exception_code = outcome.as_ref().err().map(|code| code.as_u8());

        self.sink.record(Event::new(
            &*self.clock,
            connection_id,
            Surface::Modbus,
            ModbusRequest {
                peer: peer.to_owned(),
                transaction_id: request.transaction_id,
                unit_id: request.unit_id,
                function_code,
                start_address,
                quantity,
                pdu: BASE64.encode(&request.pdu),
                exception_code,
            },
        ));

        let pdu = match outcome {
            Ok(pdu) => pdu,
            Err(code) => frame::encode_exception(function_code, code),
        };
        Frame::reply_to(request, pdu)
    }
}
