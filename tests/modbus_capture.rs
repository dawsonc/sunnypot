//! What the capture corpus records about a Modbus session.

mod support;

use std::time::Duration;

use sunnypot::capture::{EventKind, Surface};
use support::{RawModbus, TestApp};
use tokio::net::TcpStream;

#[tokio::test]
async fn a_connection_that_opens_and_closes_is_captured_as_a_pair() {
    let app = TestApp::start().await;

    let stream = TcpStream::connect(app.modbus_addr())
        .await
        .expect("connect");
    let peer = stream.local_addr().expect("local addr");
    app.wait_for_events(1).await; // the connection is open and recorded
    app.clock().advance(Duration::from_millis(2500));
    drop(stream);

    let events = app.wait_for_events(2).await;
    assert_eq!(events.len(), 2, "one open and one close: {events:#?}");

    let opened = &events[0];
    let closed = &events[1];
    assert_eq!(opened.surface, Surface::Modbus);
    assert_eq!(
        opened.connection_id, closed.connection_id,
        "lifecycle events share a connection identifier"
    );

    match &opened.kind {
        EventKind::ConnectionOpened(o) => {
            assert_eq!(o.peer, peer.to_string(), "records the source address");
            assert_eq!(o.local_port, app.modbus_addr().port());
        }
        other => panic!("expected connection_opened, got {other:?}"),
    }

    match &closed.kind {
        EventKind::ConnectionClosed(c) => {
            assert_eq!(c.peer, peer.to_string());
            assert_eq!(c.duration_ms, 2500, "duration comes from the clock");
            assert_eq!(c.requests, 0, "a bare connect carried no requests");
        }
        other => panic!("expected connection_closed, got {other:?}"),
    }
}

#[tokio::test]
async fn every_request_is_captured_and_joined_to_its_connection() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;
    let peer = client.local_addr().to_string();

    client.read_holding(40000, 2).await;
    client.request(1, &[0x04, 0x00, 0x01, 0x00, 0x01]).await;
    drop(client);

    let events = app.wait_for_events(4).await;
    assert_eq!(events.len(), 4, "open, two requests, close: {events:#?}");

    let connection_id = &events[0].connection_id;
    assert!(
        events.iter().all(|e| &e.connection_id == connection_id),
        "one connection, one identifier"
    );

    match &events[1].kind {
        EventKind::ModbusRequest(r) => {
            assert_eq!(r.peer, peer);
            assert_eq!(r.transaction_id, 1);
            assert_eq!(r.unit_id, 1);
            assert_eq!(r.function_code, 3);
            assert_eq!(r.start_address, Some(40000), "the register range asked for");
            assert_eq!(r.quantity, Some(2));
            assert_eq!(r.exception_code, None, "the read succeeded");
            // base64 of 03 9c 40 00 02, the request PDU as sent.
            assert_eq!(r.pdu, "A5xAAAI=");
        }
        other => panic!("expected the read to be captured, got {other:?}"),
    }

    match &events[2].kind {
        EventKind::ModbusRequest(r) => {
            assert_eq!(r.function_code, 4, "an unsupported code is still recorded");
            assert_eq!(
                r.start_address, None,
                "sunnypot does not parse FC 4 payloads"
            );
            assert_eq!(r.quantity, None);
            assert_eq!(r.exception_code, Some(1), "illegal function");
            // base64 of 04 00 01 00 01 — the raw PDU is the only record of what was attempted.
            assert_eq!(r.pdu, "BAABAAE=");
        }
        other => panic!("expected the unsupported request to be captured, got {other:?}"),
    }

    match &events[3].kind {
        EventKind::ConnectionClosed(c) => assert_eq!(c.requests, 2),
        other => panic!("expected connection_closed, got {other:?}"),
    }
}

#[tokio::test]
async fn separate_connections_get_separate_identifiers() {
    let app = TestApp::start().await;

    let mut first = RawModbus::connect(app.modbus_addr()).await;
    first.read_holding(40000, 2).await;
    let mut second = RawModbus::connect(app.modbus_addr()).await;
    second.read_holding(40000, 2).await;

    let events = app.wait_for_events(4).await;
    let ids: std::collections::HashSet<&String> = events.iter().map(|e| &e.connection_id).collect();
    assert_eq!(
        ids.len(),
        2,
        "two sessions are distinguishable: {events:#?}"
    );
}

/// A month of captures has to be analysable as one corpus, so the wire format is pinned here.
#[tokio::test]
async fn an_event_is_one_flat_ndjson_line() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;
    client.read_holding(40000, 2).await;

    let events = app.wait_for_events(2).await;
    let line = events[1].to_ndjson();
    assert!(line.ends_with('\n'), "one event, one line");
    assert_eq!(line.matches('\n').count(), 1, "no embedded newlines");

    let json: serde_json::Value =
        serde_json::from_str(line.trim_end()).expect("each line is valid JSON");
    assert_eq!(json["v"], 1, "schema version");
    assert_eq!(json["event"], "modbus_request");
    assert_eq!(json["surface"], "modbus");
    assert_eq!(json["function_code"], 3);
    assert_eq!(json["start_address"], 40000);
    assert_eq!(json["ts"], "2026-06-21T16:00:00Z");
    assert!(json["connection_id"].is_string());
    assert!(
        json.get("exception_code").is_none(),
        "absent rather than null on success, so jq filters read cleanly"
    );
}
