//! Benchmarks for the pure, CPU-bound header-processing logic on the response path.
//!
//! These cover the work `PhpProxy::request_filter` does for every response: deciding
//! which PHP-supplied headers the front owns (`skip_response_header`) and parsing any
//! `Connection` header into the hop-by-hop field names it lists (`connection_named_headers`).

use rapira_http::{connection_named_headers, skip_response_header};

fn main() {
    divan::main();
}

/// A representative set of response header names a PHP app might emit, mixing
/// server-owned framing/connection fields with normal application headers.
const RESPONSE_HEADER_NAMES: &[&str] = &[
    "Content-Type",
    "Content-Length",
    "Cache-Control",
    "Set-Cookie",
    "X-Powered-By",
    "Vary",
    "Transfer-Encoding",
    "Connection",
    "Keep-Alive",
    "Date",
    "Server",
    "Location",
    "ETag",
    "Content-Encoding",
    "Upgrade",
    "Link",
];

/// The common case: classify every header name in a typical response once.
#[divan::bench]
fn classify_response_headers() -> usize {
    let mut owned = 0usize;
    for name in RESPONSE_HEADER_NAMES {
        if skip_response_header(divan::black_box(name)) {
            owned += 1;
        }
    }
    owned
}

/// Worst case for the matcher: a header name that is not in the skip list, so every
/// entry is compared before returning false.
#[divan::bench]
fn skip_header_miss() -> bool {
    skip_response_header(divan::black_box("x-application-specific-header"))
}

/// A hit near the front of the skip list (framing header, the frequent case).
#[divan::bench]
fn skip_header_hit() -> bool {
    skip_response_header(divan::black_box("content-length"))
}

/// Parsing a multi-token `Connection` header value into its named fields — the
/// rare-but-nontrivial path (split, trim, lower-case each token).
#[divan::bench]
fn parse_connection_header() -> Vec<String> {
    let mut out = Vec::new();
    connection_named_headers(
        divan::black_box(b"keep-alive, X-Custom-Hop , Upgrade,Trailer"),
        &mut out,
    );
    out
}
