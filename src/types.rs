
use serde_json;
use serde_derive;
use std::error::Error;
use std::sync::Arc;
use discord;
use discord::model;
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
    discord: Arc<discord::Discord>,
    current_server: Option<model::ServerId>,
}

impl Architecture {
    pub fn set_servers(&mut self, map: HashMap<model::ServerId, Vec<model::PublicChannel>>) {
        self.servers = map;
    }

    pub fn new(discord: Arc<discord::Discord>) -> Architecture {
        Architecture {
            discord,
            servers: HashMap::new(),
            current_server: None
        }
    }

    pub fn get_current_server_display(&self) -> String {
        match self.get_current_server() {
            Some(serv) => serv.name,
            None => String::new()
        }
    }

    pub fn display_servers(&self) -> String {
        format!("Servers {}", self.servers.keys().len())
    }

    pub fn get_current_server(&self) -> Option<model::ServerId> {
        self.current_server
    }

    pub fn get_current_channels(&self) -> Option<&Vec<model::PublicChannel>> {
        self.current_server.map(|s| self.get_server_map().get(&s).unwrap() )
    }

    pub fn get_discord(&self) -> Arc<discord::Discord> {
        self.discord.clone()
    }

    pub fn get_server_with_names(&self) -> Vec<String> {
        self.get_servers().into_iter().map(|s: model::Server| s.name).collect()
    }

    pub fn get_servers(&self) -> Vec<model::Server> {
        self.servers.keys().map(|s| self.discord.get_server(*s).expect("Failed to get server")).collect()
    }

    pub fn get_server_map(&self) -> &HashMap<model::ServerId, Vec<model::PublicChannel>> {
        &self.servers
    }
}