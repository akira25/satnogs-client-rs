use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Network {
	pub token: String,
	pub url: String,
	pub query_interval: u32,
	pub post_interval: u32,
	pub timeout: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
	pub token: String,
	pub url: String,
	pub timeout: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Station {
	pub id: u32,
	pub lat: f32,
	pub lon: f32,
	pub alt: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Scripting {
	pub pre_observation_script: bool,
	pub pre_obs_path: String,
	pub post_observation_script: bool,
	pub post_obs_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Storage {
	pub remove_raw_files: bool,
	pub keep_artifacts: bool,
	pub artifacts_path: String,
	// pub output: String,
	// pub complete_output: String,
	// pub incomplete_output: String,
	// pub artifacts_output: String,
	pub enable_iq_dump: bool,
	pub iq_dump_filename: String,
}

//#[derive(Debug, Deserialize)]
//struct Misc {
//    id: u8,
//}

//#[derive(Debug, Deserialize)]
//struct Rotor {
//    id: u8,
//}

//#[derive(Debug, Deserialize)]
//struct Rig {
//    id: u8,
//}

//#[derive(Debug, Deserialize)]
//struct Sdr {
//    id: u8,
//}

//#[derive(Debug, Deserialize)]
//struct Sparkpost {
//    version: u8,
//}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
	pub network: Network,
	pub db: Database,
	pub station: Station,
	pub scripting: Scripting,
	pub storage: Storage,
}

impl Settings {
	pub fn new() -> Result<Self, ConfigError> {
		// let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

		let s = Config::builder()
			// default configuration values
			.add_source(config::File::with_name("/etc/satnogs-client-rs/defaults.toml"))
			.add_source(config::File::with_name("/etc/satnogs-client-rs/config.toml"))
			.add_source(config::File::with_name(".defaults.toml").required(false))
			.add_source(config::File::with_name("config.toml").required(false))
			// Add in settings from the environment (with a prefix of SATNOGS), e.g. enable debug
			.add_source(config::Environment::with_prefix("SATNOGS"))
			.build()
			.unwrap();

		// You can deserialize (and thus freeze) the entire configuration as
		s.try_deserialize()
	}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_cfg() {
        let settings = Settings::new();

        println!("{:?}", settings);
        assert!(settings.unwrap().network.url == "https://network.satnogs.org/api/");
        // assert!(false)
    }
}
