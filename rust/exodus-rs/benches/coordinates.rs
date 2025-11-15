use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams};
use tempfile::NamedTempFile;

fn benchmark_write_coords(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_coords");

    for num_nodes in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_nodes),
            num_nodes,
            |b, &num_nodes| {
                let x: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();
                let y: Vec<f64> = (0..num_nodes).map(|i| i as f64 * 2.0).collect();
                let z: Vec<f64> = (0..num_nodes).map(|i| i as f64 * 3.0).collect();

                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut file = ExodusFile::create(temp.path(), {
                        let mut opts = CreateOptions::default();
                        opts.mode = CreateMode::Clobber;
                        opts
                    })
                    .unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes,
                        ..Default::default()
                    };
                    file.init(&params).unwrap();

                    black_box(file.put_coords(&x, Some(&y), Some(&z)).unwrap());
                })
            },
        );
    }

    group.finish();
}

fn benchmark_read_coords(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_coords");

    for num_nodes in [100, 1000, 10000, 100000].iter() {
        // Setup: create a file with coordinates
        let temp = NamedTempFile::new().unwrap();
        {
            let mut file = ExodusFile::create(temp.path(), {
                let mut opts = CreateOptions::default();
                opts.mode = CreateMode::Clobber;
                opts
            })
            .unwrap();
            let params = InitParams {
                title: "Benchmark".to_string(),
                num_dim: 3,
                num_nodes: *num_nodes,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x: Vec<f64> = (0..*num_nodes).map(|i| i as f64).collect();
            let y: Vec<f64> = (0..*num_nodes).map(|i| i as f64 * 2.0).collect();
            let z: Vec<f64> = (0..*num_nodes).map(|i| i as f64 * 3.0).collect();
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_nodes),
            num_nodes,
            |b, _num_nodes| {
                b.iter(|| {
                    let file = ExodusFile::open(temp.path()).unwrap();
                    let coords = black_box(file.coords::<f64>().unwrap());
                    black_box(coords);
                })
            },
        );
    }

    group.finish();
}

fn benchmark_coords_f32_f64_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("coords_conversion");

    let num_nodes = 10000;

    // Write as f32, read as f64
    group.bench_function("write_f32_read_f64", |b| {
        b.iter(|| {
            let temp = NamedTempFile::new().unwrap();
            let mut file = ExodusFile::create(temp.path(), {
                let mut opts = CreateOptions::default();
                opts.mode = CreateMode::Clobber;
                opts
            })
            .unwrap();

            let params = InitParams {
                title: "Benchmark".to_string(),
                num_dim: 3,
                num_nodes,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x: Vec<f32> = (0..num_nodes).map(|i| i as f32).collect();
            let y: Vec<f32> = (0..num_nodes).map(|i| i as f32 * 2.0).collect();
            let z: Vec<f32> = (0..num_nodes).map(|i| i as f32 * 3.0).collect();
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            drop(file);

            let file = ExodusFile::open(temp.path()).unwrap();
            let _coords = black_box(file.coords::<f64>().unwrap());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_write_coords,
    benchmark_read_coords,
    benchmark_coords_f32_f64_conversion
);
criterion_main!(benches);
