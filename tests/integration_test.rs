use glob::glob;

use std::path::Path;

use rexif::*;

const JPEG_TEST_DIR: &str = "./tests/img/jpg";
const JPEG_PATTERN: &str = "**/*.jpg";
const TIFF_TEST_DIR: &str = "./tests/img/tiff";
const TIFF_PATTERN: &str = "*.tiff";
const APP_MARKER: &[u8] = &[0xff, 0xd8, 0xff, 0xe1];

#[cfg(test)]
fn check_tags(entries: &[ExifEntry], expected_tags: Vec<ExifTag>) {
    let tags: Vec<ExifTag> = entries.iter().map(|entry| entry.tag).collect();
    assert_eq!(tags, expected_tags);
}

#[test]
fn test_parse_simple_morotola_jpeg() {
    let exif = rexif::parse_file("./tests/img/profile.jpg");
    assert!(exif.is_ok(), "{exif:?}");

    let exif = exif.unwrap();
    assert_eq!(exif.mime, "image/jpeg");
    assert_eq!(exif.entries.len(), 7);

    let expected_tags = vec![
        ExifTag::Orientation,
        ExifTag::XResolution,
        ExifTag::YResolution,
        ExifTag::ResolutionUnit,
        ExifTag::ExifOffset,
        ExifTag::UnknownToMe,
        ExifTag::UnknownToMe,
    ];
    check_tags(&exif.entries, expected_tags);

    assert!(exif.entries.iter().all(|e| e.namespace == Namespace::Standard),
            "Expected all tags to be from the standard namespace");
}

#[test]
fn test_parse_jpeg_without_metadata() {
    let exif = rexif::parse_file("./tests/img/invalid/no_exif.jpg");
    if matches!(exif, Err(ExifError::FileTypeUnknown)) {
        // succeed
    } else {
        panic!("Expected ExifError::FileTypeUnknown, found {exif:?}")
    }
}

#[test]
fn test_parse_jpeg_with_gps() -> Result<(), std::io::Error> {
    let exif = rexif::parse_file("./tests/img/jpg/gps/DSCN0029.jpg");
    assert!(exif.is_ok(), "{exif:?}");

    let exif = exif.unwrap();
    assert_eq!(exif.mime, "image/jpeg");

    let expected_tags = vec![
        ExifTag::GPSOffset,
        ExifTag::GPSLatitudeRef,
        ExifTag::GPSLatitude,
        ExifTag::GPSLongitudeRef,
        ExifTag::GPSLongitude,
        ExifTag::GPSAltitudeRef,
        ExifTag::GPSTimeStamp,
        ExifTag::GPSSatellites,
        ExifTag::GPSImgDirectionRef,
        ExifTag::GPSMapDatum,
        ExifTag::GPSDateStamp,
    ];

    for t in expected_tags {
        assert!(exif.entries.iter().any(|e| e.tag == t), "Could not find exif tag: {t:?}");
    }

    Ok(())
}

#[cfg(test)]
fn cmp_serialized_exif_with_original<P: AsRef<Path>>(file: P) -> Result<(), std::io::Error> {
    let parsed_exif1 = parse_file(&file).unwrap();

    let serialized_exif1 = parsed_exif1.serialize().unwrap();
    let serialized_exif1 = if parsed_exif1.mime == "image/jpeg" {
        let size = (serialized_exif1.len() as u16 + 2).to_be_bytes();
        [APP_MARKER, &size, &serialized_exif1].concat()
    } else {
        parsed_exif1.serialize().unwrap()
    };

    let parsed_exif2 = parse_buffer(&serialized_exif1).unwrap();

    let serialized_exif2 = parsed_exif2.serialize().unwrap();
    let serialized_exif2 = if parsed_exif2.mime == "image/jpeg" {
        let size = (serialized_exif2.len() as u16 + 2).to_be_bytes();
        [APP_MARKER, &size, &serialized_exif2].concat()
    } else {
        serialized_exif2
    };

    assert_eq!(serialized_exif1, serialized_exif2);
    assert_eq!(parsed_exif1, parsed_exif2);

    Ok(())
}

#[test]
fn test_jpeg_exif_serialization() -> Result<(), std::io::Error> {
    let jpegs = glob(
        Path::new(JPEG_TEST_DIR)
            .join(JPEG_PATTERN)
            .to_str()
            .expect("Path is not valid unicode."),
    )
    .expect("Failed to read glob pattern")
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    for jpeg in jpegs {
        cmp_serialized_exif_with_original(&jpeg)?;
    }

    Ok(())
}

#[test]
fn test_tiff_exif_serialization() -> Result<(), std::io::Error> {
    let tiffs = glob(
        Path::new(TIFF_TEST_DIR)
            .join(TIFF_PATTERN)
            .to_str()
            .expect("Path is not valid unicode."),
    )
    .expect("Failed to read glob pattern")
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    for tiff in tiffs {
        cmp_serialized_exif_with_original(&tiff)?;
    }

    Ok(())
}

#[test]
fn test_serialize_empty() {
    let exif = ExifData::new("image/jpeg", vec![], false);
    let tiff_header = [b'M', b'M', 0, 42, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0];
    assert_eq!(exif.serialize().unwrap(), [EXIF_HEADER, &tiff_header].concat());
}
