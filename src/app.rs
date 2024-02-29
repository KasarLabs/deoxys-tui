use starknet::{core::types::SyncStatusType, providers::{jsonrpc::{self, HttpTransport}, Provider, Url}};

pub struct App {
    pub name: String,
    pub should_quit: bool,
    pub data: Metrics,
    radar: Radar,
}

pub struct Metrics {
    pub block_number: Result<u64, String>,
    pub syncing: Result<SyncStatusType, String>
}

impl App {
    pub fn new(name: &str, rpc_endpoint: &str) -> Self {
        Self {
            name: name.to_string(),
            should_quit: false,
            radar: Radar::new(rpc_endpoint),
            data: Metrics {
                block_number: Ok(0),
                syncing: Ok(SyncStatusType::NotSyncing),
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
    async fn get_block_number(&self) -> Result<u64, String> {
        self.rpc_client.block_number().await.map_err(|err| format!("Error: {:?}", err))
    }
    async fn get_syncing(&self) -> Result<SyncStatusType, String> {
        self.rpc_client.syncing().await.map_err(|err| format!("Error: {:?}", err))
    }
}