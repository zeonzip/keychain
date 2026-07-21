use std::io;
use std::io::{Write};
use clap::{Parser, Subcommand};
use interprocess::local_socket::{GenericNamespaced, Stream as LocalStream, ToNsName};
use interprocess::local_socket::traits::Stream;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use keychain_protocol::packet::{ClientPacket, ServerPacket};
use keychain_protocol::stream::StreamConnection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = LocalStream::connect("keychain.sock".to_ns_name::<GenericNamespaced>().unwrap())?;
    let mut conn = StreamConnection::<LocalStream>::new(stream);

    conn.send_packet(&ClientPacket::Ping("hello".to_string()))?;

    while let Ok(packet) = conn.read_packet::<ServerPacket>() {
        println!("{:?}", packet);
    }

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