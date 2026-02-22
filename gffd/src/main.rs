mod args;
mod config;
mod films;
use crate::args::{Args, GlobalOptions, Subcommands};
use crate::config::Config;
use crate::films::{fetch_ids, id_map, load_ids, BrochureEntry, FestivalEvent};

fn main() {
    let args = Args::read_args();
    let GlobalOptions {
        ref directory,
        ref auth_file,
        ref calendar_main_id,
        ref calendar_filter_id,
        ref debug,
        ref live,
    } = args.options;
    let mut config = match Config::read_config_file(directory) {
        Err(e) => {
            println!("gffd: {}", e);
            return;
        }
        Ok(c) => c,
    };

    if *debug {
        config.set_debug();
    }
    if *live {
        config.set_live();
    }

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

        Subcommands::FetchScreenings { id } => {
            if let Ok(id) = id.parse::<u32>() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &FestivalEvent::fetch_from_gft(&config, id).unwrap()
                    )
                    .unwrap()
                );
            } else {
                let map = id_map(&config).unwrap();
                for id in map.id_to_film.keys() {
                    FestivalEvent::fetch_from_gft(&config, *id).unwrap();
                }
            }
        }
        Subcommands::List {} => {}
        Subcommands::Showings {} => {
            let map = id_map(&config).unwrap();
            let mut showings = map
                .id_to_film
                .keys()
                .map(|id| FestivalEvent::fetch_from_gft(&config, *id).unwrap())
                .map(|event| BrochureEntry::from_event(&event))
                .collect::<Vec<_>>();
            showings.sort_by(|a, b| a.sortname.cmp(&b.sortname));
            println!("{}", serde_json::to_string_pretty(&showings).unwrap());
        }
        Subcommands::Ids {} => {
            println!("{:?}", id_map(&config));
        }
    };
}
