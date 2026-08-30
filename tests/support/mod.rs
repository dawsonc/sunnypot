//! Shared harness for the surface tests.
//!
//! Every test drives a real `App` over a real loopback socket. The only things swapped out are the
//! two seams the spec allows: the clock and the capture sink.

#![allow(dead_code)]

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use sunnypot::app::App;
use sunnypot::capture::{Event, MemorySink};
use sunnypot::clock::FixedClock;
use sunnypot::config::Config;

/// Identity values used by the tests. Deliberately *not* the deployed identity: the tests assert
/// that whatever config supplies reaches the wire, not that a particular vendor string does.
pub const TEST_CONFIG: &str = r#"
[site]
latitude = 42.36
longitude = -71.06
timezone = "America/New_York"
array_capacity_kw = 6.2
inverter_rating_kw = 5.0

[identity]
manufacturer = "Testvendor"
product = "Testproduct 5.0-1 208-240"
options = "3.28.1-3"
version = "1.19.10-0"
serial = "30514231"
device_address = 1

[modbus]
bind = "127.0.0.1:0"
"#;

/// The test config with extra keys appended to its `[modbus]` section, which is last.
pub fn test_config_with_modbus(extra: &str) -> String {
    format!("{TEST_CONFIG}{extra}\n")
}

/// The committed example, bound to an ephemeral loopback port instead of 0.0.0.0:502.
pub fn example_config() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/sunnypot.example.toml");
    let text = std::fs::read_to_string(path).expect("the example config is committed");
    text.replace(r#"bind = "0.0.0.0:502""#, r#"bind = "127.0.0.1:0""#)
}

pub struct TestApp {
    app: App,
    clock: Arc<FixedClock>,
    sink: Arc<MemorySink>,
}

impl TestApp {
    /// Start an app on an ephemeral loopback port with the default test config.
    pub async fn start() -> Self {
        Self::start_with(TEST_CONFIG).await
    }

    pub async fn start_with(config_toml: &str) -> Self {
        let config = Config::from_toml_str(config_toml).expect("test config should parse");
        let clock = Arc::new(FixedClock::new(at(2026, 6, 21, 16, 0, 0)));
        let sink = Arc::new(MemorySink::new());
        let app = App::bind(config, clock.clone(), sink.clone())
            .await
            .expect("app should bind");
        Self { app, clock, sink }
    }

    pub fn modbus_addr(&self) -> std::net::SocketAddr {
        self.app.modbus_addr()
    }

    pub fn clock(&self) -> &FixedClock {
        &self.clock
    }

    pub fn events(&self) -> Vec<Event> {
        self.sink.events()
    }

    /// Wait until the sink holds at least `n` events, or fail. Connection close is recorded by the
    /// server task after the client has already moved on, so tests need a settling point.
    pub async fn wait_for_events(&self, n: usize) -> Vec<Event> {
        for _ in 0..200 {
            let events = self.sink.events();
            if events.len() >= n {
                return events;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        panic!(
            "expected at least {n} captured events, saw {}: {:#?}",
            self.sink.events().len(),
            self.sink.events()
        );
    }
}

pub fn at(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, min, s)
        .single()
        .expect("valid instant")
}

// ---------------------------------------------------------------------------
// A raw Modbus/TCP client.
//
// Deliberately not a Modbus library: these tests assert on the exact bytes a scanner receives,
// including malformed and unsupported requests a client library would refuse to send.
// ---------------------------------------------------------------------------

use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct RawModbus {
    stream: TcpStream,
    next_transaction_id: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawResponse {
    pub transaction_id: u16,
    pub protocol_id: u16,
    pub unit_id: u8,
    pub pdu: Vec<u8>,
}

impl RawModbus {
    pub async fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).await.expect("connect");
        Self {
            stream,
            next_transaction_id: 1,
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.stream.local_addr().expect("local addr")
    }

    /// Send a PDU inside an MBAP header and read one response frame back.
    pub async fn request(&mut self, unit_id: u8, pdu: &[u8]) -> RawResponse {
        let transaction_id = self.next_transaction_id;
        self.next_transaction_id += 1;

        let mut frame = Vec::with_capacity(7 + pdu.len());
        frame.extend_from_slice(&transaction_id.to_be_bytes());
        frame.extend_from_slice(&0u16.to_be_bytes()); // protocol id
        frame.extend_from_slice(&((pdu.len() + 1) as u16).to_be_bytes());
        frame.push(unit_id);
        frame.extend_from_slice(pdu);
        self.stream.write_all(&frame).await.expect("write request");

        let mut header = [0u8; 7];
        read_exact_within(&mut self.stream, &mut header)
            .await
            .expect("read MBAP header");
        let length = u16::from_be_bytes([header[4], header[5]]) as usize;
        assert!(length >= 1, "MBAP length covers at least the unit id");
        let mut pdu = vec![0u8; length - 1];
        read_exact_within(&mut self.stream, &mut pdu)
            .await
            .expect("read PDU");

        RawResponse {
            transaction_id: u16::from_be_bytes([header[0], header[1]]),
            protocol_id: u16::from_be_bytes([header[2], header[3]]),
            unit_id: header[6],
            pdu,
        }
    }

    /// FC 3 — read holding registers.
    pub async fn read_holding(&mut self, start: u16, quantity: u16) -> RawResponse {
        let mut pdu = vec![0x03];
        pdu.extend_from_slice(&start.to_be_bytes());
        pdu.extend_from_slice(&quantity.to_be_bytes());
        self.request(1, &pdu).await
    }

    /// The register values from a successful FC 3 response.
    pub async fn read_registers(&mut self, start: u16, quantity: u16) -> Vec<u16> {
        let response = self.read_holding(start, quantity).await;
        assert_eq!(
            response.pdu.first().copied(),
            Some(0x03),
            "expected an FC 3 response, got {:02x?}",
            response.pdu
        );
        assert_eq!(
            response.pdu[1] as usize,
            quantity as usize * 2,
            "byte count matches the requested quantity"
        );
        let (pairs, _) = response.pdu[2..].as_chunks::<2>();
        pairs.iter().map(|pair| u16::from_be_bytes(*pair)).collect()
    }
}

/// Read exactly `into.len()` bytes, or give up. A honeypot that never answers is a test failure,
/// not a test that hangs.
async fn read_exact_within(stream: &mut TcpStream, into: &mut [u8]) -> Result<(), String> {
    match tokio::time::timeout(std::time::Duration::from_secs(5), stream.read_exact(into)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(format!("{err}")),
        Err(_) => Err("timed out waiting for a response".to_owned()),
    }
}

/// Decode a SunSpec fixed-width string field: big-endian register pairs, zero-padded.
pub fn sunspec_string(registers: &[u16]) -> String {
    let bytes: Vec<u8> = registers
        .iter()
        .flat_map(|reg| reg.to_be_bytes())
        .take_while(|byte| *byte != 0)
        .collect();
    String::from_utf8(bytes).expect("SunSpec strings are ASCII")
}
