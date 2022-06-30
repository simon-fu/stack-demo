use std::time::Duration;
use anyhow::Result;
use tracing::debug;

use crate::monitor::{rcpu::{SThreadCpuSnapshot, self}, spawn_burning_monitor};

pub fn run() -> Result<()> {
    // let pid = Option::<u32>::Some(10153);
    let pid = Option::<u32>::None;
    
    let pid = match pid {
        Some(v) => v,
        None =>  {
            build_some_threads();
            std::process::id()
        },
    };
    
    let threshold = 96.0_f64;
    let unit_interval = Duration::from_secs(10);
    let duration = Duration::from_secs(3);

    let _guard = spawn_burning_monitor(pid, threshold, unit_interval, duration, 
        move |states: &Vec<SThreadCpuSnapshot>| {
        debug!("-----{}-----", rcpu::current_thread()?);
        for state in states {
            debug!(
                "  thread cpu usage: [{}]",
                state
            );
        }
        Ok(())
    })?;

    std::thread::sleep(Duration::from_secs(99999));
    println!("sleep done");
    Ok(())
}

fn build_some_threads() {
    for _ in 0..5 {
        std::thread::spawn(|| loop {
            let _ = (0..9_000).into_iter().sum::<i128>();
        });
    }
}
