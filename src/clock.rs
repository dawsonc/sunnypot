//! The clock seam.
//!
//! Every timestamp and every duration in sunnypot comes from a [`Clock`], so tests can assert on
//! them exactly. Nothing reads the wall clock directly.

use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> DateTime<Utc>;
}

/// The wall clock. Plant state is a function of wall-clock time rather than uptime, so a restart
/// produces no discontinuity.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A clock that only moves when a test moves it.
#[derive(Debug)]
pub struct FixedClock {
    now: Mutex<DateTime<Utc>>,
}

impl FixedClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, by: Duration) {
        let mut now = self.now.lock().expect("clock lock");
        *now += by;
    }

    pub fn set(&self, to: DateTime<Utc>) {
        *self.now.lock().expect("clock lock") = to;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().expect("clock lock")
    }
}
