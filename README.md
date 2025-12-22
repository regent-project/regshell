# Regent Console

Interactive REPL console for [regent-sdk](https://github.com/regent-project/regent-sdk).

Similar to `ansible-console`, regshell provides an interactive shell for executing ad-hoc tasks on remote hosts with multiplexed parallel execution.

## Features

- **Interactive REPL**: Command-line interface with history and editing support
- **Host Pattern Matching**: Select hosts by address patterns or groups
- **Parallel Execution**: Execute tasks across multiple hosts simultaneously
- **Variable Management**: Set and use templated variables in module parameters
- **Module Invocation**: Direct execution of all regent-sdk modules
- **Inventory Support**: Load hosts from YAML inventory files

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/regshell
```

## Usage

### Starting the Console

```bash
# Start with localhost only
regshell

# Start with inventory file
regshell -i inventory.yaml

# Start with specific host pattern
regshell webservers
```

### Console Commands

```
cd <pattern>           Change to host pattern or group
list                   List hosts in current selection
list groups            List available groups
vars                   Show current variables
set <key> <value>      Set a variable
sudo on|off            Toggle sudo for subsequent commands
forks <num>            Set number of parallel forks
verbosity <num>        Set verbosity level
help, ?                Show this help message
exit, quit             Exit the console
```

### Module Execution

Execute modules directly with key=value parameters:

```bash
# Package management
apt package=nginx state=present
yum package=httpd state=present
dnf package=nginx state=present

# Service management
service name=nginx state=started enabled=true

# Shell commands
command cmd="uptime"

# Connectivity test
ping

# Debug output
debug msg="Hello from regent"

# File editing
lineinfile path=/etc/hosts line="127.0.0.1 localhost"
```

### Example Session

```bash
# Load inventory and navigate
cd webservers
list

# Set variables
set package_name nginx
set service_name nginx

# Execute tasks
sudo on
apt package={{package_name}} state=present
service name={{service_name}} state=started enabled=true

# Verify
command cmd="systemctl status {{service_name}}"
```

## Prompt Format

The prompt shows: `username@pattern (host_count)[f:forks]$`

Example: `dave@webservers (3)[f:5]$`

## Available Modules

- **apt**: Package management (Debian/Ubuntu)
- **yum**: Package management (RHEL/CentOS)
- **dnf**: Package management (Fedora)
- **service**: Service management
- **command**: Execute shell commands
- **ping**: Test connectivity
- **debug**: Display debug information
- **lineinfile**: Edit lines in files

## Dependencies

- [regent-sdk](../regent-sdk): Core automation library
- [rustyline](https://crates.io/crates/rustyline): Line editing and history
- [clap](https://crates.io/crates/clap): Command-line argument parsing

## License

Apache-2.0
