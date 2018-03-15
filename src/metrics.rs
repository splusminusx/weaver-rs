use futures::future;
use futures::future::Future;
use hyper;
use hyper::header::ContentType;
use hyper::mime::Mime;
use hyper::server::{Request, Response, Service};
use prometheus::*;

lazy_static! {
    pub static ref CONSUME_COUNTER: Counter =
        register_counter!(
            "weaver_event_handled_total",
            "Total number of snapshot task"
        ).unwrap();

    pub static ref GRPC_MSG_HISTOGRAM_VEC: HistogramVec =
        register_histogram_vec!(
            "weaver_grpc_msg_duration_seconds",
            "Bucketed histogram of grpc server messages",
            &["type"],
            exponential_buckets(0.0005, 2.0, 20).unwrap()
        ).unwrap();

    pub static ref GRPC_MSG_FAIL_COUNTER_VEC: CounterVec =
        register_counter_vec!(
            "weaver_grpc_msg_fail_total",
            "Total number of handle grpc message failure",
            &["type"]
        ).unwrap();
}

pub struct MetricService {
    encoder: TextEncoder,
}

impl Service for MetricService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _: Self::Request) -> Self::Future {
        let metric_families = gather();
        let mut buffer = vec![];
        self.encoder.encode(&metric_families, &mut buffer).unwrap();

        Box::new(future::ok(
            Response::new()
                .with_header(ContentType(
                    self.encoder.format_type().parse::<Mime>().unwrap(),
                ))
                .with_body(buffer),
        ))
    }
}

impl MetricService {
    pub fn new() -> MetricService {
        MetricService {
            encoder: TextEncoder::new(),
        }
    }
}
