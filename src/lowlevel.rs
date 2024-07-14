use super::rational::{IRational, URational};
use std::convert::TryInto;

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_u16(le: bool, raw: &[u8]) -> Option<u16> {
    let bytes = raw.get(..2)?.try_into().ok()?;
    Some(if le {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_i16(le: bool, raw: &[u8]) -> Option<i16> {
    let bytes = raw.get(..2)?.try_into().ok()?;
    Some(if le {
        i16::from_le_bytes(bytes)
    } else {
        i16::from_be_bytes(bytes)
    })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_u32(le: bool, raw: &[u8]) -> Option<u32> {
    let bytes = raw.get(..4)?.try_into().ok()?;
    Some(if le {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    })
}

/// Read value from a stream of bytes
#[inline(always)]
pub(crate) fn read_i32(le: bool, raw: &[u8]) -> Option<i32> {
    let bytes = raw.get(..4)?.try_into().ok()?;
    Some(if le {
        i32::from_le_bytes(bytes)
    } else {
        i32::from_be_bytes(bytes)
    })
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

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_i8_array(count: u32, raw: &[u8]) -> Option<Vec<i8>> {
    Some(raw.get(..count as usize)?.iter().map(|&i| i as i8).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_u16_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<u16>> {
    Some(raw.get(..count as usize * 2)?.chunks_exact(2).take(count as usize).map(|ch| read_u16(le, ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_i16_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<i16>> {
    Some(raw.get(..count as usize * 2)?.chunks_exact(2).take(count as usize).map(|ch| read_i16(le, ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_u32_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<u32>> {
    Some(raw.get(..count as usize * 4)?.chunks_exact(4).take(count as usize).map(|ch| read_u32(le, ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_i32_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<i32>> {
    Some(raw.get(..count as usize * 4)?.chunks_exact(4).take(count as usize).map(|ch| read_i32(le, ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_f32_array(count: u32, raw: &[u8]) -> Option<Vec<f32>> {
    Some(raw.get(..count as usize * 4)?.chunks_exact(4).take(count as usize).map(|ch| read_f32(ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_f64_array(count: u32, raw: &[u8]) -> Option<Vec<f64>> {
    Some(raw.get(..count as usize * 8)?.chunks_exact(8).take(count as usize).map(|ch| read_f64(ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_urational_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<URational>> {
    Some(raw.get(..count as usize * 8)?.chunks_exact(8).take(count as usize).map(|ch| read_urational(le, ch).unwrap()).collect())
}

/// Read array from a stream of bytes. Caller must be sure of count and buffer size
pub(crate) fn read_irational_array(le: bool, count: u32, raw: &[u8]) -> Option<Vec<IRational>> {
    Some(raw.get(..count as usize * 8)?.chunks_exact(8).take(count as usize).map(|ch| read_irational(le, ch).unwrap()).collect())
}
