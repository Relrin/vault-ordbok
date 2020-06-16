use std::fs::read_to_string;

use quick_error::ResultExt;

use crate::cli::Command;
use crate::error::Result;
use crate::parser::{ParsedCommand, VaultCommand, VaultCommandParser};

pub struct VaultClient;

enum HttpMethod {
    Get,
}

impl VaultClient {
    pub fn new() -> Self {
        VaultClient {}
    }

    pub fn run(&self, command: &Command) -> Result<()> {
        let file_path = command.file.clone();
        let data = read_to_string(&file_path).context(&file_path)?;
        let parser = VaultCommandParser::new();
        let vault_commands = parser.parse(&data);

        let host = command.host.clone().trim_end_matches("/").to_string();
        let token = command.token.clone();

        let substitutions = vault_commands
            .iter()
            .map(|command| {
                let raw_match = command.regex_match();
                let url = self.get_url_by_command(command, &host);
                // TODO: Add request to remote host
            })
            .collect::<Vec<(String, String)>>();

        // TODO: Add replacing the placeholders
        Ok(())
    }

    fn get_url_by_command(
        &self,
        parsed_command: &ParsedCommand,
        host: &String,
    ) -> (HttpMethod, String) {
        match parsed_command.command() {
            VaultCommand::Lookup { path, key } => {
                let url = format!("{}/{}/{}", path, key, host);
                (HttpMethod::Get, url)
            }
        }
    }
}
