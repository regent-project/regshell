use regent_sdk::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleCommand {
    Cd(String),
    List,
    ListGroups,
    Help,
    Exit,
    Vars,
    Set(String, String),
    Sudo(bool),
    Forks(usize),
    Verbosity(u8),
    Module(String, Vec<(String, String)>), // module_name, [(key, value)]
}

pub struct ConsoleParser;

impl ConsoleParser {
    pub fn parse(input: &str) -> Result<ConsoleCommand, Error> {
        let trimmed = input.trim();
        
        if trimmed.is_empty() {
            return Err(Error::FailureToParseContent("Empty command".to_string()));
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let command = parts[0];

        match command {
            "cd" => {
                if parts.len() < 2 {
                    return Err(Error::FailureToParseContent(
                        "cd requires a pattern argument".to_string(),
                    ));
                }
                Ok(ConsoleCommand::Cd(parts[1].to_string()))
            }
            "list" => {
                if parts.len() > 1 && parts[1] == "groups" {
                    Ok(ConsoleCommand::ListGroups)
                } else {
                    Ok(ConsoleCommand::List)
                }
            }
            "help" | "?" => Ok(ConsoleCommand::Help),
            "exit" | "quit" => Ok(ConsoleCommand::Exit),
            "vars" => Ok(ConsoleCommand::Vars),
            "set" => {
                if parts.len() < 3 {
                    return Err(Error::FailureToParseContent(
                        "set requires key and value arguments".to_string(),
                    ));
                }
                Ok(ConsoleCommand::Set(
                    parts[1].to_string(),
                    parts[2..].join(" "),
                ))
            }
            "sudo" => {
                if parts.len() < 2 {
                    return Err(Error::FailureToParseContent(
                        "sudo requires 'on' or 'off' argument".to_string(),
                    ));
                }
                match parts[1] {
                    "on" | "true" | "yes" => Ok(ConsoleCommand::Sudo(true)),
                    "off" | "false" | "no" => Ok(ConsoleCommand::Sudo(false)),
                    _ => Err(Error::FailureToParseContent(
                        "sudo argument must be 'on' or 'off'".to_string(),
                    )),
                }
            }
            "forks" => {
                if parts.len() < 2 {
                    return Err(Error::FailureToParseContent(
                        "forks requires a number argument".to_string(),
                    ));
                }
                match parts[1].parse::<usize>() {
                    Ok(num) => Ok(ConsoleCommand::Forks(num)),
                    Err(_) => Err(Error::FailureToParseContent(
                        "forks argument must be a number".to_string(),
                    )),
                }
            }
            "verbosity" => {
                if parts.len() < 2 {
                    return Err(Error::FailureToParseContent(
                        "verbosity requires a number argument".to_string(),
                    ));
                }
                match parts[1].parse::<u8>() {
                    Ok(num) => Ok(ConsoleCommand::Verbosity(num)),
                    Err(_) => Err(Error::FailureToParseContent(
                        "verbosity argument must be a number".to_string(),
                    )),
                }
            }
            _ => {
                // Try to parse as module invocation
                Self::parse_module_invocation(trimmed)
            }
        }
    }

    fn parse_module_invocation(input: &str) -> Result<ConsoleCommand, Error> {
        // Parse: "module_name key1=value1 key2=value2"
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(Error::FailureToParseContent(
                "Invalid module invocation".to_string(),
            ));
        }

        let module_name = parts[0].to_string();
        let mut params = Vec::new();

        for part in &parts[1..] {
            if let Some(eq_pos) = part.find('=') {
                let key = part[..eq_pos].to_string();
                let value = part[eq_pos + 1..].to_string();
                params.push((key, value));
            } else {
                return Err(Error::FailureToParseContent(format!(
                    "Invalid parameter format: '{}'. Expected key=value",
                    part
                )));
            }
        }

        Ok(ConsoleCommand::Module(module_name, params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cd() {
        let result = ConsoleParser::parse("cd webservers");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConsoleCommand::Cd("webservers".to_string()));
    }

    #[test]
    fn test_parse_list() {
        let result = ConsoleParser::parse("list");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConsoleCommand::List);
    }

    #[test]
    fn test_parse_list_groups() {
        let result = ConsoleParser::parse("list groups");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConsoleCommand::ListGroups);
    }

    #[test]
    fn test_parse_module() {
        let result = ConsoleParser::parse("apt package=nginx state=present");
        assert!(result.is_ok());
        if let ConsoleCommand::Module(name, params) = result.unwrap() {
            assert_eq!(name, "apt");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], ("package".to_string(), "nginx".to_string()));
            assert_eq!(params[1], ("state".to_string(), "present".to_string()));
        } else {
            panic!("Expected Module command");
        }
    }
}
