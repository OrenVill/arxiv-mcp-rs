//! Criterion stub for SPEC §13.5 (expand after M5 PDF golden job).

use std::hint::black_box;

use arxiv_core::ArxivId;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_arxiv_id_parse(c: &mut Criterion) {
    c.bench_function("arxiv_id_parse_new_format", |b| {
        b.iter(|| ArxivId::parse(black_box("1706.03762v7")).unwrap());
    });
    c.bench_function("arxiv_id_parse_legacy_slash", |b| {
        b.iter(|| ArxivId::parse(black_box("hep-th/9901001")).unwrap());
    });
}

criterion_group!(benches, bench_arxiv_id_parse);
criterion_main!(benches);
