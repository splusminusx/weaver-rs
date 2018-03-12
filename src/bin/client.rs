#![feature(lookup_host)]

extern crate bytes;
extern crate clap;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate log;
extern crate prost;
extern crate tokio_core;
extern crate tower_grpc;
extern crate tower_h2;
extern crate tower_http;
extern crate weaver;

use clap::{App, Arg};
use futures::Future;
use std::net;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tower_grpc::Request;
use tower_h2::client::Connection;
use tower_http::add_origin;
use weaver::error::*;
use weaver::protocol::StatsRequest;
use weaver::protocol::client::Weaver;

pub fn main() {
    env_logger::init();

    let matches = App::new("Weaver Client")
        .version("0.1.0")
        .author("Roman M. <splusminusx@gmail.com>")
        .about("Access aggregated metrics.")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOSTNAME")
                .default_value("localhost")
                .help("Hostname of Weaver's RPC server.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .default_value("5000")
                .help("Port of Weaver's RPC server.")
                .takes_value(true),
        )
        .get_matches();

    let host = matches
        .value_of("host")
        .ok_or::<Error>(ErrorKind::OtherError("Host parameter not present.".to_owned()).into())
        .unwrap();

    let port = matches
        .value_of("port")
        .ok_or::<Error>(ErrorKind::OtherError("Port parameter not present.".to_owned()).into())
        .and_then(|p| p.parse::<u16>().map_err(|e| e.into()))
        .unwrap();

    let mut core = Core::new().unwrap();
    let reactor = core.handle();

    let mut addr = net::lookup_host(host).unwrap().next().unwrap();
    addr.set_port(port);
    let uri: http::Uri = format!("http://{}:{}", host, port).parse().unwrap();

    println!("ADDR = {:?}", addr);

    let client_call = TcpStream::connect(&addr.into(), &reactor)
        .and_then(move |socket| {
            // Bind the HTTP/2.0 connection
            Connection::handshake(socket, reactor).map_err(|_| panic!("failed HTTP/2.0 handshake"))
        })
        .map(move |conn| {
            let conn = add_origin::Builder::new().uri(uri).build(conn).unwrap();

            Weaver::new(conn)
        })
        .and_then(|mut client| {
            client
                .get_items_stats(Request::new(StatsRequest { item_id: 2 }))
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|response| {
            println!("RESPONSE = {:?}", response);
            Ok(())
        })
        .map_err(|e| {
            println!("ERR = {:?}", e);
        });

    core.run(client_call).unwrap();
}
