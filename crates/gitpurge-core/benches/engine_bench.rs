use criterion::{criterion_group, criterion_main, Criterion};
use gitpurge_core::{
    model::ScanOptions,
    report::{ReportFormat, ReportType},
    testkit, Config, Engine,
};
use std::time::Duration;

fn bench_engine_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine-ops");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));

    // Set up a 500-branch fixture repo
    let fixture = testkit::benchmark_repo(500);

    let temp_data_dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: Some(temp_data_dir.path().to_path_buf()),
        ..Config::default()
    };

    let engine = Engine::open(config).unwrap();
    let repo_model =
        gitpurge_core::model::Repository::new_local(fixture.path().to_path_buf()).unwrap();
    let repo_id = repo_model.id.clone();

    // Add repo to engine
    engine.add_repo(repo_model).unwrap();

    // Benchmark Scan
    group.bench_function("scan_500_branches", |b| {
        b.iter(|| {
            let _ = engine.scan(&repo_id, ScanOptions::default()).unwrap();
        })
    });

    // Benchmark Plan
    group.bench_function("plan_500_branches", |b| {
        b.iter(|| {
            let filter = gitpurge_core::model::ActionFilter::default();
            let _ = engine.plan(&repo_id, &filter).unwrap();
        })
    });

    // Benchmark Report Generation
    group.bench_function("report_500_branches", |b| {
        b.iter(|| {
            let _ = engine
                .report(&repo_id, ReportType::Audit, ReportFormat::Markdown)
                .unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_engine_operations);
criterion_main!(benches);
