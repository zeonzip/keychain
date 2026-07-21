use bytes::{Buf, BufMut};
use crate::encode::{Decode, Encode};
use crate::error::DecodeError;

pub mod encode;
pub mod error;
pub mod stream;
pub mod packet;

pub const VERSION: &str = env!("CARGO_PKG_VERSION", "Please compile keychain using cargo to yield proper environment variables.");