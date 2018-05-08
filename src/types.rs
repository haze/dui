
use serde_json;
use serde_derive;
use std::error::Error;


// located at ~/.duirc
#[derive(Serialize, Deserialize)]
pub struct UserConfiguration {

    #[serde(default)]
    token: String,

    #[serde(default)]
    home_channel: String,

    #[serde(default)]
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