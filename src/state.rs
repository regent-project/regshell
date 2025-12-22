use regent_sdk::connection::host_connection::HostConnectionInfo;
use regent_sdk::host::hostlist::HostList;
use regent_sdk::host::hosts::Host;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConsoleState {
    pub current_pattern: String,
    pub hostlist: HostList,
    pub current_hosts: Vec<Host>,
    pub variables: HashMap<String, String>,
    pub connection_info: HostConnectionInfo,
    pub with_sudo: bool,
    pub forks: usize,
    pub verbosity: u8,
    pub username: String,
}

impl ConsoleState {
    pub fn new(username: String) -> ConsoleState {
        ConsoleState {
            current_pattern: "all".to_string(),
            hostlist: HostList { hosts: None },
            current_hosts: Vec::new(),
            variables: HashMap::new(),
            connection_info: HostConnectionInfo::localhost_current_user(),
            with_sudo: false,
            forks: 5,
            verbosity: 0,
            username,
        }
    }

    pub fn set_hostlist(&mut self, hostlist: HostList) {
        self.hostlist = hostlist;
        self.update_current_hosts();
    }

    pub fn set_pattern(&mut self, pattern: &str) {
        self.current_pattern = pattern.to_string();
        self.update_current_hosts();
    }

    pub fn update_current_hosts(&mut self) {
        self.current_hosts.clear();
        
        if let Some(ref hosts) = self.hostlist.hosts {
            if self.current_pattern == "all" {
                self.current_hosts = hosts.clone();
            } else {
                // Filter by pattern (simple matching for now)
                for host in hosts {
                    // Check if host address matches pattern
                    if host.address.contains(&self.current_pattern) {
                        self.current_hosts.push(host.clone());
                        continue;
                    }
                    
                    // Check if host is in a group matching pattern
                    if let Some(ref groups) = host.groups {
                        if groups.contains(&self.current_pattern) {
                            self.current_hosts.push(host.clone());
                        }
                    }
                }
            }
        }
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

    pub fn list_groups(&self) -> Vec<String> {
        let mut groups = Vec::new();
        
        if let Some(ref hosts) = self.hostlist.hosts {
            for host in hosts {
                if let Some(ref host_groups) = host.groups {
                    for group in host_groups {
                        if !groups.contains(group) {
                            groups.push(group.clone());
                        }
                    }
                }
            }
        }
        
        // Always include "all" and "ungrouped"
        if !groups.contains(&"all".to_string()) {
            groups.insert(0, "all".to_string());
        }
        
        groups
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
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
