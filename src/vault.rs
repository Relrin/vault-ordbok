use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;

use quick_error::ResultExt;
use serde_json::Value;
use ureq::{get as http_get, Request};

use crate::cli::Command;
use crate::error::Result;
use crate::parser::{ParsedCommand, VaultCommand, VaultCommandParser};

pub struct VaultClient {
    cache: HashMap<String, Value>,
}

enum HttpMethod {
    Get,
}

impl VaultClient {
    pub fn new() -> Self {
        VaultClient {
            cache: HashMap::new(),
        }
    }

    pub fn run(&mut self, args: &Command) -> Result<()> {
        let file_path = args.file.clone();
        let mut data = read_to_string(&file_path).context(&file_path)?;
        let parser = VaultCommandParser::new();
        let vault_commands = parser.parse(&data);

        let host = args.host.clone().trim_end_matches("/").to_string();
        let token = args.token.clone();

        let substitutions = vault_commands
            .iter()
            .map(|command| {
                let raw_match = command.regex_match().clone();
                let (http_method, url, key) = self.get_url_by_command(command, &host);

                let data = match self.cache.contains_key(&url) {
                    false => {
                        let mut request = self.build_request(http_method, &url, &token);
                        let response = request.call();

                        let empty_value = Value::String(String::new());
                        let value = match response.into_json() {
                            Ok(parsed_json) => {
                                let json_value = &parsed_json.as_object().unwrap()["data"]["data"];
                                let value = json_value
                                    .get(&key)
                                    .unwrap_or(&empty_value)
                                    .as_str()
                                    .unwrap();
                                self.cache.insert(url.clone(), json_value.to_owned());
                                format!("{}", value)
                            }
                            Err(_) => {
                                println!(
                                    "Cant parse response body into JSON for the `{}` key",
                                    raw_match.clone()
                                );
                                String::new()
                            }
                        };

                        value
                    }
                    true => {
                        let empty_value = Value::String(String::new());
                        let json_value = self.cache.get(&url.clone()).unwrap();
                        let value = json_value
                            .get(&key)
                            .unwrap_or(&empty_value)
                            .as_str()
                            .unwrap();
                        format!("{}", value)
                    }
                };

                (raw_match, data)
            })
            .filter(|(_raw_match, value)| !value.is_empty())
            .collect::<Vec<(String, String)>>();

        for (raw_match, value) in substitutions {
            data = data.replace(&raw_match, &value);
        }

        let out_file_path = args.destination.clone().unwrap_or(args.file.clone());
        let mut file = File::create(&out_file_path).context(&out_file_path)?;
        file.write_all(data.as_bytes()).context(&out_file_path)?;
        Ok(())
    }

    fn get_url_by_command(
        &self,
        parsed_command: &ParsedCommand,
        host: &String,
    ) -> (HttpMethod, String, String) {
        match parsed_command.command() {
            VaultCommand::Lookup { path, key } => {
                let url = format!(
                    "{}/{}",
                    host,
                    path.trim_start_matches("/").trim_end_matches("/")
                );
                (HttpMethod::Get, url, key.clone())
            }
        }
    }

    fn build_request(&self, http_method: HttpMethod, url: &String, token: &String) -> Request {
        match http_method {
            HttpMethod::Get => http_get(&url.clone())
                .set("X-Vault-Token", &token.clone())
                .build(),
        }
    }
}
