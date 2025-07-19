use colored::*;
use std::process::Command;
use tokio;
use tokio::sync::Semaphore;
use std::sync::Arc;
use clap::Parser;

mod config;
use config::Config;

#[derive(Parser)]
#[command(
  name = "mssh",
  about = "Multi-Server SSH CLI - Run commands across multiple servers simultaneously",
  version,
  long_about = r#"
A fast, concurrent SSH command executor for running commands across multiple servers.

CONFIGURATION:
  Create ~/.mssh_groups file with your server groups:

  [web]
  user@server1.example.com
  user@server2.example.com

  [prod:g]
  web
  db

  Groups ending with :g can reference other groups.

EXAMPLES:
  mssh @web ls -la                    # List files on web servers
  mssh @prod ps aux                   # Check processes on production
  mssh @all df -h                     # Check disk usage on all servers
  mssh @staging "find . -name '*.log'" # Find log files on staging
"#
)]
struct Args {
  /// Group name (must start with @)
  #[arg(value_name = "GROUP")]
  group: String,

  /// Command to run on all servers in the group
  #[arg(value_name = "COMMAND", trailing_var_arg = true)]
  command: Vec<String>,
}

#[tokio::main]
async fn main() {
  let args = Args::parse();

  if !args.group.starts_with('@') {
    eprintln!("Error: Group must start with @ (e.g., @web, @prod)");
    std::process::exit(1);
  }

  if args.command.is_empty() {
    eprintln!("Error: No command specified");
    std::process::exit(1);
  }

  let group_name = &args.group[1..]; // Remove the @ prefix

  let config = match Config::load() {
    Ok(config) => config,
    Err(e) => {
      eprintln!("Error loading config: {}", e);
      std::process::exit(1);
    }
  };

  let servers = match config.get_servers_resolved(group_name) {
    Some(servers) => servers,
    None => {
      eprintln!("Group '{}' not found in config", group_name);
      std::process::exit(1);
    }
  };

  let semaphore = Arc::new(Semaphore::new(5)); // Limit to 5 concurrent connections
  let mut handles = vec![];

  for server in servers {
    let command_args = args.command.clone();
    let server_display = server.split('@').nth(1).unwrap_or(&server).to_string();
    let semaphore = semaphore.clone();

    let handle = tokio::spawn(async move {
      let _permit = semaphore.acquire().await.unwrap();

      let output = Command::new("ssh")
        .arg(&server)
        .args(&command_args)
        .output()
        .expect("Failed to execute ssh command");

      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", format!("[{}]", server_display).green().bold());
        println!("{}", stdout);
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{}", format!("[{}] ERROR", server_display).red().bold());
        eprintln!("{}", stderr);
      }
    });
    handles.push(handle);
  }

  for handle in handles {
    handle.await.unwrap();
  }
}
