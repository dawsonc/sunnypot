//! The committed example is the documentation of the config's shape, so it has to stay true.
//!
//! The first test drives the example through the same seam as every other test: a config that
//! parses but cannot serve a device is not a working example.

mod support;

use sunnypot::config::Config;
use support::{RawModbus, TestApp, example_config, sunspec_string};

#[tokio::test]
async fn the_committed_example_serves_a_coherent_device() {
    let app = TestApp::start_with(&example_config()).await;
    let mut client = RawModbus::connect(app.modbus_addr()).await;

    let body = client.read_registers(40004, 65).await;
    assert_eq!(
        sunspec_string(&body[0..16]),
        "Fronius",
        "the example carries the identity ADR 0002 fixed, so a fresh checkout serves a real product"
    );
    assert_eq!(sunspec_string(&body[16..32]), "Primo 5.0-1 208-240");
}

#[test]
fn an_unknown_key_is_rejected_rather_than_ignored() {
    let text = example_config().replace("[modbus]", "[modbus]\nport = 502");
    let error = Config::from_toml_str(&text).expect_err("a typo must not be silently dropped");
    assert!(
        error.to_string().contains("port"),
        "the error names the offending key: {error}"
    );
}

/// A string longer than its SunSpec field would be truncated on the wire — invisible here, obvious
/// to a scanner. Config load is where that gets caught.
#[test]
fn an_identity_string_too_long_for_its_sunspec_field_is_rejected() {
    let text = example_config().replace(
        r#"options = "3.28.1-3""#,
        r#"options = "0123456789abcdefg""#,
    );
    let error = Config::from_toml_str(&text).expect_err("17 bytes does not fit a String16");
    let message = error.to_string();
    assert!(message.contains("identity.options"), "{message}");
    assert!(message.contains("16"), "{message}");

    let long = "x".repeat(33);
    let text = example_config().replace(
        r#"manufacturer = "Fronius""#,
        &format!(r#"manufacturer = "{long}""#),
    );
    let error = Config::from_toml_str(&text).expect_err("33 bytes does not fit a String32");
    assert!(
        error.to_string().contains("identity.manufacturer"),
        "{error}"
    );
}

/// An unfilled identity field would serve an all-zero Common block — a device that is obviously
/// nobody's product. Better to refuse to start than to sit on the internet looking like that.
#[test]
fn an_empty_identity_string_is_rejected() {
    let text = example_config().replace(r#"serial = "00000000""#, r#"serial = """#);
    let error = Config::from_toml_str(&text).expect_err("an empty serial is not a device");
    assert!(error.to_string().contains("identity.serial"), "{error}");
}

#[test]
fn limits_that_would_refuse_every_connection_are_rejected() {
    let text = example_config().replace("max_connections = 256", "max_connections = 0");
    let error = Config::from_toml_str(&text).expect_err("a surface that accepts nothing is a bug");
    assert!(error.to_string().contains("max_connections"), "{error}");

    let text = example_config().replace("idle_timeout_secs = 120", "idle_timeout_secs = 0");
    let error = Config::from_toml_str(&text).expect_err("a zero timeout closes every connection");
    assert!(error.to_string().contains("idle_timeout_secs"), "{error}");
}
