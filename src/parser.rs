use lazy_static::lazy_static;
use regex::{Match, Regex};

lazy_static! {
    static ref COMMAND_REGEX: Regex =
        Regex::new(r"\{\{\s*(?P<command>[^\s\(\)]+)\s*\((?P<args>.+)\)\s*\}\}").unwrap();
    static ref ARG_REGEX: Regex = Regex::new("\'(.*?)\'|\"(.*?)\"").unwrap();
}

#[derive(Debug)]
pub struct VaultCommandParser;

#[derive(Debug)]
pub struct ParsedCommand<'a> {
    regex_match: Match<'a>,
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

    pub(crate) fn parse(&self, data: &String) -> Vec<ParsedCommand> {
        Vec::new()
    }
}

impl<'a> ParsedCommand<'a> {
    pub(crate) fn new(regex_match: Match<'a>, command: VaultCommand) -> Self {
        ParsedCommand {
            regex_match,
            command,
        }
    }

    pub(crate) fn regex_match(&self) -> &Match<'a> {
        &self.regex_match
    }

    pub(crate) fn command(&self) -> &VaultCommand {
        &self.command
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parser_returns_lookup_command() {}

    #[test]
    fn test_parser_returns_multiple_lookup_commands() {}

    #[test]
    fn test_parser_returns_no_commands_for_empty_input() {}

    #[test]
    fn test_parser_returns_no_commands_for_input_without_defined_commands() {}
}
