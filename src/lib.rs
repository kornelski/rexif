#![allow(clippy::upper_case_acronyms)]

//! RExif is a native Rust create, written to extract EXIF data from JPEG and TIFF images.
//!
//! Note that it is in very early stages of development. Any sort of feedback is welcome!
//!
//! The crate contains a
//! sample binary called 'rexiftool' that accepts files as arguments and prints the EXIF data. It gives
//! a rough idea on how to use the crate. Get some sample images and run
//!
//!
//! `cargo run [image file 1] [image file 2] ...`
//!
//!
//! To learn to use this crate, start by the documentation of function `parse_file()`,
//! and the struct `ExifData` that is returned by the parser. The rest falls more or less into place.
//!
//! Code sample lightly edited from src/bin.rs:
//!
//! ```
//! use std::error::Error;
//!
//! let file_name = "foo.jpg";
//! match rexif::parse_file(&file_name) {
//!     Ok(exif) => {
//!         println!("{} {} exif entries: {}", file_name, exif.mime, exif.entries.len());
//!         for entry in &exif.entries {
//!             println!("\t{}: {}", entry.tag, entry.value_more_readable);
//!         }
//!     },
//!     Err(e) => {
//!         eprintln!("Error in {}: {}", &file_name, e)
//!     },
//! }
//! ```

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

mod lowlevel;
mod rational;
pub use self::rational::*;
mod types;
pub use self::types::*;
mod types_impl;
pub use self::types_impl::*;
mod image;
use self::image::*;
mod ifdformat;
mod tiff;
use self::tiff::parse_tiff;
mod exif;
mod exifpost;
mod exifreadable;

/// Parse a byte buffer that should contain a TIFF or JPEG image.
/// Tries to detect format and parse EXIF data.
///
/// Prints warnings to stderr.
pub fn parse_buffer(contents: &[u8]) -> ExifResult {
    let (res, warnings) = parse_buffer_quiet(contents);
    warnings.into_iter().for_each(|w| eprintln!("{w}"));
    res
}

/// Parse a byte buffer that should contain a TIFF or JPEG image.
/// Tries to detect format and parse EXIF data.
///
/// Returns warnings alongside result.
pub fn parse_buffer_quiet(contents: &[u8]) -> (ExifResult, Vec<String>) {
    let mime = detect_type(contents);
    let mut warnings = vec![];
    let (entries, le) = match mime {
        FileType::Unknown => return (Err(ExifError::FileTypeUnknown), warnings),
        FileType::TIFF => parse_tiff(contents, &mut warnings),
        FileType::JPEG => {
            match find_embedded_tiff_in_jpeg(contents)
                .map(|(offset, size)| parse_tiff(&contents[offset..offset + size], &mut warnings))
            {
                Ok(r) => r,
                Err(e) => return (Err(e), warnings),
            }
        }
    };

    (entries.map(|entries| ExifData { mime: mime.as_str(), entries, le }), warnings)
}

/// Try to read and parse an open file that is expected to contain an image
pub fn read_file(f: &mut File) -> ExifResult {
    f.seek(SeekFrom::Start(0))?;

    // TODO: should read only the relevant parts of a file,
    // and pass a StringIO-like object instead of a Vec buffer

    let mut contents: Vec<u8> = Vec::new();
    f.read_to_end(&mut contents)?;
    parse_buffer(&contents)
}

/// Opens an image (passed as a file name), tries to read and parse it.
pub fn parse_file<P: AsRef<Path>>(fname: P) -> ExifResult {
    read_file(&mut File::open(fname)?)
}
