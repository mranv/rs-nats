use rs_nats_lib::{Command, CommandResult, DEFAULT_NATS_URL, DEFAULT_SUBJECT_PREFIX, RsNatsError, SystemInfo};
use anyhow::Result;
use async_nats::Client;
use log::{info, warn};
use futures_util::stream::StreamExt;
use serde_json::{from_slice, to_string};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

pub struct Server {
    nats_client: Client,
    subject_prefix: String,
    connected_clients: Arc<RwLock<HashMap<String, SystemInfo>>>,
}

impl Server {
    pub async fn new(nats_url: Option<&str>, subject_prefix: Option<&str>) -> Result<Self> {
        let url = nats_url.unwrap_or(DEFAULT_NATS_URL);
        let prefix = subject_prefix.unwrap_or(DEFAULT_SUBJECT_PREFIX).to_string();
        
        info!("Connecting to NATS server at {}", url);
        let nats_client = async_nats::connect(url).await.map_err(|e| {
            RsNatsError::ConnectionError(format!("Failed to connect to NATS: {}", e))
        })?;
        
        Ok(Self {
            nats_client,
            subject_prefix: prefix,
            connected_clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<bool>(1);
        
        // Subscribe to client registration
        let reg_subject = format!("{}.register", self.subject_prefix);
        let registration_subscription = self.nats_client.subscribe(reg_subject).await?;
        
        info!("Server started, waiting for client connections");
        
        // Handle client registrations
        let clients = self.connected_clients.clone();
        let nats = self.nats_client.clone();
        let prefix = self.subject_prefix.clone();
        
        tokio::spawn(async move {
            let mut reg_stream = registration_subscription;
            while let Some(msg) = reg_stream.next().await {
                match from_slice::<SystemInfo>(&msg.payload) {
                    Ok(system_info) => {
                        let client_id = msg.reply.clone().unwrap_or_default();
                        
                        info!("New client connected: {} ({})", client_id, system_info.hostname);
                        
                        // Store client info
                        {
                            let mut clients_map = clients.write().unwrap();
                            clients_map.insert(client_id.clone(), system_info.clone());
                        }
                        
                        // Reply to client with acknowledgment
                        if let Some(reply) = msg.reply {
                            let _ = nats.publish(reply, "ACK".into()).await;
                        }
                        
                        // Subscribe to client response channel
                        let response_subject = format!("{}.response.{}", prefix, client_id);
                        let response_subscription = nats.subscribe(response_subject).await.unwrap();
                        
                        // Handle responses from this specific client
                        tokio::spawn(async move {
                            let mut resp_stream = response_subscription;
                            while let Some(response_msg) = resp_stream.next().await {
                                match from_slice::<CommandResult>(&response_msg.payload) {
                                    Ok(result) => {
                                        info!("Received response from {}: {}", client_id, 
                                            if result.success { "Success" } else { "Failed" });
                                        
                                        println!("\nClient: {}", client_id);
                                        println!("Command result: {}", if result.success { "Success" } else { "Failed" });
                                        println!("Output:\n{}", result.output);
                                        if let Some(err) = result.error {
                                            println!("Error: {}", err);
                                        }
                                    },
                                    Err(e) => {
                                        warn!("Failed to parse response from {}: {}", client_id, e);
                                    }
                                }
                            }
                        });
                    },
                    Err(e) => {
                        warn!("Failed to parse client registration: {}", e);
                    }
                }
            }
        });
        
        // Handle interactive console
        let clients = self.connected_clients.clone();
        let nats = self.nats_client.clone();
        let prefix = self.subject_prefix.clone();
        let shutdown_tx_clone = shutdown_tx.clone();
        
        tokio::spawn(async move {
            loop {
                println!("\nAvailable commands:");
                println!("  list                - List connected clients");
                println!("  execute <id> <cmd>  - Execute command on client");
                println!("  sysinfo <id>        - Get system info from client");
                println!("  ping <id>           - Ping client");
                println!("  exit                - Exit server");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                
                match parts[0] {
                    "list" => {
                        let clients_map = clients.read().unwrap();
                        if clients_map.is_empty() {
                            println!("No clients connected");
                        } else {
                            println!("Connected clients:");
                            for (id, info) in clients_map.iter() {
                                println!("  {} - {} ({} / {})", 
                                    id, info.hostname, info.username, info.os_type);
                            }
                        }
                    },
                    "execute" => {
                        if parts.len() < 3 {
                            println!("Usage: execute <client_id> <command>");
                            continue;
                        }
                        
                        let client_id = parts[1];
                        let command = parts[2..].join(" ");
                        
                        {
                            let clients_map = clients.read().unwrap();
                            if !clients_map.contains_key(client_id) {
                                println!("Client {} not found", client_id);
                                continue;
                            }
                        }
                        
                        let command_subject = format!("{}.command.{}", prefix, client_id);
                        let cmd = Command::Execute(command.clone());
                        
                        match to_string(&cmd) {
                            Ok(json) => {
                                println!("Executing command on {}: {}", client_id, command);
                                let _ = nats.publish(command_subject, json.into()).await;
                            },
                            Err(e) => {
                                println!("Failed to serialize command: {}", e);
                            }
                        }
                    },
                    "sysinfo" => {
                        if parts.len() < 2 {
                            println!("Usage: sysinfo <client_id>");
                            continue;
                        }
                        
                        let client_id = parts[1];
                        
                        {
                            let clients_map = clients.read().unwrap();
                            if !clients_map.contains_key(client_id) {
                                println!("Client {} not found", client_id);
                                continue;
                            }
                        }
                        
                        let command_subject = format!("{}.command.{}", prefix, client_id);
                        let cmd = Command::GetSystemInfo;
                        
                        match to_string(&cmd) {
                            Ok(json) => {
                                println!("Requesting system info from {}", client_id);
                                let _ = nats.publish(command_subject, json.into()).await;
                            },
                            Err(e) => {
                                println!("Failed to serialize command: {}", e);
                            }
                        }
                    },
                    "ping" => {
                        if parts.len() < 2 {
                            println!("Usage: ping <client_id>");
                            continue;
                        }
                        
                        let client_id = parts[1];
                        
                        {
                            let clients_map = clients.read().unwrap();
                            if !clients_map.contains_key(client_id) {
                                println!("Client {} not found", client_id);
                                continue;
                            }
                        }
                        
                        let command_subject = format!("{}.command.{}", prefix, client_id);
                        let cmd = Command::Ping;
                        
                        match to_string(&cmd) {
                            Ok(json) => {
                                println!("Pinging client {}", client_id);
                                let _ = nats.publish(command_subject, json.into()).await;
                            },
                            Err(e) => {
                                println!("Failed to serialize command: {}", e);
                            }
                        }
                    },
                    "exit" => {
                        println!("Shutting down server...");
                        let _ = shutdown_tx_clone.send(true).await;
                        break;
                    },
                    _ => {
                        println!("Unknown command: {}", parts[0]);
                    }
                }
            }
        });
        
        // Wait for shutdown signal
        let _ = shutdown_rx.recv().await;
        info!("Server shutting down");
        
        Ok(())
    }
}