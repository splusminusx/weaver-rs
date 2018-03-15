use byteorder::{BigEndian, WriteBytesExt};
use futures::future;
use metrics;
use prost::Message;
use protocol::{server, Item, StatsRequest, StatsResponse};
use rocksdb::DB;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::Arc;
use tower_grpc::{Error, Request, Response};

#[derive(Clone, Debug)]
pub struct Weaver {
    pub db: Arc<DB>,
}

impl server::Weaver for Weaver {
    type GetItemsStatsFuture = future::FutureResult<Response<StatsResponse>, Error>;

    fn get_items_stats(&mut self, request: Request<StatsRequest>) -> Self::GetItemsStatsFuture {
        let label = "get_items_stats";
        let timer = metrics::GRPC_MSG_HISTOGRAM_VEC
            .with_label_values(&[label])
            .start_timer();

        debug!("REQUEST = {:?}", request);

        let mut key = vec![];
        key.write_i64::<BigEndian>(request.get_ref().item_id)
            .unwrap();
        let res = match self.db.get(&key) {
            Ok(Some(value)) => future::ok(Response::new(StatsResponse {
                total: 1,
                items: vec![Item::decode(&mut Cursor::new(value.deref())).unwrap()],
            })),
            Ok(None) => future::ok(Response::new(StatsResponse {
                total: 0,
                items: vec![],
            })),
            Err(e) => {
                metrics::GRPC_MSG_FAIL_COUNTER_VEC
                    .with_label_values(&[label])
                    .inc();
                error!("DB Error {}", e);
                future::failed(Error::Inner(()))
            }
        };

        timer.observe_duration();
        res
    }
}
