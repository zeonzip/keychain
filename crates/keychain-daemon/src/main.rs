mod client;

use interprocess::local_socket::traits::tokio::{Listener};
use interprocess::local_socket::{GenericFilePath, GenericNamespaced, ListenerOptions, Name, NameType, ToFsName, ToNsName};
use crate::client::handle_client;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let listener = ListenerOptions::default()
        .name(determine_socket_name())
        .reclaim_name(true)
        .create_tokio().unwrap();

    while let Ok(stream) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                tracing::error!("Error handling client: {}", e);
            }
        });
    }
}

fn determine_socket_name() -> Name<'static> {
    if GenericNamespaced::is_supported() { "keychain.sock".to_ns_name::<GenericNamespaced>().unwrap() } else { "/tmp/keychain.sock".to_fs_name::<GenericFilePath>().unwrap() }
}