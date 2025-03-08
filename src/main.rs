use clap::{Parser, Subcommand};
use env_logger::Env;
use log::info;
use anyhow::Result;

// Import local modules
mod client;
mod server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// NATS server URL
    #[arg(short, long, value_name = "URL")]
    nats_url: Option<String>,
    
    /// Subject prefix for NATS messages
    #[arg(short, long, value_name = "PREFIX")]
    subject_prefix: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in server mode (support provider)
    Server {},
    
    /// Run in client mode (support recipient)
    Client {
        /// Override the auto-generated client ID
        #[arg(short, long, value_name = "ID")]
        client_id: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Server {} => {
            info!("Starting in server mode");
            let server = server::Server::new(
                cli.nats_url.as_deref(),
                cli.subject_prefix.as_deref(),
            ).await?;
            
            server.run().await?;
        },
        Commands::Client { client_id } => {
            info!("Starting in client mode");
            let client = client::SupportClient::new(
                cli.nats_url.as_deref(),
                cli.subject_prefix.as_deref(),
                client_id.as_deref(),
            ).await?;
            
            client.run().await?;
        }
    }
    
    Ok(())
}