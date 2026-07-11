//! Domain model — the core vocabulary of Git Purge (see `docs/03-domain-model.md`).
//!
//! These types are the shape the whole product is built from. They are faithful to
//! the domain-model spec; concrete constructors, invariants, and conversions land in
//! later phases. Every type derives `serde` so it can cross the Tauri IPC boundary and
//! be snapshot-tested.
//!
//! Timestamps are `time::OffsetDateTime` stored in UTC; ages/durations are
//! `std::time::Duration` or a parsed [`AgeThreshold`].

mod classification;
mod objects;
mod plan;
mod policy;
mod repo;
mod snapshot;
mod trend;
mod value_objects;

pub use classification::*;
pub use objects::*;
pub use plan::*;
pub use policy::*;
pub use repo::*;
pub use snapshot::*;
pub use trend::*;
pub use value_objects::*;
