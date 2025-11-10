use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams};
use tempfile::NamedTempFile;

fn benchmark_file_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_create");

    group.bench_function("create_small", |b| {
        b.iter(|| {
            let temp = NamedTempFile::new().unwrap();
            let mut opts = CreateOptions::default();
            opts.mode = CreateMode::Clobber;
            let _file = black_box(
                ExodusFile::create(temp.path(), opts).unwrap()
            );
        })
    });

    group.finish();
}

fn benchmark_file_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_init");

    for num_nodes in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_nodes),
            num_nodes,
            |b, &num_nodes| {
                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut opts = CreateOptions::default();
                    opts.mode = CreateMode::Clobber;
                    let mut file = ExodusFile::create(temp.path(), opts).unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes,
                        num_elems: num_nodes / 2,
                        num_elem_blocks: 1,
                        ..Default::default()
                    };

                    black_box(file.init(&params).unwrap());
                })
            },
        );
    }

    group.finish();
}

fn benchmark_file_open(c: &mut Criterion) {
    // Create a test file
    let temp = NamedTempFile::new().unwrap();
    {
        let mut opts = CreateOptions::default();
        opts.mode = CreateMode::Clobber;
        let mut file = ExodusFile::create(temp.path(), opts).unwrap();
        let params = InitParams {
            title: "Benchmark".to_string(),
            num_dim: 3,
            num_nodes: 1000,
            num_elems: 500,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();
    }

    let mut group = c.benchmark_group("file_open");

    group.bench_function("open_existing", |b| {
        b.iter(|| {
            let _file = black_box(ExodusFile::open(temp.path()).unwrap());
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_file_create, benchmark_file_init, benchmark_file_open);
criterion_main!(benches);
