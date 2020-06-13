use crate::cli::Command;

pub struct VaultClient;

impl VaultClient {
    pub fn new() -> Self {
        VaultClient {}
    }

    // TODO: Form requests to the remote node
    // TODO: Extract and parse data from template
    pub fn run(&self, command: &Command) {
        let _host = command.host.clone();
        let _token = command.token.clone();
    }
}
