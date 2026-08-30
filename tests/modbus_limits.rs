//! Resource limits on an exposed surface.
//!
//! The Modbus surface is hostile ground. A peer that opens sockets and never speaks must cost a
//! bounded amount, and the attempt must still reach the corpus.

mod support;

use std::time::Duration;

use sunnypot::capture::EventKind;
use support::{RawModbus, TestApp, test_config_with_modbus};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[tokio::test]
async fn a_connection_beyond_the_limit_is_refused_and_still_recorded() {
    let app = TestApp::start_with(&test_config_with_modbus("max_connections = 1")).await;

    // Hold the one permit.
    let mut held = RawModbus::connect(app.modbus_addr()).await;
    held.read_holding(40000, 2).await;

    let mut refused = TcpStream::connect(app.modbus_addr())
        .await
        .expect("connect");
    let mut buffer = [0u8; 1];
    let read = tokio::time::timeout(Duration::from_secs(5), refused.read(&mut buffer))
        .await
        .expect("the surface closes a refused connection promptly");
    assert_eq!(
        read.expect("read"),
        0,
        "refused connections are closed, not queued"
    );

    // open + request for the held connection, then open + close for the refused one.
    let events = app.wait_for_events(4).await;
    let closed = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConnectionClosed(c) => Some(c),
            _ => None,
        })
        .expect("the refusal reaches the corpus");
    assert_eq!(
        closed.error.as_deref(),
        Some("at the connection limit"),
        "a peer exhausting the limit is itself an observation"
    );
    assert_eq!(closed.requests, 0);
}

#[tokio::test]
async fn a_peer_that_never_speaks_is_eventually_closed() {
    let app = TestApp::start_with(&test_config_with_modbus("idle_timeout_secs = 1")).await;

    let mut silent = TcpStream::connect(app.modbus_addr())
        .await
        .expect("connect");
    let mut buffer = [0u8; 1];
    let read = tokio::time::timeout(Duration::from_secs(10), silent.read(&mut buffer))
        .await
        .expect("the idle timeout fires");
    assert_eq!(
        read.expect("read"),
        0,
        "the surface closes an idle connection"
    );

    let events = app.wait_for_events(2).await;
    match &events[1].kind {
        EventKind::ConnectionClosed(c) => {
            assert_eq!(c.requests, 0);
            assert!(
                c.error.as_deref().is_some_and(|e| e.contains("idle")),
                "the corpus records why the connection ended: {:?}",
                c.error
            );
        }
        other => panic!("expected connection_closed, got {other:?}"),
    }
}

/// A connection released by one peer is available to the next — the limit is concurrency, not a
/// lifetime budget.
#[tokio::test]
async fn a_permit_is_returned_when_a_connection_ends() {
    let app = TestApp::start_with(&test_config_with_modbus("max_connections = 1")).await;

    let mut first = RawModbus::connect(app.modbus_addr()).await;
    assert_eq!(first.read_registers(40000, 2).await.len(), 2);
    drop(first);
    app.wait_for_events(3).await; // open, request, close

    let mut second = RawModbus::connect(app.modbus_addr()).await;
    assert_eq!(
        second.read_registers(40000, 2).await.len(),
        2,
        "the next peer gets the freed permit"
    );
}
