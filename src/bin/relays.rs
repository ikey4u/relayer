//! Process SOCKS5 connection
//!
//! 1. Client sends establish request
//!
//!    The format of request data is showed in below
//!
//!        version(1)|auth_method_count(1)|auth_methods(auth_method_count * 1)
//!
//!     - `verison`: Version of socks protocol, 0x05 for socks5
//!     - `auth_method_count`: The number of authentication methods
//!     - `auth_methods`: List of authentication methods, each holds one byte
//!
//!         Some common authentication methods are listed below
//!
//!         - `0x00`: No authentication required
//!         - `0x01`: GSSAPI(Generic Security Services API)
//!         - `0x02`: USERNAME/PASSWORD
//!         - `[0x03, 0x7F]`: IANA assigned. See [socks-methods](https://www.iana.org/assignments/socks-methods/socks-methods.xhtml#socks-methods-1) for more details.
//!         - `[0x80, 0xFE]`: Reserved for private methods
//!         - `0xFF`: No acceptable methods
//!
//! 2. Server sends back establish response
//!
//!     The format is
//!
//!         version(1)|auth_method(1)|
//!
//!     - `version`: 0x5
//!     - `auth_method`: The server select one authentication method for client
//!
//! 3. Client normal request data
//!
//!     The format is
//!
//!         version(1)|cmd(1)|rsv(1)|atyp(1)|addr(?)|port(2)
//!
//!     - `version`: 0x05
//!     - `cmd`:
//!
//!         - 0x01: CONNECT
//!         - 0x02: BIND
//!         - 0x03: UDP associate
//!
//!     - `rsv`: Alwayes be 0x00
//!     - `atyp`: Address type.
//!
//!         - 0x01: IPv4, the addres length is always 4 bytes
//!         - 0x03: Domain name, the first byte of `addr` indicates the length of `addr`
//!         - 0x04: IPv6, the length of address is always 16 bytes
//!
//!     - `addr`: desired destination address
//!     - `port`:  desired destination port in network octet order
//!

extern crate tokio;
#[macro_use]
extern crate futures;

use std::net::{SocketAddr};
use crate::tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::net::Shutdown;
use std::io::{Error, ErrorKind};

use relayer::{RelayerConfig, RelayerType, Result, Context, errlog};

#[derive(Debug)]
struct Socks5Request {
    pub version: u8,
    pub cmd: u8,
    pub reserved: u8,
    pub addr_type: u8,
    pub addr: Vec<u8>,
    pub port: u16,
}

impl std::fmt::Display for Socks5Request {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "version: {}, cmd: {}, addr: {:?}, port: {}", self.version, self.cmd, self.addr, self.port)
        }
}

impl Socks5Request {
    async fn from_stream(stream: &mut TcpStream) -> Result<Self> {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await?;

        let version = buf[0];
        let cmd = buf[1];
        let reserved = buf[2];
        let addr_type = buf[3];
        let addr = match addr_type {
            0x01 => {
                let mut ipv4 = [0u8; 4];
                stream.read_exact(&mut ipv4).await?;
                let addr = &ipv4.iter().map(std::string::ToString::to_string).collect::<Vec<String>>().join(".");
                println!("IPv4: {}", addr);
                ipv4.to_vec()
            },
            0x03 => {
                let mut buf = [0u8; 1];
                stream.read_exact(&mut buf).await?;
                let domain = vec![0u8; buf[0] as usize];
                println!("Domain: {}", String::from_utf8_lossy(&domain).to_string());
                domain.to_vec()
            },
            0x04 => {
                let mut ipv6 = [0u8; 16];
                stream.read_exact(&mut ipv6).await?;
                println!("IPv6: TODO(...)");
                ipv6.to_vec()
            },
            _ => panic!(),
        };

        let mut port = [0u8; 2];
        stream.read_exact(&mut port).await?;
        let port = ((port[0] as u16) << 8) | port[1] as u16;
        Ok(Socks5Request {
            version,
            cmd,
            reserved,
            addr_type,
            addr,
            port,
        })
    }
}

async fn handle(mut stream: TcpStream) -> Result<()> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await
        .context("read header error")?;
    let (version, auth_method_count) = (header[0], header[1]);
    if version != 0x5 || auth_method_count <= 0 {
        //stream.shutdown(Shutdown::Both)?;
        return Err(errlog!("Not a valid socks5 connection"));
    }
    let supported_auths = vec![0x00, 0x02];
    let mut allowed_auths: Vec<u8> = vec![];
    for _ in 0..auth_method_count {
        let mut auth_method = [0u8; 1];
        stream.read_exact(&mut auth_method).await?;
        if supported_auths.contains(&auth_method[0]) {
            allowed_auths.push(auth_method[0]);
        }
    }
    println!("version: {}, method counts: {}, selected authentication: {}", version, auth_method_count, allowed_auths[0]);
    let resp: Vec<u8> = vec![0x05, allowed_auths[0]];
    stream.write_all(&resp).await?;
    let socks5req = Socks5Request::from_stream(&mut stream).await?;
    println!("Socks5 Request: {:?}", socks5req);
    Ok(())
}

pub async fn run(srvconf: RelayerConfig) -> Result<()> {
    let local_addr = srvconf.get_local_addr().parse::<SocketAddr>()
        .context(errlog!("Parse {:?} into local address failed", srvconf))?;
    let mut listener = TcpListener::bind(&local_addr).await
        .context(errlog!("Failed to listen {}", local_addr))?;
    println!("Listening at {} ...", local_addr.to_string());
    loop {
        let (local_stream, local_peer) = listener.accept().await?;
        println!("New connection from {} ...", local_peer.to_string());
        let srvconf = srvconf.clone();
        tokio::spawn(
            async move {
                handle(local_stream).await;
            }
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let srvconf = relayer::load_config(RelayerType::SERVER)?;
    println!("{}", srvconf);
    run(srvconf).await?;
    Ok(())
}
