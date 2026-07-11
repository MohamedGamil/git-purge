#![allow(deprecated)]
//! Report generation port (docs/02 §3, docs/10 reporting-and-history).
//!
//! `ReportSink` abstracts report output; `Report` and `ReportFormat` are the domain
//! types.

/// Markdown report generator.
pub mod markdown;
/// JSON report generator.
pub mod json;
/// HTML report generator.
pub mod html;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// Supported report types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Audit report showing branch statuses and naming violations.
    Audit,
    /// Trend report showing cleanup progress over time.
    Trend,
}

/// A generated report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// The repo this report covers.
    pub repo: RepoId,
    /// The type of report.
    pub report_type: ReportType,
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
    reports: std::sync::Mutex<Vec<Report>>,
}

impl FakeReportSink {
    /// Create a new empty FakeReportSink.
    pub fn new() -> Self {
        Self {
            reports: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Retrieve the captured reports.
    pub fn get_reports(&self) -> Vec<Report> {
        self.reports.lock().unwrap().clone()
    }
}

impl ReportSink for FakeReportSink {
    fn write_report(&self, report: &Report) -> Result<()> {
        self.reports.lock().unwrap().push(report.clone());
        Ok(())
    }
}

/// Production report sink writing to stdout/files and archiving.
#[derive(Debug)]
pub struct FileReportSink {
    data_dir: PathBuf,
    out_path: Option<PathBuf>,
}

impl FileReportSink {
    /// Create a new FileReportSink.
    pub fn new(data_dir: PathBuf, out_path: Option<PathBuf>) -> Self {
        Self { data_dir, out_path }
    }
}

impl ReportSink for FileReportSink {
    fn write_report(&self, report: &Report) -> Result<()> {
        let type_str = match report.report_type {
            ReportType::Audit => "audit",
            ReportType::Trend => "trend",
        };

        // 1. Output to user's desired target (stdout or file)
        if let Some(ref out) = self.out_path {
            let path = if out.is_dir() {
                let date_str = report.generated_at.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
                let ext = match report.format {
                    ReportFormat::Markdown => "md",
                    ReportFormat::Json => "json",
                    ReportFormat::Html => "html",
                };
                let sanitized_id = report.repo.0.replace([':', '#'], "_");
                out.join(format!("{}-{}-{}.{}", sanitized_id, type_str, date_str, ext))
            } else {
                out.clone()
            };
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::GitPurgeError::Config(format!("Failed to create output directory: {}", e))
                })?;
            }
            std::fs::write(&path, &report.content).map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to write report file: {}", e))
            })?;
        } else {
            // Write to stdout by default
            println!("{}", report.content);
        }

        // 2. Archive report copy in data dir
        let timestamp_str = report.generated_at.format(&time::format_description::parse("[year][month][day]-[hour][minute][second]").unwrap()).unwrap_or_default();
        let ext = match report.format {
            ReportFormat::Markdown => "md",
            ReportFormat::Json => "json",
            ReportFormat::Html => "html",
        };
        let sanitized_repo_id = report.repo.0.replace([':', '#'], "_");
        let archive_path = self.data_dir
            .join("reports")
            .join(&sanitized_repo_id)
            .join(format!("{}-{}.{}", timestamp_str, type_str, ext));

        if let Some(parent) = archive_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to create archive directory: {}", e))
            })?;
        }
        std::fs::write(&archive_path, &report.content).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to archive report: {}", e))
        })?;

        Ok(())
    }
}
