use dirs;
use serde_json as json;
use serde::{Serialize, Deserialize};
use tokio::net::TcpStream;
use std::fs::File;

pub use anyhow::{Context, Result};

/// anyhow error 封装, 实现了行号打印, 使用示例如下
///
/// - return 返回 anyhow::Result
///
///         return Err(errlog!("Unkown file type"));
///
/// - context 信息
///
///         File:open(filepath).context(errlog!("Cannot open file {}", filepath))?;
///
#[macro_export]
macro_rules! errlog {
    ($msg:literal $(,)?) => {
        anyhow::anyhow!(format!("[{}].[{}]: {}", file!(), line!(), $msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        anyhow::anyhow!(format!("[{}].[{}]: {}", file!(), line!(), format!($fmt, $($arg)*)))
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RelayerConfig {
    pub lhost: String,
    pub lport: u16,
    pub rhost: Option<String>,
    pub rport: Option<u16>,
}

pub enum RelayerType {
    CLIENT,
    SERVER,
}

impl RelayerConfig {
    pub fn get_server_addr(&self) -> Option<String> {
        if let Some(rhost) = self.rhost.as_ref() {
            if let Some(rport) = self.rport.as_ref() {
                return Some(format!("{}:{}", rhost, rport));
            }
        }
        None
    }

    pub fn get_local_addr(&self) -> String {
        format!("{}:{}", self.lhost, self.lport)
    }
}

impl std::fmt::Display for RelayerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let local = format!("local: {}:{}", self.lhost, self.lport);
        if let Some(rhost) = self.rhost.as_ref() {
            if let Some(rport) = self.rport.as_ref() {
                let remote = format!("remote: {}:{}", rhost, rport);
                return write!(f, "{}", format!("{}; remote: {}", local, remote));
            }
        }
        write!(f, "{}", local)
    }
}

pub fn load_config(conftype: RelayerType) -> Result<RelayerConfig> {
    let confpath = dirs::home_dir()
        .context(errlog!("Cannot get home directory!"))?
        .join(".config").join("relayer").join(
            match conftype {
                RelayerType::CLIENT => {
                    "relayc.json"
                },
                RelayerType::SERVER => {
                    "relays.json"
                }
            }
        );
    let config = File::open(&confpath)
        .context(errlog!("Cannot open {:?}", confpath))?;
    let config: RelayerConfig = json::from_reader(config)
        .context(errlog!("Cannot load json from {:?}", confpath))?;
    Ok(config)
}

pub async fn forward(mut srcstream: TcpStream, mut dststream: TcpStream) -> Result<()> {
    let (mut local_recv, mut local_send) = srcstream.split();
    let (mut remote_recv, mut remote_send) = dststream.split();
    let (_remote_bytes_copied, _local_bytes_copied) = futures::join!(
        tokio::io::copy(&mut remote_recv, &mut local_send),
        tokio::io::copy(&mut local_recv, &mut remote_send),
    );
    Ok(())
}
