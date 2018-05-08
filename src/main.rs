extern crate scoped_pool;
extern crate discord;
extern crate evmap;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate serde;

use discord::model::{ServerInfo, PublicChannel};
use discord::Discord;
mod types;

fn main() {
    use types::UserConfiguration;
    use evmap::*;

    println!("Reading configuration...");
    let config = UserConfiguration::read().expect("Failed to load configuration");
    if config.get_token().is_empty() {
        println!("Token is empty. Exiting...")
    } else {
        let discord = Discord::from_user_token(config.get_token()).expect("Failed to connect to Discord api");
        let servers = discord.get_servers().expect("Failed to get servers");

        let (s_chans_r, mut s_chans_w) = evmap::new();

        let pool = scoped_pool::Pool::new(4);

        pool.scoped(|s, | {
            for serv in &servers {
                s.execute(move || {

                });
            }
        });
        s_chans_w.refresh();
    }
}
