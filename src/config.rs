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
    pub cachedir: Option<String>,
    pub appdir: PathBuf,
    pub database: String,
    #[serde(default)]
    pub listen: Vec<String>,
    #[serde(default)]
    pub tls_cert: Option<String>,
    #[serde(default)]
    pub tls_key: Option<String>,
    #[serde(default)]
    pub tls_listen: Vec<String>,

    #[serde(default, skip)]
    pub addrs: Vec<SocketAddr>,
    #[serde(default, skip)]
    pub tls_addrs: Vec<SocketAddr>,
}

pub fn from_file(path: &str) -> anyhow::Result<Config> {
    let mut cfg: Config = curlyconf::from_file(path)?;
    if cfg.server.listen.len() == 0 && cfg.server.tls_listen.len() == 0 {
        bail!("{}: no listen addresses configured", path);
    }
    if cfg.server.tls_listen.len() > 0 && (cfg.server.tls_cert.is_none() || cfg.server.tls_key.is_none()) {
        bail!("{}: must set tls_cert and tls_key", path);
    }
    cfg.server.addrs = parse_listeners(&cfg.server.listen).with_context(|| format!("file: {}", path))?;
    cfg.server.tls_addrs = parse_listeners(&cfg.server.tls_listen).with_context(|| format!("file: {}", path))?;
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
