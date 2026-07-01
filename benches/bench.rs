use std::io::Write;

use criterion::{criterion_group, criterion_main, Criterion};

use rsomics_closeness_vitality::{closeness_vitality, io};

fn load() -> io::Graph {
    let mut f = tempfile::Builder::new()
        .tempfile_in("/Volumes/KIOXIA/tmp")
        .unwrap();
    f.write_all(include_bytes!("bench_edges.txt")).unwrap();
    f.flush().unwrap();
    io::read_edgelist(Some(f.path())).unwrap()
}

fn bench(c: &mut Criterion) {
    // Parse once; measure compute-only so comparison with nx.closeness_vitality
    // (also compute-only, G pre-built) is fair.
    let g = load();

    c.bench_function("closeness_vitality gnm(150,600) all nodes", |b| {
        b.iter(|| closeness_vitality(&g))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
