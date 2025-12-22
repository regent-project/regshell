use crate::parser::ConsoleCommand;
use crate::state::ConsoleState;
use regent_sdk::error::Error;
use regent_sdk::job::joblist::JobList;
use regent_sdk::task::tasklist::TaskListFileType;

pub struct ConsoleCommandHandler;

impl ConsoleCommandHandler {
    pub fn handle(command: ConsoleCommand, state: &mut ConsoleState) -> Result<String, Error> {
        match command {
            ConsoleCommand::Cd(pattern) => {
                state.set_pattern(&pattern);
                Ok(format!(
                    "Changed to pattern '{}' ({} hosts)",
                    pattern,
                    state.get_host_count()
                ))
            }
            ConsoleCommand::List => {
                let mut output = String::new();
                for host in &state.current_hosts {
                    output.push_str(&format!("{}\n", host.address));
                }
                if output.is_empty() {
                    output = "No hosts match current pattern".to_string();
                }
                Ok(output)
            }
            ConsoleCommand::ListGroups => {
                let groups = state.list_groups();
                let mut output = String::new();
                for group in groups {
                    output.push_str(&format!("{}\n", group));
                }
                if output.is_empty() {
                    output = "No groups found".to_string();
                }
                Ok(output)
            }
            ConsoleCommand::Help => Ok(Self::help_text()),
            ConsoleCommand::Exit => Ok("exit".to_string()),
            ConsoleCommand::Vars => Ok(state.get_variables_display()),
            ConsoleCommand::Set(key, value) => {
                state.set_variable(&key, &value);
                Ok(format!("Set {} = {}", key, value))
            }
            ConsoleCommand::Sudo(enabled) => {
                state.with_sudo = enabled;
                Ok(format!(
                    "Sudo {}",
                    if enabled { "enabled" } else { "disabled" }
                ))
            }
            ConsoleCommand::Forks(num) => {
                state.forks = num;
                Ok(format!("Forks set to {}", num))
            }
            ConsoleCommand::Verbosity(level) => {
                state.verbosity = level;
                Ok(format!("Verbosity set to {}", level))
            }
            ConsoleCommand::Module(module_name, params) => {
                Self::execute_module(module_name, params, state)
            }
        }
    }

    fn execute_module(
        module_name: String,
        params: Vec<(String, String)>,
        state: &ConsoleState,
    ) -> Result<String, Error> {
        if state.current_hosts.is_empty() {
            return Err(Error::MissingInitialization(
                "No hosts selected. Use 'cd' to select hosts.".to_string(),
            ));
        }

        // Build a YAML task from the module invocation
        let yaml_task = Self::build_yaml_task(&module_name, &params, state.with_sudo);

        // Create a JobList from current hosts
        let mut joblist = JobList::new();
        
        for host in &state.current_hosts {
            let mut job = regent_sdk::job::job::Job::from_host(host.clone());
            job.set_connection(state.connection_info.clone())?;
            
            // Add variables from state
            for (key, value) in &state.variables {
                job.add_var(key, value);
            }
            
            // Set the tasklist from the generated YAML
            job.set_tasklist_from_str(&yaml_task, TaskListFileType::Yaml)?;
            
            joblist.add_job(job);
        }

        // Execute the task
        joblist.apply();

        // Display results
        Ok(joblist.display_pretty())
    }

    fn build_yaml_task(module_name: &str, params: &[(String, String)], with_sudo: bool) -> String {
        let mut yaml = String::from("---\n");
        yaml.push_str("- name: Console task\n");
        yaml.push_str("  steps:\n");
        yaml.push_str("    - name: Execute module\n");
        
        if with_sudo {
            yaml.push_str("      with_sudo: true\n");
        }
        
        yaml.push_str(&format!("      {}:\n", module_name));
        
        for (key, value) in params {
            yaml.push_str(&format!("        {}: '{}'\n", key, value));
        }
        
        yaml
    }

    fn help_text() -> String {
        r#"Regent Console - Interactive REPL for regent-sdk

Commands:
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

Module Execution:
  <module> key=value ... Execute a module with parameters
  
  Available modules:
    apt                  Package management (Debian/Ubuntu)
    yum                  Package management (RHEL/CentOS)
    dnf                  Package management (Fedora)
    service              Service management
    command              Execute shell commands
    ping                 Test connectivity
    debug                Display debug information
    lineinfile           Edit lines in files

Examples:
  cd webservers
  apt package=nginx state=present
  service name=nginx state=started enabled=true
  command cmd="uptime"
  set my_var my_value
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regent_sdk::connection::host_connection::HostConnectionInfo;
    use regent_sdk::host::hostlist::HostList;
    use regent_sdk::host::hosts::Host;

    fn create_test_state() -> ConsoleState {
        let mut state = ConsoleState::new("testuser".to_string());
        
        // Create a simple hostlist with test hosts
        let mut hostlist = HostList { hosts: Some(Vec::new()) };
        
        let mut host1 = Host::from_string("192.168.1.10".to_string());
        host1.add_to_group(&"webservers".to_string());
        
        let mut host2 = Host::from_string("192.168.1.20".to_string());
        host2.add_to_group(&"webservers".to_string());
        
        let mut host3 = Host::from_string("192.168.1.30".to_string());
        host3.add_to_group(&"databases".to_string());
        
        hostlist.hosts = Some(vec![host1, host2, host3]);
        state.set_hostlist(hostlist);
        state.connection_info = HostConnectionInfo::localhost_current_user();
        
        state
    }

    #[test]
    fn test_handle_cd_command_updates_state() {
        let mut state = create_test_state();
        
        // Initially should be "all" pattern with 3 hosts
        assert_eq!(state.current_pattern, "all");
        assert_eq!(state.get_host_count(), 3);
        
        // Change to webservers pattern
        let command = ConsoleCommand::Cd("webservers".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.current_pattern, "webservers");
        assert_eq!(state.get_host_count(), 2);
        let output = result.unwrap();
        assert!(output.contains("webservers"));
        assert!(output.contains("2 hosts"));
    }

    #[test]
    fn test_handle_cd_command_with_group() {
        let mut state = create_test_state();
        
        // Change to databases group
        let command = ConsoleCommand::Cd("databases".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.current_pattern, "databases");
        assert_eq!(state.get_host_count(), 1);
    }

    #[test]
    fn test_handle_list_command_returns_current_hosts() {
        let mut state = create_test_state();
        
        // Set pattern to webservers
        state.set_pattern("webservers");
        
        let command = ConsoleCommand::List;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("192.168.1.10"));
        assert!(output.contains("192.168.1.20"));
        assert!(!output.contains("192.168.1.30")); // databases host should not be listed
    }

    #[test]
    fn test_handle_list_command_with_no_matching_hosts() {
        let mut state = create_test_state();
        
        // Set pattern that matches nothing
        state.set_pattern("nonexistent");
        
        let command = ConsoleCommand::List;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No hosts match current pattern");
    }

    #[test]
    fn test_handle_set_command_updates_state_variables() {
        let mut state = create_test_state();
        
        // Initially no variables
        assert!(state.variables.is_empty());
        
        // Set a variable
        let command = ConsoleCommand::Set("package_name".to_string(), "nginx".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.variables.get("package_name"), Some(&"nginx".to_string()));
        let output = result.unwrap();
        assert!(output.contains("package_name"));
        assert!(output.contains("nginx"));
    }

    #[test]
    fn test_handle_set_command_with_multiple_values() {
        let mut state = create_test_state();
        
        // Set a variable with multi-word value
        let command = ConsoleCommand::Set("message".to_string(), "hello world test".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.variables.get("message"), Some(&"hello world test".to_string()));
    }

    #[test]
    fn test_handle_sudo_command_enables_sudo() {
        let mut state = create_test_state();
        
        // Initially sudo should be off
        assert_eq!(state.with_sudo, false);
        
        // Enable sudo
        let command = ConsoleCommand::Sudo(true);
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.with_sudo, true);
        assert!(result.unwrap().contains("enabled"));
    }

    #[test]
    fn test_handle_sudo_command_disables_sudo() {
        let mut state = create_test_state();
        
        // Enable sudo first
        state.with_sudo = true;
        
        // Disable sudo
        let command = ConsoleCommand::Sudo(false);
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.with_sudo, false);
        assert!(result.unwrap().contains("disabled"));
    }

    #[test]
    fn test_handle_forks_command() {
        let mut state = create_test_state();
        
        let command = ConsoleCommand::Forks(10);
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.forks, 10);
    }

    #[test]
    fn test_handle_verbosity_command() {
        let mut state = create_test_state();
        
        let command = ConsoleCommand::Verbosity(3);
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(state.verbosity, 3);
    }

    #[test]
    fn test_handle_vars_command() {
        let mut state = create_test_state();
        
        // No variables initially
        let command = ConsoleCommand::Vars;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No variables set");
        
        // Add some variables
        state.set_variable("key1", "value1");
        state.set_variable("key2", "value2");
        
        let command = ConsoleCommand::Vars;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("key1"));
        assert!(output.contains("value1"));
        assert!(output.contains("key2"));
        assert!(output.contains("value2"));
    }

    #[test]
    fn test_handle_help_command() {
        let mut state = create_test_state();
        
        let command = ConsoleCommand::Help;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Regent Console"));
        assert!(output.contains("Commands:"));
        assert!(output.contains("cd <pattern>"));
    }

    #[test]
    fn test_handle_exit_command() {
        let mut state = create_test_state();
        
        let command = ConsoleCommand::Exit;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "exit");
    }

    #[test]
    fn test_handle_list_groups_command() {
        let mut state = create_test_state();
        
        let command = ConsoleCommand::ListGroups;
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("webservers"));
        assert!(output.contains("databases"));
    }

    #[test]
    fn test_build_yaml_task_without_sudo() {
        let params = vec![
            ("package".to_string(), "nginx".to_string()),
            ("state".to_string(), "present".to_string()),
        ];
        
        let yaml = ConsoleCommandHandler::build_yaml_task("apt", &params, false);
        
        assert!(yaml.contains("---"));
        assert!(yaml.contains("- name: Console task"));
        assert!(yaml.contains("apt:"));
        assert!(yaml.contains("package: 'nginx'"));
        assert!(yaml.contains("state: 'present'"));
        assert!(!yaml.contains("with_sudo: true"));
    }

    #[test]
    fn test_build_yaml_task_with_sudo() {
        let params = vec![
            ("package".to_string(), "nginx".to_string()),
        ];
        
        let yaml = ConsoleCommandHandler::build_yaml_task("apt", &params, true);
        
        assert!(yaml.contains("with_sudo: true"));
        assert!(yaml.contains("apt:"));
        assert!(yaml.contains("package: 'nginx'"));
    }

    #[test]
    fn test_execute_module_fails_with_no_hosts() {
        let mut state = create_test_state();
        
        // Set pattern that matches no hosts
        state.set_pattern("nonexistent");
        
        let command = ConsoleCommand::Module(
            "ping".to_string(),
            Vec::new(),
        );
        
        let result = ConsoleCommandHandler::handle(command, &mut state);
        
        assert!(result.is_err());
        if let Err(Error::MissingInitialization(msg)) = result {
            assert!(msg.contains("No hosts selected"));
        } else {
            panic!("Expected MissingInitialization error");
        }
    }

    #[test]
    fn test_module_command_builds_correct_yaml() {
        // This is an integration-style test that verifies the YAML is built correctly
        // but doesn't actually execute the job
        let params = vec![
            ("name".to_string(), "nginx".to_string()),
            ("state".to_string(), "started".to_string()),
            ("enabled".to_string(), "true".to_string()),
        ];
        
        let yaml = ConsoleCommandHandler::build_yaml_task("service", &params, true);
        
        assert!(yaml.contains("---"));
        assert!(yaml.contains("with_sudo: true"));
        assert!(yaml.contains("service:"));
        assert!(yaml.contains("name: 'nginx'"));
        assert!(yaml.contains("state: 'started'"));
        assert!(yaml.contains("enabled: 'true'"));
    }
}
