//! Walking the SunSpec model chain with a third-party Modbus client.
//!
//! The other tests speak raw bytes, which means they check our encoder against our own decoder.
//! This one puts `tokio-modbus` in between, so the framing sunnypot emits is read back by code that
//! knows nothing about it.
//!
//! What this does *not* do is run a real SunSpec discovery implementation: the chain walk below is
//! written out by hand, and tokio-modbus contributes framing only. It shows that the chain is
//! traversable by the standard algorithm, not that a particular scanner's implementation succeeds.
//! Confirming that against a real client is ticket 11's job.

mod support;

use support::TestApp;
use tokio_modbus::client::{Reader, tcp};
use tokio_modbus::slave::Slave;

const BASE: u16 = 40000;

/// The chain walk every SunSpec client implements: find the marker, then step model to model by the
/// length each header declares, until the terminator.
#[tokio::test]
async fn a_standard_client_walks_the_model_chain_to_the_end_marker() {
    let app = TestApp::start().await;
    let mut client = tcp::connect_slave(app.modbus_addr(), Slave(1))
        .await
        .expect("connect");

    let identifier = client
        .read_holding_registers(BASE, 2)
        .await
        .expect("transport")
        .expect("no exception");
    let marker: Vec<u8> = identifier.iter().flat_map(|r| r.to_be_bytes()).collect();
    assert_eq!(&marker, b"SunS", "the marker that starts a SunSpec map");

    let mut address = BASE + 2;
    let mut chain = Vec::new();
    loop {
        assert!(chain.len() < 32, "chain walk is not terminating: {chain:?}");
        let header = client
            .read_holding_registers(address, 2)
            .await
            .expect("transport")
            .expect("no exception");
        let (model_id, length) = (header[0], header[1]);
        if model_id == 0xFFFF {
            assert_eq!(length, 0, "the end marker declares a zero length");
            break;
        }
        chain.push((model_id, length));
        address += 2 + length;
    }

    assert_eq!(
        chain,
        vec![(1, 65)],
        "Common, 65 registers — the Fronius length, not the textbook 66"
    );
    assert_eq!(
        address, 40069,
        "the walk lands exactly on the end marker, so no model is misaligned"
    );
}
