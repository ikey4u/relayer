extern crate tokio;
#[macro_use]
extern crate futures;

use std::net::{SocketAddr};
use tokio::net::{TcpListener, TcpStream};

use relayer::{ServerAddr, load_config};

pub async fn run(local: ServerAddr, remote: ServerAddr) -> Result<(), Box<dyn std::error::Error>> {
    let local_addr = local.to_string().parse::<SocketAddr>().unwrap();
    let remote_addr = remote.to_string().parse::<SocketAddr>().unwrap();
    let mut listener = TcpListener::bind(&local_addr).await.unwrap();
    println!("Listening at {} ...", local_addr.to_string());
    loop {
        let (mut local, local_peer) = listener.accept().await?;
        tokio::spawn(async move {
            println!("Accept connection from {} ...", local_peer.to_string());
            // Fix: why cannot use `?` here?
            // let mut remote = TcpStream::connect(&remote_addr).await?;
            println!("Connect to relay server {} ...", remote_addr.to_string());
            let mut remote = TcpStream::connect(&remote_addr).await.unwrap();
            let (mut local_recv, mut local_send) = local.split();
            let (mut remote_recv, mut remote_send) = remote.split();
            let (_remote_bytes_copied, _local_bytes_copied) = join!(
                tokio::io::copy(&mut remote_recv, &mut local_send),
                tokio::io::copy(&mut local_recv, &mut remote_send),
            );
        });
    }
}

#[tokio::main]
async fn main() -> Result<(),  Box<dyn std::error::Error>> {
    let server_config = load_config()?;
    run(server_config.local, server_config.remote).await
}
