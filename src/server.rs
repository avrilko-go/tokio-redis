use anyhow::{Error, Result};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Semaphore;
use tokio::time;
use tracing::{debug, error, info, instrument};
use crate::command::Command;
use crate::connection::Connection;
use crate::db::Db;
use crate::shutdown::Shutdown;


const MAX_CONNECTIONS: usize = 1024;

#[instrument(skip_all)]
pub async fn run(listener: TcpListener, shutdown: impl Future) -> Result<()> {
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);
    let (notify_shutdown, _) = broadcast::channel(1);

    let mut server = Listener {
        db: Default::default(),
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_rx,
        shutdown_complete_tx,
        listener,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                error!(cause = %e, "fail to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down...");
        }
    }

    let Listener {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;

    info!("shutdown complete");

    Ok(())
}


#[derive(Debug)]
struct Listener {
    db: HashMap<String, String>,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: Sender<()>,
    shutdown_complete_rx: Receiver<()>,
    listener: TcpListener,
}

impl Listener {
    pub async fn run(&self) -> Result<()> {
        info!("accepting inbound connections");

        loop {
            let semaphore = self.limit_connections.clone().acquire_owned().await?;

            let stream = self.accept().await?;

            let mut handler = Handler {
                db: Db {},
                connection: Connection::new(stream),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete_tx: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = %err, "handler fail by");
                }
                drop(semaphore);
            });
        }
    }

    pub async fn accept(&self) ->Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                },
                Ok((tcp_stream,addr)) => {
                    info!("{:?} try to connect",addr);
                    return Ok(tcp_stream);
                }
            }
            time::sleep(time::Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

#[derive(Debug)]
struct Handler {
    db:Db,
    connection:Connection,
    shutdown:Shutdown,
    _shutdown_complete_tx :Sender<()>
}

impl Handler {
    async fn run(&mut self) ->Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                _ = self.shutdown.recv() =>  {
                    return Ok(());
                },

                res = self.connection.read_frame() => res?
            };

            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(())
            };


            let cmd = Command::from(frame)?;
            debug!(?cmd);
            cmd.apply(&self.db,&mut self.connection, &mut self.shutdown).await?;
        }
        Ok(())
    }
}

