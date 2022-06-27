use std::future::Future;

use anyhow::Result;

use super::{on_thread_start, on_thread_stop};

pub fn run_multi_thread<F: Future>(future: F) -> Result<F::Output>{
    let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    // .worker_threads(8)
    // .on_thread_start(||{
    //     debug!("tokio thread started");
    // })
    .on_thread_start(on_thread_start)
    .on_thread_stop(on_thread_stop)
    .build()?;

    Ok(rt.block_on(future))
}

pub fn run_single_thread<F: Future>(future: F) -> Result<F::Output> {
    let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .on_thread_start(||{
        // debug!("tokio thread started");
    })
    .build()?;

    Ok(rt.block_on(future))
}

#[inline]
pub fn spawn_with_name<I, T>(name: I, fut: T) -> tokio::task::JoinHandle<T::Output>
where
    I: Into<String>,
    T: std::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    let name:String = name.into();
    let span = tracing::span!(parent:None, tracing::Level::INFO, "", s = &name[..]);
    tokio::spawn(tracing::Instrument::instrument(fut, span))
}
