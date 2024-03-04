use starknet::core::types::SyncStatusType;
use starknet::providers::jsonrpc::{self, HttpTransport};
use starknet::providers::{Provider, Url};
use sysinfo::System;

pub struct App {
    pub should_quit: bool,
    pub data: Metrics,
    radar: Radar,
}

pub struct Metrics {
    pub block_number: Result<u64, String>,
    pub syncing: Result<SyncStatusType, String>,
    pub cpu_name: String,
    pub cpu_usage: Vec<f64>, // BALISE: N2: faire des vecteurs d'option: ratatui le gère de base
    pub memory_usage: Vec<u64>,
    pub total_memory: u64,
}

impl App {
    pub fn new(process_name: &str, rpc_endpoint: &str) -> Result<Self, String> {
        let mut radar = Radar::new(rpc_endpoint, process_name)?;
        let total_memory = radar.get_total_system_memory();
        Ok(Self {
            should_quit: false,
            radar,
            data: Metrics {
                block_number: Ok(0),
                syncing: Ok(SyncStatusType::NotSyncing),
                cpu_name: "N/A".to_string(),
                cpu_usage: vec![0.; 100],   // Le nombre de point doit être réglable: BALISE: N0
                memory_usage: vec![0; 100], // idem BALISE: N1
                total_memory,
            },
        })
    }
    pub async fn update_metrics(&mut self) {
        self.radar.snapshot();
        self.data.syncing = self.radar.get_syncing().await;

        self.data.cpu_usage.rotate_left(1);
        self.data.cpu_usage[99] = self.radar.get_cpu_usage().unwrap_or(0.); //BALISE: N0

        self.data.memory_usage.rotate_left(1);
        self.data.memory_usage[99] = self.radar.get_memory_usage().unwrap_or(0); //BALISE: N2
    }
}

struct Radar {
    rpc_client: jsonrpc::JsonRpcClient<HttpTransport>,
    system: System,
    process_name: String,
}

impl Radar {
    fn new(jsonrpc_endpoint: &str, process_name: &str) -> Result<Self, String> {
        let url = Url::parse(jsonrpc_endpoint).map_err(|_| "Error: Not a Valid URL for RPC endpoint")?;
        let rpc_provider = jsonrpc::JsonRpcClient::new(HttpTransport::new(url));
        let sys = System::new();

        Ok(Self { rpc_client: rpc_provider, system: sys, process_name: process_name.to_string() })
    }
    async fn _get_block_number(&self) -> Result<u64, String> {
        self.rpc_client.block_number().await.map_err(|err| format!("Error: {:?}", err))
    }
    async fn get_syncing(&self) -> Result<SyncStatusType, String> {
        self.rpc_client.syncing().await.map_err(|err| format!("Error: {:?}", err))
    }
    fn snapshot(&mut self) {
        self.system.refresh_processes();
    }
    fn get_cpu_usage(&mut self) -> Option<f64> {
        match self.system.processes_by_exact_name(&self.process_name).next() {
            Some(target) => Some(target.cpu_usage() as f64),
            _ => None,
        }
    }
    fn get_memory_usage(&mut self) -> Option<u64> {
        match self.system.processes_by_exact_name(&self.process_name).next() {
            Some(target) => Some(target.memory()),
            _ => None,
        }
    }
    fn get_total_system_memory(&mut self) -> u64 {
        self.system.refresh_all(); //BALISE: si appellée plusieurs fois: refresh que les infos memoire
        self.system.total_memory() as u64
    }
}
