extern crate clap;
extern crate env_logger;
extern crate kafka;
#[macro_use]
extern crate log;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate tower_grpc;

use clap::{App, Arg};
use kafka::producer::{Producer, Record, RequiredAcks};
use prost::Message;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.weaver.rs"));
}

fn main() {
    env_logger::init();

    let matches = App::new("Producer")
        .version("0.1.0")
        .author("Roman M. <splusminusx@gmail.com>")
        .about("Produces items.")
        .arg(
            Arg::with_name("brokers")
                .short("b")
                .long("brokers")
                .value_name("HOST:PORT")
                .help("Coma separated list of Kafka brokers")
                .takes_value(true)
                .default_value("localhost:9092"),
        )
        .arg(
            Arg::with_name("topic")
                .short("t")
                .long("topic")
                .value_name("TOPIC")
                .help("Kafka topic to produce records")
                .takes_value(true)
                .default_value("test"),
        )
        .arg(
            Arg::with_name("sleep")
                .short("s")
                .long("sleep")
                .value_name("MILLIS")
                .help("Sleep duration")
                .takes_value(true)
                .default_value("100"),
        )
        .get_matches();

    let brokers = matches
        .value_of("brokers")
        .map(|b| {
            b.split(",")
                .collect::<Vec<&str>>()
                .iter()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| {
            error!("Incorrect brokers.");
            process::exit(1);
        });

    let sleep_duration: Duration = matches
        .value_of("sleep")
        .ok_or("Sleep timeout should be set.".to_string())
        .and_then(|s| u64::from_str(s).map_err(|e| e.to_string()))
        .map(|s| Duration::from_millis(s))
        .unwrap_or_else(|e| {
            error!("Incorrect brokers. {}", e);
            process::exit(1);
        });

    let topic = matches.value_of("topic").unwrap_or_else(|| {
        error!("Incorrect topic.");
        process::exit(1);
    });

    let mut producer = Producer::from_hosts(brokers)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();

    let mut id = 0;
    loop {
        id += 1;
        info!("About to publish a message {}", id);
        producer.send(&to_record(topic, new_item(id))).unwrap();
        thread::sleep(sleep_duration);
    }
}

fn new_item(id: i64) -> protocol::Item {
    protocol::Item {
        item_id: id,
        description: "Unique item".to_owned(),
        counter: 7,
        percent: 0.6,
        item_type: protocol::item::ItemType::Single as i32,
    }
}

fn to_record(topic: &str, item: protocol::Item) -> Record<(), Vec<u8>> {
    let mut buf = Vec::new();
    buf.reserve(item.encoded_len());
    item.encode(&mut buf).unwrap();
    Record::from_value(topic, buf)
}
