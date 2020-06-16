use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "vault-ordbok")]
pub struct Command {
    #[structopt(
        short = "f",
        long = "file",
        help = "Path to file which requires injecting Vault values"
    )]
    pub file: String,
    #[structopt(short = "h", long = "host", help = "URL to the remote Vault node")]
    pub host: String,
    #[structopt(
        short = "t",
        long = "token",
        help = "API token for requests to Vault storage"
    )]
    pub token: String,
}
