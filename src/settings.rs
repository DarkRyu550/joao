use serde_derive::{Serialize, Deserialize};

use std::path::PathBuf;
use std::time::Duration;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct FilesystemLogger {
	pub enabled: bool,
	pub name: String,
	pub max_queue_size: u64,
	pub max_file_size: u64,
	pub history_size: u64,
	pub directory: PathBuf,
	pub flush_period: Duration
}
impl Default for FilesystemLogger {
	fn default() -> FilesystemLogger {
		FilesystemLogger {
			enabled:        false,
			name:           "log".to_owned(),
			max_queue_size: 67108864,
			max_file_size:  67108864,
			history_size:   8,
			directory:      PathBuf::from("./log/"),
			flush_period:   Duration::new(10, 0)
		}
	}
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Logging {
	pub level: log::Level
}
impl Default for Logging {
	fn default() -> Logging {
		let level = if cfg!(debug_assertions) {
			log::Level::Trace
		} else { log::Level::Info };

		Logging {
			level: level
		}
	}
}

use std::net::SocketAddr;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Settings {
	pub listen_address: SocketAddr,
	pub database_address: SocketAddr,
	pub logging: Logging,
	pub filesystem_logger: FilesystemLogger,
}
impl Default for Settings {
	fn default() -> Settings {
		Settings {
			listen_address:    SocketAddr::from(([0,   0, 0, 0], 80)),
			database_address:  SocketAddr::from(([127, 0, 0, 1], 6380)),
			logging:           Default::default(),
			filesystem_logger: Default::default() 
		}
	}
}

