use std::borrow::Cow;

use super::exifreadable::*;
use super::types::*;
use ExifTag::*;
use IfdFormat::{Ascii, URational, Undefined};

type ReadableFn = fn(u16, &TagValue) -> Option<Cow<'static, str>>;

/// Convert a numeric tag into `ExifTag` enumeration, and yields information about the tag. This information
/// is used by the main body of the parser to sanity-check the tags found in image
/// and make sure that EXIF tags have the right data types
///
/// Returns (tag, unit, format, `min_count`, `max_count`, `more_readable`)
pub(crate) fn tag_to_exif(f: u16) -> (ExifTag, &'static str, IfdFormat, i32, i32, ReadableFn) {
    match f {
        0x010e => (ImageDescription, "none", Ascii, -1i32, -1i32, strpass),

        0x010f => (Make, "none", Ascii, -1i32, -1i32, strpass),

        0x013c => (HostComputer, "none", Ascii, -1i32, -1i32, strpass),

        0x0110 => (Model, "none", Ascii, -1i32, -1i32, strpass),

        0x0112 => (Orientation, "none", IfdFormat::U16, 1, 1, orientation),

        0x011a => (XResolution, "pixels per res unit", URational, 1, 1, rational_value),

        0x011b => (YResolution, "pixels per res unit", URational, 1, 1, rational_value),

        0x0128 => (ResolutionUnit, "none", IfdFormat::U16, 1, 1, resolution_unit),

        0x0131 => (Software, "none", Ascii, -1i32, -1i32, strpass),

        0x0132 => (DateTime, "none", Ascii, -1i32, -1i32, strpass),

        0x013e => (WhitePoint, "CIE 1931 coordinates", URational, 2, 2, rational_values),

        0x013f => (PrimaryChromaticities, "CIE 1931 coordinates", URational, 6, 6, rational_values),

        0x0211 => (YCbCrCoefficients, "none", URational, 3, 3, rational_values),

        0x0214 => (ReferenceBlackWhite, "RGB or YCbCr", URational, 6, 6, rational_values),

        0x8298 => (Copyright, "none", Ascii, -1i32, -1i32, strpass),

        0x8769 => (ExifOffset, "byte offset", IfdFormat::U32, 1, 1, strpass),

        0x8825 => (GPSOffset, "byte offset", IfdFormat::U32, 1, 1, strpass),

        0x829a => (ExposureTime, "s", URational, 1, 1, exposure_time),

        0x829d => (FNumber, "f-number", URational, 1, 1, f_number),

        0x8822 => (ExposureProgram, "none", IfdFormat::U16, 1, 1, exposure_program),

        0x8824 => (SpectralSensitivity, "ASTM string", Ascii, -1i32, -1i32, strpass),

        0x8830 => (SensitivityType, "none", IfdFormat::U16, 1, 1, sensitivity_type),

        0x8827 => (ISOSpeedRatings, "ISO", IfdFormat::U16, 1, 3, iso_speeds),

        0x8828 => (OECF, "none", Undefined, -1i32, -1i32, undefined_as_blob),

        0x9000 => (ExifVersion, "none", Undefined, -1i32, -1i32, undefined_as_ascii),

        0x9003 => (DateTimeOriginal, "none", Ascii, -1i32, -1i32, strpass),

        0x9004 => (DateTimeDigitized, "none", Ascii, -1i32, -1i32, strpass),

        0x9201 => (ShutterSpeedValue, "APEX", IfdFormat::IRational, 1, 1, apex_tv),

        0x9202 => (ApertureValue, "APEX", URational, 1, 1, apex_av),

        0x9203 => (BrightnessValue, "APEX", IfdFormat::IRational, 1, 1, apex_brightness),

        0x9204 => (ExposureBiasValue, "APEX", IfdFormat::IRational, 1, 1, apex_ev),

        0x9205 => (MaxApertureValue, "APEX", URational, 1, 1, apex_av),

        0x9206 => (SubjectDistance, "m", URational, 1, 1, meters),

        0x9207 => (MeteringMode, "none", IfdFormat::U16, 1, 1, metering_mode),

        0x9208 => (LightSource, "none", IfdFormat::U16, 1, 1, light_source),

        0x9209 => (Flash, "none", IfdFormat::U16, 1, 2, flash),

        0x920a => (FocalLength, "mm", URational, 1, 1, focal_length),

        0x9214 => (SubjectArea, "px", IfdFormat::U16, 2, 4, subject_area),

        0x927c => (MakerNote, "none", Undefined, -1i32, -1i32, undefined_as_blob),

        0x9286 => (UserComment, "none", Undefined, -1i32, -1i32, undefined_as_encoded_string),

        0xa000 => (FlashPixVersion, "none", Undefined, -1i32, -1i32, undefined_as_ascii),

        0xa001 => (ColorSpace, "none", IfdFormat::U16, 1, 1, color_space),

        0xa004 => (RelatedSoundFile, "none", Ascii, -1i32, -1i32, strpass),

        0xa20b => (FlashEnergy, "BCPS", URational, 1, 1, flash_energy),

        0xa20e => {
            (FocalPlaneXResolution, "@FocalPlaneResolutionUnit", URational, 1, 1, rational_value)
        }

        0xa20f => {
            (FocalPlaneYResolution, "@FocalPlaneResolutionUnit", URational, 1, 1, rational_value)
        }

        0xa210 => (FocalPlaneResolutionUnit, "none", IfdFormat::U16, 1, 1, resolution_unit),

        0xa214 => (SubjectLocation, "X,Y", IfdFormat::U16, 2, 2, subject_location),

        // TODO check if rational as decimal value is the best for this one
        0xa215 => (ExposureIndex, "EI", URational, 1, 1, rational_value),

        0xa217 => (SensingMethod, "none", IfdFormat::U16, 1, 1, sensing_method),

        0xa300 => (FileSource, "none", Undefined, 1, 1, file_source),

        0xa301 => (SceneType, "none", Undefined, 1, 1, scene_type),

        0xa302 => (CFAPattern, "none", Undefined, -1i32, -1i32, undefined_as_u8),

        0xa401 => (CustomRendered, "none", IfdFormat::U16, 1, 1, custom_rendered),

        0xa402 => (ExposureMode, "none", IfdFormat::U16, 1, 1, exposure_mode),

        0xa403 => (WhiteBalanceMode, "none", IfdFormat::U16, 1, 1, white_balance_mode),

        0xa404 => (DigitalZoomRatio, "none", URational, 1, 1, rational_value),

        0xa405 => (FocalLengthIn35mmFilm, "mm", IfdFormat::U16, 1, 1, focal_length_35),

        0xa406 => (SceneCaptureType, "none", IfdFormat::U16, 1, 1, scene_capture_type),

        0xa407 => (GainControl, "none", IfdFormat::U16, 1, 1, gain_control),

        0xa408 => (Contrast, "none", IfdFormat::U16, 1, 1, contrast),

        0xa409 => (Saturation, "none", IfdFormat::U16, 1, 1, saturation),

        0xa40a => (Sharpness, "none", IfdFormat::U16, 1, 1, sharpness),

        0xa432 => (LensSpecification, "none", URational, 4, 4, lens_spec),

        0xa433 => (LensMake, "none", Ascii, -1i32, -1i32, strpass),

        0xa434 => (LensModel, "none", Ascii, -1i32, -1i32, strpass),

        0xa500 => (Gamma, "none", URational, 1, 1, rational_value),

        // collaborate if you have any idea how to interpret this
        0xa40b => (DeviceSettingDescription, "none", Undefined, -1i32, -1i32, undefined_as_blob),

        0xa40c => (SubjectDistanceRange, "none", IfdFormat::U16, 1, 1, subject_distance_range),

        0xa420 => (ImageUniqueID, "none", Ascii, -1i32, -1i32, strpass),

        0x0 => (GPSVersionID, "none", IfdFormat::U8, 4, 4, strpass),

        0x1 => (GPSLatitudeRef, "none", Ascii, -1i32, -1i32, strpass),

        0x2 => (GPSLatitude, "D/M/S", URational, 3, 3, dms),

        0x3 => (GPSLongitudeRef, "none", Ascii, -1i32, -1i32, strpass),

        0x4 => (GPSLongitude, "D/M/S", URational, 3, 3, dms),

        0x5 => (GPSAltitudeRef, "none", IfdFormat::U8, 1, 1, gps_alt_ref),

        0x6 => (GPSAltitude, "m", URational, 1, 1, meters),

        0x7 => (GPSTimeStamp, "UTC time", URational, 3, 3, gpstimestamp),

        0x8 => (GPSSatellites, "none", Ascii, -1i32, -1i32, strpass),

        0x9 => (GPSStatus, "none", Ascii, -1i32, -1i32, gpsstatus),

        0xa => (GPSMeasureMode, "none", Ascii, -1i32, -1i32, gpsmeasuremode),

        0xb => (GPSDOP, "none", URational, 1, 1, rational_value),

        0xc => (GPSSpeedRef, "none", Ascii, -1i32, -1i32, gpsspeedref),

        0xd => (GPSSpeed, "@GPSSpeedRef", URational, 1, 1, gpsspeed),

        0xe => (GPSTrackRef, "none", Ascii, -1i32, -1i32, gpsbearingref),

        0xf => (GPSTrack, "deg", URational, 1, 1, gpsbearing),

        0x10 => (GPSImgDirectionRef, "none", Ascii, -1i32, -1i32, gpsbearingref),

        0x11 => (GPSImgDirection, "deg", URational, 1, 1, gpsbearing),

        0x12 => (GPSMapDatum, "none", Ascii, -1i32, -1i32, strpass),

        0x13 => (GPSDestLatitudeRef, "none", Ascii, -1i32, -1i32, strpass),

        0x14 => (GPSDestLatitude, "D/M/S", URational, 3, 3, dms),

        0x15 => (GPSDestLongitudeRef, "none", Ascii, -1i32, -1i32, strpass),

        0x16 => (GPSDestLongitude, "D/M/S", URational, 3, 3, dms),

        0x17 => (GPSDestBearingRef, "none", Ascii, -1i32, -1i32, gpsbearingref),

        0x18 => (GPSDestBearing, "deg", URational, 1, 1, gpsbearing),

        0x19 => (GPSDestDistanceRef, "none", Ascii, -1i32, -1i32, gpsdestdistanceref),

        0x1a => (GPSDestDistance, "@GPSDestDistanceRef", URational, 1, 1, gpsdestdistance),

        0x1b => (GPSProcessingMethod, "none", Undefined, -1i32, -1i32, undefined_as_encoded_string),

        0x1c => (GPSAreaInformation, "none", Undefined, -1i32, -1i32, undefined_as_encoded_string),

        0x1d => (GPSDateStamp, "none", Ascii, -1i32, -1i32, strpass),

        0x1e => (GPSDifferential, "none", IfdFormat::U16, 1, 1, gpsdiff),

        _ => (UnknownToMe, "Unknown unit", IfdFormat::Unknown, -1i32, -1i32, unknown),
    }
}
