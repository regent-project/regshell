use clap::Parser;
use regent_console::commands::ConsoleCommandHandler;
use regent_console::parser::ConsoleParser;
use regent_console::state::ConsoleState;
use regent_sdk::host::hostlist::HostList;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

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

fn main() {
    let args = Args::parse();

    // Get username from environment
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    // Initialize console state
    let mut state = ConsoleState::new(username);

    // Load inventory if provided
    if let Some(inventory_path) = args.inventory {
        match HostList::from_file(&inventory_path) {
            Ok(hostlist) => {
                state.set_hostlist(hostlist);
                println!("Loaded inventory from {}", inventory_path);
            }
            Err(e) => {
                eprintln!("Error loading inventory: {:?}", e);
                eprintln!("Continuing with localhost only...");
            }
        }
    } else {
        // Default to localhost
        let localhost = regent_sdk::host::hosts::Host::from_string("localhost".to_string());
        let hostlist = HostList { hosts: Some(vec![localhost]) };
        state.set_hostlist(hostlist);
    }

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
                    Ok(command) => {
                        match ConsoleCommandHandler::handle(command, &mut state) {
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
                        }
                    }
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
