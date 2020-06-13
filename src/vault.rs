use crate::cli::Command;

pub struct VaultClient;

impl VaultClient {
    pub fn new() -> Self {
        VaultClient {}
    }

    pub fn run(&self, command: &Command) {
        let host = command.host.clone();
        let token = command.token.clone();
    }
}
