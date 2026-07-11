//! Progress reporting port (docs/02 §5).
//!
//! Long-running operations report progress through this trait. The CLI adapter
//! renders to `indicatif`; the Tauri adapter forwards to `gitpurge://progress` events.

/// Port for progress reporting.
///
/// `Send + Sync` so it can be called from async tasks and Tauri command handlers.
pub trait ProgressSink: Send + Sync + std::fmt::Debug {
    /// Set the total number of steps.
    fn set_total(&self, total: u64);

    /// Advance by one step with an optional message.
    fn tick(&self, message: Option<&str>);

    /// Set the current position.
    fn set_position(&self, pos: u64);

    /// Mark the operation as finished.
    fn finish(&self, message: Option<&str>);
}

/// No-op progress sink (used when progress reporting is disabled).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopProgressSink;

impl ProgressSink for NoopProgressSink {
    fn set_total(&self, _total: u64) {}
    fn tick(&self, _message: Option<&str>) {}
    fn set_position(&self, _pos: u64) {}
    fn finish(&self, _message: Option<&str>) {}
}

/// Fake progress sink for tests — records calls for assertions.
#[derive(Debug, Default)]
pub struct FakeProgressSink {
    // TODO(P0-T4): add fields to record progress calls.
}

impl ProgressSink for FakeProgressSink {
    fn set_total(&self, _total: u64) {}
    fn tick(&self, _message: Option<&str>) {}
    fn set_position(&self, _pos: u64) {}
    fn finish(&self, _message: Option<&str>) {}
}
