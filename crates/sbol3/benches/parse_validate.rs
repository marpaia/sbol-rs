//! End-to-end parse + check bench. Measures combined cost (a closer model
//! of `sbol validate <path>` than the validate-only bench).

use criterion::{Criterion, criterion_group, criterion_main};
use sbol3::Document;

const SMALL: &str = include_str!("fixtures/small.ttl");

fn bench_parse_check_small(criterion: &mut Criterion) {
    criterion.bench_function("parse_validate/small", |bencher| {
        bencher.iter(|| {
            let document = Document::read_turtle(SMALL).unwrap();
            std::hint::black_box(document.check().is_ok());
        });
    });
}

criterion_group!(benches, bench_parse_check_small);
criterion_main!(benches);
