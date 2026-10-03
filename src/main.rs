use clap::Parser;
use regent_sdk::hosts::handlers::{ConnectionMethod, TargetUser};
use regent_sdk::hosts::managed_host::ManagedHost;
use regent_sdk::hosts::managed_host::ManagedHostBuilder;
use regent_sdk::secrets::{SecretProvider, SecretProvidersPoolBuilder};
use regshell::commands::ConsoleCommandHandler;
use regshell::parser::ConsoleParser;
use regshell::state::ConsoleState;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "regshell")]
#[command(about = "Interactive REPL console for regent-sdk", long_about = None)]
struct Args {
    /// Inventory file (YAML format)
    #[arg(short = 'i', long = "inventory")]
    inventory: Option<String>,

    /// Initial host pattern
    #[arg(value_name = "PATTERN")]
    pattern: Option<String>,
}

/// Subset of the regent-sdk inventory YAML schema needed by the console.
///
/// regent-sdk inventories expose no per-host accessors, so regshell builds and
/// owns its own pool of connected ManagedHost hosts.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ConsoleInventory {
    #[serde(default)]
    default_connection_method: Option<ConnectionMethod>,
    hosts: Vec<ManagedHostBuilder>,
}

fn build_secrets_pool() -> Result<regent_sdk::secrets::SecretProvidersPool, String> {
    SecretProvidersPoolBuilder::new()
        .add_default_provider("env_var", SecretProvider::env_var())
        .add_provider("file", SecretProvider::files())
        .build()
        .map_err(|details| format!("Failed to build secret providers pool: {}", details))
}

async fn connect_inventory(raw_yaml: &str) -> Result<HashMap<String, ManagedHost>, String> {
    let inventory: ConsoleInventory = yaml_serde::from_str(raw_yaml)
        .map_err(|details| format!("Invalid inventory: {:?}", details))?;

    let secrets_pool = build_secrets_pool()?;
    let mut hosts = HashMap::new();

    for mut builder in inventory.hosts {
        if builder.host_connection_method.is_none() {
            match &inventory.default_connection_method {
                Some(connection_method) => {
                    builder.set_connection_method(connection_method.clone());
                }
                None => {
                    return Err(format!(
                        "No HostConnectionMethod or DefaultConnectionMethod set for host '{}'",
                        builder.id()
                    ));
                }
            }
        }

        let mut managed_host = builder
            .build(Some(secrets_pool.clone()))
            .await
            .map_err(|details| format!("Failed to build host '{}': {}", "host", details))?;

        managed_host.connect().await.map_err(|details| {
            format!("Failed to connect to '{}': {}", managed_host.id(), details)
        })?;

        hosts.insert(managed_host.id().to_string(), managed_host);
    }

    Ok(hosts)
}

async fn connect_default_localhost() -> Result<HashMap<String, ManagedHost>, String> {
    let builder = ManagedHostBuilder::new(
        "localhost",
        "localhost",
        Some(ConnectionMethod::Localhost(TargetUser::current_user())),
    );

    let mut localhost = builder
        .build(None)
        .await
        .map_err(|details| format!("Failed to build localhost: {}", details))?;

    localhost
        .connect()
        .await
        .map_err(|details| format!("Failed to connect to localhost: {}", details))?;

    Ok(HashMap::from([("localhost".to_string(), localhost)]))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Get username from environment
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    // Initialize console state
    let mut state = ConsoleState::new(username);

    // Load hosts from inventory if provided, default to localhost otherwise
    let hosts = match &args.inventory {
        Some(inventory_path) => match fs::read_to_string(inventory_path) {
            Ok(raw_yaml) => match connect_inventory(&raw_yaml).await {
                Ok(hosts) => {
                    println!(
                        "Loaded inventory from {} ({} hosts)",
                        inventory_path,
                        hosts.len()
                    );
                    hosts
                }
                Err(details) => {
                    eprintln!("Error loading inventory: {}", details);
                    eprintln!("Continuing with localhost only...");
                    connect_default_localhost().await.unwrap_or_else(|details| {
                        eprintln!("Error connecting to localhost: {}", details);
                        std::process::exit(1);
                    })
                }
            },
            Err(details) => {
                eprintln!("Error reading inventory file: {}", details);
                eprintln!("Continuing with localhost only...");
                connect_default_localhost().await.unwrap_or_else(|details| {
                    eprintln!("Error connecting to localhost: {}", details);
                    std::process::exit(1);
                })
            }
        },
        None => connect_default_localhost().await.unwrap_or_else(|details| {
            eprintln!("Error connecting to localhost: {}", details);
            std::process::exit(1);
        }),
    };

    state.set_hosts(hosts);

    // Set initial pattern if provided
    if let Some(pattern) = args.pattern {
        state.set_pattern(&pattern);
    }

    // Print welcome message
    println!("Welcome to the Regent Console.");
    println!("Type 'help' or '?' to list commands.\n");

    // Initialize readline
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Error initializing readline: {}", e);
            return;
        }
    };

    // REPL loop
    loop {
        let prompt = state.get_prompt();

        match rl.readline(&prompt) {
            Ok(line) => {
                // Add to history
                let _ = rl.add_history_entry(&line);

                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                // Parse and execute command
                match ConsoleParser::parse(&line) {
                    Ok(command) => match ConsoleCommandHandler::handle(command, &mut state).await {
                        Ok(output) => {
                            if output == "exit" {
                                println!("Goodbye!");
                                break;
                            }
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {:?}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Parse error: {:?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
