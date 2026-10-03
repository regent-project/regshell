use regent_sdk::hosts::managed_host::ManagedHost;
use std::collections::HashMap;

pub struct ConsoleState {
    pub current_pattern: String,
    pub hosts: HashMap<String, ManagedHost>,
    pub current_hosts: Vec<String>,
    pub variables: HashMap<String, String>,
    pub with_sudo: bool,
    pub forks: usize,
    pub verbosity: u8,
    pub username: String,
}

impl ConsoleState {
    pub fn new(username: String) -> ConsoleState {
        ConsoleState {
            current_pattern: "all".to_string(),
            hosts: HashMap::new(),
            current_hosts: Vec::new(),
            variables: HashMap::new(),
            with_sudo: false,
            forks: 5,
            verbosity: 0,
            username,
        }
    }

    pub fn set_hosts(&mut self, hosts: HashMap<String, ManagedHost>) {
        self.hosts = hosts;
        self.update_current_hosts();
    }

    pub fn set_pattern(&mut self, pattern: &str) {
        self.current_pattern = pattern.to_string();
        self.update_current_hosts();
    }

    pub fn update_current_hosts(&mut self) {
        self.current_hosts.clear();

        if self.current_pattern == "all" {
            self.current_hosts = self.hosts.keys().cloned().collect();
        } else {
            for host_id in self.hosts.keys() {
                if host_id.contains(&self.current_pattern) {
                    self.current_hosts.push(host_id.clone());
                }
            }
        }

        self.current_hosts.sort();
    }

    pub fn get_host_count(&self) -> usize {
        self.current_hosts.len()
    }

    pub fn get_prompt(&self) -> String {
        format!(
            "{}@{} ({})[f:{}]$ ",
            self.username,
            self.current_pattern,
            self.get_host_count(),
            self.forks
        )
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());

        // Propagate the variable to every host so it is available in templates
        for managed_host in self.hosts.values_mut() {
            managed_host.add_var(key.to_string(), value.to_string());
        }
    }

    pub fn get_variables_display(&self) -> String {
        if self.variables.is_empty() {
            "No variables set".to_string()
        } else {
            let mut output = String::from("Variables:\n");
            for (key, value) in &self.variables {
                output.push_str(&format!("  {} = {}\n", key, value));
            }
            output
        }
    }
}
