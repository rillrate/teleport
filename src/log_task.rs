use crate::logparser::{LogFormat, LogParser, LogRecord};
use crate::supplier::Supplier;
use anyhow::Error;
use async_trait::async_trait;
use futures::{select, FutureExt, StreamExt};
use meio::prelude::{LiteTask, ShutdownReceiver};
use rill::{
    pathfinder::{Pathfinder, Record},
    provider::LogProvider,
};

pub struct LogTask<T: Supplier> {
    supplier: T,
    format: LogFormat,
}

impl<T: Supplier> LogTask<T> {
    pub fn new(supplier: T, format: LogFormat) -> Self {
        Self { supplier, format }
    }
}

#[async_trait]
impl<T: Supplier> LiteTask for LogTask<T> {
    async fn routine(mut self, signal: ShutdownReceiver) -> Result<(), Error> {
        let log_parser = LogParser::build(self.format)?;
        let mut providers: Pathfinder<LogProvider> = Pathfinder::new();
        let done = signal.just_done().fuse();
        tokio::pin!(done);
        let supplier = &mut self.supplier;
        loop {
            select! {
                line = supplier.next() => {
                    if let Some(line) = line.transpose()? {
                        let res = log_parser.parse(&line);
                        match res {
                            Ok(LogRecord { path, timestamp, message }) => {
                                let provider = providers.find(&path).and_then(Record::get_link);
                                if let Some(provider) = provider {
                                    if provider.is_active() {
                                        provider.log(timestamp, message);
                                    }
                                } else {
                                    let provider = LogProvider::new(path.clone());
                                    providers.dig(path).set_link(provider);
                                }
                            }
                            Err(err) => {
                                log::error!("Can't parse line \"{}\": {}", line, err);
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ = done => {
                    break;
                }
            }
        }
        Ok(())
    }
}