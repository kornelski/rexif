use super::exif::*;
use super::exifpost::*;
use super::ifdformat::*;
use super::lowlevel::*;
use super::types::*;
use std::borrow::Cow;

type InExifResult = Result<(), ExifError>;

/// Parse of raw IFD entry into EXIF data, if it is of a known type, and returns
/// an `ExifEntry` object. If the tag is unknown, the enumeration is set to `UnknownToMe`,
/// but the raw information of tag is still available in the ifd member.
pub(crate) fn parse_exif_entry(ifd: IfdEntry, warnings: &mut Vec<String>, kind: IfdKind) -> ExifEntry {
    let (tag, unit, format, min_count, max_count, more_readable) = tag_to_exif(ifd.tag);
    let value = match tag_value_new(&ifd) {
        Some(v) => v,
        None => TagValue::Invalid(ifd.data.clone(), ifd.le, ifd.format as u16, ifd.count),
    };

    let e = ExifEntry {
        value_more_readable: more_readable(ifd.tag, &value).unwrap_or(Cow::Borrowed("")),
        namespace: ifd.namespace,
        ifd,
        tag,
        unit: unit.into(),
        value,
        kind,
    };

    if tag == ExifTag::UnknownToMe {
        // Unknown EXIF tag type
        return e;
    }

    // Internal assert:
    // 1) tag must match enum
    // 2) all types except Ascii, Undefined, Unknown must have definite length
    // 3) Str type must not have a definite length
    if (((tag as u32) & 0xffff) as u16) != e.ifd.tag
        || (min_count == -1
            && (format != IfdFormat::Ascii
                && format != IfdFormat::Undefined
                && format != IfdFormat::Unknown))
        || (min_count != -1 && format == IfdFormat::Ascii)
    {
        panic!("Internal error {:x}", e.ifd.tag);
    }

    if format != e.ifd.format {
        warnings.push(format!(
            "EXIF tag {:x} {} ({}), expected format {} ({:?}), found {} ({:?})",
            e.ifd.tag, e.ifd.tag, tag, format as u8, format, e.ifd.format as u8, e.ifd.format
        ));
    }

    if min_count != -1 && ((e.ifd.count as i32) < min_count || (e.ifd.count as i32) > max_count) {
        warnings.push(format!(
            "EXIF tag {:x} {} ({:?}), format {}, expected count {}..{} found {}",
            e.ifd.tag, e.ifd.tag, tag, format as u8, min_count, max_count, e.ifd.count
        ));
    }
    e
}

/// Superficial parse of IFD that can't fail
pub fn parse_ifd(
    subifd: bool,
    le: bool,
    count: u16,
    contents: &[u8]
) -> Option<(Vec<IfdEntry>, usize)> {
    let mut entries: Vec<IfdEntry> = Vec::new();

    for i in 0..count {
        let mut offset = (i as usize) * 12;
        let tag = read_u16(le, contents.get(offset..)?)?;
        offset += 2;
        let format = read_u16(le, contents.get(offset..)?)?;
        offset += 2;
        let count = read_u32(le, contents.get(offset..)?)?;
        offset += 4;
        let data = contents.get(offset..offset + 4)?.to_vec();

        let entry = IfdEntry {
            namespace: Namespace::Standard,
            tag,
            format: IfdFormat::new(format),
            count,
            ifd_data: data,
            le,
            ext_data: Vec::new(),
            data: Vec::new(),
        };
        entries.push(entry);
    }

    let next_ifd = if subifd {
        0
    } else {
        read_u32(le, &contents[count as usize * 12..])? as usize
    };

    Some((entries, next_ifd))
}

/// Deep parse of IFD that grabs EXIF data from IFD0, `SubIFD` and GPS IFD
fn parse_exif_ifd(
    le: bool,
    contents: &[u8],
    ioffset: usize,
    exif_entries: &mut Vec<ExifEntry>,
    warnings: &mut Vec<String>,
    kind: IfdKind,
) -> InExifResult {
    let mut offset = ioffset;

    if contents.len() < (offset + 2) {
        return Err(ExifError::ExifIfdTruncated(
            format!("Truncated {:?} at dir entry count ({} < {})", kind, contents.len(), (offset + 2)),
        ));
    }

    let count = read_u16(
        le,
        contents
            .get(offset..)
            .ok_or(ExifError::IfdTruncated)?,
    ).ok_or(ExifError::IfdTruncated)?;
    let ifd_length = (count as usize) * 12;
    offset += 2;

    if contents.len() < (offset + ifd_length) {
        return Err(ExifError::ExifIfdTruncated(
            "Truncated at dir listing".to_string(),
        ));
    }

    let ifd_content = &contents
        .get(offset..offset + ifd_length)
        .ok_or(ExifError::IfdTruncated)?;
    let (ifd, _) = parse_ifd(true, le, count, ifd_content).ok_or(ExifError::IfdTruncated)?;

    for mut entry in ifd {
        if !entry.copy_data(contents) {
            // data is probably beyond EOF
            continue;
        }
        let exif_entry = parse_exif_entry(entry, warnings, kind);
        exif_entries.push(exif_entry);
    }

    Ok(())
}

/// Parses IFD0 and looks for `SubIFD` or GPS IFD within IFD0
pub fn parse_ifds(
    le: bool,
    ifd0_offset: usize,
    contents: &[u8],
    warnings: &mut Vec<String>,
) -> ExifEntryResult {
    let mut offset = ifd0_offset;
    let mut exif_entries: Vec<ExifEntry> = Vec::new();

    // fills exif_entries with data from IFD0

    match parse_exif_ifd(le, contents, offset, &mut exif_entries, warnings, IfdKind::Ifd0) {
        Ok(()) => true,
        Err(e) => return Err(e),
    };

    // at this point we knot that IFD0 is good
    // looks for SubIFD (EXIF)

    let count = read_u16(
        le,
        contents
            .get(offset..offset + 2)
            .ok_or(ExifError::IfdTruncated)?,
    ).ok_or(ExifError::IfdTruncated)?;
    let ifd_length = (count as usize) * 12 + 4;
    offset += 2;

    let ifd_content = &contents
        .get(offset..offset + ifd_length)
        .ok_or(ExifError::IfdTruncated)?;
    let (ifd, _) = parse_ifd(false, le, count, ifd_content).ok_or(ExifError::IfdTruncated)?;

    for entry in &ifd {
        // Identify which IFD this entry belongs to (IFD-0, Exif, Gps, IFD-1 etc)
        let ifd_kind = if entry.tag == (((ExifTag::ExifOffset as u32) & 0xffff) as u16) {
            IfdKind::Exif
        } else if entry.tag == (((ExifTag::GPSOffset as u32) & 0xffff) as u16) {
            // Gps
            IfdKind::Gps
        } else {
            continue;
        };

        let exif_offset = entry.try_data_as_offset().unwrap_or(!0);
        if contents.len() < exif_offset {
            return Err(ExifError::ExifIfdTruncated(
                "Exif SubIFD goes past EOF".to_string(),
            ));
        }
        parse_exif_ifd(le, contents, exif_offset, &mut exif_entries, warnings, ifd_kind)?;
    }

    for n in 0..exif_entries.len() {
        let (begin, end) = exif_entries.split_at_mut(n);
        let (entry, end) = end.split_first_mut().unwrap();
        exif_postprocessing(entry, begin, end);
    }

    Ok(exif_entries)
}

/// Parse a TIFF image, or embedded TIFF in JPEG, in order to get IFDs and then the EXIF data
pub fn parse_tiff(contents: &[u8], warnings: &mut Vec<String>) -> (ExifEntryResult, bool) {
    let mut le = false;

    if contents.len() < 8 {
        return (Err(ExifError::TiffTruncated), false);
    } else if contents[0] == b'I' && contents[1] == b'I' && contents[2] == 42 && contents[3] == 0 {
        /* TIFF little-endian */
        le = true;
    } else if contents[0] == b'M' && contents[1] == b'M' && contents[2] == 0 && contents[3] == 42 {
        /* TIFF big-endian */
    } else {
        let err = format!(
            "Preamble is {:x} {:x} {:x} {:x}",
            contents[0], contents[1], contents[2], contents[3]
        );
        return (Err(ExifError::TiffBadPreamble(err)), false);
    }

    let offset = read_u32(le, &contents[4..]).unwrap() as usize;

    (parse_ifds(le, offset, contents, warnings), le)
}
