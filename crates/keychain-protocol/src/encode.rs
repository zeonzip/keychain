use bytes::{Buf, BufMut};
use crate::error::DecodeError;

pub trait Encode {
    fn encode<W: BufMut>(&self, buf: &mut W);
}

pub trait Decode {
    type Error;

    fn decode<W: Buf>(data: &mut W) -> Result<Self, Self::Error>
        where Self: Sized;
}

pub struct GenericBufDecoder;

impl Decode for String {
    type Error = DecodeError;

    fn decode<W: Buf>(data: &mut W) -> Result<String, DecodeError> {
        if data.remaining() < 4 {
            return Err(DecodeError::UnexpectedEof);
        }
        let buf_length = data.get_u32_le();
        if data.remaining() < buf_length as usize {
            return Err(DecodeError::MalformedLength);
        }
        let bytes = data.copy_to_bytes(buf_length as usize);

        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

impl Encode for String {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        buf.put_u32_le(self.as_bytes().len() as u32);
        buf.put_slice(self.as_bytes());
    }
}

impl Decode for bool {
    type Error = DecodeError;

    fn decode<W: Buf>(data: &mut W) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(data.get_u8() != 0)
    }
}

impl Encode for bool {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        buf.put_u8(*self as u8);
    }
}