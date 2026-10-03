# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

regshell is an interactive REPL console for regent-sdk, similar to `ansible-console`. It provides an interactive shell for executing ad-hoc tasks on remote hosts with multiplexed parallel execution.

## Build & Run Commands

### Build
```bash
cargo build --release
```

### Install
```bash
cargo install --path .
```

### Run
```bash
# Start with localhost only
./target/release/regshell

# Start with inventory file
./target/release/regshell -i inventory.yaml

# Start with specific host pattern
./target/release/regshell webservers
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_cd

# Run tests for specific module
cargo test --lib parser
cargo test --lib commands
```

## Architecture

### Core Components

**ConsoleState** (`src/state.rs`): Central state management for the REPL session
- Maintains current host pattern selection and filtered host list
- Manages user-defined variables for task templating
- Stores connection information and execution settings (sudo, forks, verbosity)
- Implements host filtering by pattern matching and group membership
- Generates dynamic prompts showing current context

**ConsoleParser** (`src/parser.rs`): Command parsing and tokenization
- Parses REPL commands (cd, list, set, sudo, etc.)
- Handles module invocation syntax: `module_name key=value key=value`
- Distinguishes between console commands and module execution
- Returns strongly-typed `ConsoleCommand` enum variants

**ConsoleCommandHandler** (`src/commands.rs`): Command execution logic
- Executes console commands by updating state or displaying information
- Converts module invocations to a regent-sdk ExpectedState YAML description
- Enforces the expected state on the selected hosts via regent-sdk, `forks` hosts at a time
- Injects user variables into each host's template context
- Returns formatted output or error results

**Main REPL** (`src/main.rs`): Entry point and interactive loop
- Uses rustyline for line editing and history
- Loads inventory from YAML files or defaults to localhost
- Orchestrates parser → handler → state workflow
- Handles interrupt signals and EOF gracefully

### Data Flow

1. User input → ConsoleParser → ConsoleCommand enum
2. ConsoleCommand + ConsoleState → ConsoleCommandHandler
3. Handler executes command:
   - Console commands: Update state or display info
   - Module commands: Build ExpectedState YAML → Parse via regent-sdk → Enforce on selected hosts → Display results
4. State changes reflected in prompt for next iteration

## Dependencies

**regent-sdk** (crates.io, version 0.12): Core automation library providing:
- ManagedHost and ManagedHostBuilder for host connection management
- ExpectedState and Attribute types for declarative expected states
- Module implementations (apt, yum/dnf, pacman, service, command, ping, debug, lineinfile, user, group, cron, hostname, dns, dhcp, iptables, template, ollama)
- Connection management and SSH handling
- Template rendering and variable substitution

**rustyline**: Provides readline-like functionality for the REPL (history, editing, completions)

**clap**: Command-line argument parsing with derive macros

## Important Design Patterns

### Module Execution Pattern
Module invocations are converted to a regent-sdk ExpectedState YAML description that regent-sdk can parse and enforce:
```yaml
# User input: "apt package=nginx state=present"
# Becomes YAML:
---
Attributes:
  - Name: Console task
    Timeout: 30
    Privilege: !None
    Detail: !Apt
      Package: 'nginx'
      State: 'Present'
```

### Variable Templating
Variables set with `set key value` are injected into each host's template context before execution, enabling template expansion in module parameters (e.g., `package={{package_name}}`).

### Host Filtering
The `cd` command updates `current_pattern`, triggering `update_current_hosts()` which filters the connected hosts:
- Pattern "all" includes all hosts
- Otherwise, matches host ids (substring)

## Console Commands Reference

- `cd <pattern>` - Filter hosts by host id pattern
- `list` - Show filtered hosts in current selection
- `vars` - Display current variable bindings
- `set <key> <value>` - Define variable for template substitution
- `sudo on|off` - Toggle sudo for subsequent module executions
- `forks <num>` - Set parallelism level (default: 5)
- `verbosity <num>` - Set output verbosity (default: 0)
- `help`, `?` - Show help text
- `exit`, `quit` - Exit the console

## Prompt Format

The dynamic prompt shows execution context: `username@pattern (host_count)[f:forks]$`

Example: `dave@webservers (3)[f:5]$` indicates:
- User: dave
- Current pattern: webservers
- 3 hosts match current pattern
- Fork level: 5 parallel executions
