use crate::types::ExifError;

use std::fmt::{self, Display};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FileType {
    Unknown,
    JPEG,
    TIFF,
}

impl Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "application/octet-stream",
            Self::JPEG => "image/jpeg",
            Self::TIFF => "image/tiff",
        }
    }
}

/// Detect the type of an image contained in a byte buffer
pub(crate) fn detect_type(contents: &[u8]) -> FileType {
    if contents.len() < 11 {
        return FileType::Unknown;
    }

    if contents[0..3] == [0xff, 0xd8, 0xff] && &contents[6..11] == b"JFIF\0" {
        return FileType::JPEG;
    }
    if contents[0..3] == [0xff, 0xd8, 0xff] && &contents[6..11] == b"Exif\0" {
        return FileType::JPEG;
    }
    if contents.starts_with(&[b'I', b'I', 42, 0]) {
        /* TIFF little-endian */
        return FileType::TIFF;
    }
    if contents.starts_with(&[b'M', b'M', 0, 42]) {
        /* TIFF big-endian */
        return FileType::TIFF;
    }
    FileType::Unknown
}

/// Find the embedded TIFF in a JPEG image (that in turn contains the EXIF data)
pub fn find_embedded_tiff_in_jpeg(contents: &[u8]) -> Result<(usize, usize), ExifError> {
    let mut offset = 2_usize;

    while offset < contents.len() {
        if contents.len() < (offset + 4) {
            return Err(ExifError::JpegWithoutExif("JPEG truncated in marker header".into()));
        }

        let marker: u16 = u16::from(contents[offset]) * 256 + u16::from(contents[offset + 1]);

        if marker < 0xff00 {
            return Err(ExifError::JpegWithoutExif(format!("Invalid marker {marker:x}")));
        }

        offset += 2;
        let size = (contents[offset] as usize) * 256 + (contents[offset + 1] as usize);

        if size < 2 {
            return Err(ExifError::JpegWithoutExif("JPEG marker size must be at least 2 (because of the size word)".into()));
        }
        if contents.len() < (offset + size) {
            return Err(ExifError::JpegWithoutExif("JPEG truncated in marker body".into()));
        }

        if marker == 0xffe1 {
            if size < 8 {
                return Err(ExifError::JpegWithoutExif("EXIF preamble truncated".into()));
            }

            if contents[offset + 2..offset + 8] != [b'E', b'x', b'i', b'f', 0, 0] {
                return Err(ExifError::JpegWithoutExif("EXIF preamble unrecognized".into()));
            }

            // The offset and size of the block, excluding size and 'Exif\0\0'.
            return Ok((offset + 8, size - 8));
        }
        if marker == 0xffda {
            // last marker
            return Err(ExifError::JpegWithoutExif("Last mark found and no EXIF".into()));
        }
        offset += size;
    }

    Err(ExifError::JpegWithoutExif("Scan past EOF and no EXIF found".into()))
}
