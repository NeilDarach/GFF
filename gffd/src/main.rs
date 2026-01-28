mod args;
use crate::args::{Args, GlobalOptions, Subcommands};

fn main() {
    let args = Args::read_args();

    println!("args are {:?}", args);
    if let Subcommands::Serve {
        port,
        callback_url,
        calendar_main_id,
        calendar_filter_id,
    } = args.subcommand
    {
        println!("Running on port {}", port)
    }
}
