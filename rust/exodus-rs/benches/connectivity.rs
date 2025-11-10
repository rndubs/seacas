use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use exodus_rs::{Block, CreateOptions, EntityType, ExodusFile, InitParams, Topology};
use tempfile::NamedTempFile;

fn benchmark_write_connectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_connectivity");

    for num_elems in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_elems),
            num_elems,
            |b, &num_elems| {
                let num_nodes = num_elems * 8; // HEX8 elements
                let connectivity: Vec<i64> = (0..num_elems * 8)
                    .map(|i| (i % num_nodes) as i64 + 1)
                    .collect();

                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes,
                        num_elems,
                        num_elem_blocks: 1,
                        ..Default::default()
                    };
                    file.init(&params).unwrap();

                    let block = Block {
                        id: 1,
                        entity_type: EntityType::ElemBlock,
                        topology: "HEX8".to_string(),
                        num_entries: num_elems,
                        num_nodes_per_entry: 8,
                        num_edges_per_entry: 0,
                        num_faces_per_entry: 0,
                        num_attributes: 0,
                    };
                    file.put_block(&block).unwrap();

                    black_box(file.put_connectivity(1, &connectivity).unwrap());
                })
            },
        );
    }

    group.finish();
}

fn benchmark_read_connectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_connectivity");

    for num_elems in [100, 1000, 10000].iter() {
        let num_nodes = num_elems * 8;
        let connectivity: Vec<i64> = (0..num_elems * 8)
            .map(|i| (i % num_nodes) as i64 + 1)
            .collect();

        // Setup: create a file with connectivity
        let temp = NamedTempFile::new().unwrap();
        {
            let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();
            let params = InitParams {
                title: "Benchmark".to_string(),
                num_dim: 3,
                num_nodes,
                num_elems: *num_elems,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: *num_elems,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(1, &connectivity).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_elems),
            num_elems,
            |b, _num_elems| {
                b.iter(|| {
                    let file = ExodusFile::open(temp.path()).unwrap();
                    let conn = black_box(file.connectivity(1).unwrap());
                    black_box(conn);
                })
            },
        );
    }

    group.finish();
}

fn benchmark_multiple_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_blocks");

    for num_blocks in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_blocks),
            num_blocks,
            |b, &num_blocks| {
                let elems_per_block = 1000;
                let num_nodes = num_blocks * elems_per_block * 8;

                b.iter(|| {
                    let temp = NamedTempFile::new().unwrap();
                    let mut file = ExodusFile::create(temp.path(), { let mut opts = CreateOptions::default(); opts.mode = CreateMode::Clobber; opts }).unwrap();

                    let params = InitParams {
                        title: "Benchmark".to_string(),
                        num_dim: 3,
                        num_nodes,
                        num_elems: num_blocks * elems_per_block,
                        num_elem_blocks: num_blocks,
                        ..Default::default()
                    };
                    file.init(&params).unwrap();

                    for i in 0..num_blocks {
                        let block = Block {
                            id: (i + 1) as i64,
                            entity_type: EntityType::ElemBlock,
                            topology: "HEX8".to_string(),
                            num_entries: elems_per_block,
                            num_nodes_per_entry: 8,
                            num_edges_per_entry: 0,
                            num_faces_per_entry: 0,
                            num_attributes: 0,
                        };
                        file.put_block(&block).unwrap();

                        let connectivity: Vec<i64> = (0..elems_per_block * 8)
                            .map(|j| (i * elems_per_block * 8 + j) as i64 + 1)
                            .collect();
                        file.put_connectivity((i + 1) as i64, &connectivity).unwrap();
                    }

                    black_box(&file);
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_write_connectivity, benchmark_read_connectivity, benchmark_multiple_blocks);
criterion_main!(benches);
