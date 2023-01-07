use std::env;
use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal::ctrl_c;
use tracing::{span};
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

use tokio_redis::server;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(default_value = "127.0.0.1:6379", short, long)]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_log()?;

    let cli = Cli::parse();
    let addr: SocketAddr = cli.addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    let root = span!(tracing::Level::INFO, "main");
    let _ = root.enter();
    server::run(listener, ctrl_c()).await?;
    Ok(())
}

fn init_log() -> Result<()> {
    env::set_var("RUST_LOG", "INFO");

    let std_log = fmt::layer().event_format(format().pretty()).compact();

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("tokio-server")
        .install_simple()?;
    let jaeger = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(jaeger)
        .with(std_log)
        .init();
    Ok(())
}
