mod args;
mod config;
mod films;
use crate::args::{Args, GlobalOptions, Subcommands};
use crate::config::Config;
use crate::films::{FestivalEvent, fetch_ids, id_map, load_ids};
use std::thread::sleep_ms;

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
    match args.subcommand {
        Subcommands::Serve { port, callback_url } => {
            if !callback_url.is_empty() {
                config.server_options.callback_url = callback_url;
            }
            if port >= 0 {
                config.server_options.port = port as u16;
            }
        }
        Subcommands::ShowConfig {} => {
            println!("{:?}", &config);
        }

        Subcommands::FetchScreenings {} => {
            let map = id_map(&config).unwrap();
            for id in map.id_to_film.keys() {
                FestivalEvent::fetch_from_gft(&config, *id).unwrap();
                sleep_ms(250);
            }
        }
        Subcommands::List {} => {}
        Subcommands::Ids {} => {
            println!("{:?}", id_map(&config));
        }
    };
}
