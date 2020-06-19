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

    // Returns a set of commands that needs to be executed and replaced
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
    // Converts the regex capture into the Vault command
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

    // Returns a full match by regex
    pub(crate) fn regex_match(&self) -> &String {
        &self.regex_match
    }

    // Returns a VaultCommand instance
    pub(crate) fn command(&self) -> &VaultCommand {
        &self.command
    }
}

impl VaultCommand {
    /// Parses the input and tries to convert it into the appropriate Vault command
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

    use crate::parser::{ParsedCommand, VaultCommand, VaultCommandParser, COMMAND_REGEX};

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
    fn test_parser_returns_single_lookup_command_when_found_duplicates() {
        let path = "./tests/manifests/k8s_duplicated_keys.yaml".to_string();
        let data = read_to_string(path).expect("Can't read the file.");

        let instance = VaultCommandParser::new();
        let commands = instance.parse(&data);

        assert_eq!(commands.len(), 1);
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

    #[test]
    fn test_parsed_command_returns_error_for_invalid_input() {
        let data = "KEY: {{ lookup ('1', '2', '3') }}";

        let mut regex_results = COMMAND_REGEX.captures_iter(data);
        let capture = regex_results.next().unwrap();
        let result = ParsedCommand::from(&capture);

        assert_eq!(result.is_err(), true);
        let error = result.unwrap_err();
        assert_eq!(
            format!("{}", error),
            "Parse error: The `{{ lookup (\'1\', \'2\', \'3\') }}` part \
            can\'t be parsed properly. Reason: The `lookup` command requires \
            two arguments for an execution."
        )
    }

    #[test]
    fn test_parsed_command_returns_error_for_invalid_command_name() {
        let data = "KEY: {{ not_a_command ('1', '2', '3') }}";

        let mut regex_results = COMMAND_REGEX.captures_iter(data);
        let capture = regex_results.next().unwrap();
        let result = ParsedCommand::from(&capture);

        assert_eq!(result.is_err(), true);
        let error = result.unwrap_err();
        assert_eq!(
            format!("{}", error),
            "Parse error: The `{{ not_a_command (\'1\', \'2\', \'3\') }}` \
            part can\'t be parsed properly. Reason: The `not_a_command` \
            command is not supported."
        )
    }

    #[test]
    fn test_vault_command_parser_returns_lookup_command() {
        let command_name = format!("lookup");
        let args = format!("'/data/key', 'test'");

        let result = VaultCommand::parse(&command_name, &args);

        assert_eq!(result.is_ok(), true);
        println!("{:?}", result);
        let (parsed_path, parsed_args) = match result.unwrap() {
            VaultCommand::Lookup { path, key } => (path, key),
        };
        assert_eq!(parsed_path, "/data/key");
        assert_eq!(parsed_args, "test");
    }

    #[test]
    fn test_vault_command_parser_returns_error_for_too_much_arguments() {
        let command_name = format!("lookup");
        let args = format!("'/data/key', 'test', 'value'");

        let result = VaultCommand::parse(&command_name, &args);

        assert_eq!(result.is_err(), true);
        let error = result.unwrap_err();
        assert_eq!(
            format!("{}", error),
            "The `lookup` command requires two arguments for an execution."
        );
    }

    #[test]
    fn test_vault_command_parser_returns_error_for_no_command_arguments() {
        let command_name = format!("lookup");
        let args = format!("");

        let result = VaultCommand::parse(&command_name, &args);

        assert_eq!(result.is_err(), true);
        let error = result.unwrap_err();
        assert_eq!(
            format!("{}", error),
            "The `lookup` command requires two arguments for an execution."
        );
    }

    #[test]
    fn test_vault_command_parser_returns_error_for_unsupported_command() {
        let command_name = format!("not_a_command");
        let args = format!("'/data/key', 'test'");

        let result = VaultCommand::parse(&command_name, &args);

        assert_eq!(result.is_err(), true);
        let error = result.unwrap_err();
        assert_eq!(
            format!("{}", error),
            "The `not_a_command` command is not supported."
        );
    }
}
