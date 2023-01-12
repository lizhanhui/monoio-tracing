use opentelemetry::sdk::export::trace::stdout;
use std::{error::Error, time::Duration};

use tracing::{event, instrument, span, Level};
use tracing_subscriber::{
    fmt, layer::Context, prelude::__tracing_subscriber_SubscriberExt, registry::LookupSpan, Layer,
};

struct MyLayer;

impl<S> Layer<S> for MyLayer
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_enter(&self, _id: &span::Id, _ctx: Context<'_, S>) {
        let metadata = _ctx.metadata(_id);
        println!("Enter {:#?}, metadata: {:#?}", _id, metadata);
    }
}

#[monoio::main(timer = true)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start configuring a `fmt` subscriber
    configure_tracing()?;

    let _ = trace_me(1, 3);

    let _ = foo(100).await?;

    Ok(())
}

fn configure_tracing() -> Result<(), Box<dyn Error>> {
    // let fmt_layer = fmt::layer().with_target(false);

    let my_layer = MyLayer;

    // Configure a custom event formatter
    let format = fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .with_line_number(true)
        .with_source_location(true)
        .with_file(true)
        .compact(); // use the `Compact` formatting style.

    let fmt_layer = fmt::layer().event_format(format).with_target(true);

    // Create a new OpenTelemetry pipeline
    let tracer = stdout::new_pipeline().install_simple();

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = tracing_subscriber::Registry::default()
        // .with(my_layer)
        .with(fmt_layer)
        .with(telemetry);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    Ok(())
}

#[instrument(level = "info", name = "Handler::run")]
async fn foo(duration: u64) -> Result<(), Box<dyn Error>> {
    monoio::time::sleep(Duration::from_millis(duration)).await;
    event!(Level::INFO, "Inside foo");
    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    a + b
}
