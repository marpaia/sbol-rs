//! HTTP download helper shared by the bundled-snapshot bootstrap
//! binary and the runtime cache.
//!
//! Synchronous and blocking: callers in async contexts should wrap calls
//! in `tokio::task::spawn_blocking` or equivalent. Up to three attempts
//! are made on transient failures.

use std::io::{self, Read};

const USER_AGENT: &str = concat!("sbol-ontology/", env!("CARGO_PKG_VERSION"));

/// Fetches `url` into memory. Returns `Err` after three failed attempts.
pub fn fetch(url: &str) -> io::Result<Vec<u8>> {
    let mut last_error: Option<io::Error> = None;
    for _ in 0..3 {
        match fetch_once(url) {
            Ok(bytes) => return Ok(bytes),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("download failed without an error")))
}

fn fetch_once(url: &str) -> io::Result<Vec<u8>> {
    let response = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| io::Error::other(format!("HTTP request failed: {error}")))?;
    let mut reader = response.into_body().into_reader();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}
