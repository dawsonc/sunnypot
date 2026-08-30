//! Sunnypot: a low-interaction honeypot impersonating a grid-tied PV inverter.
//!
//! The crate is organised as the spec describes: a plant that owns device state, protocol
//! *surfaces* that expose it, and a capture sink both surfaces write to. Surfaces never own
//! values; they project them.

pub mod app;
pub mod capture;
pub mod clock;
pub mod config;
pub mod modbus;
