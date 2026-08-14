//! Sport file and container concerns: formats, metadata, identification.
//!
//! This crate is about files (GPX, FIT, TCX, ...), not HTTP. Identification
//! works on a leading chunk of bytes so it composes with streaming
//! transfers: sniff the first chunk, then stream the rest.
//!
//! Native/lossless data should be preferred for cross-provider transfers:
//! when the source provides an original FIT file and the destination
//! accepts FIT, transfer the original instead of re-encoding. Normalization
//! and re-encoding are only used when native transfer is impossible.
//!
//! No FIT parser is implemented here; existing ecosystem parsers should be
//! evaluated before adopting or wrapping one.

use std::path::Path;

use oxisport_runtime::{ContentLength, MediaType};

/// A sport file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SportFileFormat {
    /// GPS eXchange Format (XML).
    Gpx,
    /// Garmin FIT (binary).
    Fit,
    /// Training Center XML.
    Tcx,
}

impl SportFileFormat {
    /// The conventional media type for the format.
    pub fn media_type(self) -> &'static str {
        match self {
            SportFileFormat::Gpx => "application/gpx+xml",
            SportFileFormat::Fit => "application/fit",
            SportFileFormat::Tcx => "application/vnd.garmin.tcx+xml",
        }
    }

    /// The conventional file extension (without the leading dot).
    pub fn extension(self) -> &'static str {
        match self {
            SportFileFormat::Gpx => "gpx",
            SportFileFormat::Fit => "fit",
            SportFileFormat::Tcx => "tcx",
        }
    }

    /// Identifies the format from a file extension.
    pub fn from_extension(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "gpx" => Some(SportFileFormat::Gpx),
            "fit" => Some(SportFileFormat::Fit),
            "tcx" => Some(SportFileFormat::Tcx),
            _ => None,
        }
    }

    /// Identifies the format from leading bytes of the file.
    ///
    /// Works on a partial leading chunk, so it can be used on the first
    /// bytes of a stream before consuming the rest.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if is_fit(bytes) {
            return Some(SportFileFormat::Fit);
        }
        let text = leading_text(bytes);
        let text = strip_xml_preamble(&text);
        if text.starts_with("<gpx") {
            Some(SportFileFormat::Gpx)
        } else if text.starts_with("<trainingcenterdatabase") {
            Some(SportFileFormat::Tcx)
        } else {
            None
        }
    }
}

/// Skips whitespace, BOM and an optional XML declaration/DOCTYPE so
/// detection can inspect the root element directly.
fn strip_xml_preamble(text: &str) -> &str {
    let mut rest = text;
    loop {
        rest = rest.trim_start_matches(['\u{feff}', ' ', '\t', '\r', '\n']);
        if rest.starts_with("<?") || rest.starts_with("<!") {
            match rest.split_once('>') {
                Some((_, after)) => rest = after,
                None => return rest,
            }
        } else {
            return rest;
        }
    }
}

/// The FIT header starts with header size 0x0E, protocol version 0x10 and
/// the literal `.FIT` magic at offset 8.
fn is_fit(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes[0] == 0x0E && bytes[1] == 0x10 && &bytes[8..12] == b".FIT"
}

/// Lowercases and trims BOM/whitespace from the leading bytes so XML
/// declarations do not interfere with root-element detection.
fn leading_text(bytes: &[u8]) -> String {
    let len = bytes.len().min(4096);
    let text = String::from_utf8_lossy(&bytes[..len]).to_lowercase();
    text.trim_start_matches(['\u{feff}', ' ', '\t', '\r', '\n'])
        .to_string()
}

/// Metadata describing a sport file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    /// The identified format.
    pub format: SportFileFormat,
    /// Content length, when known.
    pub content_length: ContentLength,
    /// Media type, when known.
    pub media_type: Option<MediaType>,
    /// File name, when available.
    pub file_name: Option<String>,
}

impl FileMetadata {
    /// Creates metadata for a format.
    pub fn new(format: SportFileFormat) -> Self {
        Self {
            format,
            content_length: ContentLength::unknown(),
            media_type: None,
            file_name: None,
        }
    }

    /// Returns the media type, falling back to the format's conventional
    /// media type when none was announced.
    pub fn effective_media_type(&self) -> MediaType {
        self.media_type
            .clone()
            .unwrap_or_else(|| MediaType::new(self.format.media_type()))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxisport_runtime::{ContentLength, MediaType};

    use super::{FileMetadata, SportFileFormat};

    #[test]
    fn identifies_fit_from_magic_bytes() {
        let mut fit = vec![0x0E, 0x10, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00];
        fit.extend_from_slice(b".FIT");
        assert_eq!(
            SportFileFormat::from_bytes(&fit),
            Some(SportFileFormat::Fit)
        );
    }

    #[test]
    fn identifies_gpx_from_xml_root() {
        let gpx = br#"<?xml version="1.0"?>
        <gpx version="1.1" creator="oxisport"></gpx>"#;
        assert_eq!(SportFileFormat::from_bytes(gpx), Some(SportFileFormat::Gpx));
    }

    #[test]
    fn identifies_tcx_from_xml_root() {
        let tcx = br#"<?xml version="1.0" encoding="UTF-8"?>
        <TrainingCenterDatabase xmlns="http://www.garmin.com/xmlschemas/TrainingCenterDatabase/v2"></TrainingCenterDatabase>"#;
        assert_eq!(SportFileFormat::from_bytes(tcx), Some(SportFileFormat::Tcx));
    }

    #[test]
    fn handles_bom_and_leading_whitespace() {
        let mut gpx = vec![0xEF, 0xBB, 0xBF, b' ', b' ', b' '];
        gpx.extend_from_slice(b"<gpx></gpx>");
        assert_eq!(
            SportFileFormat::from_bytes(&gpx),
            Some(SportFileFormat::Gpx)
        );
    }

    #[test]
    fn rejects_unknown_content() {
        assert_eq!(SportFileFormat::from_bytes(b"hello"), None);
        assert_eq!(SportFileFormat::from_bytes(&[]), None);
    }

    #[test]
    fn identifies_from_extensions() {
        assert_eq!(
            SportFileFormat::from_extension(Path::new("ride.gpx")),
            Some(SportFileFormat::Gpx)
        );
        assert_eq!(
            SportFileFormat::from_extension(Path::new("RIDE.FIT")),
            Some(SportFileFormat::Fit)
        );
        assert_eq!(
            SportFileFormat::from_extension(Path::new("workout.tcx")),
            Some(SportFileFormat::Tcx)
        );
        assert_eq!(
            SportFileFormat::from_extension(Path::new("notes.txt")),
            None
        );
        assert_eq!(SportFileFormat::from_extension(Path::new("noext")), None);
    }

    #[test]
    fn format_conventions() {
        assert_eq!(SportFileFormat::Gpx.extension(), "gpx");
        assert_eq!(SportFileFormat::Gpx.media_type(), "application/gpx+xml");
        assert_eq!(SportFileFormat::Fit.media_type(), "application/fit");
        assert_eq!(
            SportFileFormat::Tcx.media_type(),
            "application/vnd.garmin.tcx+xml"
        );
    }

    #[test]
    fn metadata_falls_back_to_conventional_media_type() {
        let mut metadata = FileMetadata::new(SportFileFormat::Gpx);
        assert_eq!(metadata.media_type, None);
        assert_eq!(
            metadata.effective_media_type(),
            MediaType::new("application/gpx+xml")
        );

        metadata.content_length = ContentLength::new(1024);
        metadata.media_type = Some(MediaType::new("text/xml"));
        assert_eq!(metadata.content_length, ContentLength::new(1024));
        assert_eq!(metadata.effective_media_type(), MediaType::new("text/xml"));
    }
}
