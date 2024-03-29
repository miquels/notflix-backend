use std::io;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use poem::{
    listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener},
    Endpoint, EndpointExt, IntoResponse, Request, Response, Result, Route, Server,
};
use poem_openapi::{auth::ApiKey, OpenApiService, SecurityScheme};

use crate::api::Api;
use crate::config::Config;
use crate::db::Db;
use crate::media;
use crate::models;
use crate::util::ok_or_return;

#[derive(Clone)]
pub struct SharedState {
    pub db: Db,
    pub config: Arc<Config>,
}

/// ApiKey authorization
#[derive(SecurityScheme)]
#[oai(type = "api_key", key_name = "X-Session-Id", in = "header", checker = "api_checker")]
pub struct SessionFK(pub models::Session);

/// Cookie authorization
#[derive(SecurityScheme)]
#[oai(type = "api_key", key_name = "x-session-id", in = "cookie", checker = "api_checker")]
pub struct SessionFC(pub models::Session);

async fn api_checker(req: &Request, api_key: ApiKey) -> Option<models::Session> {
    let state = req.data::<SharedState>().unwrap();
    let api_key = api_key.key.as_str();
    let timeout = state.config.session.timeout;
    // println!("api key sent: {:?}", api_key);

    let mut txn = ok_or_return!(state.db.handle.begin().await, |err| {
        log::error!("api_checker: {}", err);
        None
    });

    match models::Session::find(&mut txn, api_key, timeout).await {
        Ok(Some(session)) => {
            if txn.commit().await.is_ok() {
                Some(session)
            } else {
                None
            }
        },
        Ok(None) => {
            let _ = txn.commit().await;
            None
        },
        Err(err) => {
            let _ = txn.rollback().await;
            log::error!("api_checker: {}", err);
            None
        },
    }
}

pub async fn serve(cfg: Config, db: Db) -> anyhow::Result<()> {
    let mut listeners = Vec::new();

    if cfg.server.tls_listen.len() > 0 {
        // Try to read the certificates, just to make sure.
        let tls_cert = cfg.server.tls_cert.clone().unwrap();
        let tls_key = cfg.server.tls_key.clone().unwrap();
        let mut tls_file_state = TlsFileState::new();
        load_tls_config(&mut tls_file_state, &tls_cert, &tls_key, true)
            .await
            .with_context(|| "failed to load certificate")?;

        let (tx, rx) = flume::bounded(cfg.server.tls_listen.len() + 1);

        for addr in cfg.server.tls_addrs.clone().drain(..) {
            listeners.push(TcpListener::bind(addr).rustls(rx.clone().into_stream()).boxed());
        }

        tokio::spawn(async move {
            let mut tls_file_state = TlsFileState::new();
            let mut first = true;
            loop {
                match load_tls_config(&mut tls_file_state, &tls_cert, &tls_key, first).await {
                    Ok(Some(tls_config)) => {
                        if let Err(_) = tx.send_async(tls_config).await {
                            break;
                        }
                    },
                    Ok(None) => {},
                    Err(e) => log::error!("failed to reload certificate: {}", e),
                }
                first = false;
                tokio::time::sleep(Duration::from_secs(600)).await;
            }
        });
    }

    if cfg.server.listen.len() > 0 {
        for addr in cfg.server.addrs.clone().drain(..) {
            listeners.push(TcpListener::bind(addr).boxed());
        }
    }

    let mut listener = listeners.pop().unwrap();
    for l in listeners.drain(..) {
        listener = listener.combine(l).boxed();
    }

    let state = SharedState { db, config: Arc::new(cfg) };

    let api_service = OpenApiService::new(Api::new(state.clone()), "Notflix", "0.1")
        .server("https://mx2.high5.nl:3001/api");
    let ui = api_service.rapidoc();
    // let spec = api_service.spec_endpoint_yaml();
    let media = media::routes();

    let app = Route::new()
        .nest("/api", api_service)
        // .nest("/spec", spec)
        .nest("/media", media)
        .nest("/", ui)
        .around(log)
        .data(state);

    Server::new(listener).run(app).await?;

    Ok(())
}

#[derive(PartialEq)]
struct TlsFileState {
    cert_size: u64,
    cert_time: SystemTime,
    key_size: u64,
    key_time: SystemTime,
}

impl TlsFileState {
    fn new() -> TlsFileState {
        TlsFileState {
            cert_size: 0,
            cert_time: SystemTime::UNIX_EPOCH,
            key_size: 0,
            key_time: SystemTime::UNIX_EPOCH,
        }
    }
}

async fn stat(file: &str, size: &mut u64, time: &mut SystemTime) -> io::Result<()> {
    let meta = tokio::fs::metadata(file).await?;
    *size = meta.len();
    *time = meta.modified()?;
    Ok(())
}

async fn load_tls_config(
    tls_file_state: &mut TlsFileState,
    tls_cert: &str,
    tls_key: &str,
    first: bool,
) -> io::Result<Option<RustlsConfig>> {
    let mut newstate = TlsFileState::new();
    stat(tls_cert, &mut newstate.cert_size, &mut newstate.cert_time).await?;
    stat(tls_key, &mut newstate.key_size, &mut newstate.key_time).await?;
    if *tls_file_state == newstate {
        return Ok(None);
    }
    if !first {
        log::info!("new tls certificate detected - reloading");
    }
    let tls_config = RustlsConfig::new().fallback(
        RustlsCertificate::new()
            .cert(tokio::fs::read(tls_cert).await?)
            .key(tokio::fs::read(tls_key).await?),
    );
    *tls_file_state = newstate;
    Ok(Some(tls_config))
}

async fn log<E: Endpoint>(next: E, req: Request) -> Result<Response> {
    // store request data.
    let start = std::time::Instant::now();
    let now = chrono::Local::now();
    let now = now - chrono::Duration::nanoseconds(now.timestamp_nanos() % 1_000_000_000);
    let pnq = req.uri().path_and_query().map(|p| p.to_string()).unwrap_or(String::from("-"));
    let addr = req.remote_addr().to_string();
    let addr = addr.trim_start_matches("socket://");
    let method = req.method().clone();
    let version = req.version();

    let res = next.call(req).await;

    match res {
        Ok(resp) => {
            // log request + response status / size / elapsed.
            let resp = resp.into_response();
            let size = resp.header("content-length").unwrap_or("-");
            println!(
                "{} {} \"{} {} {:?}\" {} {} {:?}",
                now.to_rfc3339(),
                addr,
                method,
                pnq,
                version,
                resp.status().as_u16(),
                size,
                start.elapsed(),
            );
            Ok(resp)
        },
        Err(err) => {
            println!(
                "{} {} \"{} {} {:?}\" {} - {:?} \"{}\"",
                now.to_rfc3339(),
                addr,
                method,
                pnq,
                version,
                err.status().as_u16(),
                start.elapsed(),
                err
            );
            Err(err)
        },
    }
}
