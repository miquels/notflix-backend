#[macro_use]
extern crate anyhow;

pub mod api;
pub mod collections;
pub mod config;
pub mod db;
pub mod genres;
pub(crate) mod id;
pub mod jvec;
pub mod kodifs;
pub mod media;
pub mod models;
pub mod server;
pub mod sqlx;
pub mod util;
