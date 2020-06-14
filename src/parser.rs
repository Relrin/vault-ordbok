use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::error::{Error, Result};

lazy_static! {
    static ref COMMAND_REGEX: Regex =
        Regex::new(r"\{\{\s*(?P<command>[^\s\(\)]+)\s*\((?P<args>.+)\)\s*\}\}").unwrap();
    static ref ARG_REGEX: Regex =
        Regex::new(r"\u0027(.*?[^\\])\u0027|\u0022(.*?[^\\])\u0022").unwrap();
}

#[derive(Debug)]
pub struct VaultCommandParser;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct ParsedCommand {
    regex_match: String,
    command: VaultCommand,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum VaultCommand {
    Lookup { path: String, key: String },
}

impl VaultCommandParser {
    pub(crate) fn new() -> Self {
        VaultCommandParser {}
    }

    pub(crate) fn parse(&self, data: &str) -> HashSet<ParsedCommand> {
        let mut parsed_commands = HashSet::new();

        for group in COMMAND_REGEX.captures_iter(data) {
            match ParsedCommand::from(&group) {
                Ok(command) => {
                    parsed_commands.insert(command);
                }
                Err(err) => println!("{}", err),
            }
        }

        parsed_commands
    }
}

impl ParsedCommand {
    pub(crate) fn from(group_match: &Captures) -> Result<Self> {
        let raw_match = group_match.get(0).map_or("", |v| v.as_str()).to_string();
        let command_name = group_match.name("command").map_or("", |v| v.as_str());
        let args = group_match.name("args").map_or("", |v| v.as_str());

        match VaultCommand::parse(command_name, args) {
            Ok(vault_command) => Ok(ParsedCommand {
                regex_match: raw_match,
                command: vault_command,
            }),
            Err(err) => {
                let message = format!(
                    "The `{}` part can't be parsed properly. Reason: {}",
                    raw_match, err
                );
                Err(Error::Parse(message))
            }
        }
    }

    pub(crate) fn regex_match(&self) -> &String {
        &self.regex_match
    }

    pub(crate) fn command(&self) -> &VaultCommand {
        &self.command
    }
}

impl VaultCommand {
    // TODO: Add tests for this method
    pub(crate) fn parse(command_name: &str, args: &str) -> Result<Self> {
        let mut extracted_args: Vec<String> = Vec::new();
        for capture in ARG_REGEX.captures_iter(args) {
            let mut data = capture.get(1);
            if data.is_none() {
                data = capture.get(2)
            }

            let value = data
                .map_or("", |v| v.as_str())
                .to_string()
                .replace("\\", "");
            extracted_args.push(value);
        }

        match command_name {
            "lookup" => {
                if extracted_args.len() <= 1 || extracted_args.len() > 2 {
                    let message = format!(
                        "The `{}` command requires two arguments for an execution.",
                        command_name
                    );
                    return Err(Error::Arguments(message));
                }

                let path = extracted_args.get(0).unwrap().to_string();
                let key = extracted_args.get(1).unwrap().to_string();
                Ok(VaultCommand::Lookup { path, key })
            }
            unknown_command => Err(Error::UnknownCommand(unknown_command.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::parser::{ParsedCommand, VaultCommandParser};

    #[test]
    fn test_parser_returns_lookup_command() {
        let data = "KEY_1: {{ lookup ('/data/storage/dev/', 'aws_key') }}";

        let instance = VaultCommandParser::new();
        let commands = instance.parse(data);

        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_parser_returns_multiple_lookup_commands() {
        let path = "./tests/manifests/k8s_multiple_keys.yaml".to_string();
        let data = read_to_string(path).expect("Can't read the file.");

        let instance = VaultCommandParser::new();
        let commands = instance.parse(&data);

        assert_eq!(commands.len(), 6);
    }

    #[test]
    fn test_parser_returns_no_commands_for_empty_input() {
        let data = "";

        let instance = VaultCommandParser::new();
        let commands = instance.parse(data);

        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parser_returns_no_commands_for_input_without_defined_commands() {
        let data = "KEY: VALUE";

        let instance = VaultCommandParser::new();
        let commands = instance.parse(data);

        assert_eq!(commands.len(), 0);
    }
}
