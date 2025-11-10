use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use exodus_rs::{CreateMode, CreateOptions, EntityType, ExodusFile, InitParams};
use tempfile::NamedTempFile;

fn benchmark_write_nodal_vars(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_nodal_vars");

    for num_nodes in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_nodes),
            num_nodes,
            |b, &num_nodes| {
                let values: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();

                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes,
                        ..Default::default()
                    };
                    file.init(&params).unwrap();

                    file.define_variables(EntityType::Nodal, &["temperature"]).unwrap();
                    file.put_time(1, 0.0).unwrap();

                    black_box(file.put_var(1, EntityType::Nodal, 0, 0, &values).unwrap());
                })
            },
        );
    }

    group.finish();
}

fn benchmark_read_nodal_vars(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_nodal_vars");

    for num_nodes in [1000, 10000, 100000].iter() {
        let values: Vec<f64> = (0..*num_nodes).map(|i| i as f64).collect();

        // Setup: create a file with variables
        let temp = NamedTempFile::new().unwrap();
        {
            let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();
            let params = InitParams {
                title: "Benchmark".to_string(),
                num_dim: 3,
                num_nodes: *num_nodes,
                ..Default::default()
            };
            file.init(&params).unwrap();

            file.define_variables(EntityType::Nodal, &["temperature"]).unwrap();
            file.put_time(1, 0.0).unwrap();
            file.put_var(1, EntityType::Nodal, 0, 0, &values).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_nodes),
            num_nodes,
            |b, _num_nodes| {
                b.iter(|| {
                    let file = ExodusFile::open(temp.path()).unwrap();
                    let vals: Vec<f64> = black_box(file.var(1, EntityType::Nodal, 0, 0).unwrap());
                    black_box(vals);
                })
            },
        );
    }

    group.finish();
}

fn benchmark_write_global_vars(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_global_vars");

    for num_vars in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_vars),
            num_vars,
            |b, &num_vars| {
                let var_names: Vec<String> = (0..num_vars)
                    .map(|i| format!("var_{}", i))
                    .collect();
                let var_names_ref: Vec<&str> = var_names.iter().map(|s| s.as_str()).collect();
                let values: Vec<f64> = (0..num_vars).map(|i| i as f64).collect();

                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes: 100,
                        ..Default::default()
                    };
                    file.init(&params).unwrap();

                    file.define_variables(EntityType::Global, &var_names_ref).unwrap();
                    file.put_time(1, 0.0).unwrap();

                    for (i, &val) in values.iter().enumerate() {
                        file.put_var(1, EntityType::Global, i as i64, 0, &[val]).unwrap();
                    }

                    black_box(&file);
                })
            },
        );
    }

    group.finish();
}

fn benchmark_multiple_time_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_time_steps");

    for num_steps in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_steps),
            num_steps,
            |b, &num_steps| {
                let num_nodes = 1000;
                let values: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();

                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes,
                        ..Default::default()
                    };
                    file.init(&params).unwrap();

                    file.define_variables(EntityType::Nodal, &["temperature"]).unwrap();

                    for step in 0..num_steps {
                        file.put_time((step + 1) as usize, step as f64 * 0.1).unwrap();
                        file.put_var((step + 1) as usize, EntityType::Nodal, 0, 0, &values).unwrap();
                    }

                    black_box(&file);
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_write_nodal_vars,
    benchmark_read_nodal_vars,
    benchmark_write_global_vars,
    benchmark_multiple_time_steps
);
criterion_main!(benches);
