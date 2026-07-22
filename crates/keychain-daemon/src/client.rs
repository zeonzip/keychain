use interprocess::local_socket::tokio::Stream as AsyncStream;
use keychain_protocol::packet::{ClientPacket, ServerPacket};
use keychain_protocol::stream::{ProtocolError, StreamConnection};
use keychain_protocol::VERSION;

pub async fn handle_client(stream: AsyncStream) -> Result<(), ProtocolError> {
    let mut conn = StreamConnection::<AsyncStream>::new(stream);
    let mut handshaked = false;

    while let Ok(packet) = conn.read_packet::<ClientPacket>().await {
        if !handshaked {
            if let ClientPacket::Ping(version) = packet {
                let matches = version == VERSION;

                conn.send_packet(
                    &ServerPacket::Pong {
                        incompatible: !matches,
                        version: VERSION
                    }
                ).await?;

                if matches { handshaked = true } else { break }
            } else { conn.send_packet(&ServerPacket::DropConnection).await?; break; }
        }

        // handle actual client interaction
    }

    Ok(())
}