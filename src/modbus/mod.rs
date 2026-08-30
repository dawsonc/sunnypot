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
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::capture::{
    CaptureSink, ConnectionClosed, ConnectionOpened, Event, EventKind, ModbusRequest, Surface,
};
use crate::clock::Clock;
use crate::config::Config;

use frame::{ExceptionCode, Frame, Request};
use registers::RegisterMap;

/// One peer's session, and the identity that joins everything it did in the corpus.
struct Connection {
    id: String,
    peer: String,
    local_port: u16,
}

impl Connection {
    fn accept(peer: SocketAddr, local_port: u16) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            peer: peer.to_string(),
            local_port,
        }
    }
}

pub struct ModbusSurface {
    registers: RegisterMap,
    clock: Arc<dyn Clock>,
    sink: Arc<dyn CaptureSink>,
    /// Bounds how many peers can hold a socket open at once.
    permits: Arc<Semaphore>,
    idle_timeout: Duration,
}

impl ModbusSurface {
    pub fn new(config: &Config, clock: Arc<dyn Clock>, sink: Arc<dyn CaptureSink>) -> Self {
        Self {
            registers: RegisterMap::new(&config.identity),
            clock,
            sink,
            permits: Arc::new(Semaphore::new(config.modbus.max_connections)),
            idle_timeout: config.modbus.idle_timeout(),
        }
    }

    /// Accept forever. One task per connection; a failing connection never takes the listener down.
    ///
    /// `local_port` is passed in rather than read back off the listener, so a record in the corpus
    /// can never carry a placeholder port.
    pub async fn serve(self: Arc<Self>, listener: TcpListener, local_port: u16) {
        loop {
            let Ok((stream, peer)) = listener.accept().await else {
                // Accept errors are per-connection (fd limits, a peer that vanished between SYN and
                // accept). Keep listening.
                continue;
            };

            let connection = Connection::accept(peer, local_port);
            match Arc::clone(&self.permits).try_acquire_owned() {
                Ok(permit) => {
                    let surface = Arc::clone(&self);
                    tokio::spawn(async move {
                        surface.handle_connection(stream, connection).await;
                        drop(permit);
                    });
                }
                Err(_) => {
                    // At the cap. Refuse, but still record the attempt: a peer opening more
                    // connections than the device will hold is itself an observation.
                    self.record_refusal(&connection);
                }
            }
        }
    }

    fn record(&self, connection: &Connection, kind: impl Into<EventKind>) {
        self.sink.record(Event::new(
            &*self.clock,
            &connection.id,
            Surface::Modbus,
            kind,
        ));
    }

    fn opened(connection: &Connection) -> ConnectionOpened {
        ConnectionOpened {
            peer: connection.peer.clone(),
            local_port: connection.local_port,
        }
    }

    fn record_refusal(&self, connection: &Connection) {
        self.record(connection, Self::opened(connection));
        self.record(
            connection,
            ConnectionClosed {
                peer: connection.peer.clone(),
                local_port: connection.local_port,
                duration_ms: 0,
                requests: 0,
                error: Some("at the connection limit".to_owned()),
            },
        );
    }

    async fn handle_connection(&self, mut stream: TcpStream, connection: Connection) {
        let opened_at = self.clock.now();
        self.record(&connection, Self::opened(&connection));

        let mut requests = 0u64;
        let error = self
            .converse(&mut stream, &connection, &mut requests)
            .await
            .err();

        // Best-effort: the peer may already be gone.
        let _ = stream.shutdown().await;

        let duration_ms = (self.clock.now() - opened_at).num_milliseconds().max(0) as u64;
        self.record(
            &connection,
            ConnectionClosed {
                peer: connection.peer.clone(),
                local_port: connection.local_port,
                duration_ms,
                requests,
                error,
            },
        );
    }

    /// Read requests until the peer goes away or goes quiet. `Err` carries the reason the
    /// conversation ended badly, for the close event.
    async fn converse(
        &self,
        stream: &mut TcpStream,
        connection: &Connection,
        requests: &mut u64,
    ) -> Result<(), String> {
        loop {
            let read = tokio::time::timeout(self.idle_timeout, frame::read_frame(stream));
            let request = match read.await {
                Ok(Ok(Some(frame))) => frame,
                Ok(Ok(None)) => return Ok(()),
                Ok(Err(err)) => return Err(err.to_string()),
                Err(_) => {
                    return Err(format!("idle for {} seconds", self.idle_timeout.as_secs()));
                }
            };
            *requests += 1;

            let response = self.respond(&request, connection);

            if let Err(err) = stream.write_all(&response.encode()).await {
                return Err(err.to_string());
            }
        }
    }

    /// Answer one request, and record what was asked and what came back.
    fn respond(&self, request: &Frame, connection: &Connection) -> Frame {
        let function_code = request.function_code();

        // One match over the parsed request: what to answer, and what range to record.
        let (outcome, start_address, quantity) = match Request::parse(&request.pdu) {
            Request::ReadHoldingRegisters { start, quantity } => (
                self.registers
                    .read(start, quantity)
                    .map(frame::encode_read_response),
                Some(start),
                Some(quantity),
            ),
            Request::UnsupportedFunction => (Err(ExceptionCode::IllegalFunction), None, None),
            Request::MalformedPayload => (Err(ExceptionCode::IllegalDataValue), None, None),
        };

        self.record(
            connection,
            ModbusRequest {
                peer: connection.peer.clone(),
                transaction_id: request.transaction_id,
                unit_id: request.unit_id,
                function_code,
                start_address,
                quantity,
                pdu: BASE64.encode(&request.pdu),
                exception_code: outcome.as_ref().err().map(|code| code.as_u8()),
            },
        );

        let pdu = match outcome {
            Ok(pdu) => pdu,
            Err(code) => frame::encode_exception(function_code, code),
        };
        Frame::reply_to(request, pdu)
    }
}
