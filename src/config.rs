#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct KafkaConsumerConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    pub group_id: String,
    pub fetch_max_wait_time_millis: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub ttl_seconds: u32,
    pub data_path: String,
    pub consumer: KafkaConsumerConfig,
    pub queue_size: usize,
    pub worker_count: usize,
    pub rpc_port: u16,
    pub health_check_port: u16,
}
