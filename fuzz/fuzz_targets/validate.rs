#![no_main]

//! Fuzzes `Document::check` for panics. Inputs that don't parse as
//! Turtle are skipped — we're hunting for validator-side panics or
//! infinite loops, not for parser bugs (those are covered by
//! `read_turtle`).

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(document) = sbol3::Document::read_turtle(text) else {
        return;
    };
    let _ = document.check();
});
