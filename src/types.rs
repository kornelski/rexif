use super::ifdformat::tag_value_eq;
use super::rational::{IRational, URational};
use std::borrow::Cow;
use std::{fmt, io};

/// The value of the Exif header.
pub const EXIF_HEADER: &[u8] = &[b'E', b'x', b'i', b'f', 0x00, 0x00];
const INTEL_TIFF_HEADER: &[u8] = &[b'I', b'I', 0x2a, 0x00];
const MOTOROLA_TIFF_HEADER: &[u8] = &[b'M', b'M', 0x00, 0x2a];
const DATA_WIDTH: usize = 4;

/// Top-level structure that contains all parsed metadata inside an image
#[derive(Debug, PartialEq)]
pub struct ExifData {
    /// MIME type of the parsed image. It may be "image/jpeg", "image/tiff", or empty if unrecognized.
    pub mime: &'static str,
    /// Collection of EXIF entries found in the image
    pub entries: Vec<ExifEntry>,
    /// If `true`, this uses little-endian byte ordering for the raw bytes. Otherwise, it uses big-endian ordering.
    pub le: bool,
}

impl ExifData {
    #[must_use]
    pub fn new(mime: &'static str, entries: Vec<ExifEntry>, le: bool) -> Self {
        Self { mime, entries, le }
    }
}

impl ExifData {
    /// Serialize the metadata entries, and return the result.
    ///
    /// *Note*: this serializes the metadata according to its original endianness (specified
    /// through the `le` attribute).
    pub fn serialize(&self) -> Result<Vec<u8>, ExifError> {
        // Select the right TIFF header based on the endianness.
        let tiff_header = if self.le {
            INTEL_TIFF_HEADER
        } else {
            MOTOROLA_TIFF_HEADER
        };

        // The result buffer.
        let mut serialized = vec![];

        // Generate the TIFF header.
        serialized.extend(tiff_header);

        // The offset to IFD-0. IFD-0 follows immediately after the TIFF header.
        // The offset is a 4-byte value - serialize it to bytes:
        let offset = if self.le {
            (tiff_header.len() as u32 + std::mem::size_of::<u32>() as u32).to_le_bytes()
        } else {
            (tiff_header.len() as u32 + std::mem::size_of::<u32>() as u32).to_be_bytes()
        };
        serialized.extend(&offset);

        let mut ifd0 = vec![];
        let mut ifd1 = vec![];
        let mut exif = vec![];
        let mut gps = vec![];

        for e in &self.entries {
            match e.kind {
                IfdKind::Ifd0 => ifd0.push(e),
                IfdKind::Ifd1 => ifd1.push(e),
                IfdKind::Exif => exif.push(e),
                IfdKind::Gps => gps.push(e),
                _ => {
                    // XXX Silently ignore Makernote and Interoperability IFDs
                },
            }
        }

        // IFD-1 contains the thumbnail. For now, the parser discards IFD-1, so its serialization
        // has not yet been implemented.
        assert!(ifd1.is_empty());

        // Serialize the number of directory entries in this IFD.
        serialized.extend(&if self.le {
            (ifd0.len() as u16).to_le_bytes()
        } else {
            (ifd0.len() as u16).to_be_bytes()
        });

        // The position of the data in an Exif Offset entry.
        let mut exif_ifd_pointer = None;

        // The position of the data in an GPS Offset entry.
        let mut gps_ifd_pointer = None;

        // The positions which contain offsets pointing to values in the data section of IFD-0.
        // These offsets will be filled out (patched) later.
        let mut data_patches = vec![];
        for entry in ifd0 {
            entry.ifd.serialize(&mut serialized, &mut data_patches)?;

            // If IFD-0 points to an Exif/GPS sub-IFD, the offset of the sub-IFD must be serialized
            // inside IFD-0. Subtract `DATA_WIDTH` from the length, because the pointer to the
            // sub-IFD will be written in the data section of the previously serialized entry
            // (which is of type ExifOffset/GPSOffset precisely because its data section contains
            // an offset to a sub-IFD).
            if entry.tag == ExifTag::ExifOffset {
                exif_ifd_pointer = Some(serialized.len() - DATA_WIDTH);
            }
            if entry.tag == ExifTag::GPSOffset {
                gps_ifd_pointer = Some(serialized.len() - DATA_WIDTH);
            }
        }

        if ifd1.is_empty() {
            serialized.extend(&[0, 0, 0, 0]);
        } else {
            // Otherwise, serialize the pointer to IFD-1 (which is just the offset of IFD-1 in the
            // file).
            unimplemented!("IFD-1");
        }

        // Patch the offsets serialized above.
        for patch in &data_patches {
            // The position of the data pointed to by the IFD entries serialized above.
            let bytes = if self.le {
                (serialized.len() as u32).to_le_bytes()
            } else {
                (serialized.len() as u32).to_be_bytes()
            };

            serialized.extend(patch.data);
            for (place, byte) in serialized.iter_mut().skip(patch.offset_pos as usize).zip(bytes.iter()) {
                *place = *byte;
            }
        }

        if !exif.is_empty() {
            self.serialize_ifd(&mut serialized, exif, exif_ifd_pointer)?;
        }

        if !gps.is_empty() {
            self.serialize_ifd(&mut serialized, gps, gps_ifd_pointer)?;
        }

        // TODO Makernote, Interoperability IFD, Thumbnail image

        Ok(if self.mime == "image/jpeg" {
            [EXIF_HEADER, &serialized].concat()
        } else {
            serialized
        })
    }

    /// Serialize GPS/Exif IFD entries.
    fn serialize_ifd(
        &self,
        serialized: &mut Vec<u8>,
        entries: Vec<&ExifEntry>,
        pos: Option<usize>,
    ) -> Result<(), ExifError> {
        let bytes = if self.le {
            (serialized.len() as u32).to_le_bytes()
        } else {
            (serialized.len() as u32).to_be_bytes()
        };

        // Serialize the number of directory entries in this IFD
        serialized.extend(&if self.le {
            (entries.len() as u16).to_le_bytes()
        } else {
            (entries.len() as u16).to_be_bytes()
        });

        // Write the offset of this IFD in IFD-0.
        let pos = pos.ok_or(ExifError::MissingExifOffset)?;
        for (place, byte) in serialized.iter_mut().skip(pos).zip(bytes.iter()) {
            *place = *byte;
        }

        let mut data_patches = vec![];

        for entry in entries {
            entry.ifd.serialize(serialized, &mut data_patches)?;
        }

        serialized.extend(&[0, 0, 0, 0]);
        for patch in &data_patches {
            // The position of the data pointed to by the IFD entries serialized above.
            let bytes = if self.le {
                (serialized.len() as u32).to_le_bytes()
            } else {
                (serialized.len() as u32).to_be_bytes()
            };
            serialized.extend(patch.data);
            for (place, byte) in serialized.iter_mut().skip(patch.offset_pos as usize).zip(bytes.iter()) {
                *place = *byte;
            }
        }
        Ok(())
    }
}

pub(super) struct Patch<'a> {
    /// The position where to write the offset in the file where the data will be located.
    offset_pos: u32,
    /// The data to add to the data section of the current IFD.
    data: &'a [u8],
}

impl Patch<'_> {
    #[must_use]
    pub const fn new(offset_pos: u32, data: &[u8]) -> Patch {
        Patch { offset_pos, data }
    }
}

/// Possible fatal errors that may happen when an image is parsed.
#[derive(Debug)]
pub enum ExifError {
    IoError(io::Error),
    FileTypeUnknown,
    JpegWithoutExif(String),
    TiffTruncated,
    TiffBadPreamble(String),
    IfdTruncated,
    ExifIfdTruncated(String),
    ExifIfdEntryNotFound,
    UnsupportedNamespace,
    MissingExifOffset,
}

/// Structure that represents a parsed IFD entry of a TIFF image
#[derive(Clone, Debug)]
pub struct IfdEntry {
    /// Namespace of the entry. Standard is a tag found in normal TIFF IFD structure,
    /// other namespaces are entries found e.g. within `MarkerNote` blobs that are
    /// manufacturer-specific.
    pub namespace: Namespace,
    /// IFD tag value, may or not be an EXIF tag
    pub tag: u16,
    /// IFD data format
    pub format: IfdFormat,
    /// Number of items, each one in the data format specified by format
    pub count: u32,
    /// Raw data as a vector of bytes. Length is sizeof(format) * count.
    /// Depending on its size, it came from different parts of the image file.
    pub data: Vec<u8>,
    /// Raw data contained within the IFD structure. If count * sizeof(format) >= 4,
    /// this item contains the offset where the actual data can be found
    pub ifd_data: Vec<u8>,
    /// Raw data contained outside of the IFD structure and pointed by `ifd_data`,
    /// if data would not fit within the IFD structure
    pub ext_data: Vec<u8>,
    /// If true, integer and offset formats must be parsed from raw data as little-endian.
    /// If false, integer and offset formats must be parsed from raw data as big-endian.
    ///
    /// It is important to have 'endianess' per IFD entry, because some manufacturer-specific
    /// entries may have fixed endianess (regardeless of TIFF container's general endianess).
    pub le: bool,
}

// Do not include `ifd_data` in the comparison, as it may in fact contain the offset to the data,
// and two Exif entries may contain the same data, but at different offsets. In that case, the
// entries should still be considered equal.
impl PartialEq for IfdEntry {
    fn eq(&self, other: &Self) -> bool {
        let data_eq = if self.in_ifd() && !self.tag == ExifTag::ExifOffset as u16 && !self.tag == ExifTag::GPSOffset as u16 {
            self.data == other.data && self.ifd_data == other.ifd_data && self.ext_data == other.ext_data
        } else {
            true
        };

        self.namespace == other.namespace && self.tag == other.tag && self.count == other.count &&
            data_eq && self.le == other.le
    }
}

impl IfdEntry {
    pub(crate) fn serialize<'a>(
        &'a self,
        serialized: &mut Vec<u8>,
        data_patches: &mut Vec<Patch<'a>>,
    ) -> Result<(), ExifError> {
        // Serialize the entry
        if self.namespace != Namespace::Standard {
            return Err(ExifError::UnsupportedNamespace);
        }

        // Serialize the tag (2 bytes)
        serialized.extend(&if self.le {
            self.tag.to_le_bytes()
        } else {
            self.tag.to_be_bytes()
        });

        // Serialize the data format (2 bytes)
        serialized.extend(&if self.le {
            (self.format as u16).to_le_bytes()
        } else {
            (self.format as u16).to_be_bytes()
        });

        // Serialize the number of components (4 bytes)
        serialized.extend(&if self.le {
            self.count.to_le_bytes()
        } else {
            self.count.to_be_bytes()
        });

        // Serialize the data value/offset to data value (4 bytes)
        if self.in_ifd() {
            serialized.extend(&self.data);
        } else {
            data_patches.push(Patch::new(serialized.len() as u32, &self.data));
            // 4 bytes that will be filled out later
            serialized.extend(&[0, 0, 0, 0]);
        }
        Ok(())
    }
}

/// Enumeration that represent EXIF tag namespaces. Namespaces exist to
/// accomodate future parsing of the manufacturer-specific tags embedded within
/// the `MarkerNote` tag.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Namespace {
    Standard = 0x0000,
    Nikon = 0x0001,
    Canon = 0x0002,
}

/// Enumeration that represents recognized EXIF tags found in TIFF IFDs.
///
/// Items can be cast to u32 in order to get the namespace (most significant word)
/// and tag code (least significant word). The tag code matches the Exif, or the
/// Makernote standard, depending on the namespace that the tag belongs to.
///
/// On the other hand, the namespace code is arbitrary, it only matches
/// the `Namespace` enumeration. The namespace is 0 for standard Exif tags.
/// The non-standard namespaces exist to accomodate future parsing of the
/// `MarkerNote` tag, that contains embedded manufacturer-specific tags.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
#[repr(u32)]
pub enum ExifTag {
    /// Tag not recognized are partially parsed. The client may still try to interpret
    /// the tag by reading into the `IfdFormat` structure.
    UnknownToMe = 0x0000_ffff,
    ImageDescription = 0x0000_010e,
    Make = 0x0000_010f,
    Model = 0x0000_0110,
    Orientation = 0x0000_0112,
    XResolution = 0x0000_011a,
    YResolution = 0x0000_011b,
    ResolutionUnit = 0x0000_0128,
    Software = 0x0000_0131,
    DateTime = 0x0000_0132,
    HostComputer = 0x0000_013c,
    WhitePoint = 0x0000_013e,
    PrimaryChromaticities = 0x0000_013f,
    YCbCrCoefficients = 0x0000_0211,
    ReferenceBlackWhite = 0x0000_0214,
    Copyright = 0x0000_8298,
    ExifOffset = 0x0000_8769,
    GPSOffset = 0x0000_8825,

    ExposureTime = 0x0000_829a,
    FNumber = 0x0000_829d,
    ExposureProgram = 0x0000_8822,
    SpectralSensitivity = 0x0000_8824,
    ISOSpeedRatings = 0x0000_8827,
    OECF = 0x0000_8828,
    SensitivityType = 0x0000_8830,
    ExifVersion = 0x0000_9000,
    DateTimeOriginal = 0x0000_9003,
    DateTimeDigitized = 0x0000_9004,
    ShutterSpeedValue = 0x0000_9201,
    ApertureValue = 0x0000_9202,
    BrightnessValue = 0x0000_9203,
    ExposureBiasValue = 0x0000_9204,
    MaxApertureValue = 0x0000_9205,
    SubjectDistance = 0x0000_9206,
    MeteringMode = 0x0000_9207,
    LightSource = 0x0000_9208,
    Flash = 0x0000_9209,
    FocalLength = 0x0000_920a,
    SubjectArea = 0x0000_9214,
    MakerNote = 0x0000_927c,
    UserComment = 0x0000_9286,
    FlashPixVersion = 0x0000_a000,
    ColorSpace = 0x0000_a001,
    RelatedSoundFile = 0x0000_a004,
    FlashEnergy = 0x0000_a20b,
    FocalPlaneXResolution = 0x0000_a20e,
    FocalPlaneYResolution = 0x0000_a20f,
    FocalPlaneResolutionUnit = 0x0000_a210,
    SubjectLocation = 0x0000_a214,
    ExposureIndex = 0x0000_a215,
    SensingMethod = 0x0000_a217,
    FileSource = 0x0000_a300,
    SceneType = 0x0000_a301,
    CFAPattern = 0x0000_a302,
    CustomRendered = 0x0000_a401,
    ExposureMode = 0x0000_a402,
    WhiteBalanceMode = 0x0000_a403,
    DigitalZoomRatio = 0x0000_a404,
    FocalLengthIn35mmFilm = 0x0000_a405,
    SceneCaptureType = 0x0000_a406,
    GainControl = 0x0000_a407,
    Contrast = 0x0000_a408,
    Saturation = 0x0000_a409,
    Sharpness = 0x0000_a40a,
    DeviceSettingDescription = 0x0000_a40b,
    SubjectDistanceRange = 0x0000_a40c,
    ImageUniqueID = 0x0000_a420,
    LensSpecification = 0x0000_a432,
    LensMake = 0x0000_a433,
    LensModel = 0x0000_a434,
    Gamma = 0xa500,

    GPSVersionID = 0x00000,
    GPSLatitudeRef = 0x00001,
    GPSLatitude = 0x00002,
    GPSLongitudeRef = 0x00003,
    GPSLongitude = 0x00004,
    GPSAltitudeRef = 0x00005,
    GPSAltitude = 0x00006,
    GPSTimeStamp = 0x00007,
    GPSSatellites = 0x00008,
    GPSStatus = 0x00009,
    GPSMeasureMode = 0x0000a,
    GPSDOP = 0x0000b,
    GPSSpeedRef = 0x0000c,
    GPSSpeed = 0x0000d,
    GPSTrackRef = 0x0000e,
    GPSTrack = 0x0000f,
    GPSImgDirectionRef = 0x0000_0010,
    GPSImgDirection = 0x0000_0011,
    GPSMapDatum = 0x0000_0012,
    GPSDestLatitudeRef = 0x0000_0013,
    GPSDestLatitude = 0x0000_0014,
    GPSDestLongitudeRef = 0x0000_0015,
    GPSDestLongitude = 0x0000_0016,
    GPSDestBearingRef = 0x0000_0017,
    GPSDestBearing = 0x0000_0018,
    GPSDestDistanceRef = 0x0000_0019,
    GPSDestDistance = 0x0000_001a,
    GPSProcessingMethod = 0x0000_001b,
    GPSAreaInformation = 0x0000_001c,
    GPSDateStamp = 0x0000_001d,
    GPSDifferential = 0x0000_001e,
}

impl Eq for ExifTag {}

impl fmt::Display for ExifTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ExifTag::ImageDescription => "Image Description",
                ExifTag::Make => "Manufacturer",
                ExifTag::HostComputer => "Host computer",
                ExifTag::Model => "Model",
                ExifTag::Orientation => "Orientation",
                ExifTag::XResolution => "X Resolution",
                ExifTag::YResolution => "Y Resolution",
                ExifTag::ResolutionUnit => "Resolution Unit",
                ExifTag::Software => "Software",
                ExifTag::DateTime => "Image date",
                ExifTag::WhitePoint => "White Point",
                ExifTag::PrimaryChromaticities => "Primary Chromaticities",
                ExifTag::YCbCrCoefficients => "YCbCr Coefficients",
                ExifTag::ReferenceBlackWhite => "Reference Black/White",
                ExifTag::Copyright => "Copyright",
                ExifTag::ExifOffset => "This image has an Exif SubIFD",
                ExifTag::GPSOffset => "This image has a GPS SubIFD",
                ExifTag::ExposureTime => "Exposure time",
                ExifTag::SensitivityType => "Sensitivity type",
                ExifTag::FNumber => "Aperture",
                ExifTag::ExposureProgram => "Exposure program",
                ExifTag::SpectralSensitivity => "Spectral sensitivity",
                ExifTag::ISOSpeedRatings => "ISO speed ratings",
                ExifTag::OECF => "OECF",
                ExifTag::ExifVersion => "Exif version",
                ExifTag::DateTimeOriginal => "Date of original image",
                ExifTag::DateTimeDigitized => "Date of image digitalization",
                ExifTag::ShutterSpeedValue => "Shutter speed",
                ExifTag::ApertureValue => "Aperture value",
                ExifTag::BrightnessValue => "Brightness value",
                ExifTag::ExposureBiasValue => "Exposure bias value",
                ExifTag::MaxApertureValue => "Maximum aperture value",
                ExifTag::SubjectDistance => "Subject distance",
                ExifTag::MeteringMode => "Meteting mode",
                ExifTag::LightSource => "Light source",
                ExifTag::Flash => "Flash",
                ExifTag::FocalLength => "Focal length",
                ExifTag::SubjectArea => "Subject area",
                ExifTag::MakerNote => "Maker note",
                ExifTag::UserComment => "User comment",
                ExifTag::FlashPixVersion => "Flashpix version",
                ExifTag::ColorSpace => "Color space",
                ExifTag::FlashEnergy => "Flash energy",
                ExifTag::RelatedSoundFile => "Related sound file",
                ExifTag::FocalPlaneXResolution => "Focal plane X resolution",
                ExifTag::FocalPlaneYResolution => "Focal plane Y resolution",
                ExifTag::FocalPlaneResolutionUnit => "Focal plane resolution unit",
                ExifTag::SubjectLocation => "Subject location",
                ExifTag::ExposureIndex => "Exposure index",
                ExifTag::SensingMethod => "Sensing method",
                ExifTag::FileSource => "File source",
                ExifTag::SceneType => "Scene type",
                ExifTag::CFAPattern => "CFA Pattern",
                ExifTag::CustomRendered => "Custom rendered",
                ExifTag::ExposureMode => "Exposure mode",
                ExifTag::WhiteBalanceMode => "White balance mode",
                ExifTag::DigitalZoomRatio => "Digital zoom ratio",
                ExifTag::FocalLengthIn35mmFilm => "Equivalent focal length in 35mm",
                ExifTag::SceneCaptureType => "Scene capture type",
                ExifTag::GainControl => "Gain control",
                ExifTag::Contrast => "Contrast",
                ExifTag::Saturation => "Saturation",
                ExifTag::Sharpness => "Sharpness",
                ExifTag::LensSpecification => "Lens specification",
                ExifTag::LensMake => "Lens manufacturer",
                ExifTag::LensModel => "Lens model",
                ExifTag::Gamma => "Gamma",
                ExifTag::DeviceSettingDescription => "Device setting description",
                ExifTag::SubjectDistanceRange => "Subject distance range",
                ExifTag::ImageUniqueID => "Image unique ID",
                ExifTag::GPSVersionID => "GPS version ID",
                ExifTag::GPSLatitudeRef => "GPS latitude ref",
                ExifTag::GPSLatitude => "GPS latitude",
                ExifTag::GPSLongitudeRef => "GPS longitude ref",
                ExifTag::GPSLongitude => "GPS longitude",
                ExifTag::GPSAltitudeRef => "GPS altitude ref",
                ExifTag::GPSAltitude => "GPS altitude",
                ExifTag::GPSTimeStamp => "GPS timestamp",
                ExifTag::GPSSatellites => "GPS satellites",
                ExifTag::GPSStatus => "GPS status",
                ExifTag::GPSMeasureMode => "GPS measure mode",
                ExifTag::GPSDOP => "GPS Data Degree of Precision (DOP)",
                ExifTag::GPSSpeedRef => "GPS speed ref",
                ExifTag::GPSSpeed => "GPS speed",
                ExifTag::GPSTrackRef => "GPS track ref",
                ExifTag::GPSTrack => "GPS track",
                ExifTag::GPSImgDirectionRef => "GPS image direction ref",
                ExifTag::GPSImgDirection => "GPS image direction",
                ExifTag::GPSMapDatum => "GPS map datum",
                ExifTag::GPSDestLatitudeRef => "GPS destination latitude ref",
                ExifTag::GPSDestLatitude => "GPS destination latitude",
                ExifTag::GPSDestLongitudeRef => "GPS destination longitude ref",
                ExifTag::GPSDestLongitude => "GPS destination longitude",
                ExifTag::GPSDestBearingRef => "GPS destination bearing ref",
                ExifTag::GPSDestBearing => "GPS destination bearing",
                ExifTag::GPSDestDistanceRef => "GPS destination distance ref",
                ExifTag::GPSDestDistance => "GPS destination distance",
                ExifTag::GPSProcessingMethod => "GPS processing method",
                ExifTag::GPSAreaInformation => "GPS area information",
                ExifTag::GPSDateStamp => "GPS date stamp",
                ExifTag::GPSDifferential => "GPS differential",
                ExifTag::UnknownToMe => "Unknown to this library, or manufacturer-specific",
            }
        )
    }
}

/// Enumeration that represents the possible data formats of an IFD entry.
///
/// Any enumeration item can be cast to u16 to get the low-level format code
/// as defined by the TIFF format.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IfdFormat {
    Unknown = 0,
    U8 = 1,
    Ascii = 2,
    U16 = 3,
    U32 = 4,
    URational = 5,
    I8 = 6,
    Undefined = 7, // u8
    I16 = 8,
    I32 = 9,
    IRational = 10,
    F32 = 11,
    F64 = 12,
}

/// Structure that represents a parsed EXIF tag.
#[derive(Clone, Debug)]
pub struct ExifEntry {
    /// Namespace of the tag. If Standard (0x0000), it is an EXIF tag defined in the
    /// official standard. Other namespaces accomodate manufacturer-specific tags that
    /// may be embedded in `MarkerNote` blob tag.
    pub namespace: Namespace,
    /// Low-level IFD entry that contains the EXIF tag. The client may look into this
    /// structure to get tag's raw data, or to parse the tag herself if `tag` is `UnknownToMe`.
    pub ifd: IfdEntry,
    /// EXIF tag type as an enumeration. If `UnknownToMe`, the crate did not know the
    /// tag in detail, and parsing will be incomplete. The client may read into
    /// `ifd` to discover more about the unparsed tag.
    pub tag: ExifTag,
    /// EXIF tag value as an enumeration. Behaves as a "variant" value
    pub value: TagValue,
    /// Unit of the value, if applicable. If tag is `UnknownToMe`, unit will be empty.
    /// If the tag has been parsed and it is indeed unitless, it will be `"none"`.
    ///
    /// Note that
    /// unit refers to the contents of `value`, not to the readable string. For example,
    /// a GPS latitude is a triplet of rational values, so unit is D/M/S, even though
    /// `value_more_readable` contains a single string with all three parts
    /// combined.
    pub unit: Cow<'static, str>,
    /// Human-readable and "pretty" version of `value`.
    /// Enumerations and tuples are interpreted and combined. If `value`
    /// has a unit, it is also added.
    /// If tag is `UnknownToMe`, this member contains tag ID.
    pub value_more_readable: Cow<'static, str>,
    pub kind: IfdKind,
}

impl PartialEq for ExifEntry {
    fn eq(&self, other: &Self) -> bool {
        // If the ExifEntry is an ExifOffset or a GPSOffset, the value it contains is an offset.
        // Two entries can be equal even if they do not point to the same offset.
        let value_eq = match self.tag {
            ExifTag::ExifOffset | ExifTag::GPSOffset => true,
            _ => {
                self.value_more_readable == other.value_more_readable && tag_value_eq(&self.value, &other.value)
            },
        };

        self.namespace == other.namespace && self.ifd == other.ifd && self.tag == other.tag &&
            self.unit == other.unit && self.kind == other.kind && value_eq
    }
}

/// Tag value enumeration. It works as a variant type. Each value is
/// actually a vector because many EXIF tags are collections of values.
/// Exif tags with single values are represented as single-item vectors.
#[derive(Clone, Debug, PartialEq)]
pub enum TagValue {
    /// Array of unsigned byte integers
    U8(Vec<u8>),
    /// ASCII string. (The standard specifies 7-bit ASCII, but this parser accepts UTF-8 strings.)
    Ascii(String),
    U16(Vec<u16>),
    U32(Vec<u32>),
    /// Array of `URational` structures (tuples with integer numerator and denominator)
    URational(Vec<URational>),
    I8(Vec<i8>),
    /// Array of bytes with opaque internal structure. Used by manufacturer-specific
    /// tags, SIG-specific tags, tags that contain Unicode (UCS-2) or Japanese (JIS)
    /// strings (i.e. strings that are not 7-bit-clean), tags that contain
    /// dissimilar or variant types, etc.
    ///
    /// This item has a "little endian"
    /// boolean parameter that reports the whole TIFF's endianness.
    /// Any sort of internal structure that is sensitive to endianess
    /// should be interpreted accordignly to this parameter (true=LE, false=BE).
    Undefined(Vec<u8>, bool),
    I16(Vec<i16>),
    I32(Vec<i32>),
    /// Array of `IRational` structures (tuples with signed integer numerator and denominator)
    IRational(Vec<IRational>),
    /// Array of IEEE 754 floating-points
    F32(Vec<f32>),
    /// Array of IEEE 754 floating-points
    F64(Vec<f64>),
    /// Array of bytes with unknown internal structure.
    /// This is different from `Undefined` because `Undefined` is actually a specified
    /// format, while `Unknown` is an unexpected format type. A tag of `Unknown` format
    /// is most likely a corrupted tag.
    ///
    /// This variant has a "little endian"
    /// boolean parameter that reports the whole TIFF's endianness.
    /// Any sort of internal structure that is sensitive to endianess
    /// should be interpreted accordignly to this parameter (true=LE, false=BE).
    Unknown(Vec<u8>, bool),
    /// Type that could not be parsed due to some sort of error (e.g. buffer too
    /// short for the count and type size). Variant contains raw data, LE/BE,
    /// format (as u16) and count.
    Invalid(Vec<u8>, bool, u16, u32),
}

impl TagValue {
    /// Get value as an integer
    /// Out of bounds indexes and invalid types return `None`
    pub fn to_i64(&self, index: usize) -> Option<i64> {
        match *self {
            TagValue::U8(ref v) => v.get(index).copied().map(From::from),
            TagValue::U16(ref v) => v.get(index).copied().map(From::from),
            TagValue::U32(ref v) => v.get(index).copied().map(From::from),
            TagValue::I8(ref v) => v.get(index).copied().map(From::from),
            TagValue::I16(ref v) => v.get(index).copied().map(From::from),
            TagValue::I32(ref v) => v.get(index).copied().map(From::from),
            _ => None,
        }
    }

    /// Get value as a floating-point number
    /// Out of bounds indexes and invalid types return `None`
    pub fn to_f64(&self, index: usize) -> Option<f64> {
        match *self {
            TagValue::U8(ref v) => v.get(index).copied().map(From::from),
            TagValue::U16(ref v) => v.get(index).copied().map(From::from),
            TagValue::U32(ref v) => v.get(index).copied().map(From::from),
            TagValue::I8(ref v) => v.get(index).copied().map(From::from),
            TagValue::I16(ref v) => v.get(index).copied().map(From::from),
            TagValue::I32(ref v) => v.get(index).copied().map(From::from),
            TagValue::F32(ref v) => v.get(index).copied().map(From::from),
            TagValue::F64(ref v) => v.get(index).copied().map(From::from),
            TagValue::IRational(ref v) => v.get(index).copied().map(|v| v.value()),
            TagValue::URational(ref v) => v.get(index).copied().map(|v| v.value()),
            _ => None,
        }
    }
}

/// Type returned by image file parsing
pub type ExifResult = Result<ExifData, ExifError>;

/// Type resturned by lower-level parsing functions
pub type ExifEntryResult = Result<Vec<ExifEntry>, ExifError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IfdKind {
    Ifd0,
    Ifd1,
    Exif,
    Gps,
    Makernote,
    Interoperability,
}
