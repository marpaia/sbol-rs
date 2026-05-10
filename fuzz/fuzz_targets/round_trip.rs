#![no_main]

//! Fuzzes the full parse → write → reparse cycle for all four RDF
//! formats. The first byte of the input selects the format; the
//! remainder is fed to the parser as UTF-8 text. If the first parse
//! succeeds, the document must serialize and reparse without panic
//! in the same format. Validation is intentionally skipped — this
//! target stresses the format backends, not the rule suite.

use libfuzzer_sys::fuzz_target;

use sbol::RdfFormat;

fuzz_target!(|data: &[u8]| {
    let Some((selector, rest)) = data.split_first() else {
        return;
    };
    let format = match selector % 4 {
        0 => RdfFormat::Turtle,
        1 => RdfFormat::RdfXml,
        2 => RdfFormat::JsonLd,
        _ => RdfFormat::NTriples,
    };
    let Ok(text) = std::str::from_utf8(rest) else {
        return;
    };
    let Ok(document) = sbol::Document::read(text, format) else {
        return;
    };
    let Ok(serialized) = document.write(format) else {
        return;
    };
    let _ = sbol::Document::read(&serialized, format);
});
