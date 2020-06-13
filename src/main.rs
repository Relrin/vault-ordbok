mod cli;
mod vault;
mod parser;

use structopt::StructOpt;

use crate::cli::Command;
use crate::vault::VaultClient;

fn main() {
    let command = Command::from_args();
    let client = VaultClient::new();
    client.run(&command);
}