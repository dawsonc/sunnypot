//! Deployment configuration.
//!
//! One file supplies everything that distinguishes one deployment from another: the site the plant
//! models, the identity strings the surfaces serve, and where to listen. The file is gitignored;
//! `sunnypot.example.toml` documents its shape.

use std::net::SocketAddr;
use std::path::Path;

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
    /// SunSpec Common `Md`.
    pub model: String,
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
        Ok(config)
    }
}

impl Identity {
    fn validate(&self) -> Result<(), ConfigError> {
        let fields = [
            ("manufacturer", self.manufacturer.as_str(), STRING32),
            ("model", self.model.as_str(), STRING32),
            ("options", self.options.as_str(), STRING16),
            ("version", self.version.as_str(), STRING16),
            ("serial", self.serial.as_str(), STRING32),
        ];
        for (field, value, max) in fields {
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
