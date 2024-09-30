use super::ifdformat::NumArray;
use super::lowlevel::read_u16_array;
use super::types::TagValue;
use std::borrow::Cow;

/// No-op for readable value tag function. Should not be used by any EXIF tag descriptor,
/// except for the catch-all match that handles unknown tags
pub(crate) fn unknown(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    Some(Cow::Owned(format!("[tag={tag:04x}] {e}")))
}

/// No-op for readable value tag function. Used for ASCII string tags, or when the
/// default readable representation of value is pretty enough.
pub(crate) fn strpass(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    Some(Cow::Owned(e.to_string()))
}

/// Indicates which one of the parameters of ISO12232 is used for `PhotographicSensitivity`
pub(crate) fn sensitivity_type(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(match v.first()? {
            0 => "Unknown",
            1 => "Standard output sensitivity (SOS)",
            2 => "Recommended exposure index (REI)",
            3 => "ISO speed",
            4 => "Standard output sensitivity (SOS) and recommended exposure index (REI)",
            5 => "Standard output sensitivity (SOS) and ISO speed",
            6 => "Recommended exposure index (REI) and ISO speed",
            7 => "Standard output sensitivity (SOS) and recommended exposure index (REI) and ISO speed",
            n => return Some(format!("Unknown ({tag:04x}={n})").into()),
        }.into()),
        _ => None,
    }
}

pub(crate) fn orientation(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                1 => "Straight",
                3 => "Upside down",
                6 => "Rotated to left",
                8 => "Rotated to right",
                9 => "Undefined",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn rational_value(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    Some(
        match e {
            TagValue::URational(v) => v.first()?.value(),
            TagValue::IRational(v) => v.first()?.value(),
            _ => return None,
        }
        .to_string()
        .into(),
    )
}

pub(crate) fn rational_values(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(
            NumArray::new(v.iter().map(|&x| x.value()))
                .to_string()
                .into(),
        ),
        _ => None,
    }
}

pub(crate) fn resolution_unit(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                1 => "Unitless",
                2 => "in",
                3 => "cm",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn exposure_time(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => {
            let r = v.first()?;
            Some(
                if r.numerator == 1 && r.denominator > 1 {
                    // traditional 1/x exposure time
                    format!("{r} s")
                } else if r.value() < 0.1 {
                    format!("1/{:.0} s", 1.0 / r.value())
                } else if r.value() < 1.0 {
                    format!("1/{:.1} s", 1.0 / r.value())
                } else {
                    format!("{:.1} s", r.value())
                }
                .into(),
            )
        }
        _ => None,
    }
}

pub(crate) fn f_number(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("f/{:.1}", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn exposure_program(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                1 => "Manual control",
                2 => "Program control",
                3 => "Aperture priority",
                4 => "Shutter priority",
                5 => "Program creative (slow program)",
                6 => "Program creative (high-speed program)",
                7 => "Portrait mode",
                8 => "Landscape mode",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn focal_length(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{} mm", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn focal_length_35(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(format!("{} mm", v.first()?).into()),
        _ => None,
    }
}

pub(crate) fn meters(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{:.1} m", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn iso_speeds(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            if v.len() == 1 {
                format!("ISO {}", v[0])
            } else if v.len() == 2 || v.len() == 3 {
                format!("ISO {} latitude {}", v[0], v[1])
            } else {
                format!("Unknown ({})", NumArray::new(v))
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn dms(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) if v.len() >= 3 => {
            let deg = v[0];
            let min = v[1];
            let sec = v[2];
            Some(
                if deg.denominator == 1 && min.denominator == 1 {
                    format!("{}°{}'{:.2}\"", deg.value(), min.value(), sec.value())
                } else if deg.denominator == 1 {
                    format!("{}°{:.4}'", deg.value(), min.value() + sec.value() / 60.0)
                } else {
                    // untypical format
                    format!(
                        "{:.7}°",
                        deg.value() + min.value() / 60.0 + sec.value() / 3600.0
                    )
                }
                .into(),
            )
        }
        _ => None,
    }
}

pub(crate) fn gps_alt_ref(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U8(ref v) => Some(
            match v.first()? {
                0 => "Above sea level",
                1 => "Below sea level",
                n => return Some(format!("Unknown, assumed below sea level ({n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn gpsdestdistanceref(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Ascii(ref v) => Some(if v == "N" {
            "kn".into()
        } else if v == "K" {
            "km".into()
        } else if v == "M" {
            "mi".into()
        } else {
            format!("Unknown ({v})").into()
        }),
        _ => None,
    }
}

pub(crate) fn gpsdestdistance(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{:.3}", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn gpsspeedref(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Ascii(ref v) => Some(if v == "N" {
            "kn".into()
        } else if v == "K" {
            "km/h".into()
        } else if v == "M" {
            "mi/h".into()
        } else {
            format!("Unknown ({v})").into()
        }),
        _ => None,
    }
}

pub(crate) fn gpsspeed(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{:.1}", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn gpsbearingref(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Ascii(ref v) => Some(if v == "T" {
            "True bearing".into()
        } else if v == "M" {
            "Magnetic bearing".into()
        } else {
            format!("Unknown ({v})").into()
        }),
        _ => None,
    }
}

pub(crate) fn gpsbearing(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{:.2}°", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn gpstimestamp(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => {
            let sec = v.get(2)?;
            let hour = v.first()?;
            let min = v.get(1)?;
            Some(
                format!(
                    "{:02.0}:{:02.0}:{:04.1} UTC",
                    hour.value(),
                    min.value(),
                    sec.value()
                )
                .into(),
            )
        }
        _ => None,
    }
}

pub(crate) fn gpsdiff(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(match v.first()? {
            0 => "Measurement without differential correction".into(),
            1 => "Differential correction applied".into(),
            n => format!("Unknown ({tag:04x}={n})").into(),
        }),
        _ => None,
    }
}

pub(crate) fn gpsstatus(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Ascii(ref v) => Some(if v == "A" {
            "Measurement in progress".into()
        } else if v == "V" {
            "Measurement is interoperability".into()
        } else {
            format!("Unknown ({v})").into()
        }),
        _ => None,
    }
}

pub(crate) fn gpsmeasuremode(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Ascii(ref v) => Some(if v == "2" {
            "2-dimension".into()
        } else if v == "3" {
            "3-dimension".into()
        } else {
            format!("Unknown ({v})").into()
        }),
        _ => None,
    }
}

/// Interprets an Undefined tag as ASCII, when the contents are guaranteed
/// by EXIF standard to be ASCII-compatible. This function accepts UTF-8
/// strings, should they be accepted by EXIF standard in the future.
pub(crate) fn undefined_as_ascii(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Undefined(ref v, _) => Some(String::from_utf8_lossy(&v[..]).into_owned().into()),
        _ => None,
    }
}

/// Outputs an Undefined tag as an array of bytes. Appropriate for tags
/// that are opaque and small-sized
pub(crate) fn undefined_as_u8(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Undefined(ref v, _) => Some(NumArray::new(v).to_string().into()),
        _ => None,
    }
}

/// Tries to parse an Undefined tag as containing a string. For some tags,
/// the string encoding /// format can be discovered by looking into the first
/// 8 bytes.
pub(crate) fn undefined_as_encoded_string(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    // "ASCII\0\0\0"
    static ASC: [u8; 8] = [0x41, 0x53, 0x43, 0x49, 0x49, 0, 0, 0];
    // "JIS\0\0\0\0\0"
    static JIS: [u8; 8] = [0x4a, 0x49, 0x53, 0, 0, 0, 0, 0];
    // "UNICODE\0"
    static UNICODE: [u8; 8] = [0x55, 0x4e, 0x49, 0x43, 0x4f, 0x44, 0x45, 0x00];

    match *e {
        TagValue::Undefined(ref v, le) => {
            Some(
                if v.len() < 8 {
                    format!("String w/ truncated preamble {}", NumArray::new(v))
                } else if v[0..8] == ASC[..] {
                    String::from_utf8_lossy(&v[8..]).into_owned()
                } else if v[0..8] == JIS[..] {
                    format!("JIS string {}", NumArray::new(&v[8..]))
                } else if v[0..8] == UNICODE[..] {
                    let v8 = &v[8..];
                    // reinterpret as vector of u16
                    let v16_size = (v8.len() / 2) as u32;
                    let v16 = read_u16_array(le, v16_size, v8)?;
                    String::from_utf16_lossy(&v16)
                } else {
                    format!("String w/ undefined encoding {}", NumArray::new(v))
                }
                .into(),
            )
        }
        _ => None,
    }
}

/// Prints an opaque and long Undefined tag simply as as "blob", noting its length
pub(crate) fn undefined_as_blob(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Undefined(ref v, _) => Some(format!("Blob of {} bytes", v.len()).into()),
        _ => None,
    }
}

pub(crate) fn apex_tv(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::IRational(ref v) => Some(format!("{:.1} Tv APEX", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn apex_av(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{:.1} Av APEX", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn apex_brightness(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::IRational(ref v) => {
            // numerator 0xffffffff = unknown
            Some(if v.first()?.numerator == -1 {
                "Unknown".into()
            } else {
                format!("{:.1} APEX", v.first()?.value()).into()
            })
        }
        _ => None,
    }
}

pub(crate) fn apex_ev(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::IRational(ref v) => Some(format!("{:.2} EV APEX", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn file_source(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Undefined(ref v, _) => Some(
            if !v.is_empty() && v[0] == 3 {
                "DSC"
            } else {
                "Unknown"
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn flash_energy(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) => Some(format!("{} BCPS", v.first()?.value()).into()),
        _ => None,
    }
}

pub(crate) fn metering_mode(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Unknown",
                1 => "Average",
                2 => "Center-weighted average",
                3 => "Spot",
                4 => "Multi-spot",
                5 => "Pattern",
                6 => "Partial",
                255 => "Other",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn light_source(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Unknown",
                1 => "Daylight",
                2 => "Fluorescent",
                3 => "Tungsten",
                4 => "Flash",
                9 => "Fine weather",
                10 => "Cloudy weather",
                11 => "Shade",
                12 => "Daylight fluorescent (D)",
                13 => "Day white fluorescent (N)",
                14 => "Cool white fluorescent (W)",
                15 => "White fluorescent (WW)",
                17 => "Standard light A",
                18 => "Standard light B",
                19 => "Standard light C",
                20 => "D55",
                21 => "D65",
                22 => "D75",
                23 => "D50",
                24 => "ISO studio tungsten",
                255 => "Other",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn color_space(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                1 => "sRGB",
                65535 => "Uncalibrated",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn flash(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => {
            let n = v.first()?;
            let mut b0 = "Did not fire. ";
            let mut b12 = "";
            let mut b34 = "";
            let mut b6 = "";

            if (n & (1 << 5)) > 0 {
                return Some("Does not have a flash.".into());
            }

            if (n & 1) > 0 {
                b0 = "Fired. ";
                if (n & (1 << 6)) > 0 {
                    b6 = "Redeye reduction. ";
                } else {
                    b6 = "No redeye reduction. ";
                }

                // bits 1 and 2
                let m = (n >> 1) & 3;
                if m == 2 {
                    b12 = "Strobe ret not detected. ";
                } else if m == 3 {
                    b12 = "Strobe ret detected. ";
                }
            }

            // bits 3 and 4
            let m = (n >> 3) & 3;
            if m == 1 {
                b34 = "Forced fire. ";
            } else if m == 2 {
                b34 = "Forced suppresion. ";
            } else if m == 3 {
                b12 = "Auto mode. ";
            }

            Some(format!("{b0}{b12}{b34}{b6}").into())
        }
        _ => None,
    }
}

pub(crate) fn subject_area(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.len() {
                2 => format!("at pixel {},{}", v[0], v[1]),
                3 => format!("at center {},{} radius {}", v[0], v[1], v[2]),
                4 => format!(
                    "at rectangle {},{} width {} height {}",
                    v[0], v[1], v[2], v[3]
                ),
                _ => format!("Unknown ({}) ", NumArray::new(v)),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn subject_location(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) if v.len() >= 2 => Some(format!("at pixel {},{}", v[0], v[1]).into()),
        _ => None,
    }
}

pub(crate) fn sharpness(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Normal",
                1 => "Soft",
                2 => "Hard",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn saturation(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Normal",
                1 => "Low",
                2 => "High",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn contrast(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Normal",
                1 => "Soft",
                2 => "Hard",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn gain_control(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "None",
                1 => "Low gain up",
                2 => "High gain up",
                3 => "Low gain down",
                4 => "High gain down",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn exposure_mode(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Auto exposure",
                1 => "Manual exposure",
                2 => "Auto bracket",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn scene_capture_type(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Standard",
                1 => "Landscape",
                2 => "Portrait",
                3 => "Night scene",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn scene_type(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::Undefined(ref v, _) => Some(
            match v.first()? {
                1 => "Directly photographed image",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn white_balance_mode(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Auto",
                1 => "Manual",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn sensing_method(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                1 => "Not defined",
                2 => "One-chip color area sensor",
                3 => "Two-chip color area sensor",
                4 => "Three-chip color area sensor",
                5 => "Color sequential area sensor",
                7 => "Trilinear sensor",
                8 => "Color sequential linear sensor",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn custom_rendered(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Normal",
                1 => "Custom",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn subject_distance_range(tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::U16(ref v) => Some(
            match v.first()? {
                0 => "Unknown",
                1 => "Macro",
                2 => "Close view",
                3 => "Distant view",
                n => return Some(format!("Unknown ({tag:04x}={n})").into()),
            }
            .into(),
        ),
        _ => None,
    }
}

pub(crate) fn lens_spec(_tag: u16, e: &TagValue) -> Option<Cow<'static, str>> {
    match *e {
        TagValue::URational(ref v) if v.len() >= 4 => {
            let f0 = v[0].value();
            let f1 = v[1].value();
            let a0 = v[2].value();
            let a1 = v[3].value();

            Some(
                if v[0] == v[1] {
                    if a0.is_finite() {
                        format!("{f0} mm f/{a0:.1}")
                    } else {
                        format!("{f0} mm f/unknown")
                    }
                } else if a0.is_finite() && a1.is_finite() {
                    format!("{f0}-{f1} mm f/{a0:.1}-{a1:.1}")
                } else {
                    format!("{f0}-{f1} mm f/unknown")
                }
                .into(),
            )
        }
        _ => None,
    }
}
