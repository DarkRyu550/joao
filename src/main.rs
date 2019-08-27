#[macro_use]
extern crate log;

const PKG_NAME:    &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_TITLE:   &'static str = "The Impenetrable";

mod logger;
mod settings;

mod cmdargs {
	pub struct Arguments {
		pub config: String,
	}
	impl Default for Arguments {
		fn default() -> Arguments {
			Arguments { 
				config: "config.toml".to_owned()
			}
		}
	}
	use core::iter::FromIterator;
	impl FromIterator<Bits> for Arguments {
		fn from_iter<T: IntoIterator<Item = Bits>>(iter: T) -> Arguments {
			let mut iter = iter.into_iter();
			let mut args: Arguments = Default::default();

			while let Some(bit) = iter.next() {
				match bit {
					Bits::Config(config) => args.config = config,
				}
			}

			args
		}
	}

	pub enum Bits {
		Config(String)
	}

	pub struct Parser<I: Iterator<Item = String>>(pub I);
	impl<I: Iterator<Item = String>> Iterator for Parser<I> {
		type Item = Result<Bits, String>;

		fn next(&mut self) -> Option<Self::Item> {
			let bit = if let Some(bit) = self.0.next() {
				bit 
			} else { return None };

			Some(match bit.as_str() {
				"-c" | "--config" => 
					if let Some(path) = self.0.next() {
						Ok(Bits::Config(path))
					} else {
						Err("expected path to config file".to_owned())
					}
				_ => Err(format!("unknown command line parameter: {}", bit))
			})
		}
	}
}

fn main() {
	/* Initialize settings. */
	let args = cmdargs::Parser(std::env::args().skip(1))
		.map(|parsed|
			match parsed {
				Ok(bit) => bit,
				Err(what) => {
					eprintln!("{}", what);
					eprintln!("Usage: {} [-c <config file>]", PKG_NAME);
					std::process::exit(1)
				}
			})
		.collect::<cmdargs::Arguments>();
	
	let settings: settings::Settings = {
		use std::fs::File;
		let data = match File::open(&args.config) {
			Ok(mut file) => {
				let mut string = String::new();

				use std::io::Read;
				file.read_to_string(&mut string)
					.expect("Could not read config file");

				string
			},
			Err(what) => {
				eprintln!("Cannot open config file {}, using defaults: {:?}",
					&args.config, what);

				"".to_owned()
			}
		};

		toml::from_str(&data)
			.expect("Could not parse config file")
	};

	/* Initialize logger. */	
	let mut dispatch = {
		let sclone = settings.clone();
		fern::Dispatch::new()
			.format(|fmt, message, record| { 
				fmt.finish(
					format_args!("[{}]{}[{}] {}",
						record.level(),
						match (record.file(), record.line()) {
							(Some(file), Some(line)) =>
								format!("[{}:{}]", file, line),
							(Some(file), None) =>
								format!("[{}]", file),
							(None, Some(line)) =>
								format!("[Line {}]", line),
							(None, None) =>
								"".to_owned()
						},
						record.target(),
						message)
				)
			})
			.filter(move |meta| {
				meta.level() <= sclone.logging.level
			})
			.chain(std::io::stderr())
	};
	
	let fslog = if settings.filesystem_logger.enabled {
		let (fslog, fslog_thread) = logger::AsyncFlusher::new(
			logger::FsLogger::new(
				settings.filesystem_logger.name.clone(),
				settings.filesystem_logger.max_queue_size,
				settings.filesystem_logger.max_file_size,
				settings.filesystem_logger.history_size,
				settings.filesystem_logger.directory.clone()
			).expect("Could not initialize filesystem logger"),
			settings.filesystem_logger.flush_period.clone()
		).expect("Could not initialize async flusher");

		dispatch = dispatch.chain(Box::new(fslog.clone()) as Box<log::Log>);
		Some((fslog, fslog_thread))
	} else { None };

	dispatch.apply()
		.expect("Could not initialize logger");
	
	trace!("We have a logger!");
	trace!("Our settings are: {:?}", settings);
	
	/* Start up. */
	info!("{} - {}", PKG_NAME, PKG_TITLE);
	info!("Version {}", PKG_VERSION);

	/* Shut down filesystem logger. */
	fslog.and_then(|(fslog, fslog_thread)|{
		fslog.stop();
		fslog_thread.thread().unpark();
		fslog_thread.join();

		Some(())
	});
}
