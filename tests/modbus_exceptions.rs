//! Error handling. A device that answers wrong requests wrongly is a device that stands out.

mod support;

use support::{RawModbus, TestApp};

const BASE: u16 = 40000;

/// The last mapped register: Common ends at 40068, the end marker occupies 40069 and 40070.
const LAST_MAPPED: u16 = 40070;

#[tokio::test]
async fn an_unsupported_function_code_returns_illegal_function() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    // FC 4, read input registers: a real function code this device does not serve.
    let response = client.request(1, &[0x04, 0x9c, 0x40, 0x00, 0x02]).await;
    assert_eq!(
        response.pdu,
        vec![0x84, 0x01],
        "function code with the exception bit set, then illegal function"
    );

    // FC 0x41, in the vendor-defined range: not a function code at all, as far as this device goes.
    let response = client.request(1, &[0x41, 0xff]).await;
    assert_eq!(response.pdu, vec![0xc1, 0x01]);
}

#[tokio::test]
async fn reads_outside_the_map_return_illegal_data_address() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    // Well below the map.
    assert_eq!(client.read_holding(0, 1).await.pdu, vec![0x83, 0x02]);
    // Just below the base address.
    assert_eq!(client.read_holding(BASE - 1, 2).await.pdu, vec![0x83, 0x02]);
    // Starts inside the map but runs off the end.
    assert_eq!(
        client.read_holding(LAST_MAPPED, 2).await.pdu,
        vec![0x83, 0x02],
        "a read may not straddle the end of the map"
    );
    // Well above the map.
    assert_eq!(client.read_holding(50000, 1).await.pdu, vec![0x83, 0x02]);
    // The last mapped register on its own is fine.
    assert_eq!(client.read_registers(LAST_MAPPED, 1).await, vec![0x0000]);
}

#[tokio::test]
async fn an_illegal_quantity_returns_illegal_data_value() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    assert_eq!(
        client.read_holding(BASE, 0).await.pdu,
        vec![0x83, 0x03],
        "zero registers is not a legal read"
    );
    assert_eq!(
        client.read_holding(BASE, 126).await.pdu,
        vec![0x83, 0x03],
        "126 registers exceeds what a response PDU can carry"
    );
    assert_eq!(
        client.read_holding(0, 126).await.pdu,
        vec![0x83, 0x03],
        "quantity is validated before the address, as the Modbus spec sequences it"
    );
    assert_eq!(
        client.read_registers(BASE, 69).await.len(),
        69,
        "a single read may span the identifier and the whole Common block"
    );
}

#[tokio::test]
async fn a_truncated_read_request_returns_illegal_data_value() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    // FC 3 with a payload too short to name a register range.
    let response = client.request(1, &[0x03, 0x9c]).await;
    assert_eq!(response.pdu, vec![0x83, 0x03]);
}

#[tokio::test]
async fn the_response_echoes_the_transaction_and_unit_of_the_request() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    let first = client.read_holding(BASE, 2).await;
    let second = client.request(0x2a, &[0x03, 0x9c, 0x40, 0x00, 0x02]).await;

    assert_eq!(first.transaction_id, 1);
    assert_eq!(first.protocol_id, 0);
    assert_eq!(first.unit_id, 1);
    assert_eq!(second.transaction_id, 2, "each request gets its own reply");
    assert_eq!(
        second.unit_id, 0x2a,
        "sunnypot answers whatever unit id it is asked for, so a scanner probing unit ids \
         always gets an answer"
    );
}
