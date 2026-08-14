//! Small shared utilities used by generated clients.

use std::fmt::Write;

/// Percent-encodes a path segment.
///
/// Encodes everything outside the RFC 3986 unreserved set, so values
/// containing `/`, `?`, `#` and friends are safe inside a URL path.
pub fn encode_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for &byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::encode_path_segment;

    #[test]
    fn leaves_unreserved_characters_alone() {
        assert_eq!(encode_path_segment("abc123-_.~"), "abc123-_.~");
    }

    #[test]
    fn encodes_special_characters() {
        assert_eq!(encode_path_segment("a/b?c#d e"), "a%2Fb%3Fc%23d%20e");
    }

    #[test]
    fn encodes_utf8_bytes() {
        assert_eq!(
            encode_path_segment("привет"),
            "%D0%BF%D1%80%D0%B8%D0%B2%D0%B5%D1%82"
        );
    }
}
