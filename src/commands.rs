use crate::parser::ConsoleCommand;
use crate::state::ConsoleState;
use regent_sdk::ExpectedState;
use regent_sdk::error::RegentError;
use regent_sdk::state::compliance::ManagedHostStatus;
use tokio::task::JoinSet;

pub struct ConsoleCommandHandler;

impl ConsoleCommandHandler {
    pub async fn handle(
        command: ConsoleCommand,
        state: &mut ConsoleState,
    ) -> Result<String, RegentError> {
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
                for host_id in &state.current_hosts {
                    output.push_str(&format!("{}\n", host_id));
                }
                if output.is_empty() {
                    output = "No hosts match current pattern".to_string();
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
                Self::execute_module(module_name, params, state).await
            }
        }
    }

    async fn execute_module(
        module_name: String,
        params: Vec<(String, String)>,
        state: &mut ConsoleState,
    ) -> Result<String, RegentError> {
        if state.current_hosts.is_empty() {
            return Err(RegentError::MissingInitialization(
                "No hosts selected. Use 'cd' to select hosts.".to_string(),
            ));
        }

        let module_tag = match Self::module_tag(&module_name) {
            Some(tag) => tag,
            None => {
                return Err(RegentError::FailureToParseContent(format!(
                    "Unknown module '{}'",
                    module_name
                )));
            }
        };

        // Build an ExpectedState description from the module invocation
        let yaml_state = Self::build_expected_state_yaml(module_tag, &params, state.with_sudo);
        let expected_state = ExpectedState::from_raw_yaml(&yaml_state)?;

        // Enforce the expected state on the selected hosts, `forks` at a time
        let mut output = String::new();
        let fork_count = state.forks.max(1);

        for batch in state.current_hosts.chunks(fork_count) {
            let mut tasks = JoinSet::new();

            for host_id in batch {
                let managed_host = match state.hosts.get(host_id) {
                    Some(host) => host.clone(),
                    None => {
                        return Err(RegentError::MissingInitialization(format!(
                            "Host '{}' not found",
                            host_id
                        )));
                    }
                };
                let expected_state = expected_state.clone();

                tasks.spawn(async move {
                    let mut managed_host = managed_host;
                    let result = managed_host.reach_compliance(&expected_state).await;
                    (managed_host.id().to_string(), result)
                });
            }

            while let Some(task_result) = tasks.join_next().await {
                match task_result {
                    Ok((host_id, Ok(status))) => {
                        output.push_str(&format!("{}: {}\n", host_id, Self::status_text(&status)));
                    }
                    Ok((host_id, Err(error_detail))) => {
                        output.push_str(&format!("{}: error: {}\n", host_id, error_detail));
                    }
                    Err(join_error) => {
                        output.push_str(&format!("Internal task error: {}\n", join_error));
                    }
                }
            }
        }

        Ok(output)
    }

    /// Map a console module name to its regent-sdk attribute YAML tag.
    fn module_tag(module_name: &str) -> Option<&'static str> {
        match module_name.to_lowercase().as_str() {
            "apt" => Some("!Apt"),
            "aptrepo" | "apt_repo" => Some("!AptRepo"),
            "yum" | "dnf" => Some("!YumDnf"),
            "dnfrepo" | "dnf_repo" => Some("!DnfRepo"),
            "pacman" => Some("!Pacman"),
            "service" => Some("!Service"),
            "command" => Some("!Command"),
            "ping" => Some("!Ping"),
            "debug" => Some("!Debug"),
            "lineinfile" => Some("!LineInFile"),
            "user" => Some("!User"),
            "group" => Some("!Group"),
            "cron" => Some("!Cron"),
            "hostname" => Some("!Hostname"),
            "dns" => Some("!Dns"),
            "dhcp" => Some("!Dhcp"),
            "iptables" => Some("!Iptables"),
            "ollama" => Some("!Ollama"),
            "template" => Some("!Template"),
            _ => None,
        }
    }

    /// Build a regent-sdk ExpectedState YAML description from a module invocation.
    ///
    /// The `timeout` parameter is optional and maps to the attribute-level
    /// `Timeout` field (seconds), defaulting to 30.
    fn build_expected_state_yaml(
        module_tag: &str,
        params: &[(String, String)],
        with_sudo: bool,
    ) -> String {
        let mut timeout = "30".to_string();
        let mut detail_params: Vec<&(String, String)> = Vec::new();

        for param in params {
            if param.0.eq_ignore_ascii_case("timeout") {
                timeout = param.1.clone();
            } else {
                detail_params.push(param);
            }
        }

        let mut yaml = String::from("---\n");
        yaml.push_str("Attributes:\n");
        yaml.push_str("  - Name: Console task\n");
        yaml.push_str(&format!("    Timeout: {}\n", timeout));

        if with_sudo {
            yaml.push_str("    Privilege: !WithSudo\n");
        } else {
            yaml.push_str("    Privilege: !None\n");
        }

        yaml.push_str(&format!("    Detail: {}\n", module_tag));

        for (key, value) in detail_params {
            yaml.push_str(&format!(
                "      {}: {}\n",
                Self::pascal_case(key),
                Self::yaml_scalar(key, value)
            ));
        }

        yaml
    }

    /// Convert a console key to its PascalCase YAML counterpart (package -> Package).
    fn pascal_case(key: &str) -> String {
        let mut chars = key.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

    /// Render a console value as a YAML scalar.
    ///
    /// Booleans and numbers are emitted bare, strings are single-quoted, and
    /// enum-like values (e.g. `state=present`) are capitalized to match the
    /// regent-sdk expected-state enums (`State: Present`).
    fn yaml_scalar(key: &str, value: &str) -> String {
        let value = if key.eq_ignore_ascii_case("state") {
            Self::capitalize(value)
        } else {
            value.to_string()
        };

        if value == "true" || value == "false" || value.parse::<f64>().is_ok() {
            value
        } else {
            format!("'{}'", value.replace('\'', "''"))
        }
    }

    fn capitalize(value: &str) -> String {
        let mut chars = value.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

    fn status_text(status: &ManagedHostStatus) -> &'static str {
        if status.is_already_compliant() {
            "already compliant"
        } else if status.is_not_compliant() {
            "not compliant"
        } else if status.is_reach_compliance_success() {
            "compliance reached"
        } else {
            "failed to reach compliance"
        }
    }

    fn help_text() -> String {
        r#"Regent Console - Interactive REPL for regent-sdk

Commands:
  cd <pattern>           Change to host id pattern ("all" selects every host)
  list                   List hosts in current selection
  vars                   Show current variables
  set <key> <value>      Set a variable (available in templates)
  sudo on|off            Toggle sudo for subsequent commands
  forks <num>            Set number of parallel forks
  verbosity <num>        Set verbosity level
  help, ?                Show this help message
  exit, quit             Exit the console

Module Execution:
  <module> key=value ... Build an expected state and enforce it on the
                         selected hosts. Modules map to regent-sdk
                         attributes, keys map to their YAML fields.

  Available modules:
    apt                  Package management (Debian/Ubuntu)
    yum, dnf             Package management (RHEL/CentOS/Fedora)
    pacman               Package management (Arch)
    service              Service management
    command              Execute shell commands
    ping                 Test connectivity
    debug                Display debug information
    lineinfile           Edit lines in files
    user, group          Users and groups management
    cron                 Cron jobs management
    hostname             Hostname management
    dns, dhcp, iptables Network configuration
    template             Render a template to a destination file
    ollama               Ollama (AI) integration

Examples:
  cd web
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
    use crate::state::ConsoleState;
    use regent_sdk::hosts::handlers::{ConnectionMethod, TargetUser};
    use regent_sdk::hosts::managed_host::ManagedHostBuilder;
    use std::collections::HashMap;

    async fn create_test_state() -> ConsoleState {
        let mut state = ConsoleState::new("testuser".to_string());

        let mut hosts = HashMap::new();
        for host_id in ["web-01", "web-02", "db-01"] {
            let builder = ManagedHostBuilder::new(
                host_id,
                "localhost",
                Some(ConnectionMethod::Localhost(TargetUser::current_user())),
            );
            let managed_host = builder.build(None).await.unwrap();
            hosts.insert(host_id.to_string(), managed_host);
        }

        state.set_hosts(hosts);
        state
    }

    #[tokio::test]
    async fn test_handle_cd_command_updates_state() {
        let mut state = create_test_state().await;

        // Initially "all" pattern should match all 3 hosts
        assert_eq!(state.current_pattern, "all");
        assert_eq!(state.get_host_count(), 3);

        // Change to a pattern matching only the web hosts
        let command = ConsoleCommand::Cd("web".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(state.current_pattern, "web");
        assert_eq!(state.get_host_count(), 2);
        let output = result.unwrap();
        assert!(output.contains("web"));
        assert!(output.contains("2 hosts"));
    }

    #[tokio::test]
    async fn test_handle_list_command_returns_current_hosts() {
        let mut state = create_test_state().await;

        // Select a pattern matching a single host
        state.set_pattern("db");

        let command = ConsoleCommand::List;
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("db-01"));
        assert!(!output.contains("web-01"));
        assert!(!output.contains("web-02"));
    }

    #[tokio::test]
    async fn test_handle_list_command_with_no_matching_hosts() {
        let mut state = create_test_state().await;

        // Set pattern that matches nothing
        state.set_pattern("nonexistent");

        let command = ConsoleCommand::List;
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No hosts match current pattern");
    }

    #[tokio::test]
    async fn test_handle_set_command_updates_state_variables() {
        let mut state = create_test_state().await;

        // Initially no variables
        assert!(state.variables.is_empty());

        // Set a variable
        let command = ConsoleCommand::Set("package_name".to_string(), "nginx".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(
            state.variables.get("package_name"),
            Some(&"nginx".to_string())
        );
        let output = result.unwrap();
        assert!(output.contains("package_name"));
        assert!(output.contains("nginx"));
    }

    #[tokio::test]
    async fn test_handle_set_command_with_multiple_values() {
        let mut state = create_test_state().await;

        let command = ConsoleCommand::Set("message".to_string(), "hello world test".to_string());
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(
            state.variables.get("message"),
            Some(&"hello world test".to_string())
        );
    }

    #[tokio::test]
    async fn test_handle_sudo_command_toggles_state() {
        let mut state = create_test_state().await;

        assert!(!state.with_sudo);

        let command = ConsoleCommand::Sudo(true);
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert!(state.with_sudo);
        assert!(result.unwrap().contains("enabled"));

        let command = ConsoleCommand::Sudo(false);
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert!(!state.with_sudo);
        assert!(result.unwrap().contains("disabled"));
    }

    #[tokio::test]
    async fn test_handle_forks_command() {
        let mut state = create_test_state().await;

        let command = ConsoleCommand::Forks(10);
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(state.forks, 10);
    }

    #[tokio::test]
    async fn test_handle_verbosity_command() {
        let mut state = create_test_state().await;

        let command = ConsoleCommand::Verbosity(3);
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(state.verbosity, 3);
    }

    #[tokio::test]
    async fn test_handle_vars_command() {
        let mut state = create_test_state().await;

        // No variables initially
        let command = ConsoleCommand::Vars;
        let result = ConsoleCommandHandler::handle(command, &mut state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No variables set");

        // Add some variables
        state.set_variable("key1", "value1");
        state.set_variable("key2", "value2");

        let command = ConsoleCommand::Vars;
        let result = ConsoleCommandHandler::handle(command, &mut state).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("key1"));
        assert!(output.contains("value1"));
        assert!(output.contains("key2"));
        assert!(output.contains("value2"));
    }

    #[tokio::test]
    async fn test_handle_help_command() {
        let mut state = create_test_state().await;

        let command = ConsoleCommand::Help;
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Regent Console"));
        assert!(output.contains("Commands:"));
        assert!(output.contains("cd <pattern>"));
    }

    #[tokio::test]
    async fn test_handle_exit_command() {
        let mut state = create_test_state().await;

        let command = ConsoleCommand::Exit;
        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "exit");
    }

    #[test]
    fn test_build_expected_state_yaml_without_sudo() {
        let params = vec![
            ("package".to_string(), "nginx".to_string()),
            ("state".to_string(), "present".to_string()),
        ];

        let yaml = ConsoleCommandHandler::build_expected_state_yaml("!Apt", &params, false);

        assert!(yaml.contains("---"));
        assert!(yaml.contains("Attributes:"));
        assert!(yaml.contains("Name: Console task"));
        assert!(yaml.contains("Timeout: 30"));
        assert!(yaml.contains("Detail: !Apt"));
        assert!(yaml.contains("Package: 'nginx'"));
        assert!(yaml.contains("State: 'Present'"));
        assert!(!yaml.contains("Privilege: !WithSudo"));
    }

    #[test]
    fn test_build_expected_state_yaml_with_sudo() {
        let params = vec![("package".to_string(), "nginx".to_string())];

        let yaml = ConsoleCommandHandler::build_expected_state_yaml("!Apt", &params, true);

        assert!(yaml.contains("Privilege: !WithSudo"));
        assert!(yaml.contains("Detail: !Apt"));
        assert!(yaml.contains("Package: 'nginx'"));
    }

    #[test]
    fn test_build_expected_state_yaml_emits_bare_scalars() {
        let params = vec![
            ("name".to_string(), "nginx".to_string()),
            ("state".to_string(), "started".to_string()),
            ("enabled".to_string(), "true".to_string()),
        ];

        let yaml = ConsoleCommandHandler::build_expected_state_yaml("!Service", &params, false);

        assert!(yaml.contains("Name: 'nginx'"));
        assert!(yaml.contains("State: 'Started'"));
        assert!(yaml.contains("Enabled: true"));
        assert!(!yaml.contains("Timeout: 30\n      "));
    }

    #[test]
    fn test_build_expected_state_yaml_supports_timeout_param() {
        let params = vec![
            ("package".to_string(), "nginx".to_string()),
            ("state".to_string(), "present".to_string()),
            ("timeout".to_string(), "60".to_string()),
        ];

        let yaml = ConsoleCommandHandler::build_expected_state_yaml("!Apt", &params, false);

        assert!(yaml.contains("Timeout: 60"));
        assert!(yaml.contains("Package: 'nginx'"));
        // The timeout parameter must not leak into the attribute detail
        assert!(!yaml.contains("Timeout: 'nginx'"));
        let parsed = regent_sdk::ExpectedState::from_raw_yaml(&yaml);
        assert!(parsed.is_ok(), "parse failed: {:?}", parsed.err());
    }

    #[test]
    fn test_build_expected_state_yaml_parses_into_expected_state() {
        let params = vec![
            ("package".to_string(), "nginx".to_string()),
            ("state".to_string(), "present".to_string()),
        ];

        let yaml = ConsoleCommandHandler::build_expected_state_yaml("!Apt", &params, true);
        let expected_state = regent_sdk::ExpectedState::from_raw_yaml(&yaml).unwrap();

        assert_eq!(expected_state.attributes.len(), 1);
    }

    #[test]
    fn test_module_tag_mapping() {
        assert_eq!(ConsoleCommandHandler::module_tag("apt"), Some("!Apt"));
        assert_eq!(ConsoleCommandHandler::module_tag("YUM"), Some("!YumDnf"));
        assert_eq!(ConsoleCommandHandler::module_tag("dnf"), Some("!YumDnf"));
        assert_eq!(
            ConsoleCommandHandler::module_tag("lineinfile"),
            Some("!LineInFile")
        );
        assert_eq!(ConsoleCommandHandler::module_tag("unknown"), None);
    }

    #[tokio::test]
    async fn test_execute_module_fails_with_no_hosts() {
        let mut state = create_test_state().await;

        // Set pattern that matches no hosts
        state.set_pattern("nonexistent");

        let command = ConsoleCommand::Module("ping".to_string(), Vec::new());

        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_err());
        if let Err(RegentError::MissingInitialization(msg)) = result {
            assert!(msg.contains("No hosts selected"));
        } else {
            panic!("Expected MissingInitialization error");
        }
    }

    #[tokio::test]
    async fn test_execute_module_fails_with_unknown_module() {
        let mut state = create_test_state().await;

        let command = ConsoleCommand::Module("nosuchmodule".to_string(), Vec::new());

        let result = ConsoleCommandHandler::handle(command, &mut state).await;

        assert!(result.is_err());
        if let Err(RegentError::FailureToParseContent(msg)) = result {
            assert!(msg.contains("nosuchmodule"));
        } else {
            panic!("Expected FailureToParseContent error");
        }
    }
}
