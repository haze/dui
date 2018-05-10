
use serde_json;
use serde_derive;
use std::error::Error;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use discord;
use discord::model;
use tui::style::{Color, Style};
use tui::layout::Rect;
use tui::widgets::Item;
use std::collections::HashMap;

// located at ~/.duirc
#[derive(Serialize, Deserialize)]
pub struct UserConfiguration {

    #[serde(default)]
    #[serde(rename = "UserToken")]
    token: String,

    #[serde(default)]
    #[serde(rename = "HomeChannel")]
    home_channel: String,

    #[serde(default)]
    #[serde(rename = "HomeServer")]
    home_server: String,
}

impl UserConfiguration {

    pub fn get_token(&self) -> &String {
        &self.token
    }

    pub fn get_home_server(&self) -> &String {
        &self.home_server
    }

    pub fn get_home_channel(&self) -> &String {
        &self.home_channel
    }

    pub fn read_from(location: String) -> Result<UserConfiguration, serde_json::Error> {
        use std::fs::File;
        use std::io::Read;

        let mut buf: String = String::new();
        let mut file = File::open(location).unwrap();
        file.read_to_string(&mut buf).expect("Failed to read configuration file to buffer.");
        UserConfiguration::from_json(buf)
    }


    pub fn read() -> Result<UserConfiguration, serde_json::Error> {
        use std::env::home_dir;
        let home = home_dir().expect("Failed to read home directory.");
        let path = home.to_string_lossy() + "/.dui/conf.json";
        UserConfiguration::read_from(path.to_owned().to_string())
    }

    pub fn from_json(json: String) -> Result<UserConfiguration, serde_json::Error> {
        serde_json::from_str(&*json)
    }
}

pub struct Architecture {
    servers: HashMap<model::ServerId, Vec<model::PublicChannel>>,
    server_cache: Vec<model::Server>,
    discord: Arc<discord::Discord>,
    current_server: Option<model::ServerId>,
    current_channel: Option<model::PublicChannel>,
    cached_chat: HashMap<model::ChannelId, Vec<model::Message>>,
    pub size: Rect,
}

impl Architecture {
    pub fn set_servers(&mut self, map: HashMap<model::ServerId, Vec<model::PublicChannel>>) {
        self.servers = map;
        self.fill_server_cache();
        self.select_default();
    }

    pub fn fill_server_cache(&mut self) {
        let (snd, rcv) = mpsc::channel();
        let mut keys: Vec<model::ServerId> = Vec::new();
        for k in self.servers.keys() {
            keys.push(*k);
        }
        let server_len = *(&keys.len());
        for serv in keys {
            let sndc = snd.clone();
            let d = self.get_discord();
            thread::spawn(move || {
                sndc.send(d.get_server(serv).unwrap());
            });
        }

        let mut cache: Vec<model::Server> = Vec::new();

        for _ in 0..server_len {
            let serv = rcv.recv().unwrap();
            cache.push(serv);
        }
        self.server_cache = cache;
    }

    pub fn get_size(&self) -> &Rect {
        &self.size
    }

    pub fn get_chat_block_display(&self) -> String {
        format!("{} - {}" /* add shit for notifications maybe? */, self.get_current_server_info().unwrap().name, self.get_current_channel().unwrap().name)
    }

    pub fn get_cached_chat(&self, chan: &model::ChannelId) -> Option<&Vec<model::Message>> {
        let current = &self.get_current_channel().unwrap().id;
        for (key, val) in self.cached_chat.iter() {
            if key == current {
                return Some(val);
            }
        }
        None
    }

    // call this when updating channels...
    pub fn cache_chat(&mut self) {
        let current_channel = self.get_current_channel().unwrap();
        let current_chan_id = current_channel.id.clone();
        let mut chat_map: &mut HashMap<model::ChannelId, Vec<model::Message>> =  &mut self.cached_chat;
        let messages = self.discord.get_messages(current_chan_id, discord::GetMessages::MostRecent, Some(100));
        if (messages.is_ok()) {
            chat_map.insert(current_chan_id, messages.unwrap());
        } else {
            chat_map.insert(current_chan_id, Vec::new());
        }
    }

    pub fn new(discord: Arc<discord::Discord>) -> Architecture {
        Architecture {
            discord,
            servers: HashMap::new(),
            current_server: None,
            current_channel: None,
            cached_chat: HashMap::new(),
            server_cache: Vec::new(),
            size: Rect::new(0, 0, 0, 0) // use default rect
        }
    }

    fn server_from_id(&self, id: model::ServerId) -> model::Server {
        self.discord.get_server(id).unwrap()
    }

    pub fn get_current_server_display(&self) -> String {
        match self.get_current_server() {
            Some(serv) => self.server_from_id(serv).name,
            None => String::new()
        }
    }

    pub fn display_channels(&self) -> String {
        format!("Channels [{}]", self.get_current_channels().unwrap().len())
    }

    pub fn channel_down(&mut self) {
        let index = self.get_current_channel_index().unwrap();
        if (index + 1) <= (self.get_current_channels().unwrap().len() - 1) {
            let new_channel = self.get_current_channels().unwrap()[index + 1].clone();
            self.set_channel(Some(new_channel));
        }
    }

    pub fn set_channel(&mut self, new_channel: Option<model::PublicChannel>) {
        self.current_channel = new_channel;
        self.cache_chat();
    }

    pub fn channel_up(&mut self) {
        let index = self.get_current_channel_index().unwrap();
        if index != 0 && index - 1 >= 0 {
            let new_channel = self.get_current_channels().unwrap()[index - 1].clone();
            self.set_channel(Some(new_channel));

        }
    }

    pub fn server_up(&mut self) {
        let index = self.get_current_server_index().unwrap();
        if index != 0 && index - 1 >= 0 {
            let new_server = &self.server_cache[index - 1];
            self.current_server = Some(new_server.id);
        }
        self.select_default_channel();
    }

    pub fn server_down(&mut self) {
        let index = self.get_current_server_index().unwrap();
        if (index + 1) <= (self.server_cache.len() - 1) {
            let new_server = &self.server_cache[index + 1];
            self.current_server = Some(new_server.id);
        }
        self.select_default_channel();
    }

    pub fn display_servers(&self) -> String {
        format!("Servers [{}]", self.servers.keys().len())
    }

    pub fn get_current_server(&self) -> Option<model::ServerId> {
        self.current_server
    }

    pub fn get_current_server_info(&self) -> Option<&model::Server> {
        for server in &self.server_cache {
            if server.id == self.get_current_server().unwrap() {
                return Some(server);
            }
        }
        None
    }

    pub fn get_current_channel_index(&self) -> Option<usize> {
        let mut ind: usize = 0;
        let schan = self.get_current_channel().unwrap();
        let chans = self.get_current_channels().unwrap();
        for chan in chans {
            if chan.name == schan.name {
                return Some(ind)
            } else {
                ind += 1;
            }
        }
        None
    }

    pub fn get_current_server_index(&self) -> Option<usize> {
        //        self.get_current_server().map(|s| self.servers.keys().position(|&k| k == s).unwrap() )
        let mut ind: usize = 0;
        for server in self.get_servers() {
            if server.id == self.current_server.unwrap() {
                return Some(ind)
            } else {
                ind += 1;
            }
        }
        None
    }

    pub fn get_current_channels(&self) -> Option<&Vec<model::PublicChannel>> {
        self.current_server.map(|s| self.get_server_map().get(&s).unwrap() )
    }

    pub fn get_current_channel(&self) -> Option<model::PublicChannel> {
        self.current_channel.clone()
    }

    pub fn get_discord(&self) -> Arc<discord::Discord> {
        self.discord.clone()
    }

    pub fn select_default(&mut self) {
        self.current_server = Some(self.get_servers()[0].id);
        self.select_default_channel();
    }

    pub fn select_default_channel(&mut self) {
        self.current_channel = Some(self.get_current_channels().unwrap()[0].clone());
        self.cache_chat();
    }

    pub fn get_servers_for_display(&self) -> Vec<String> {
        // self.get_servers().into_iter().map(|s: model::Server| s.name).collect()
        let mut names: Vec<String> = Vec::new();
        for serv in self.get_servers() {
            names.push(serv.name.clone());
        }
        names
    }

    pub fn get_channels_for_display(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for chan in self.get_current_channels().unwrap() {
            names.push(chan.name.clone());
        }
        names
    }

    pub fn get_servers(&self) -> &Vec<model::Server> {
        &self.server_cache
    }

    pub fn get_server_map(&self) -> &HashMap<model::ServerId, Vec<model::PublicChannel>> {
        &self.servers
    }
}