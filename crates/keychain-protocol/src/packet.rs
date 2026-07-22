use bytes::{Buf, BufMut};
use crate::encode::{Decode, Encode};
use crate::error::DecodeError;
use crate::ProtocolVersion;

pub trait PacketBodyCodec {
    fn decode_body<B: Buf>(buf: &mut B, opcode: u32) -> Result<Self, DecodeError>
    where Self: Sized;
    fn encode_body<B: BufMut>(&self, buf: &mut B);
    fn opcode(&self) -> u32;
}
#[derive(Debug)]
pub enum ClientPacket {
    // KEEP THIS PACKET TYPE (handshake) STABLE, IT IS SUPPOSED TO WORK ACROSS DIFFERENT VERSIONS
    Ping(ProtocolVersion),
    ReqVault,
    Disconnect,
}

impl PacketBodyCodec for ClientPacket {
    fn decode_body<B: Buf>(buf: &mut B, opcode: u32) -> Result<Self, DecodeError> {
        match opcode {
            0 => {
                Ok(Self::Ping(ProtocolVersion::decode(buf)?))
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
            Self::Disconnect | Self::ReqVault => {}
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

#[derive(Debug)]
pub enum ServerPacket {
    // KEEP THIS PACKET TYPE (handshake) STABLE, IT IS SUPPOSED TO WORK ACROSS DIFFERENT VERSIONS
    Pong {
        incompatible: bool,
        version: ProtocolVersion,
    },
    DropConnection
}

impl PacketBodyCodec for ServerPacket {
    fn decode_body<B: Buf>(buf: &mut B, opcode: u32) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        match opcode {
            0 => {
                let incompatible = bool::decode(buf)?;
                let version = ProtocolVersion::decode(buf)?;

                Ok(Self::Pong { incompatible, version })
            },
            1 => Ok(Self::DropConnection),
            _ => Err(DecodeError::UnrecognizedOpcode)
        }
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) {
        match self {
            ServerPacket::Pong { incompatible: outdated, version } => {
                outdated.encode(buf);
                version.encode(buf);
            },
            ServerPacket::DropConnection => {}
        }
    }

    fn opcode(&self) -> u32 {
        match self {
            ServerPacket::Pong { .. } => 0,
            ServerPacket::DropConnection => 1,
        }
    }
}

impl<D: PacketBodyCodec> Decode for D {
    type Error = DecodeError;

    fn decode<W: Buf>(data: &mut W) -> Result<D, DecodeError>
    where
        Self: Sized,
    {
        let opcode = u32::decode(data)?;
        let mut bytes = data.copy_to_bytes(data.remaining());

        let packet = D::decode_body(&mut bytes, opcode)?;

        Ok(packet)
    }
}

impl<D: PacketBodyCodec> Encode for D {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        self.opcode().encode(buf);
        self.encode_body(buf);
    }
}