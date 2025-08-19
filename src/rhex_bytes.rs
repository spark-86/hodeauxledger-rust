// src/rhex_bytes.rs
#![allow(dead_code)]

use core::{fmt, str};

/// Decode limits (tune for DoS safety)
pub const MAX_STR: usize = 1 << 20; // 1 MiB
pub const MAX_BLOB: usize = 16 << 20; // 16 MiB

/* ---------- Error ---------- */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    Truncated,
    TooLarge { got: usize, max: usize },
    BadTag,
    Utf8,
}
impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WireError::Truncated => write!(f, "truncated input"),
            WireError::TooLarge { got, max } => write!(f, "field too large: {got} > {max}"),
            WireError::BadTag => write!(f, "unexpected tag"),
            WireError::Utf8 => write!(f, "invalid utf-8"),
        }
    }
}
impl std::error::Error for WireError {}

/* ---------- Writer (encoder) ---------- */

#[derive(Default, Debug)]
pub struct ByteWriter {
    buf: Vec<u8>,
}
impl ByteWriter {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(256),
        }
    }
    pub fn with_capacity(n: usize) -> Self {
        Self {
            buf: Vec::with_capacity(n),
        }
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    #[inline]
    pub fn put_u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    #[inline]
    pub fn put_u16_be(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    #[inline]
    pub fn put_u32_be(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    #[inline]
    pub fn put_u64_be(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    #[inline]
    pub fn put_i64_be(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    #[inline]
    pub fn put_bool(&mut self, v: bool) {
        self.put_u8(if v { 1 } else { 0 });
    }

    #[inline]
    pub fn put_opt_u64_be(&mut self, v: Option<u64>) {
        self.put_bool(v.is_some());
        if let Some(v) = v {
            self.put_u64_be(v);
        }
    }

    /// Length-prefixed bytes: u32 BE length + raw bytes
    pub fn put_bytes(&mut self, b: &[u8]) {
        self.put_u32_be(b.len() as u32);
        self.buf.extend_from_slice(b);
    }

    /// Length-prefixed UTF-8 string
    pub fn put_lp(&mut self, s: &str) {
        self.put_bytes(s.as_bytes());
    }

    /// Presence bitmap for up to 8 optional fields
    pub fn put_presence(&mut self, mask: u8) {
        self.put_u8(mask);
    }

    /// Tag/magic
    pub fn put_tag(&mut self, t: &[u8]) {
        self.buf.extend_from_slice(t);
    }
}

/* ---------- Reader (decoder) ---------- */

#[derive(Clone, Copy, Debug)]
pub struct ByteReader<'a> {
    b: &'a [u8],
    i: usize,
}
impl<'a> ByteReader<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Self { b, i: 0 }
    }
    #[inline]
    fn need(&self, n: usize) -> Result<(), WireError> {
        if self.i + n <= self.b.len() {
            Ok(())
        } else {
            Err(WireError::Truncated)
        }
    }
    pub fn remaining(&self) -> usize {
        self.b.len().saturating_sub(self.i)
    }

    pub fn expect_tag(&mut self, t: &[u8]) -> Result<(), WireError> {
        self.need(t.len())?;
        if &self.b[self.i..self.i + t.len()] != t {
            return Err(WireError::BadTag);
        }
        self.i += t.len();
        Ok(())
    }

    #[inline]
    pub fn take_u8(&mut self) -> Result<u8, WireError> {
        self.need(1)?;
        let v = self.b[self.i];
        self.i += 1;
        Ok(v)
    }
    #[inline]
    pub fn take_bool(&mut self) -> Result<bool, WireError> {
        Ok(self.take_u8()? != 0)
    }
    #[inline]
    pub fn take_u16_be(&mut self) -> Result<u16, WireError> {
        self.need(2)?;
        let v = u16::from_be_bytes(self.b[self.i..self.i + 2].try_into().unwrap());
        self.i += 2;
        Ok(v)
    }
    #[inline]
    pub fn take_u32_be(&mut self) -> Result<u32, WireError> {
        self.need(4)?;
        let v = u32::from_be_bytes(self.b[self.i..self.i + 4].try_into().unwrap());
        self.i += 4;
        Ok(v)
    }
    #[inline]
    pub fn take_u64_be(&mut self) -> Result<u64, WireError> {
        self.need(8)?;
        let v = u64::from_be_bytes(self.b[self.i..self.i + 8].try_into().unwrap());
        self.i += 8;
        Ok(v)
    }
    #[inline]
    pub fn take_i64_be(&mut self) -> Result<i64, WireError> {
        self.need(8)?;
        let v = i64::from_be_bytes(self.b[self.i..self.i + 8].try_into().unwrap());
        self.i += 8;
        Ok(v)
    }

    /// Read u32 length, then that many bytes (bounded)
    pub fn take_bytes_bounded(&mut self, max: usize) -> Result<&'a [u8], WireError> {
        let n = self.take_u32_be()? as usize;
        if n > max {
            return Err(WireError::TooLarge { got: n, max });
        }
        self.need(n)?;
        let out = &self.b[self.i..self.i + n];
        self.i += n;
        Ok(out)
    }

    /// LP UTF-8 string (bounded)
    pub fn take_lp_str_bounded(&mut self, max: usize) -> Result<&'a str, WireError> {
        let bytes = self.take_bytes_bounded(max)?;
        str::from_utf8(bytes).map_err(|_| WireError::Utf8)
    }

    /// Unbounded variants (use carefully)
    pub fn take_bytes(&mut self) -> Result<&'a [u8], WireError> {
        self.take_bytes_bounded(usize::MAX)
    }
    pub fn take_lp_str(&mut self) -> Result<&'a str, WireError> {
        self.take_lp_str_bounded(usize::MAX)
    }

    /// Presence bitmap for optionals
    pub fn take_presence(&mut self) -> Result<u8, WireError> {
        self.take_u8()
    }
    /// Optional u64 (for at)
    pub fn take_opt_u64_be(&mut self) -> Result<Option<u64>, WireError> {
        let present = self.take_bool()?;
        if present {
            Ok(Some(self.take_u64_be()?))
        } else {
            Ok(None)
        }
    }
}

/* ---------- Quick smoke test ---------- */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let mut w = ByteWriter::new();
        w.put_tag(b"RDATA1\0");
        w.put_u16_be(1);
        w.put_presence(0b0000_0001);
        w.put_lp("hello");
        w.put_lp("world");
        w.put_u64_be(3_700_000);
        w.put_lp("optional-owner");

        let mut r = ByteReader::new(&w.as_slice());
        r.expect_tag(b"RDATA1\0").unwrap();
        assert_eq!(r.take_u16_be().unwrap(), 1);
        let mask = r.take_presence().unwrap();
        let a = r.take_lp_str_bounded(MAX_STR).unwrap();
        let b = r.take_lp_str_bounded(MAX_STR).unwrap();
        let p = r.take_u64_be().unwrap();
        let owner = if (mask & 1) != 0 {
            Some(r.take_lp_str_bounded(MAX_STR).unwrap())
        } else {
            None
        };

        assert_eq!(a, "hello");
        assert_eq!(b, "world");
        assert_eq!(p, 3_700_000);
        assert_eq!(owner, Some("optional-owner"));
        assert_eq!(r.remaining(), 0);
    }
}
