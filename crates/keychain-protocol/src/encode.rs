use bytes::{Buf, BufMut};
use crate::error::DecodeError;
use crate::ProtocolVersion;

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
        check_len(data, 4)?;

        let buf_length = data.get_u32_le();
        check_len(data, buf_length as usize)?;

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
        check_len(data, 1)?;

        Ok(data.get_u8() == 1)
    }
}

impl Encode for bool {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        buf.put_u8(*self as u8);
    }
}

impl Decode for u32 {
    type Error = DecodeError;
    
    fn decode<W: Buf>(data: &mut W) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        check_len(data, 4)?;

        Ok(data.get_u32_le())
    }
}

impl Encode for u32 {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        buf.put_u32_le(*self);
    }
}

impl Decode for ProtocolVersion {
    type Error = DecodeError;

    fn decode<W: Buf>(data: &mut W) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let major = u32::decode(data)?;
        let minor = u32::decode(data)?;
        let patch = u32::decode(data)?;
        
        Ok(
            ProtocolVersion {
                major,
                minor,
                patch,
            }
        )
    }
}

impl Encode for ProtocolVersion {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        self.major.encode(buf);
        self.minor.encode(buf);
        self.patch.encode(buf);
    }
}


#[inline]
pub fn check_len<B: Buf>(buf: &B, bytes: usize) -> Result<(), DecodeError> {
    if buf.remaining() < bytes {
        Err(DecodeError::MalformedLength)
    } else {
        Ok(())
    }
}