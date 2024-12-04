use super::rational::{IRational, URational};
use std::convert::TryInto;

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_u16(le: bool, raw: &[u8]) -> Option<u16> {
    let bytes = raw.get(..2)?.try_into().ok()?;
    Some(if le { u16::from_le_bytes(bytes) } else { u16::from_be_bytes(bytes) })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_i16(le: bool, raw: &[u8]) -> Option<i16> {
    let bytes = raw.get(..2)?.try_into().ok()?;
    Some(if le { i16::from_le_bytes(bytes) } else { i16::from_be_bytes(bytes) })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_u32(le: bool, raw: &[u8]) -> Option<u32> {
    let bytes = raw.get(..4)?.try_into().ok()?;
    Some(if le { u32::from_le_bytes(bytes) } else { u32::from_be_bytes(bytes) })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_i32(le: bool, raw: &[u8]) -> Option<i32> {
    let bytes = raw.get(..4)?.try_into().ok()?;
    Some(if le { i32::from_le_bytes(bytes) } else { i32::from_be_bytes(bytes) })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_f32(raw: &[u8]) -> Option<f32> {
    raw.get(..4)?.try_into().ok().map(f32::from_le_bytes)
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_f64(raw: &[u8]) -> Option<f64> {
    raw.get(..8)?.try_into().ok().map(f64::from_le_bytes)
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_urational(le: bool, raw: &[u8]) -> Option<URational> {
    let n = read_u32(le, &raw[0..4])?;
    let d = read_u32(le, &raw[4..8])?;
    Some(URational { numerator: n, denominator: d })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_irational(le: bool, raw: &[u8]) -> Option<IRational> {
    let n = read_i32(le, &raw[0..4])?;
    let d = read_i32(le, &raw[4..8])?;
    Some(IRational { numerator: n, denominator: d })
}

#[inline(always)]
fn read_elements<T>(size: u8, count: u32, raw: &[u8], convert: impl Fn(&[u8]) -> T) -> Option<Vec<T>> {
    let count = count as usize;
    let size = size as usize;
    let byte_size = size.checked_mul(count)?;
    let bytes = raw.get(..byte_size)?;

    let mut out = Vec::new();
    out.try_reserve_exact(count).ok()?;
    out.extend(bytes.chunks_exact(size).map(convert).take(count));

    Some(out)
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_i8_array(count: u32, raw: &[u8]) -> Option<Vec<i8>> {
    Some(raw.get(..count as usize)?.iter().map(|&i| i as i8).collect())
}

#[inline(never)]
/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_u16_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<u16>> {
    read_elements(2, count, raw, move |ch| read_u16(le, ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_i16_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<i16>> {
    read_elements(2, count, raw, move |ch| read_i16(le, ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_u32_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<u32>> {
    read_elements(4, count, raw, move |ch| read_u32(le, ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_i32_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<i32>> {
    read_elements(4, count, raw, move |ch| read_i32(le, ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_f32_array(count: u32, raw: &[u8]) -> Option<Vec<f32>> {
    read_elements(4, count, raw, move |ch| read_f32(ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_f64_array(count: u32, raw: &[u8]) -> Option<Vec<f64>> {
    read_elements(8, count, raw, move |ch| read_f64(ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_urational_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<URational>> {
    read_elements(8, count, raw, move |ch| read_urational(le, ch).unwrap())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_irational_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<IRational>> {
    read_elements(8, count, raw, move |ch| read_irational(le, ch).unwrap())
}
