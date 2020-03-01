#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate dirs;
extern crate tokio;
#[macro_use]
extern crate futures;

use std::fs::File;
use std::path::{PathBuf};
use std::net::{SocketAddr};

use serde_json as json;
use tokio::net::{TcpListener, TcpStream};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerAddr {
    host: String,
    port: u16,
}

impl ServerAddr {
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    local: ServerAddr,
    remote: ServerAddr,
}

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

    let homedir = dirs::home_dir().expect("Cannot get home directory!");
    let confdir: PathBuf = [homedir.to_str().unwrap(), ".config", "relayer"].iter().collect();
    let mut config_path = confdir.clone();
    config_path.push("config.json");
    let config = File::open(config_path)?;
    let server_config: ServerConfig = json::from_reader(config)?;
    run(server_config.local, server_config.remote).await
}
