mod args;
mod config;
use crate::args::{Args, GlobalOptions, Subcommands};
use crate::config::{Config, ServerConfig};

fn main() {
    let args = Args::read_args();
    let GlobalOptions {
        ref directory,
        ref auth_file,
        ref calendar_main_id,
        ref calendar_filter_id,
    } = args.options;
    let mut config = Config::read_config_file(directory).expect(&format!(
        "Could not load the config file in {}",
        &directory[..]
    ));

    if auth_file != "" {
        config.calendar_auth_file = auth_file.clone();
    }
    if calendar_main_id != "" {
        config.calendar_main_id = calendar_main_id.clone();
    }
    if calendar_filter_id != "" {
        config.calendar_filter_id = calendar_filter_id.clone();
    }
    println!("args are {:?}", args);
    if let Subcommands::Serve { port, callback_url } = args.subcommand {
        if callback_url != "".to_string() {
            config.server_options.callback_url = callback_url;
        }
        if port >= 0 {
            config.server_options.port = port as u16;
        }
    }
    println!("Config is {:?}", config);
}
