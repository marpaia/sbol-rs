//! Criterion benches for `Document::check` over committed fixtures.
//!
//! Each bench measures only validation; parsing happens once during setup.

use criterion::{Criterion, criterion_group, criterion_main};
use sbol3::Document;

const SMALL: &str = include_str!("fixtures/small.ttl");

fn bench_check_small(criterion: &mut Criterion) {
    let document = Document::read_turtle(SMALL).expect("small fixture parses");
    criterion.bench_function("validate/small", |bencher| {
        bencher.iter(|| std::hint::black_box(document.check().is_ok()));
    });
}

criterion_group!(benches, bench_check_small);
criterion_main!(benches);
