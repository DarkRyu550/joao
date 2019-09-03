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
mod serde_resolve {
	/* NOTE: Due to some idiocy in the way FromStr is implemented for 
	 * SocketAddr, deserialization cannot resolve any names as it would
	 * do if you just used address.to_socket_addrs(). */
	use serde::de::{Deserializer, Deserialize, Visitor, Error};
	use std::net::SocketAddr;
	pub fn deserialize<'de, D: Deserializer<'de>>(de: D) 
		-> Result<SocketAddr, D::Error> {
		
		if de.is_human_readable() {
			struct ResolverVisitor;
			impl<'de> Visitor<'de> for ResolverVisitor {
				type Value = SocketAddr;

				fn expecting(&self, format: &mut std::fmt::Formatter) 
					-> std::fmt::Result {
					
					format.write_str("resolvable socket address")
				}

				fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> 
					where E: Error {
					
					use std::net::ToSocketAddrs;
					s.to_socket_addrs()
						.map_err(Error::custom)?
						.next()
						.ok_or("no match".to_owned())
						.map_err(Error::custom)
				}
			}

			de.deserialize_str(ResolverVisitor)
		} else {
			/* Carry on as usual. */
			<SocketAddr as Deserialize<'de>>::deserialize(de)
		}
	}
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Settings {
	#[serde(deserialize_with = "serde_resolve::deserialize")]
	pub listen_address: SocketAddr,

	#[serde(deserialize_with = "serde_resolve::deserialize")]
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

#[derive(Debug)]
pub enum Error {
	InvalidDatabase,
	InvalidListen,
	Poisoned,
}

use std::sync::RwLock;
lazy_static! { 
	static ref SETTINGS: RwLock<Settings> = RwLock::new(Default::default());
}

pub fn settings() -> Settings {
	(*SETTINGS.read().expect("Settings lock has been poisoned")).clone()
}

pub fn store(new: Settings) -> Result<(), (Settings, Error)> {
	use std::sync::RwLockWriteGuard;
	let mut lock: RwLockWriteGuard<Settings> = match SETTINGS.write() {
		Ok(lock) => lock,
		Err(_) => return Err((new, Error::Poisoned))
	};

	use std::mem;
	let _ = mem::replace(&mut *lock, new);

	Ok(())
}

