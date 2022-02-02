#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod api;
pub mod collections;
pub mod config;
pub mod db;
pub mod genres;
pub mod kodifs;
pub mod nfo;
pub mod server;
