use std::io;
use std::io::{Read, Write};
use clap::{Parser, Subcommand};
use interprocess::local_socket::{Name, Stream as LocalStream};
use interprocess::local_socket::traits::Stream;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use bytes::{BufMut, BytesMut};
use keychain_protocol::{ClientPacket, PacketBodyCodec, ServerPacket, VERSION};
use keychain_protocol::encode::{Decode, Encode};
use keychain_protocol::error::DecodeError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let cli = KeychainCli::parse();

    cli.command.handle(&mut stdout)?;

    Ok(())
}

pub struct PasskeyMetadata {
    label: String,
    encrypted: bool,
    hidden: bool,
}

#[derive(Parser, Debug)]
#[command(version = "0.0.1", about = "CLI client for the Keychain daemon.")]
struct KeychainCli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Lists all available keys in the keychain
    List,
    Add
}

impl Command {
    fn handle(self, stdout: &mut StandardStream) -> io::Result<()> {
        match self {
            Command::List => self.list(stdout)?,
            Command::Add => {

            }
        }

        Ok(())
    }

    fn list(&self, stdout: &mut StandardStream) -> io::Result<()> {
        let passkeys = [
            PasskeyMetadata {
                label: String::from("ntbk"),
                encrypted: true,
                hidden: false,
            },
            PasskeyMetadata {
                label: String::from("wallet"),
                encrypted: true,
                hidden: true,
            }
        ];

        stdout.set_color(ColorSpec::new().set_bold(true))?;
        writeln!(stdout, "Available keys:")?;

        stdout.reset()?;

        for passkey in passkeys {
            let mut meta = String::new();

            if passkey.encrypted || passkey.hidden {
                meta.push_str(": ");

                if passkey.encrypted {
                    meta.push_str("<encrypted>");
                }

                if passkey.hidden {
                    meta.push_str(" <hidden>");
                }
            }

            write!(stdout, "- ")?;
            stdout.set_color(ColorSpec::new().set_italic(true))?;
            write!(stdout, "{}", passkey.label)?;
            stdout.reset()?;
            stdout.set_color(ColorSpec::new().set_dimmed(true))?;
            writeln!(stdout, "{}", meta)?;
            stdout.reset()?;
        }

        Ok(())
    }
}

pub struct ServerConnection {
    stream: LocalStream
}

impl ServerConnection {
    pub fn new(name: Name) -> Result<Self, ProtocolError> {
        let mut conn = Self { stream: LocalStream::connect(name)? };

        conn.send_packet::<ClientPacket>(&ClientPacket::Ping(VERSION.to_string()))?;
        conn.read_packet::<ServerPacket>()?;

        Ok(conn)
    }

    pub fn send_packet<P: PacketBodyCodec>(&mut self, packet: &P) -> io::Result<()> {
        let mut bytes = BytesMut::new();
        bytes.put_u32_le(0u32);
        packet.encode(&mut bytes);

        let body_len = bytes.len() - 4;
        bytes[0..4].copy_from_slice(&body_len.to_le_bytes());

        self.stream.write_all(&bytes)?;

        Ok(())
    }

    pub fn read_packet<P: PacketBodyCodec>(&mut self) -> Result<P, ProtocolError> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;

        let len = u32::from_le_bytes(len_buf);

        let mut bytes = BytesMut::zeroed(len as usize);
        self.stream.read_exact(&mut bytes)?;

        Ok(P::decode(&mut bytes)?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Decode(#[from] DecodeError)
}