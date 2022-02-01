use std::io;
use std::net::{AddrParseError, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use serde::Deserialize;

use crate::collections::Collection;

#[derive(Deserialize)]
pub struct Config {
    pub server: Server,
    #[serde(rename = "collection")]
    pub collections: Vec<Collection>,
}

#[derive(Deserialize)]
pub struct Server {
    #[serde(default)]
    pub cachedir: Option<PathBuf>,
    pub appdir: PathBuf,
    pub database: String,
    #[serde(default)]
    pub tls: bool,
    #[serde(default)]
    pub tls_cert: Option<PathBuf>,
    #[serde(default)]
    pub tls_key: Option<PathBuf>,
    pub listen: Vec<String>,

    #[serde(default, skip)]
    pub addrs: Vec<SocketAddr>,
}

pub fn from_file(path: &str) -> anyhow::Result<Config> {
    let mut cfg: Config = curlyconf::from_file(path)?;
    if cfg.server.listen.len() == 0 {
        bail!("{}: no listen addresses configured", path);
    }
    cfg.server.addrs = parse_listeners(&cfg.server.listen).with_context(|| format!("file: {}", path))?;
    Ok(cfg)
}

fn parse_listener(s: impl Into<String>) -> Result<SocketAddr, AddrParseError> {
    SocketAddr::from_str(&s.into())
}

fn parse_listeners(listeners: &Vec<String>) -> io::Result<Vec<SocketAddr>> {
    let mut res = Vec::new();
    for l in listeners.iter().map(|s| s.as_str()) {
        if l.starts_with(":") {
            let p = (&l[1..])
                .parse::<u16>()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}: {}", l, e)))?;
            res.push(parse_listener(format!("0.0.0.0:{}", p)).unwrap());
            res.push(parse_listener(format!("[::]:{}", p)).unwrap());
        } else if l.starts_with("*:") {
            let a = parse_listener(format!("0.0.0.0:{}", &l[2..]))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}: {}", l, e)))?;
            res.push(a);
        } else {
            let a = parse_listener(l)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}: {}", l, e)))?;
            res.push(a);
        }
    }
    Ok(res)
}
