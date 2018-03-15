use byteorder::{BigEndian, WriteBytesExt};
use config::KafkaConsumerConfig;
use crossbeam_channel::{Receiver, Sender};
use crossbeam_channel::{bounded, unbounded};
use error::Result;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage, MessageSets};
use metrics;
use prost::Message;
use protocol;
use rocksdb::{Writable, DB};
use std::io::Cursor;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub trait Runnable {
    fn run(&self, worker_count: usize) -> ();
}

#[derive(Debug)]
pub struct KafkaItemConsumer {
    tx_input: Sender<MessageSets>,
    rx_input: Receiver<MessageSets>,
    tx_commit: Sender<MessageSets>,
    rx_commit: Receiver<MessageSets>,
    queue_size: usize,
    db: Arc<DB>,
    config: KafkaConsumerConfig,
}

impl KafkaItemConsumer {
    pub fn new(queue_size: usize, db: Arc<DB>, config: KafkaConsumerConfig) -> KafkaItemConsumer {
        let (tx_input, rx_input) = bounded::<MessageSets>(queue_size);
        let (tx_commit, rx_commit) = unbounded::<MessageSets>();

        KafkaItemConsumer {
            tx_input,
            rx_input,
            tx_commit,
            rx_commit,
            queue_size,
            db,
            config,
        }
    }

    fn consume_messages(
        config: KafkaConsumerConfig,
        tx_input: Sender<MessageSets>,
        rx_commit: Receiver<MessageSets>,
    ) -> Result<()> {
        let mut con = (Consumer::from_hosts(config.brokers)
            .with_topic(config.topic)
            .with_group(config.group_id)
            .with_fetch_max_wait_time(Duration::from_millis(config.fetch_max_wait_time_millis))
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create())?;

        loop {
            let mss = con.poll()?;
            tx_input.send(mss).unwrap();
            for x in rx_commit.try_iter() {
                for ms in x.iter() {
                    con.consume_messageset(ms).unwrap();
                }
            }
            con.commit_consumed()?;
        }
    }
}

impl Runnable for KafkaItemConsumer {
    fn run(&self, worker_count: usize) -> () {
        let mut handles = vec![];

        for i in 0..worker_count {
            let input = self.rx_input.clone();
            let commit = self.tx_commit.clone();
            let db = self.db.clone();

            handles.push(thread::spawn(move || loop {
                let mss = input.recv().unwrap();
                for ms in mss.iter() {
                    for m in ms.messages() {
                        metrics::CONSUME_COUNTER.inc();
                        let item = protocol::Item::decode(&mut Cursor::new(m.value)).unwrap();
                        info!("thread {} {:?}", i, item);

                        let mut key = vec![];
                        key.write_i64::<BigEndian>(item.item_id).unwrap();
                        db.put(&key, m.value).unwrap();
                    }
                }
                commit.send(mss).unwrap();
            }))
        }

        let tx_input = self.tx_input.clone();
        let rx_commit = self.rx_commit.clone();
        let config = self.config.clone();

        handles.push(thread::spawn(|| {
            KafkaItemConsumer::consume_messages(config, tx_input, rx_commit).unwrap()
        }));

        for handle in handles {
            if handle.join().is_err() {
                error!("Failed processing messages");
                process::exit(1)
            }
        }
    }
}
