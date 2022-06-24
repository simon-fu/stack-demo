
mod async_rt;
mod loop_test;


use anyhow::Result;
use tracing_subscriber::EnvFilter;

// #[tokio::main]
// async fn main() -> Result<()> {
//     init_log();
//     loop_test::run_async().await
// }

fn main() -> Result<()> {
    // let r = std::backtrace::Backtrace::capture();
    init_log();
    loop_test::run()
}

fn init_log() {
    tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new("debug"))
    .with_thread_ids(true)
    .init();
}


