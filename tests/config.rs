//! The committed example is the documentation of the config's shape, so it has to stay true.

use sunnypot::config::Config;

fn example() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/sunnypot.example.toml");
    std::fs::read_to_string(path).expect("the example config is committed")
}

#[test]
fn the_committed_example_still_describes_the_config() {
    let config = Config::from_toml_str(&example()).expect("the example parses");
    assert_eq!(
        config.identity.serial, "",
        "the example documents shape, never a deployment's values"
    );
}

#[test]
fn an_unknown_key_is_rejected_rather_than_ignored() {
    let text = example().replace("[modbus]", "[modbus]\nport = 502");
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
    let text = example().replace(r#"options = """#, r#"options = "0123456789abcdefg""#);
    let error = Config::from_toml_str(&text).expect_err("17 bytes does not fit a String16");
    let message = error.to_string();
    assert!(message.contains("identity.options"), "{message}");
    assert!(message.contains("16"), "{message}");

    let text = example().replace(
        r#"manufacturer = """#,
        &format!(r#"manufacturer = "{}""#, "x".repeat(33)),
    );
    let error = Config::from_toml_str(&text).expect_err("33 bytes does not fit a String32");
    assert!(
        error.to_string().contains("identity.manufacturer"),
        "{error}"
    );
}
