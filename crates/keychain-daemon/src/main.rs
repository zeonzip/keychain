use std::io;
use interprocess::local_socket::{GenericFilePath, GenericNamespaced, ListenerOptions, Name, NameType, ToFsName, ToNsName};
use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::traits::tokio::Listener;
use tokio::io::{AsyncRead, AsyncReadExt};
use bytes::BytesMut;
use keychain_protocol::ClientPacket;
use keychain_protocol::encode::Decode;

#[tokio::main]
async fn main() {
    let listener = ListenerOptions::default()
        .name(determine_socket_name())
        .reclaim_name(true)
        .create_tokio().unwrap();

    while let Ok(mut stream) = listener.accept().await {
        while let Ok(packet) = read_packet::<ClientPacket, Stream>(&mut stream).await {
            println!("new");
        }
    }
}

fn determine_socket_name() -> Name<'static> {
    if GenericNamespaced::is_supported() { "keychain.sock".to_ns_name::<GenericNamespaced>().unwrap() } else { "/tmp/keychain.sock".to_fs_name::<GenericFilePath>().unwrap() }
}

async fn read_packet<D: Decode, R: AsyncRead + Unpin>(stream: &mut R) -> Result<D, ProtocolError<D::Error>> {
    let body_length = stream.read_u32_le().await?;

    if body_length > 16 * 1024 * 1024 {
        return Err(ProtocolError::from(io::Error::new(io::ErrorKind::InvalidData, "body too large")));
    }

    let mut tmp = BytesMut::zeroed(body_length as usize);
    stream.read_exact(&mut tmp).await?;

    let decoded = D::decode(&mut tmp).map_err(|e| ProtocolError::Decoding(e))?;

    Ok(decoded)
}

#[derive(thiserror::Error, Debug)]
pub enum ProtocolError<E> {
    #[error(transparent)]
    Decoding(E),
    #[error(transparent)]
    Io(#[from] io::Error),
}