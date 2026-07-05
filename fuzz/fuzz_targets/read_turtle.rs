#![no_main]

//! Fuzzes `Document::read` over all four RDF formats. The first byte
//! of the input selects the format; the remainder is fed to the
//! parser as UTF-8 text. Any panic from the parser (including from
//! the underlying `oxrdfio` backend) is a fuzz failure. Errors are
//! expected and ignored. We're hunting for panics, infinite loops,
//! and OOMs.

use libfuzzer_sys::fuzz_target;

use sbol3::RdfFormat;

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
    if let Ok(text) = std::str::from_utf8(rest) {
        let _ = sbol3::Document::read(text, format);
    }
});
