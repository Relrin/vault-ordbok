use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "vault-ordbok")]
pub struct Command {
    #[structopt(
        short = "f",
        long = "file",
        help = "Path to source file with placeholders"
    )]
    pub file: String,
    #[structopt(
        short = "d",
        long = "dest",
        help = "Result file path with injected Vault values"
    )]
    pub destination: Option<String>,
    #[structopt(short = "h", long = "host", help = "URL to the remote Vault node")]
    pub host: String,
    #[structopt(
        short = "t",
        long = "token",
        help = "API token for requests to Vault storage"
    )]
    pub token: String,
}
