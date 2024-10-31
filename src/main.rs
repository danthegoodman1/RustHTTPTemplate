use citizenstats_server::start;
use tokio::signal;
use tracing::{level_filters, Level};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_target(false)
            // .json()
            .with_filter(level_filters::LevelFilter::from_level(Level::DEBUG)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    tokio::select! {
        _ = start("0.0.0.0:8080") => {},
        _ = signal::ctrl_c() => {
            println!("Received Ctrl-C, shutting down...");
        }
    }
}
