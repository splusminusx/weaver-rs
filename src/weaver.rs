use byteorder::{BigEndian, WriteBytesExt};
use futures::future;
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
        debug!("REQUEST = {:?}", request);

        let mut key = vec![];
        key.write_i64::<BigEndian>(request.get_ref().item_id)
            .unwrap();
        match self.db.get(&key) {
            Ok(Some(value)) => future::ok(Response::new(StatsResponse {
                total: 1,
                items: vec![Item::decode(&mut Cursor::new(value.deref())).unwrap()],
            })),
            Ok(None) => future::ok(Response::new(StatsResponse {
                total: 0,
                items: vec![],
            })),
            Err(e) => {
                error!("DB Error {}", e);
                future::failed(Error::Inner(()))
            }
        }
    }
}
