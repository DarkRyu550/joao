/// Looks like "2019-08-10.20-10-10.0456789"
const TIME_FORMAT: &'static str = "%Y-%m-%d.%H-%M-%S.%f";
const COMPRESSION: u32 = 8;

#[derive(Debug)]
pub enum Error {
    OutputPoisoned,
    QueuePoisoned,
    IoError(std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Record {
    pub level: log::Level,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: String,
}
impl Record {
    pub fn from_log(rec: &log::Record) -> Record {
        Record {
            level: rec.level(),
            target: rec.target().to_owned(),
            file: rec.file().map(|s| s.to_owned()),
            line: rec.line(),
            message: format!("{}", rec.args()),
        }
    }
}

use std::collections::LinkedList;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
pub struct FsLogger<P: AsRef<Path> + Send + Sync> {
    /// Name prefix of the log files.
    name: String,
    /// Maximum number of messages to be queued, any messages
    /// beyond this number in the queue are dropped.
    queue_max: u64,
    /// Maximum size of a file before a rotation is triggered.
    size_max: u64,
    /// Number of older log files to be kept around.
    history_size: u64,
    /// Path to the target diretory.
    dir_path: P,

    logf: Mutex<(u64, File)>,
    logq: Mutex<LinkedList<Record>>,
}
impl<P: AsRef<Path> + Send + Sync> FsLogger<P> {
    pub fn new(
        name: String,
        queue_max: u64,
        size_max: u64,
        history_size: u64,
        dir_path: P,
    ) -> Result<FsLogger<P>, Error> {
        std::fs::create_dir_all(&dir_path).map_err(|what| Error::IoError(what))?;

        let file = {
            let fname = format!("{}.log", name);
            let path = dir_path.as_ref().join(&fname);
            std::fs::File::create(path)
        }
        .map_err(|what| Error::IoError(what))?;

        Ok(FsLogger {
            name: name,
            queue_max: queue_max,
            size_max: size_max,
            history_size: history_size,
            dir_path: dir_path,

            logf: Mutex::new((0, file)),
            logq: Mutex::new(LinkedList::new()),
        })
    }

    fn checkout(&self) -> Result<(), Error> {
        let mut lock = self.logf.lock().map_err(|_| Error::OutputPoisoned)?;

        let (ref mut len, ref mut logf) = *lock;

        let iomap = |what| Error::IoError(what);
        let mut target = {
            use chrono::Local;
            let time = Local::now();
            let time = time.format(TIME_FORMAT);
            let fname = format!("{}.{}.log.xz", self.name, time);
            let path = self.dir_path.as_ref().join(&fname);

            use xz2::write::XzEncoder;
            XzEncoder::new(File::create(path).map_err(iomap)?, COMPRESSION)
        };

        use std::io::{Seek, SeekFrom};
        logf.seek(SeekFrom::Start(0)).map_err(iomap)?;

        use std::io::copy;
        copy(logf, &mut target).map_err(iomap)?;

        logf.set_len(0).map_err(iomap).map(|_| ())?;
        *len = 0;

        Ok(())
    }

    fn clean(&self) -> Result<(), Vec<std::io::Error>> {
        let mut flist = std::fs::read_dir(&self.dir_path)
            .map_err(|what| vec![what])?
            .filter(|entry| match entry {
                Ok(entry) => {
                    entry.path().is_file()
                        && entry
                            .path()
                            .file_name()
                            .map(|name| {
                                name.to_str()
                                    .map(|name| name.starts_with(&self.name))
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false)
                }
                Err(_) => false,
            })
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();

        flist.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut errors = Vec::new();
        if self.history_size < flist.len() as u64 {
            /* Clean up some files. */
            for file in &flist[..(flist.len() - self.history_size as usize)] {
                if let Err(what) = std::fs::remove_file(file) {
                    errors.push(what);
                }
            }

            if errors.len() == 0 {
                Ok(())
            } else {
                Err(errors)
            }
        } else {
            Ok(())
        }
    }
}
impl<P: AsRef<Path> + Send + Sync> log::Log for FsLogger<P> {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _flush = match self.logq.lock() {
            Ok(mut logq) => {
                logq.push_back(Record::from_log(record));
                logq.len() > 0
            }
            Err(what) => {
                eprintln!("!!! RECORD LOCK POISONED: {:?}", what);
                false
            }
        };
    }

    fn flush(&self) {
        let messages = match self.logq.lock() {
            Ok(mut logq) => {
                let messages = logq
                    .iter()
                    .map(|record| format!("{}\n", record.message))
                    .collect::<Vec<_>>();
                logq.clear();

                messages
            }
            Err(what) => {
                eprintln!("!!! RECORD QUEUE LOCK POISONED: {:?}", what);
                return;
            }
        };

        let checkout = match self.logf.lock() {
            Ok(mut logf) => {
                let (ref mut fsize, ref mut logf) = *logf;
                let wsize = messages
                    .into_iter()
                    .fold(Vec::new(), |mut vec, line| {
                        use std::io::Write;
                        let len = line.as_bytes().len();
                        vec.push(logf.write_all(line.as_bytes()).map(move |_| len as u64));
                        vec
                    })
                    .into_iter()
                    .fold(0_u64, |amm, val| {
                        if let Ok(written) = val {
                            amm + written
                        } else {
                            amm
                        }
                    });

                if *fsize + wsize < self.size_max {
                    *fsize += wsize;
                    false
                } else {
                    true
                }
            }
            Err(what) => {
                eprintln!("!!! LOG FILE LOCK POISONED: {:?}", what);
                return;
            }
        };

        if checkout {
            if let Err(what) = self.checkout() {
                eprintln!("Log file checkout failed: {:?}", what);
            }
            if let Err(what) = self.clean() {
                eprintln!("Failed to clean old log files: {:?}", what);
            }
        }
    }
}
impl<P: AsRef<Path> + Send + Sync> Drop for FsLogger<P> {
    fn drop(&mut self) {
        eprintln!(r#"FsLogger("{}") is being dropped"#, self.name);

        use log::Log;
        self.flush();
    }
}

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
fn async_flush<L: log::Log>(logger: Arc<L>, breaks: Arc<AtomicBool>, delay: Duration) {
    use std::sync::atomic::Ordering;
    while !breaks.load(Ordering::Relaxed) {
        logger.flush();

        thread::park_timeout(delay.clone());
    }

    trace!("Stopping AsyncFlusher thread");
    logger.flush();
}

pub struct AsyncFlusher<L: log::Log + 'static> {
    logger: Arc<L>,
    breaks: Arc<AtomicBool>,
    delay: Duration,
}
impl<L: log::Log + 'static> AsyncFlusher<L> {
    pub fn new(
        logger: L,
        delay: Duration,
    ) -> Result<(AsyncFlusher<L>, thread::JoinHandle<()>), std::io::Error> {
        let logger = Arc::new(logger);
        let atomic = Arc::new(AtomicBool::new(false));

        let (l1, b1, d1) = (logger.clone(), atomic.clone(), delay.clone());

        let handle = thread::Builder::new()
            .name("AsyncFlusher".to_owned())
            .spawn(move || async_flush::<L>(l1, b1, d1))?;

        Ok((
            AsyncFlusher {
                logger: logger,
                breaks: atomic,
                delay: delay,
            },
            handle,
        ))
    }

    pub fn stop(&self) {
        use std::sync::atomic::Ordering;
        self.breaks.store(true, Ordering::Relaxed);
    }
}
impl<L: log::Log + 'static> Clone for AsyncFlusher<L> {
    fn clone(&self) -> AsyncFlusher<L> {
        AsyncFlusher {
            logger: self.logger.clone(),
            breaks: self.breaks.clone(),
            delay: self.delay.clone(),
        }
    }
}
impl<L: log::Log + 'static> log::Log for AsyncFlusher<L> {
    fn enabled(&self, a: &log::Metadata) -> bool {
        self.logger.enabled(a)
    }

    fn log(&self, a: &log::Record) {
        self.logger.log(a)
    }

    fn flush(&self) {
        self.logger.flush()
    }
}
