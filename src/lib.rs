#[macro_use]
extern crate anyhow;

pub mod api;
pub mod config;
pub mod collections;
pub mod media;
pub mod db;
pub mod genres;
pub(crate) mod id;
pub mod kodifs;
pub mod models;
pub mod server;
pub mod util;
pub mod sqlx;
pub mod jvec;
