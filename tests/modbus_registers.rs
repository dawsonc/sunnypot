//! What a Modbus client reads out of the SunSpec register map.

mod support;

use support::{RawModbus, TestApp, sunspec_string};

/// The protocol (0-based) address of the SunSpec identifier, documented as holding register 40001
/// in the 1-based tables vendors publish. Standard discovery probes this address.
const BASE: u16 = 40000;

#[tokio::test]
async fn the_sunspec_identifier_sits_at_the_documented_base_address() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    let response = client.read_holding(BASE, 2).await;

    assert_eq!(
        response.pdu,
        // FC 3, 4 bytes, "SunS"
        vec![0x03, 0x04, 0x53, 0x75, 0x6e, 0x53],
        "the identifier is the ASCII bytes of \"SunS\", big-endian"
    );
}

#[tokio::test]
async fn the_common_block_serves_the_identity_from_config() {
    let app = TestApp::start().await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    // The Common block header sits immediately after the identifier.
    let header = client.read_registers(BASE + 2, 2).await;
    assert_eq!(header[0], 1, "model id 1, Common");
    assert_eq!(
        header[1], 65,
        "Fronius serves a 65-register Common block, not the textbook 66"
    );

    let body = client.read_registers(BASE + 4, 65).await;
    assert_eq!(sunspec_string(&body[0..16]), "Testvendor", "Mn");
    assert_eq!(
        sunspec_string(&body[16..32]),
        "Testmodel 5.0-1 208-240",
        "Md"
    );
    assert_eq!(sunspec_string(&body[32..40]), "3.28.1-3", "Opt");
    assert_eq!(sunspec_string(&body[40..48]), "1.19.10-0", "Vr");
    assert_eq!(sunspec_string(&body[48..64]), "30514231", "SN");
    assert_eq!(body[64], 1, "DA, the Modbus device address");
}
