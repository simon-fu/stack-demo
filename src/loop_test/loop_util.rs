use std::{sync::atomic::{AtomicBool, AtomicU64, Ordering}, time::Duration};
use anyhow::{Result, Context};
use tokio::task::JoinHandle;
use tracing::debug;
use crate::async_rt::spawn_with_name;


pub fn kick_dead_loop() -> Result<()> {
    lazy_static::lazy_static! {
        static ref FLAG: AtomicBool = AtomicBool::new(false);
        static ref COUNT: AtomicU64 = AtomicU64::new(0);
    }
    const ORDERING: Ordering = Ordering::Relaxed;
    
    if !FLAG.swap(true, ORDERING) {
        debug!("kick dead_loop");
        spawn_with_name("dead_loop_task", async move {
            COUNT.store(0, ORDERING);
            let mut n = 0_u64;
            debug!("dead_loop begin with n={}", n);
            while n < u64::MAX/2 {
                n +=1; 
                COUNT.fetch_add(1, ORDERING);
            }
            debug!("dead_loop end with n={}, count={}", n, COUNT.load(ORDERING));
            FLAG.store(false, ORDERING);
        });
    } else {
        debug!("dead_loop already started");
    }
    Ok(())
}

pub fn kick_print_loop_task(arg: &str) -> Result<Option<JoinHandle<()>>> {
    lazy_static::lazy_static! {
        // static ref FLAG: AtomicBool = AtomicBool::new(false);
        static ref COUNT: AtomicU64 = AtomicU64::new(0);
    }
    const ORDERING: Ordering = Ordering::Relaxed;
    
    let num: u64 = arg.parse().with_context(|| "loop arg must be u64")?;

    // if !FLAG.swap(true, ORDERING) {
        let session_no = COUNT.fetch_add(1, ORDERING);
        let ss_name = format!("print_loop_task-{}", session_no);
        debug!("kick {}", ss_name);
        let task = spawn_with_name(ss_name.as_str(), async move {
            debug!("will loop seconds {}", num);
            for n in 0..num {
                debug!("Loop No.{}/{}", n+1, num);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            debug!("loop end");
            // FLAG.store(false, ORDERING);
        });
    // } else {
    //     debug!("print_loop_task already started");
    // }
    Ok(Some(task))
}

pub fn kick_print_loop_thread(arg: &str) -> Result<()> {
    lazy_static::lazy_static! {
        static ref FLAG: AtomicBool = AtomicBool::new(false);
    }
    const ORDERING: Ordering = Ordering::Relaxed;
    
    let num: u64 = arg.parse().with_context(|| "loop arg must be u64")?;

    if !FLAG.swap(true, ORDERING) {
        debug!("kick print_loop_thread");
        std::thread::spawn(move || {
            let span = tracing::span!(parent:None, tracing::Level::INFO, "", s = "print_loop_thread");
            let _enter = span.enter();
            debug!("will loop seconds {}", num);
            for n in 0..num {
                debug!("Loop No.{}/{}", n+1, num);
                std::thread::sleep(Duration::from_secs(1));
            }
            debug!("loop end");
            FLAG.store(false, ORDERING);
        });
    } else {
        debug!("print_loop_thread already started");
    }
    Ok(())
}


