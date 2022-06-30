
use std::time::Duration;
use anyhow::Result;
use strum_macros::EnumString;
use tracing::debug;

use crate::async_rt;

use super::loop_util;

pub fn run() -> Result<()> {
    run_with_runtime(true)
}

pub async fn run_async() -> Result<()> {
    run_it().await
}


fn run_with_runtime(is_mt: bool) -> Result<()> {
    debug!("run with multi-thread={}", is_mt);
    if is_mt {
        async_rt::run_multi_thread(run_async())??;
    } else {
        async_rt::run_single_thread(run_async())??;
    }

    Ok(())
}


#[derive(Debug, PartialEq, EnumString)]
enum KickType {
    AllInMain, // 所有循环都在主线程里启动
    AllInTask, // 所有循环都在同一个task里启动
    DeadMainAndPrintTask, // 死循环task在主线程启动，打印task在一个task里启动
    DeadTaskAndPrintMain, // 死循环task在task里启动，打印task在主线程里启动
    DeadTaskAndPrintTask, // 死循环task在task里启动，打印task在另一个task里启动
}

/// MacOS-M1 总结：
/// - print thread永远不会卡死 
/// - 死循环task只要在主线程中启动，不会卡死print task
/// - 死循环task只要是在task中启动，就会卡死print task
async fn run_it() -> Result<()> {

    let kick_type: KickType = "AllInTask".parse()?;
    debug!("kick_type = {:?}", kick_type);
    debug!("runtime threads = {:?}", async_rt::threads().len());
    
    // std::thread::spawn(||{
    //     std::thread::sleep(Duration::from_secs(5));
    //     for thread in async_rt::threads() {
    //         unsafe { 
    //             libc::pthread_kill(thread.os_id, signal_hook::consts::SIGUSR2);
    //         }
    //     }
    // });

    match kick_type {
        KickType::AllInMain => {
            // 正常打印，都不会卡死
            kick_print().await?;
            kick_dead().await?;
        },
        KickType::AllInTask => {
            // print task 卡死
            // print thread 正常打印
            async_rt::spawn_with_name("launch-all-main", async move {
                kick_print().await?;
                kick_dead().await?;
                Result::<()>::Ok(())
            });
        },
        KickType::DeadMainAndPrintTask => {
            // 正常打印，都不会卡死
            async_rt::spawn_with_name("launch-print", async move {
                kick_print().await
            });
            kick_dead().await?;
        },
        KickType::DeadTaskAndPrintMain => {
            // print task 卡死
            // print thread 正常打印
            async_rt::spawn_with_name("launch-dead", async move {
                kick_dead().await
            });
            kick_print().await?;
        },
        KickType::DeadTaskAndPrintTask => {
            // print task 卡死
            // print thread 正常打印
            async_rt::spawn_with_name("launch-print", async move {
                kick_print().await
            });

            async_rt::spawn_with_name("launch-dead", async move {
                kick_dead().await
            });
        }
    }

    // waiting for ever
    tokio::time::sleep(Duration::MAX).await;
    
    Ok(())
}

async fn kick_print() -> Result<()> {
    for _ in 0..10 {
        loop_util::kick_print_loop_task("30")?;
    }
    loop_util::kick_print_loop_thread("30")?;
    Ok(())
}

async fn kick_dead() -> Result<()> {
    // 等待2秒再启动死循环
    tokio::time::sleep(Duration::from_secs(2)).await;
    loop_util::kick_dead_loop()
}

