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

    /// Fetch past executions for a repository with pagination support.
    pub fn executions(
        &self,
        repo: &RepoId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::model::RunRecord>> {
        self.history.get_runs(repo, limit, offset)
    }
}
