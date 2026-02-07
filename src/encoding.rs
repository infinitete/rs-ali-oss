//! Shared percent-encoding sets for OSS request signing and URL construction.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

/// Encode everything except unreserved chars (RFC 3986) and forward slash.
/// Used for URI paths where `/` separators must be preserved.
pub(crate) const URI_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'/');

/// Encode everything except unreserved chars (RFC 3986).
/// Forward slash IS encoded — used for query parameter keys and values.
pub(crate) const QUERY_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');
