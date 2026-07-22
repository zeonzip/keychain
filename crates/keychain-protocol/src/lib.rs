use bytes::{Buf, BufMut};
use crate::encode::{Decode, Encode};
use crate::error::DecodeError;

pub mod encode;
pub mod error;
pub mod stream;
pub mod packet;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

const fn parse_u32(bytes: &[u8], offset: usize) -> (u32, usize) {
    let mut value: u32 = 0;

    let mut i = 0;
    while offset + i < bytes.len() && bytes[offset + i].is_ascii_digit() {
        value = (value & !(0xFF << (i * 8))) | ((bytes[offset + i] as u32) << (i * 8));

        i += 1;
    }

    (value, offset + i)
}

const fn parse_semver(semver: &str) -> ProtocolVersion {
    let bytes = semver.as_bytes();

    let (major, i) = parse_u32(bytes, 0);
    let (minor, i) = parse_u32(bytes, i + 1);
    let (patch, _) = parse_u32(bytes, i + 1);

    ProtocolVersion {
        major,
        minor,
        patch,
    }
}

pub const RAW_VERSION: &str = env!("CARGO_PKG_VERSION", "Please compile keychain using cargo to yield proper environment variables.");
pub const VERSION: ProtocolVersion = parse_semver(RAW_VERSION);