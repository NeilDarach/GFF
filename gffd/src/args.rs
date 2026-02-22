use clap::{Parser, Subcommand};
#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub options: GlobalOptions,

    #[clap(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Parser, Debug)]
#[command(author,version,about,long_about = None)]
pub struct GlobalOptions {
    #[arg(short, long)]
    pub directory: String,

    #[arg(long = "auth", short = 'a', default_value_t = ("").to_string())]
    pub auth_file: String,
    #[arg(long = "main", short = 'm', default_value_t = ("").to_string())]
    pub calendar_main_id: String,
    #[arg(long = "fiter", short = 'f', default_value_t = ("").to_string())]
    pub calendar_filter_id: String,
    #[arg(long = "verbose", short = 'v', default_value_t = false)]
    pub debug: bool,
    #[arg(long = "live", short = 'l', default_value_t = false)]
    pub live: bool,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Subcommands {
    Serve {
        #[arg(long = "port", short = 'p', default_value_t = -1)]
        port: i16,
        #[arg(long = "url", short = 'u', default_value_t = ("").to_string())]
        callback_url: String,
    },
    Ids {},
    Showings {},
    FetchScreenings {
        #[arg(long = "id", short = 'i', default_value_t = ("").to_string())]
        id: String,
    },
    ShowConfig {},
    List {},
}

impl Args {
    pub fn read_args() -> Self {
        Self::parse()
    }
}
