use starknet::core::types::SyncStatusType;
use starknet::providers::jsonrpc::{self, HttpTransport};
use starknet::providers::{Provider, Url};
use sysinfo::System;

pub struct App {
    pub name: String,
    pub should_quit: bool,
    pub data: Metrics,
    radar: Radar,
}

pub struct Metrics {
    pub block_number: Result<u64, String>,
    pub syncing: Result<SyncStatusType, String>,
    pub cpu_name: String,
    pub cpu_usage: Vec<Vec<f64>>,
}

impl App {
    pub fn new(name: &str, rpc_endpoint: &str) -> Self {
        let mut radar = Radar::new(rpc_endpoint);
        let cpu_number = radar.get_cpu_usage().len();
        Self {
            name: name.to_string(),
            should_quit: false,
            radar,
            data: Metrics {
                block_number: Ok(0),
                syncing: Ok(SyncStatusType::NotSyncing),
                cpu_name: "N/A".to_string(),
                cpu_usage: vec![vec![0.; 100]; cpu_number],
            },
        }
    }
    pub async fn update_metrics(&mut self) {
        self.data.syncing = self.radar.get_syncing().await;
        let usages = self.radar.get_cpu_usage();
        self.data.cpu_usage.iter_mut().for_each(|elm| elm.rotate_left(1));
        self.data.cpu_usage.iter_mut().zip(usages).for_each(|(v, val)| {
            let last = v.last_mut().unwrap();
            *last = val as f64;
        });
    }
}

struct Radar {
    rpc_client: jsonrpc::JsonRpcClient<HttpTransport>,
    system: System,
}

impl Radar {
    fn new(jsonrpc_endpoint: &str) -> Self {
        let rpc_provider = jsonrpc::JsonRpcClient::new(HttpTransport::new(Url::parse(jsonrpc_endpoint).unwrap()));
        let sys = System::new();
        Self { rpc_client: rpc_provider, system: sys }
    }
    async fn _get_block_number(&self) -> Result<u64, String> {
        self.rpc_client.block_number().await.map_err(|err| format!("Error: {:?}", err))
    }
    async fn get_syncing(&self) -> Result<SyncStatusType, String> {
        self.rpc_client.syncing().await.map_err(|err| format!("Error: {:?}", err))
    }
    fn get_cpu_usage(&mut self) -> Vec<f32> {
        self.system.refresh_cpu();
        let usages: Vec<f32> = self.system.cpus().into_iter().map(|elm| elm.cpu_usage()).collect();
        usages
    }
}
