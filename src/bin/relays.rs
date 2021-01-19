use relayer::{RelayerConfig, RelayerType, Result, Context, errlog};

use std::net::{SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct Handshake<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> Handshake<'a> {
    fn new(stream: &'a mut TcpStream) -> Self {
        Handshake {
            stream
        }
    }

    /*

        第一次握手

        Client request format is

           The format of request data is showed in below

               version(1)|auth_method_count(1)|auth_methods(auth_method_count * 1)

            - `verison`: Version of socks protocol, 0x05 for socks5
            - `auth_method_count`: The number of authentication methods
            - `auth_methods`: List of authentication methods, each holds one byte

                Some common authentication methods are listed below

                - `0x00`: No authentication required
                - `0x01`: GSSAPI(Generic Security Services API)
                - `0x02`: USERNAME/PASSWORD
                - `[0x03, 0x7F]`: IANA assigned. See [socks-methods](https://www.iana.org/assignments/socks-methods/socks-methods.xhtml#socks-methods-1) for more details.
                - `[0x80, 0xFE]`: Reserved for private methods
                - `0xFF`: No acceptable methods

        Server response format is

                version(1)|auth_method(1)|

            - `version`: 0x5
            - `auth_method`: The server select one authentication method for client

    */
    async fn first(&mut self) -> Result<()> {
        let mut header = [0u8; 2];
        self.stream.read_exact(&mut header).await
            .context("read header error")?;
        let (version, auth_method_count) = (header[0], header[1]);
        println!("[1: client -> server]");
        println!("version: {}; auth_method_count: {}", version, auth_method_count);
        if version != 0x5 || auth_method_count <= 0 {
            self.stream.shutdown().await?;
            return Err(
                errlog!("Cannot recognize socks5 version or auth_method_count, shut down the stream")
            );
        }
        let supported_auths = vec![0x00, 0x02];
        let mut allowed_auths: Vec<u8> = vec![];
        for _ in 0..auth_method_count {
            let mut auth_method = [0u8; 1];
            self.stream.read_exact(&mut auth_method).await?;
            println!("method identifier: {}", &auth_method[0]);
            if supported_auths.contains(&auth_method[0]) {
                allowed_auths.push(auth_method[0]);
            }
        }
        println!("[1: server -> client]");
        println!("version: {}, method counts: {}, selected authentication: {}", version, auth_method_count, allowed_auths[0]);
        let resp: Vec<u8> = vec![0x05, allowed_auths[0]];
        self.stream.write_all(&resp).await?;
        Ok(())
    }

    /*

        第二次握手

        Client request format is

            version(1)|cmd(1)|rsv(1)|atyp(1)|addr(?)|port(2)

        - `version`: 0x05
        - `cmd`:

            - 0x01: CONNECT
            - 0x02: BIND
            - 0x03: UDP associate

        - `rsv`: Alwayes be 0x00
        - `atyp`: Address type.

            - 0x01: IPv4, the addres length is always 4 bytes
            - 0x03: Domain name, the first byte of `addr` indicates the length of `addr`
            - 0x04: IPv6, the length of address is always 16 bytes

        - `addr`: desired destination address
        - `port`:  desired destination port in network octet order

        Server response format is

            TODO

    */
    async fn second(&mut self) -> Result<()> {
        let mut buf = [0u8; 4];
        self.stream.read_exact(&mut buf).await?;

        let version = buf[0];
        let cmd = buf[1];
        let reserved = buf[2];
        let addr_type = buf[3];
        let addr = match addr_type {
            0x01 => {
                let mut ipv4 = [0u8; 4];
                self.stream.read_exact(&mut ipv4).await?;
                let addr = &ipv4.iter().map(std::string::ToString::to_string).collect::<Vec<String>>().join(".");
                println!("IPv4: {}", addr);
                ipv4.to_vec()
            },
            0x03 => {
                let mut buf = [0u8; 1];
                self.stream.read_exact(&mut buf).await?;
                let domain = vec![0u8; buf[0] as usize];
                println!("Domain: {}", String::from_utf8_lossy(&domain).to_string());
                domain.to_vec()
            },
            0x04 => {
                let mut ipv6 = [0u8; 16];
                self.stream.read_exact(&mut ipv6).await?;
                println!("IPv6: TODO(...)");
                ipv6.to_vec()
            },
            _ => panic!(),
        };

        let mut port = [0u8; 2];
        self.stream.read_exact(&mut port).await?;
        let port = ((port[0] as u16) << 8) | port[1] as u16;

        println!("[2: client -> server]");
        println!("version: {}; cmd: {}; reserved: {}", version, cmd, reserved);
        println!("addr_type: {}; addr: {:?}; port: {}", addr_type, addr, port);
        Ok(())
    }
}

async fn handle(mut stream: TcpStream) -> Result<()> {
    let mut handshake = Handshake::new(&mut stream);
    handshake.first().await?;
    handshake.second().await?;
    Ok(())
}

pub async fn run(srvconf: RelayerConfig) -> Result<()> {
    let local_addr = srvconf.get_local_addr().parse::<SocketAddr>()
        .context(errlog!("Parse {:?} into local address failed", srvconf))?;
    let listener = TcpListener::bind(&local_addr).await
        .context(errlog!("Failed to listen {}", local_addr))?;
    println!("[+] Listening at {} ...", local_addr.to_string());
    loop {
        let (local_stream, local_peer) = listener.accept().await?;
        println!("[+] New connection from {} ...", local_peer.to_string());
        tokio::spawn(
            async move {
                if let Err(e) = handle(local_stream).await {
                    println!("Close connection from {}, caused by {} ...", {local_peer}, e);
                };
            }
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let srvconf = relayer::load_config(RelayerType::SERVER)?;
    run(srvconf).await?;
    Ok(())
}
