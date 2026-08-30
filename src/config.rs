//! Deployment configuration.
//!
//! One file supplies everything that distinguishes one deployment from another: the site the plant
//! models, the identity strings the surfaces serve, and where to listen. The file is gitignored;
//! `sunnypot.example.toml` documents its shape.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

/// Widths of the SunSpec Common block string fields, in bytes. A string that does not fit would be
/// silently truncated on the wire, which is exactly the kind of quiet realism bug that only a
/// scanner would notice, so config load rejects it instead.
const STRING32: usize = 32;
const STRING16: usize = 16;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub site: Site,
    pub identity: Identity,
    pub modbus: ModbusConfig,
}

/// The physical installation the plant models. Unused until the plant lands; carried here because
/// a second deployment must be a config change rather than a rebuild.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub array_capacity_kw: f64,
    pub inverter_rating_kw: f64,
}

/// The product sunnypot impersonates, as a scanner can observe it. Values come from
/// `docs/adr/0002-impersonate-fronius-primo.md`; the surfaces read them from here rather than
/// deciding for themselves.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    /// SunSpec Common `Mn`.
    pub manufacturer: String,
    /// SunSpec Common `Md`. The product name — ADR 0002 calls this field `Product`, and
    /// `CONTEXT.md` reserves "model" for a SunSpec register-block definition.
    pub product: String,
    /// SunSpec Common `Opt`.
    pub options: String,
    /// SunSpec Common `Vr`.
    pub version: String,
    /// SunSpec Common `SN`. Must not be copied from a real unit — a real serial would misattribute
    /// sunnypot's traffic to a real owner's device.
    pub serial: String,
    /// SunSpec Common `DA`, the Modbus device address the identity advertises.
    pub device_address: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModbusConfig {
    pub bind: SocketAddr,
    /// How many connections may be open at once. The surface is hostile ground: without a cap, a
    /// peer that opens sockets and never speaks walks the process to fd exhaustion.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    /// How long a connection may sit without sending a complete request before it is closed.
    /// Generous on purpose — holding an attacker's attention is the point — but bounded.
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

fn default_max_connections() -> usize {
    256
}

fn default_idle_timeout_secs() -> u64 {
    120
}

impl ModbusConfig {
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("identity.{field} is {len} bytes; the SunSpec field holds at most {max}")]
    StringTooLong {
        field: &'static str,
        len: usize,
        max: usize,
    },
    #[error("identity.{field} is empty; the device would advertise nothing there")]
    StringEmpty { field: &'static str },
    #[error("modbus.{field} is {value}; {expectation}")]
    OutOfRange {
        field: &'static str,
        value: u64,
        expectation: &'static str,
    },
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_toml_str(&text)
    }

    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(text)?;
        config.identity.validate()?;
        config.modbus.validate()?;
        Ok(config)
    }
}

impl Identity {
    fn validate(&self) -> Result<(), ConfigError> {
        let fields = [
            ("manufacturer", self.manufacturer.as_str(), STRING32),
            ("product", self.product.as_str(), STRING32),
            ("options", self.options.as_str(), STRING16),
            ("version", self.version.as_str(), STRING16),
            ("serial", self.serial.as_str(), STRING32),
        ];
        for (field, value, max) in fields {
            // An empty field is not a shape the device can serve: it would advertise an all-zero
            // Common block, which is a config nobody filled in rather than a device.
            if value.is_empty() {
                return Err(ConfigError::StringEmpty { field });
            }
            if value.len() > max {
                return Err(ConfigError::StringTooLong {
                    field,
                    len: value.len(),
                    max,
                });
            }
        }
        Ok(())
    }
}

impl ModbusConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.max_connections == 0 {
            return Err(ConfigError::OutOfRange {
                field: "max_connections",
                value: 0,
                expectation: "the surface must accept at least one connection",
            });
        }
        if self.idle_timeout_secs == 0 {
            return Err(ConfigError::OutOfRange {
                field: "idle_timeout_secs",
                value: 0,
                expectation: "a zero timeout would close every connection before it spoke",
            });
        }
        Ok(())
    }
}
