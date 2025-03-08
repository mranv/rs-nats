use rs_nats_lib::{Command, CommandResult, CommandType, DEFAULT_NATS_URL, DEFAULT_SUBJECT_PREFIX, RsNatsError, SystemInfo, get_client_id, get_os_type, LogLevel};
use anyhow::Result;
use async_nats::Client;
use log::{debug, error, info, warn};
use futures_util::stream::StreamExt;
use serde_json::{from_slice, to_string};
use std::process::Command as ProcessCommand;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct SupportClient {
    nats_client: Client,
    subject_prefix: String,
    client_id: String,
}

impl SupportClient {
    pub async fn new(
        nats_url: Option<&str>, 
        subject_prefix: Option<&str>,
        client_id: Option<&str>,
    ) -> Result<Self> {
        let url = nats_url.unwrap_or(DEFAULT_NATS_URL);
        let prefix = subject_prefix.unwrap_or(DEFAULT_SUBJECT_PREFIX).to_string();
        let id = client_id.map(|s| s.to_string()).unwrap_or_else(get_client_id);
        
        info!("Connecting to NATS server at {}", url);
        let nats_client = async_nats::connect(url).await.map_err(|e| {
            RsNatsError::ConnectionError(format!("Failed to connect to NATS: {}", e))
        })?;
        
        Ok(Self {
            nats_client,
            subject_prefix: prefix,
            client_id: id,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<bool>(1);
        
        // Register with the server - keep trying indefinitely until successful
        info!("Registering with server as {}", self.client_id);
        self.register_with_retry(true).await?;
        
        // Subscribe to commands
        let command_subject = format!("{}.command.{}", self.subject_prefix, self.client_id);
        info!("Subscribing to commands on {}", command_subject);
        
        let command_subscription = self.nats_client.subscribe(command_subject).await?;
        
        let nats = self.nats_client.clone();
        let client_id = self.client_id.clone();
        let prefix = self.subject_prefix.clone();
        let shutdown_tx_clone = shutdown_tx.clone();
        
        // Handle incoming commands
        tokio::spawn(async move {
            let mut command_stream = command_subscription;
            while let Some(msg) = command_stream.next().await {
                match from_slice::<Command>(&msg.payload) {
                    Ok(command) => {
                        info!("Received command: {}", command);
                        
                        let result = match command {
                            Command::Ping => {
                                CommandResult {
                                    success: true,
                                    output: "Pong".to_string(),
                                    error: None,
                                    command_type: CommandType::Internal,
                                }
                            },
                            Command::Execute(cmd) => {
                                execute_command(&cmd)
                            },
                            Command::GetSystemInfo => {
                                let sys_info = get_system_info();
                                // Use serde_json to serialize the system info properly
                                match to_string(&sys_info) {
                                    Ok(json) => {
                                        CommandResult {
                                            success: true,
                                            output: json,
                                            error: None,
                                            command_type: CommandType::Internal,
                                        }
                                    },
                                    Err(e) => {
                                        CommandResult {
                                            success: false,
                                            output: String::new(),
                                            error: Some(format!("Failed to serialize system info: {}", e)),
                                            command_type: CommandType::Internal,
                                        }
                                    }
                                }
                            },
                            Command::Shutdown => {
                                info!("Received shutdown command");
                                let _ = shutdown_tx_clone.send(true).await;
                                CommandResult {
                                    success: true,
                                    output: "Client shutting down".to_string(),
                                    error: None,
                                    command_type: CommandType::Internal,
                                }
                            },
                            Command::LogEvent { level, message } => {
                                match level {
                                    LogLevel::Debug => debug!("{}", message),
                                    LogLevel::Info => info!("{}", message),
                                    LogLevel::Warning => warn!("{}", message),
                                    LogLevel::Error => error!("{}", message),
                                }
                                
                                CommandResult {
                                    success: true,
                                    output: format!("Logged: [{}] {}", level, message),
                                    error: None,
                                    command_type: CommandType::Internal,
                                }
                            }
                        };
                        
                        // Send the result back
                        let response_subject = format!("{}.response.{}", prefix, client_id);
                        match to_string(&result) {
                            Ok(json) => {
                                info!("Sending response to {}: {}", response_subject, json);
                                let send_result = nats.publish(response_subject, json.into()).await;
                                match send_result {
                                    Ok(_) => info!("Successfully sent response"),
                                    Err(e) => error!("Failed to send response: {}", e),
                                }
                            },
                            Err(e) => {
                                error!("Failed to serialize result: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                    }
                }
            }
        });
        
        // Heartbeat to server
        let nats = self.nats_client.clone();
        let client_id = self.client_id.clone();
        let prefix = self.subject_prefix.clone();
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                
                let heartbeat_subject = format!("{}.heartbeat", prefix);
                let _ = nats.publish(heartbeat_subject, client_id.clone().into()).await;
                debug!("Sent heartbeat");
            }
        });
        
        // Wait for shutdown signal
        let _ = shutdown_rx.recv().await;
        info!("Client shutting down");
        
        Ok(())
    }
    
    // Modified to run indefinitely if needed
    async fn register_with_retry(&self, run_indefinitely: bool) -> Result<()> {
        let mut attempts = 0;
        let initial_backoff = Duration::from_secs(2);
        let max_backoff = Duration::from_secs(60); // Cap at 1 minute between retries
        let mut current_backoff = initial_backoff;
        
        loop {
            attempts += 1;
            
            match self.register().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    // Just check error message for "no responders" string
                    let error_msg = e.to_string();
                    let is_no_responders = error_msg.contains("no responders");
                    
                    if is_no_responders {
                        warn!("No server ready yet, retrying in {:?} (attempt {})", 
                            current_backoff, attempts);
                        sleep(current_backoff).await;
                        
                        // Increase backoff for next attempt (exponential backoff)
                        // but cap at max_backoff
                        current_backoff = std::cmp::min(
                            Duration::from_secs_f32(current_backoff.as_secs_f32() * 1.5),
                            max_backoff
                        );
                        
                        // Keep trying indefinitely if specified
                        if run_indefinitely {
                            continue;
                        }
                        
                        // Otherwise give up after 10 attempts
                        if attempts >= 10 {
                            return Err(anyhow::anyhow!("Failed to register after {} attempts: {}", attempts, e));
                        }
                    } else {
                        // If it's some other error, don't retry
                        return Err(anyhow::anyhow!("Registration failed: {}", e));
                    }
                }
            }
        }
    }
    
    async fn register(&self) -> Result<()> {
        let register_subject = format!("{}.register", self.subject_prefix);
        let system_info = get_system_info();
        
        match to_string(&system_info) {
            Ok(json) => {
                // Create headers with client_id
                let mut headers = async_nats::HeaderMap::new();
                headers.insert("client_id", self.client_id.as_str());
                
                // Use request_with_headers with timeout
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    self.nats_client.request_with_headers(
                        register_subject, 
                        headers,
                        json.into()
                    )
                ).await {
                    Ok(resp_result) => {
                        match resp_result {
                            Ok(resp) => {
                                let resp_data = String::from_utf8_lossy(&resp.payload);
                                
                                if resp_data == "ACK" {
                                    info!("Successfully registered with server");
                                } else {
                                    warn!("Unexpected registration response: {}", resp_data);
                                }
                                
                                Ok(())
                            },
                            Err(e) => Err(anyhow::anyhow!("Registration request failed: {}", e))
                        }
                    },
                    Err(_) => Err(anyhow::anyhow!("Registration request timed out"))
                }
            },
            Err(e) => {
                Err(RsNatsError::SerializationError(format!("Failed to serialize system info: {}", e)).into())
            }
        }
    }
}

fn get_system_info() -> SystemInfo {
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
    let username = whoami::username();
    let os_type = get_os_type();
    let os_version = get_os_version();
    
    SystemInfo {
        hostname,
        username,
        os_type,
        os_version,
    }
}

fn get_os_version() -> Option<String> {
    if cfg!(target_os = "windows") {
        // Windows-specific implementation
        let output = ProcessCommand::new("cmd")
            .args(&["/c", "ver"])
            .output();
            
        match output {
            Ok(out) => {
                let version = String::from_utf8_lossy(&out.stdout);
                Some(version.trim().to_string())
            },
            Err(_) => None,
        }
    } else if cfg!(target_os = "linux") {
        // Linux-specific implementation
        let output = ProcessCommand::new("cat")
            .arg("/etc/os-release")
            .output();
            
        match output {
            Ok(out) => {
                let release_info = String::from_utf8_lossy(&out.stdout);
                for line in release_info.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return Some(line.trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string());
                    }
                }
                None
            },
            Err(_) => None,
        }
    } else {
        // Fallback for other platforms
        None
    }
}

fn execute_command(cmd: &str) -> CommandResult {
    let command_result = if cfg!(target_os = "windows") {
        ProcessCommand::new("cmd")
            .args(&["/c", cmd])
            .output()
    } else {
        ProcessCommand::new("sh")
            .args(&["-c", cmd])
            .output()
    };
    
    match command_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            if output.status.success() {
                CommandResult {
                    success: true,
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                    command_type: CommandType::Shell,
                }
            } else {
                CommandResult {
                    success: false,
                    output: stdout,
                    error: Some(stderr),
                    command_type: CommandType::Shell,
                }
            }
        },
        Err(e) => {
            CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute command: {}", e)),
                command_type: CommandType::Shell,
            }
        }
    }
}