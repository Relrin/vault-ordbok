use lazy_static::lazy_static;
use regex::{Match, Regex};

use crate::error::{Error, Result};

lazy_static! {
    static ref COMMAND_REGEX: Regex =
        Regex::new(r"\{\{\s*(?P<command>[^\s\(\)]+)\s*\((?P<args>.+)\)\s*\}\}").unwrap();
    static ref ARG_REGEX: Regex = Regex::new(r"\u0027(.*?)\u0027|\u0022(.*?)\u0022").unwrap();
}

#[derive(Debug)]
pub struct VaultCommandParser;

#[derive(Debug)]
pub struct ParsedCommand {
    regex_match: String,
    command: VaultCommand,
}

#[derive(Debug)]
pub enum VaultCommand {
    Lookup { path: String, key: String },
}

impl VaultCommandParser {
    pub(crate) fn new() -> Self {
        VaultCommandParser {}
    }

    pub(crate) fn parse(&self, data: &str) -> Vec<ParsedCommand> {
        let mut parsed_commands = Vec::new();

        for group in COMMAND_REGEX.captures_iter(data) {
            let raw_match = group.get(0).map_or("", |v| v.as_str()).to_string();
            let command_name = group.name("command").map_or("", |v| v.as_str());
            let args = group.name("args").map_or("", |v| v.as_str());

            match VaultCommand::parse(command_name, args) {
                Ok(vault_command) => {
                    let command = ParsedCommand::new(raw_match, vault_command);
                    parsed_commands.push(command);
                }
                Err(err) => println!("{}", err),
            }
        }

        parsed_commands
    }
}

impl ParsedCommand {
    pub(crate) fn new(regex_match: String, command: VaultCommand) -> Self {
        ParsedCommand {
            regex_match,
            command,
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
            let value = capture.get(0).map_or("", |v| v.as_str()).to_string();
            extracted_args.push(value)
        }

        match command_name {
            "lookup" => match (extracted_args.get(0), extracted_args.get(1)) {
                (Some(path), Some(key)) => Ok(VaultCommand::Lookup { path, key }),
                _ => {
                    let message = format!(
                        "The `{}` command requires two arguments for execution",
                        command_name
                    );
                    Err(Error::Parse(message))
                }
            },
            unknown_command => {}
        }
    }
}

#[cfg(test)]
mod tests {
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
        // Use these examples
        // KEY_1: {{ lookup ('/data/storage/dev/', 'key') }}
        // KEY_2: {{ lookup ('/data/storage/dev/', 'pass') }}
        // KEY_3: {{ lookup (NOT A KEY) }}
        // KEY_4: {{ test a commnad }}
        // KEY_5: {{lookup("/data/storage/dev/",'random_stuff')}}
        // KEY_6: {{lookup('/data/storage/dev/',"random_stuff2")}}
        // KEY_7: {{lookup("/data/storage/dev/",'try parse\' this')}}
        // KEY_8: {{ lookup ("SOME_KEY") }}
        // KEY_9: {{ lookup ("first", "second", "third") }}
        // KEY_10: {{ lookup ("escape\" this",) }}
    }

    #[test]
    fn test_parser_returns_no_commands_for_empty_input() {}

    #[test]
    fn test_parser_returns_no_commands_for_input_without_defined_commands() {}
}
