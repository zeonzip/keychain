use crate::encode::{Decode, Encode};
use crate::error::DecodeError;
use crate::packet::PacketBodyCodec;
use bytes::{BufMut, BytesMut};
use interprocess::local_socket::tokio::Stream as AsyncStream;
use interprocess::local_socket::Stream as LocalStream;
use std::io;
use std::io::{Read, Write};
#[cfg(feature = "tokio")]
use tokio::io::AsyncReadExt;
#[cfg(feature = "tokio")]
use tokio::io::AsyncWriteExt;

pub struct StreamConnection<S> {
    pub stream: S,
    reuse_buf: BytesMut
}

impl<S> StreamConnection<S> {
    pub fn new(stream: S) -> Self {
        Self { stream, reuse_buf: BytesMut::new() }
    }
}

impl StreamConnection<LocalStream> {
    pub fn send_packet<P: PacketBodyCodec>(&mut self, packet: &P) -> io::Result<()> {
        let mut bytes = &mut self.reuse_buf;
        bytes.put_u32_le(0u32);
        packet.encode(&mut bytes);

        let packet_len = (bytes.len() - 4) as u32;
        bytes[0..4].copy_from_slice(&packet_len.to_le_bytes());

        self.stream.write_all(&bytes.split().freeze())?;

        Ok(())
    }

    pub fn read_packet<P: PacketBodyCodec>(&mut self) -> Result<P, ProtocolError> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;

        let len = u32::from_le_bytes(len_buf);

        self.reuse_buf.clear();
        self.reuse_buf.resize(len as usize, 0);
        self.stream.read_exact(&mut self.reuse_buf)?;

        Ok(P::decode(&mut self.reuse_buf.split().freeze())?)
    }
}

#[cfg(feature = "tokio")]
impl StreamConnection<AsyncStream> {
    pub async fn send_packet<P: PacketBodyCodec>(&mut self, packet: &P) -> io::Result<()> {
        let mut bytes = &mut self.reuse_buf;
        bytes.put_u32_le(0u32);
        packet.encode(&mut bytes);

        let packet_len = (bytes.len() - 4) as u32;
        bytes[0..4].copy_from_slice(&packet_len.to_le_bytes());

        self.stream.write_all(&bytes.split().freeze()).await?;

        Ok(())
    }

    pub async fn read_packet<P: PacketBodyCodec>(&mut self) -> Result<P, ProtocolError> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;

        let len = u32::from_le_bytes(len_buf);

        self.reuse_buf.clear();
        self.reuse_buf.resize(len as usize, 0);
        self.stream.read_exact(&mut self.reuse_buf).await?;

        Ok(P::decode(&mut self.reuse_buf.split().freeze())?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Decode(#[from] DecodeError)
}