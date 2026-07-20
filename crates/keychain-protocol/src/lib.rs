use bytes::{Buf, BufMut};
use crate::encode::{Decode, Encode};
use crate::error::DecodeError;

pub mod encode;
pub mod error;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait PacketBodyCodec {
    fn decode_body<B: Buf>(buf: &mut B, opcode: u32) -> Result<Self, DecodeError>
        where Self: Sized;
    fn encode_body<B: BufMut>(&self, buf: &mut B);
    fn opcode(&self) -> u32;
}

pub enum ClientPacket {
    Ping(String),
    ReqVault,
    Disconnect,
}

impl PacketBodyCodec for ClientPacket {
    fn decode_body<B: Buf>(buf: &mut B, opcode: u32) -> Result<Self, DecodeError> {
        match opcode {
            0 => {
                Ok(Self::Ping(String::decode(buf)?))
            },
            1 => {
                Ok(Self::ReqVault)
            },
            2 => {
                Ok(Self::Disconnect)
            },
            _ => Err(DecodeError::UnrecognizedOpcode)
        }
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) {
        match self {
            Self::Ping(p) => p.encode(buf),
            _ => {}
        }
    }

    fn opcode(&self) -> u32 {
        match self {
            ClientPacket::Ping(_) => 0,
            ClientPacket::ReqVault => 1,
            ClientPacket::Disconnect => 2,
        }
    }
}

pub enum ServerPacket {
    Pong {
        outdated: bool,
        version: String,
    }
}

impl PacketBodyCodec for ServerPacket {
    fn decode_body<B: Buf>(buf: &mut B, opcode: u32) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) {
        
    }

    fn opcode(&self) -> u32 {
        match self {
            ServerPacket::Pong { .. } => 0,
        }
    }
}

impl<D: PacketBodyCodec> Decode for D {
    type Error = DecodeError;

    fn decode<W: Buf>(data: &mut W) -> Result<D, DecodeError>
    where
        Self: Sized,
    {
        let opcode = data.get_u32_le();
        let mut bytes = data.copy_to_bytes(data.remaining());

        let packet = D::decode_body(&mut bytes, opcode)?;

        Ok(packet)
    }
}

impl<D: PacketBodyCodec> Encode for D {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        buf.put_u32_le(self.opcode());
        self.encode_body(buf);
    }
}