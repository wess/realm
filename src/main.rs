use clap::Parser;
use realm::{Cli, CliHandler};
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let handler = match CliHandler::new() {
        Ok(handler) => handler,
        Err(e) => {
            eprintln!("Failed to initialize realm: {}", e);
            process::exit(1);
        }
    };
    
    if let Err(e) = handler.handle_command(cli.command).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
