//! Clock port — abstraction over system time for deterministic tests (docs/02 §3).
//!
//! The `Clock` port lets tests inject a fixed time, ensuring fixture repos produce
//! byte-identical commit SHAs and deterministic staleness classifications.

use time::OffsetDateTime;

/// Port for the system clock.
///
/// `Send + Sync` so `Engine` (which holds the clock) can be shared across threads.
pub trait Clock: Send + Sync + std::fmt::Debug {
    /// Return the current time in UTC.
    fn now(&self) -> OffsetDateTime;
}

/// Production clock — delegates to the system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// Fake clock for deterministic tests. Returns a fixed time.
#[derive(Debug, Clone)]
pub struct FakeClock {
    /// The fixed time to return.
    pub now: OffsetDateTime,
}

impl FakeClock {
    /// Create a fake clock pinned to the given time.
    pub fn new(now: OffsetDateTime) -> Self {
        Self { now }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> OffsetDateTime {
        self.now
    }
}
