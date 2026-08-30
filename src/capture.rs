//! The capture corpus.
//!
//! One flat stream of newline-delimited JSON. Connection-lifecycle events carry the source address
//! and duration; request events carry protocol payload. A shared `connection_id` joins them, so a
//! full session can be reconstructed from a flat file.

use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Bumped only when the shape of an event changes incompatibly. A month of captures has to be
/// analysable as one corpus, so every record carries the version it was written under.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "v")]
    pub schema_version: u32,
    #[serde(rename = "ts")]
    pub timestamp: DateTime<Utc>,
    pub connection_id: String,
    pub surface: Surface,
    #[serde(flatten)]
    pub kind: EventKind,
}

impl Event {
    pub fn new(
        clock: &dyn crate::clock::Clock,
        connection_id: &str,
        surface: Surface,
        kind: impl Into<EventKind>,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            timestamp: clock.now(),
            connection_id: connection_id.to_owned(),
            surface,
            kind: kind.into(),
        }
    }

    /// One line of the corpus.
    pub fn to_ndjson(&self) -> String {
        let mut line = serde_json::to_string(self).expect("event serialises");
        line.push('\n');
        line
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Modbus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EventKind {
    ConnectionOpened(ConnectionOpened),
    ConnectionClosed(ConnectionClosed),
    ModbusRequest(ModbusRequest),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionOpened {
    /// Source address, `ip:port`.
    pub peer: String,
    /// Which exposed port the connection landed on.
    pub local_port: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionClosed {
    pub peer: String,
    pub local_port: u16,
    /// How long the peer stayed. Distinguishes sustained interaction from a single-packet sweep.
    pub duration_ms: u64,
    /// How many protocol requests the connection carried.
    pub requests: u64,
    /// Set when the connection ended on a transport or framing failure rather than a clean close.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModbusRequest {
    pub peer: String,
    pub transaction_id: u16,
    pub unit_id: u8,
    pub function_code: u8,
    /// Present for the function codes whose payload names a register range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_address: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u16>,
    /// The request PDU as sent, base64. For function codes sunnypot does not parse this is the only
    /// evidence of what was attempted.
    pub pdu: String,
    /// The Modbus exception code returned, if the request was refused. Absent on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_code: Option<u8>,
}

impl From<ConnectionOpened> for EventKind {
    fn from(value: ConnectionOpened) -> Self {
        EventKind::ConnectionOpened(value)
    }
}

impl From<ConnectionClosed> for EventKind {
    fn from(value: ConnectionClosed) -> Self {
        EventKind::ConnectionClosed(value)
    }
}

impl From<ModbusRequest> for EventKind {
    fn from(value: ModbusRequest) -> Self {
        EventKind::ModbusRequest(value)
    }
}

/// Where captured events go.
///
/// Deliberately synchronous: a surface must never block on the corpus, so implementations hand the
/// event off (a queue, a buffer) and return immediately.
pub trait CaptureSink: Send + Sync + 'static {
    fn record(&self, event: Event);
}

/// Holds events in memory for tests.
#[derive(Debug, Default)]
pub struct MemorySink {
    events: Mutex<Vec<Event>>,
}

impl MemorySink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<Event> {
        self.events.lock().expect("sink lock").clone()
    }
}

impl CaptureSink for MemorySink {
    fn record(&self, event: Event) {
        self.events.lock().expect("sink lock").push(event);
    }
}

/// Writes the corpus to stdout, one JSON line per event.
///
/// What the binary ships with until the object-storage sink lands: `sunnypot > captures.jsonl`
/// already produces an analysable corpus.
#[derive(Debug, Default)]
pub struct StdoutSink {
    /// Serialises writers so two surfaces can never interleave halves of a line.
    lock: Mutex<()>,
}

impl StdoutSink {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CaptureSink for StdoutSink {
    fn record(&self, event: Event) {
        use std::io::Write as _;

        let line = event.to_ndjson();
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        // A honeypot must not die because its output pipe went away.
        let _ = stdout.write_all(line.as_bytes());
        let _ = stdout.flush();
    }
}
