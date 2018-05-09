extern crate termion;
extern crate tui;
extern crate scoped_pool;
extern crate discord;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate serde;

use discord::model::{ServerId, ServerInfo, PublicChannel};
use discord::Discord;

mod input;
mod types;
mod ui;

fn main() {
    use types::UserConfiguration;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::mpsc::{Sender, Receiver};
    use std::sync::mpsc;
    use std::thread;

    println!("Reading configuration...");
    let config = UserConfiguration::read().expect("Failed to load configuration");
    if config.get_token().is_empty() {
        println!("Token is empty. Exiting...")
    } else {
        let discord = Discord::from_user_token(config.get_token()).expect("Failed to connect to Discord api");
        let servers = discord.get_servers().expect("Failed to get servers");
        let server_len = servers.len();


        let arc = Arc::new(discord);
        let mut  arch: types::Architecture = types::Architecture::new(arc);
        let (snd, rcv) = mpsc::channel();

        println!("Loading servers...");
        for serv in servers {
            let sndc = snd.clone();
            let d = arch.get_discord();
            thread::spawn(move || {
                let id = serv.id;
                let servs = d.get_server_channels(id).expect("Failed to get channels");
                sndc.send((id, servs));
            });
        }

        let mut map: HashMap<ServerId, Vec<PublicChannel>> = HashMap::new();
        for _ in 0..server_len {
            let (id, chans) = rcv.recv().unwrap();
            println!("got server: {}", id); // debug statement
            map.insert(id, chans);
        }
        println!("Done getting servers ({})", map.len());
        arch.set_servers(map);
        println!("Drawing UI...");
        ui::draw_ui(arch);
    }
}
