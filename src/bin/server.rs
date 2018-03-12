extern crate clap;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate rocksdb;
extern crate tokio_core;
extern crate toml;
extern crate tower_grpc;
extern crate tower_h2;
extern crate weaver;

use clap::{App, Arg};
use futures::{Future, Stream};
use rocksdb::DB;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddrV4;
use std::panic;
use std::process;
use std::sync::Arc;
use std::thread;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tower_h2::Server;
use weaver::config::ServerConfig;
use weaver::consumer::{KafkaItemConsumer, Runnable};
use weaver::error::*;
use weaver::protocol::server;
use weaver::weaver::Weaver;

pub fn main() {
    env_logger::init();
    info!("Logger Initialized!");

    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        error!("Exiting by panic hook!");
        process::exit(1);
    }));

    let matches = App::new("Weaver")
        .version("0.1.0")
        .author("Roman M. <splusminusx@gmail.com>")
        .about("Aggregates and exposes metrics.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches
        .value_of("config")
        .map(|path| {
            File::open(&path)
                .map_err::<Error, _>(|e| e.into())
                .and_then(|mut f| {
                    let mut s = String::new();
                    f.read_to_string(&mut s)?;
                    let c: ServerConfig = toml::from_str(&s)?;
                    Ok(c)
                })
                .unwrap_or_else(|e| {
                    error!("invalid configuration file {:?}: {}", path, e);
                    process::exit(1);
                })
        })
        .unwrap();
    info!("Using config {:?}", config);

    let db = Arc::new(DB::open_default(&config.data_path).unwrap());

    let mut core = Core::new().unwrap();
    let reactor = core.handle();

    let weaver = server::WeaverServer::new(Weaver { db: db.clone() });
    let consumer = KafkaItemConsumer::new(config.queue_size, db.clone(), config.consumer.clone());

    let worker_count = config.worker_count;
    thread::spawn(move || consumer.run(worker_count));

    let h2 = Server::new(weaver, Default::default(), reactor.clone());

    let addr = SocketAddrV4::new("0.0.0.0".parse().unwrap(), config.rpc_port);
    let bind = TcpListener::bind(&addr.into(), &reactor).expect("bind");

    let serve = bind.incoming()
        .fold((h2, reactor), |(h2, reactor), (sock, _)| {
            if let Err(e) = sock.set_nodelay(true) {
                return Err(e);
            }

            let serve = h2.serve(sock);
            reactor.spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));

            Ok((h2, reactor))
        });

    core.run(serve).unwrap();
}
