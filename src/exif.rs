use super::exifreadable::*;
use super::types::*;
use std::borrow::Cow;

type ReadableFn = fn(u16, &TagValue) -> Option<Cow<'static, str>>;

/// Convert a numeric tag into `ExifTag` enumeration, and yields information about the tag. This information
/// is used by the main body of the parser to sanity-check the tags found in image
/// and make sure that EXIF tags have the right data types
///
/// Returns (tag, unit, format, `min_count`, `max_count`, `more_readable`)
pub(crate) fn tag_to_exif(f: u16) -> (ExifTag, &'static str, IfdFormat, i32, i32, ReadableFn) {
    match f {
        0x010e => (
            ExifTag::ImageDescription,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x010f => (
            ExifTag::Make,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x013c => (
            ExifTag::HostComputer,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x0110 => (
            ExifTag::Model,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x0112 => (
            ExifTag::Orientation,
            "none",
            IfdFormat::U16,
            1,
            1,
            orientation,
        ),

        0x011a => (
            ExifTag::XResolution,
            "pixels per res unit",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0x011b => (
            ExifTag::YResolution,
            "pixels per res unit",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0x0128 => (
            ExifTag::ResolutionUnit,
            "none",
            IfdFormat::U16,
            1,
            1,
            resolution_unit,
        ),

        0x0131 => (
            ExifTag::Software,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x0132 => (
            ExifTag::DateTime,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x013e => (
            ExifTag::WhitePoint,
            "CIE 1931 coordinates",
            IfdFormat::URational,
            2,
            2,
            rational_values,
        ),

        0x013f => (
            ExifTag::PrimaryChromaticities,
            "CIE 1931 coordinates",
            IfdFormat::URational,
            6,
            6,
            rational_values,
        ),

        0x0211 => (
            ExifTag::YCbCrCoefficients,
            "none",
            IfdFormat::URational,
            3,
            3,
            rational_values,
        ),

        0x0214 => (
            ExifTag::ReferenceBlackWhite,
            "RGB or YCbCr",
            IfdFormat::URational,
            6,
            6,
            rational_values,
        ),

        0x8298 => (
            ExifTag::Copyright,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x8769 => (
            ExifTag::ExifOffset,
            "byte offset",
            IfdFormat::U32,
            1,
            1,
            strpass,
        ),

        0x8825 => (
            ExifTag::GPSOffset,
            "byte offset",
            IfdFormat::U32,
            1,
            1,
            strpass,
        ),

        0x829a => (
            ExifTag::ExposureTime,
            "s",
            IfdFormat::URational,
            1,
            1,
            exposure_time,
        ),

        0x829d => (
            ExifTag::FNumber,
            "f-number",
            IfdFormat::URational,
            1,
            1,
            f_number,
        ),

        0x8822 => (
            ExifTag::ExposureProgram,
            "none",
            IfdFormat::U16,
            1,
            1,
            exposure_program,
        ),

        0x8824 => (
            ExifTag::SpectralSensitivity,
            "ASTM string",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x8830 => (
            ExifTag::SensitivityType,
            "none",
            IfdFormat::U16,
            1,
            1,
            sensitivity_type,
        ),

        0x8827 => (
            ExifTag::ISOSpeedRatings,
            "ISO",
            IfdFormat::U16,
            1,
            3,
            iso_speeds,
        ),

        0x8828 => (
            ExifTag::OECF,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_blob,
        ),

        0x9000 => (
            ExifTag::ExifVersion,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_ascii,
        ),

        0x9003 => (
            ExifTag::DateTimeOriginal,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x9004 => (
            ExifTag::DateTimeDigitized,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x9201 => (
            ExifTag::ShutterSpeedValue,
            "APEX",
            IfdFormat::IRational,
            1,
            1,
            apex_tv,
        ),

        0x9202 => (
            ExifTag::ApertureValue,
            "APEX",
            IfdFormat::URational,
            1,
            1,
            apex_av,
        ),

        0x9203 => (
            ExifTag::BrightnessValue,
            "APEX",
            IfdFormat::IRational,
            1,
            1,
            apex_brightness,
        ),

        0x9204 => (
            ExifTag::ExposureBiasValue,
            "APEX",
            IfdFormat::IRational,
            1,
            1,
            apex_ev,
        ),

        0x9205 => (
            ExifTag::MaxApertureValue,
            "APEX",
            IfdFormat::URational,
            1,
            1,
            apex_av,
        ),

        0x9206 => (
            ExifTag::SubjectDistance,
            "m",
            IfdFormat::URational,
            1,
            1,
            meters,
        ),

        0x9207 => (
            ExifTag::MeteringMode,
            "none",
            IfdFormat::U16,
            1,
            1,
            metering_mode,
        ),

        0x9208 => (
            ExifTag::LightSource,
            "none",
            IfdFormat::U16,
            1,
            1,
            light_source,
        ),

        0x9209 => (ExifTag::Flash, "none", IfdFormat::U16, 1, 2, flash),

        0x920a => (
            ExifTag::FocalLength,
            "mm",
            IfdFormat::URational,
            1,
            1,
            focal_length,
        ),

        0x9214 => (
            ExifTag::SubjectArea,
            "px",
            IfdFormat::U16,
            2,
            4,
            subject_area,
        ),

        0x927c => (
            ExifTag::MakerNote,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_blob,
        ),

        0x9286 => (
            ExifTag::UserComment,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_encoded_string,
        ),

        0xa000 => (
            ExifTag::FlashPixVersion,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_ascii,
        ),

        0xa001 => (
            ExifTag::ColorSpace,
            "none",
            IfdFormat::U16,
            1,
            1,
            color_space,
        ),

        0xa004 => (
            ExifTag::RelatedSoundFile,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0xa20b => (
            ExifTag::FlashEnergy,
            "BCPS",
            IfdFormat::URational,
            1,
            1,
            flash_energy,
        ),

        0xa20e => (
            ExifTag::FocalPlaneXResolution,
            "@FocalPlaneResolutionUnit",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0xa20f => (
            ExifTag::FocalPlaneYResolution,
            "@FocalPlaneResolutionUnit",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0xa210 => (
            ExifTag::FocalPlaneResolutionUnit,
            "none",
            IfdFormat::U16,
            1,
            1,
            resolution_unit,
        ),

        0xa214 => (
            ExifTag::SubjectLocation,
            "X,Y",
            IfdFormat::U16,
            2,
            2,
            subject_location,
        ),

        // TODO check if rational as decimal value is the best for this one
        0xa215 => (
            ExifTag::ExposureIndex,
            "EI",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0xa217 => (
            ExifTag::SensingMethod,
            "none",
            IfdFormat::U16,
            1,
            1,
            sensing_method,
        ),

        0xa300 => (
            ExifTag::FileSource,
            "none",
            IfdFormat::Undefined,
            1,
            1,
            file_source,
        ),

        0xa301 => (
            ExifTag::SceneType,
            "none",
            IfdFormat::Undefined,
            1,
            1,
            scene_type,
        ),

        0xa302 => (
            ExifTag::CFAPattern,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_u8,
        ),

        0xa401 => (
            ExifTag::CustomRendered,
            "none",
            IfdFormat::U16,
            1,
            1,
            custom_rendered,
        ),

        0xa402 => (
            ExifTag::ExposureMode,
            "none",
            IfdFormat::U16,
            1,
            1,
            exposure_mode,
        ),

        0xa403 => (
            ExifTag::WhiteBalanceMode,
            "none",
            IfdFormat::U16,
            1,
            1,
            white_balance_mode,
        ),

        0xa404 => (
            ExifTag::DigitalZoomRatio,
            "none",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0xa405 => (
            ExifTag::FocalLengthIn35mmFilm,
            "mm",
            IfdFormat::U16,
            1,
            1,
            focal_length_35,
        ),

        0xa406 => (
            ExifTag::SceneCaptureType,
            "none",
            IfdFormat::U16,
            1,
            1,
            scene_capture_type,
        ),

        0xa407 => (
            ExifTag::GainControl,
            "none",
            IfdFormat::U16,
            1,
            1,
            gain_control,
        ),

        0xa408 => (ExifTag::Contrast, "none", IfdFormat::U16, 1, 1, contrast),

        0xa409 => (
            ExifTag::Saturation,
            "none",
            IfdFormat::U16,
            1,
            1,
            saturation,
        ),

        0xa40a => (ExifTag::Sharpness, "none", IfdFormat::U16, 1, 1, sharpness),

        0xa432 => (
            ExifTag::LensSpecification,
            "none",
            IfdFormat::URational,
            4,
            4,
            lens_spec,
        ),

        0xa433 => (
            ExifTag::LensMake,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0xa434 => (
            ExifTag::LensModel,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0xa500 => (
            ExifTag::Gamma,
            "none",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        // collaborate if you have any idea how to interpret this
        0xa40b => (
            ExifTag::DeviceSettingDescription,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_blob,
        ),

        0xa40c => (
            ExifTag::SubjectDistanceRange,
            "none",
            IfdFormat::U16,
            1,
            1,
            subject_distance_range,
        ),

        0xa420 => (
            ExifTag::ImageUniqueID,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x0 => (ExifTag::GPSVersionID, "none", IfdFormat::U8, 4, 4, strpass),

        0x1 => (
            ExifTag::GPSLatitudeRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x2 => (
            ExifTag::GPSLatitude,
            "D/M/S",
            IfdFormat::URational,
            3,
            3,
            dms,
        ),

        0x3 => (
            ExifTag::GPSLongitudeRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x4 => (
            ExifTag::GPSLongitude,
            "D/M/S",
            IfdFormat::URational,
            3,
            3,
            dms,
        ),

        0x5 => (
            ExifTag::GPSAltitudeRef,
            "none",
            IfdFormat::U8,
            1,
            1,
            gps_alt_ref,
        ),

        0x6 => (
            ExifTag::GPSAltitude,
            "m",
            IfdFormat::URational,
            1,
            1,
            meters,
        ),

        0x7 => (
            ExifTag::GPSTimeStamp,
            "UTC time",
            IfdFormat::URational,
            3,
            3,
            gpstimestamp,
        ),

        0x8 => (
            ExifTag::GPSSatellites,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x9 => (
            ExifTag::GPSStatus,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsstatus,
        ),

        0xa => (
            ExifTag::GPSMeasureMode,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsmeasuremode,
        ),

        0xb => (
            ExifTag::GPSDOP,
            "none",
            IfdFormat::URational,
            1,
            1,
            rational_value,
        ),

        0xc => (
            ExifTag::GPSSpeedRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsspeedref,
        ),

        0xd => (
            ExifTag::GPSSpeed,
            "@GPSSpeedRef",
            IfdFormat::URational,
            1,
            1,
            gpsspeed,
        ),

        0xe => (
            ExifTag::GPSTrackRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsbearingref,
        ),

        0xf => (
            ExifTag::GPSTrack,
            "deg",
            IfdFormat::URational,
            1,
            1,
            gpsbearing,
        ),

        0x10 => (
            ExifTag::GPSImgDirectionRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsbearingref,
        ),

        0x11 => (
            ExifTag::GPSImgDirection,
            "deg",
            IfdFormat::URational,
            1,
            1,
            gpsbearing,
        ),

        0x12 => (
            ExifTag::GPSMapDatum,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x13 => (
            ExifTag::GPSDestLatitudeRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x14 => (
            ExifTag::GPSDestLatitude,
            "D/M/S",
            IfdFormat::URational,
            3,
            3,
            dms,
        ),

        0x15 => (
            ExifTag::GPSDestLongitudeRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x16 => (
            ExifTag::GPSDestLongitude,
            "D/M/S",
            IfdFormat::URational,
            3,
            3,
            dms,
        ),

        0x17 => (
            ExifTag::GPSDestBearingRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsbearingref,
        ),

        0x18 => (
            ExifTag::GPSDestBearing,
            "deg",
            IfdFormat::URational,
            1,
            1,
            gpsbearing,
        ),

        0x19 => (
            ExifTag::GPSDestDistanceRef,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            gpsdestdistanceref,
        ),

        0x1a => (
            ExifTag::GPSDestDistance,
            "@GPSDestDistanceRef",
            IfdFormat::URational,
            1,
            1,
            gpsdestdistance,
        ),

        0x1b => (
            ExifTag::GPSProcessingMethod,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_encoded_string,
        ),

        0x1c => (
            ExifTag::GPSAreaInformation,
            "none",
            IfdFormat::Undefined,
            -1i32,
            -1i32,
            undefined_as_encoded_string,
        ),

        0x1d => (
            ExifTag::GPSDateStamp,
            "none",
            IfdFormat::Ascii,
            -1i32,
            -1i32,
            strpass,
        ),

        0x1e => (
            ExifTag::GPSDifferential,
            "none",
            IfdFormat::U16,
            1,
            1,
            gpsdiff,
        ),

        _ => (
            ExifTag::UnknownToMe,
            "Unknown unit",
            IfdFormat::Unknown,
            -1i32,
            -1i32,
            unknown,
        ),
    }
}
