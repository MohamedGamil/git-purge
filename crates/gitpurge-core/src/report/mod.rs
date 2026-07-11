//! Report generation port (docs/02 §3, docs/10 reporting-and-history).
//!
//! `ReportSink` abstracts report output; `Report` and `ReportFormat` are the domain
//! types.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::RepoId;

/// Supported report output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// Markdown.
    Markdown,
    /// JSON.
    Json,
    /// HTML.
    Html,
}

/// A generated report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// The repo this report covers.
    pub repo: RepoId,
    /// Output format.
    pub format: ReportFormat,
    /// The rendered report content.
    pub content: String,
    /// When the report was generated.
    pub generated_at: time::OffsetDateTime,
}

/// Port for report output.
///
/// Implementations must be `Send + Sync` for shared `Engine` access.
pub trait ReportSink: Send + Sync + std::fmt::Debug {
    /// Write a report to the appropriate destination.
    fn write_report(&self, report: &Report) -> Result<()>;
}

/// In-memory fake for tests.
#[derive(Debug, Default)]
pub struct FakeReportSink {
    // TODO(P5): capture written reports for assertions.
}

impl ReportSink for FakeReportSink {
    fn write_report(&self, _report: &Report) -> Result<()> {
        Ok(())
    }
}
