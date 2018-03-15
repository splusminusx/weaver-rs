#![crate_type = "lib"]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![feature(std_panic, panic_handler)]

extern crate byteorder;
extern crate bytes;
extern crate clap;
extern crate crossbeam_channel;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate kafka;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate rocksdb;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;
extern crate tokio_core;
extern crate toml;
extern crate tower;
extern crate tower_grpc;
extern crate tower_h2;
extern crate uuid;

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.weaver.rs"));
}
pub mod metrics;
pub mod config;
pub mod weaver;
pub mod consumer;
pub mod error;
