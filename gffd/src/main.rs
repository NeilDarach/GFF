mod args;
mod config;
use crate::args::{Args, GlobalOptions, Subcommands};
use crate::config::Config;

fn main() {
    let args = Args::read_args();
    let GlobalOptions {
        ref directory,
        ref auth_file,
        ref calendar_main_id,
        ref calendar_filter_id,
    } = args.options;
    let mut config = match Config::read_config_file(directory) {
        Err(e) => {
            println!("gffd: {}", e);
            return;
        }
        Ok(c) => c,
    };

    if !auth_file.is_empty() {
        config.set_auth_file(auth_file);
    }
    if !calendar_main_id.is_empty() {
        config.calendar_main_id = calendar_main_id.clone();
    }
    if !calendar_filter_id.is_empty() {
        config.calendar_filter_id = calendar_filter_id.clone();
    }
    if let Subcommands::Serve { port, callback_url } = args.subcommand {
        if !callback_url.is_empty() {
            config.server_options.callback_url = callback_url;
        }
        if port >= 0 {
            config.server_options.port = port as u16;
        }
    }
    println!("Config is {:?}", config);
}
