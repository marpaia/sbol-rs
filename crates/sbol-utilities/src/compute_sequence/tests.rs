use super::*;
use crate::prelude::*;
use sbol3::design::Design;

const NS: &str = "https://github.com/DRAGGON-Lab";

fn region_iri(document: &Document, display_id: &str) -> Resource {
    document
        .components()
        .find(|component| component.display_id() == Some(display_id))
        .expect("region present")
        .identity
        .clone()
}

#[test]
fn concatenates_parts_in_meets_order_with_inclusive_ranges() {
    let mut d = Design::new(NS).unwrap();
    let plac = d.promoter("pLac", "aaa").add();
    let b0034 = d.rbs("B0034", "cc").add();
    let tetr = d.cds("tetR", "gggg").add();
    d.engineered_region("tu", [plac, b0034, tetr]).add();
    let doc = d.finish().unwrap();

    let region = region_iri(&doc, "tu");
    let doc = compute_sequence(&doc, &region).unwrap();

    let region = doc
        .components()
        .find(|component| component.display_id() == Some("tu"))
        .unwrap();
    assert_eq!(region.sequences.len(), 1);

    let sequence = doc
        .sequences()
        .find(|sequence| sequence.display_id() == Some("tu_sequence"))
        .unwrap();
    assert_eq!(sequence.elements.as_deref(), Some("aaaccgggg"));
    assert_eq!(sequence.encoding.as_ref(), Some(&EDAM_IUPAC_DNA));

    let mut ranges: Vec<_> = doc.ranges().collect();
    ranges.sort_by_key(|range| range.start);
    assert_eq!(ranges.len(), 3);
    assert_eq!((ranges[0].start, ranges[0].end), (Some(1), Some(3)));
    assert_eq!((ranges[1].start, ranges[1].end), (Some(4), Some(5)));
    assert_eq!((ranges[2].start, ranges[2].end), (Some(6), Some(9)));

    // Each sub-component now carries exactly one location.
    for sub in doc.sub_components() {
        assert_eq!(
            sub.locations.len(),
            1,
            "sub {:?} location count",
            sub.display_id()
        );
    }

    assert!(doc.check().is_ok(), "computed document should validate");
}

#[test]
fn compute_all_sequences_handles_every_region() {
    let mut d = Design::new(NS).unwrap();
    let p1 = d.promoter("p1", "aa").add();
    let t1 = d.terminator("t1", "gg").add();
    d.engineered_region("tu1", [p1, t1]).add();
    let p2 = d.promoter("p2", "cc").add();
    let t2 = d.terminator("t2", "tt").add();
    d.engineered_region("tu2", [p2, t2]).add();
    let doc = d.finish().unwrap();

    let doc = compute_all_sequences(&doc).unwrap();

    assert_eq!(
        doc.sequences()
            .find(|s| s.display_id() == Some("tu1_sequence"))
            .and_then(|s| s.elements.as_deref()),
        Some("aagg")
    );
    assert_eq!(
        doc.sequences()
            .find(|s| s.display_id() == Some("tu2_sequence"))
            .and_then(|s| s.elements.as_deref()),
        Some("cctt")
    );
    assert!(doc.check().is_ok());
}

#[test]
fn single_part_region_needs_no_ordering() {
    let mut d = Design::new(NS).unwrap();
    let only = d.promoter("solo", "acgt").add();
    d.engineered_region("tu", [only]).add();
    let doc = d.finish().unwrap();

    let doc = compute_sequence(&doc, &region_iri(&doc, "tu")).unwrap();
    let sequence = doc
        .sequences()
        .find(|s| s.display_id() == Some("tu_sequence"))
        .unwrap();
    assert_eq!(sequence.elements.as_deref(), Some("acgt"));
}

#[test]
fn missing_part_sequence_is_reported() {
    let mut d = Design::new(NS).unwrap();
    // A bare part component with a role but no sequence.
    let bare = d
        .component("bare")
        .dna()
        .role(sbol3::constants::SO_PROMOTER)
        .add();
    let region = d.component("tu").dna().role(SO_ENGINEERED_REGION).add();
    d.sub_component(region, "bare_sub")
        .instance_of(bare)
        .role(sbol3::constants::SO_PROMOTER)
        .add();
    let doc = d.finish().unwrap();

    let err = compute_sequence(&doc, &region_iri(&doc, "tu"))
        .expect_err("a part with no sequence should be reported");
    assert!(matches!(
        err,
        ComputeSequenceError::PartSequenceCount { count: 0, .. }
    ));
}
