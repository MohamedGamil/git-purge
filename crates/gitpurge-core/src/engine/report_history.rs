//! Reporting & history methods (P10-T2).

use crate::error::Result;
use crate::model::RepoId;
use crate::report::ReportFormat;

impl super::Engine {
    /// Generate an audit/trend report in the requested format.
    pub fn report(
        &self,
        repo: &RepoId,
        report_type: crate::report::ReportType,
        fmt: ReportFormat,
    ) -> Result<crate::report::Report> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        // 1. Run a scan to get the current classifications
        let scan_result = self.scan(repo, crate::model::ScanOptions::default())?;

        // 2. Fetch trend history
        let history = self.history.get_history(repo)?;

        // 3. Generate content based on format and type
        let generated_at = self.clock.now();
        let content = match report_type {
            crate::report::ReportType::Audit => match fmt {
                ReportFormat::Markdown => crate::report::markdown::generate_audit_report(
                    &repo_model,
                    &scan_result,
                    generated_at,
                ),
                ReportFormat::Json => crate::report::json::generate_json_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                )?,
                ReportFormat::Html => crate::report::html::generate_html_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                ),
            },
            crate::report::ReportType::Trend => match fmt {
                ReportFormat::Markdown => crate::report::markdown::generate_trend_report(
                    &repo_model,
                    &history,
                    generated_at,
                    None,
                ),
                ReportFormat::Json => crate::report::json::generate_json_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                )?,
                ReportFormat::Html => crate::report::html::generate_html_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                ),
            },
        };

        let report = crate::report::Report {
            repo: repo.clone(),
            report_type,
            format: fmt,
            content,
            generated_at,
        };

        // 4. Write/archive the report using the configured report sink
        self.report_sink.write_report(&report)?;

        Ok(report)
    }

    /// Fetch the recorded trend history for a repository.
    pub fn history(&self, repo: &RepoId) -> Result<crate::model::TrendHistory> {
        self.history.get_history(repo)
    }

    /// Import legacy JSON history data.
    pub fn import_history(
        &self,
        json_data: &str,
        repo_mappings: &std::collections::HashMap<String, String>,
        execute: bool,
    ) -> Result<crate::history::ImportSummary> {
        self.history.import_legacy_json(json_data, repo_mappings, execute)
    }

    /// Fetch past executions for a repository with pagination support.
    pub fn executions(
        &self,
        repo: &RepoId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::model::RunRecord>> {
        self.history.get_runs(repo, limit, offset)
    }

    /// Compare two past execution runs and return a detailed trend diff.
    pub fn trends(
        &self,
        repo: &RepoId,
        old_run_id: &str,
        new_run_id: &str,
    ) -> Result<crate::model::TrendDiff> {
        let old_run = self.history.get_run_record(old_run_id)?.ok_or_else(|| {
            crate::GitPurgeError::Config(format!("Run record not found: {}", old_run_id))
        })?;
        let new_run = self.history.get_run_record(new_run_id)?.ok_or_else(|| {
            crate::GitPurgeError::Config(format!("Run record not found: {}", new_run_id))
        })?;

        let old_classifications = self.history.get_run_classifications(old_run_id)?;
        let new_classifications = self.history.get_run_classifications(new_run_id)?;

        let old_scan = crate::model::ScanResult {
            repo: repo.clone(),
            total_branches: old_classifications.len(),
            excluded_count: 0,
            classifications: old_classifications,
        };

        let new_scan = crate::model::ScanResult {
            repo: repo.clone(),
            total_branches: new_classifications.len(),
            excluded_count: 0,
            classifications: new_classifications,
        };

        Ok(crate::history::trends::compare_scans(
            &old_scan,
            &new_scan,
            old_run.started_at,
            new_run.started_at,
        ))
    }
}
