use super::lowlevel::*;
use super::types::*;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;

pub(crate) struct NumArray<I>(RefCell<Option<I>>);

impl<I> NumArray<I> {
    pub fn new(i: I) -> Self {
        Self(RefCell::new(Some(i)))
    }
}

impl<T: Display, I: IntoIterator<Item = T>> Display for NumArray<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        let iter = self.0.borrow_mut().take().unwrap();
        for number in iter {
            if !first {
                write!(f, ", {number}")?;
                first = false;
            } else {
                write!(f, "{number}")?;
            }
        }
        Ok(())
    }
}

/// Convert a `IfdEntry` into a tuple of `TagValue`
pub(crate) fn tag_value_new(f: &IfdEntry) -> Option<TagValue> {
    Some(match f.format {
        IfdFormat::Ascii => {
            // Remove \0, there may be more than one
            let mut data = &f.data[..];
            while let Some((&val, rest)) = data.split_last() {
                if val != 0 {
                    break;
                }
                data = rest;
            }
            // In theory it should be pure ASCII but we admit UTF-8
            let s = String::from_utf8_lossy(data);
            let s = s.into_owned();
            TagValue::Ascii(s)
        }
        IfdFormat::U16 => {
            let a = read_u16_array(f.le, f.count, &f.data)?;
            TagValue::U16(a)
        }
        IfdFormat::I16 => {
            let a = read_i16_array(f.le, f.count, &f.data)?;
            TagValue::I16(a)
        }
        IfdFormat::U8 => {
            if f.data.len() < f.count as usize {
                return None;
            }
            TagValue::U8(f.data.clone())
        }
        IfdFormat::I8 => {
            let a = read_i8_array(f.count, &f.data)?;
            TagValue::I8(a)
        }
        IfdFormat::U32 => {
            let a = read_u32_array(f.le, f.count, &f.data)?;
            TagValue::U32(a)
        }
        IfdFormat::I32 => {
            let a = read_i32_array(f.le, f.count, &f.data)?;
            TagValue::I32(a)
        }
        IfdFormat::F32 => {
            let a = read_f32_array(f.count, &f.data)?;
            TagValue::F32(a)
        }
        IfdFormat::F64 => {
            let a = read_f64_array(f.count, &f.data)?;
            TagValue::F64(a)
        }
        IfdFormat::URational => {
            let a = read_urational_array(f.le, f.count, &f.data)?;
            TagValue::URational(a)
        }
        IfdFormat::IRational => {
            let a = read_irational_array(f.le, f.count, &f.data)?;
            TagValue::IRational(a)
        }

        IfdFormat::Undefined => {
            let a = f.data.clone();
            TagValue::Undefined(a, f.le)
        }
        _ => TagValue::Unknown(f.data.clone(), f.le),
    })
}

/// Compare two vectors of floats, and always consider NaN == NaN.
fn vec_cmp_f32(va: &[f32], vb: &[f32]) -> bool {
    (va.len() == vb.len()) && // zip stops at the shortest
     va.iter().zip(vb)
       .all(|(a, b)| (a == b) || (a.is_nan() && b.is_nan()))
}

fn vec_cmp_f64(va: &[f64], vb: &[f64]) -> bool {
    (va.len() == vb.len()) && // zip stops at the shortest
     va.iter().zip(vb)
       .all(|(a, b)| (a == b) || (a.is_nan() && b.is_nan()))
}

/// Check if `left` == `right`. If the `left` and `right` are float vectors, this returns `true` even
/// if they contain NaN values (as long as the two vectors are otherwise identical, and contain NaN
/// values at the same positions).
pub(crate) fn tag_value_eq(left: &TagValue, right: &TagValue) -> bool {
    match (left, right) {
        (TagValue::F32(x), TagValue::F32(y)) => vec_cmp_f32(x, y),
        (TagValue::F64(x), TagValue::F64(y)) => vec_cmp_f64(x, y),
        (x, y) => x == y,
    }
}
