use relayer::{RelayerConfig, RelayerType, Context, Result};
use relayer::{load_config, errlog};

use std::net::{SocketAddr};
use tokio::net::{TcpListener, TcpStream};

pub async fn run(config: RelayerConfig) -> Result<()> {
    let local_addr = config.get_local_addr().parse::<SocketAddr>()
        .context(errlog!("Parse {:?} into local address failed!"))?;
    let remote_addr = config.get_server_addr()
        .context(errlog!("Failed to get server address"))?
        .parse::<SocketAddr>()
        .context(errlog!("Parse {:?} into remote address failed!"))?;
    let listener = TcpListener::bind(&local_addr).await
        .context(errlog!("Bind to {:?} failed", local_addr))?;
    println!("Listening at {} ...", local_addr.to_string());
    loop {
        let (mut local, local_peer) = listener.accept().await?;
        tokio::spawn(async move {
            println!("Accept connection from {} ...", local_peer.to_string());
            println!("Connect to relay server {} ...", remote_addr.to_string());
            let mut remote = match TcpStream::connect(&remote_addr).await {
                Ok(remote) => remote,
                Err(e) => {
                    println!("{}", errlog!("Failed to connect to {:?}: {:?}", remote_addr, e));
                    return
                }
            };
            let (mut local_recv, mut local_send) = local.split();
            let (mut remote_recv, mut remote_send) = remote.split();
            let (_remote_bytes_copied, _local_bytes_copied) = futures::join!(
                tokio::io::copy(&mut remote_recv, &mut local_send),
                tokio::io::copy(&mut local_recv, &mut remote_send),
            );
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config(RelayerType::CLIENT)
        .context(errlog!("Cannot load client config"))?;
    run(config).await
}
