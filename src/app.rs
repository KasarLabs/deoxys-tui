use starknet::{core::types::SyncStatusType, providers::{jsonrpc::{self, HttpTransport}, Provider, Url}};

pub struct App {
    pub name: String,
    pub should_quit: bool,
    pub data: Metrics,
    radar: Radar,
}

pub struct Metrics {
    pub block_number: u64,
    pub syncing: SyncStatusType,
}

impl App {
    pub fn new(name: &str, rpc_endpoint: &str) -> Self {
        Self {
            name: name.to_string(),
            should_quit: false,
            radar: Radar::new(rpc_endpoint),
            data: Metrics {
                block_number: 0,
                syncing: SyncStatusType::NotSyncing,
            }
        }
    }
    pub async fn update_metrics(&mut self) {
        self.data.block_number = self.radar.get_block_number().await;
        self.data.syncing = self.radar.get_syncing().await;
    }
}

struct Radar {
    rpc_client: jsonrpc::JsonRpcClient<HttpTransport>,
}

impl Radar {
    fn new (jsonrpc_endpoint: &str) -> Self {
        let rpc_provider = jsonrpc::JsonRpcClient::new(HttpTransport::new(Url::parse(jsonrpc_endpoint).unwrap()));
        Self {
            rpc_client: rpc_provider,
        }
    }
    async fn get_block_number(&self) -> u64 {
        self.rpc_client.block_number().await.unwrap()
    }
    async fn get_syncing(&self) -> SyncStatusType {
        self.rpc_client.syncing().await.unwrap()
    }
}