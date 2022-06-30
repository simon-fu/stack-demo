
mod async_rt;
mod loop_test;
mod monitor;


use std::time::Duration;
use anyhow::Result;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use crate::monitor::rcpu::{self, SThreadCpuSnapshot};

// #[tokio::main]
// async fn main() -> Result<()> {
//     init_log();
//     loop_test::run_async().await
// }

fn main() -> Result<()> {
    // test_crash()?;
    // if test_crash().is_ok() {
    //     return Ok(())
    // }

    // let r = std::backtrace::Backtrace::capture();
    init_log();

    {
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
        let handler = Box::new(monitor::BuringThreadsMonitor::new(
            threshold, 
            Duration::from_secs(10), 
            Duration::from_secs(3), 
            move |states: &Vec<SThreadCpuSnapshot>| {
                debug!("-----{}-----", rcpu::current_thread()?);
                for state in states {
                    println!(
                        "  thread cpu usage: [{}]",
                        state
                    );
                }
                Ok(())
            }
        ));
        let _monitor_guard = monitor::monitor_process_cpu(
            pid, 
            handler
        )?;
        std::thread::sleep(Duration::from_secs(99999));
        println!("sleep done");
    }


    // #[cfg(any(target_os = "linux", target_os = "android"))]
    // {
    //     let pid = 13324;
    //     let filename = "/tmp/dump.dmp";
    //     let r = linux::dump_pid(pid, true, filename);
    //     match r {
    //         Ok(_r) => {
    //             println!("dumped: pid=[{}], file=[{}]", pid, filename);
    //             return Ok(())
    //         },
    //         Err(_) => {
    //             r?;
    //         }
    //     }
    // }


    // let signal = unsafe {
    //     signal_hook::low_level::register(signal_hook::consts::SIGUSR2, || {
    //         let backtrace = backtrace::Backtrace::new();
    //         tracing::error!("backtrace {:?}", backtrace);
    //     })
    // }?;


    
    // use std::sync::Arc;
    // use std::sync::atomic::{AtomicBool, Ordering};
    // let term = Arc::new(AtomicBool::new(false));

    // signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    // while !term.load(Ordering::Relaxed) {
    //     // Do some time-limited stuff here
    //     // (if this could block forever, then there's no guarantee the signal will have any
    //     // effect).
    // }

    loop_test::run()?;

    // signal_hook::low_level::unregister(signal); // Not really necessary.
    Ok(())
}

fn init_log() {

    // tracing_subscriber::fmt()
    // .with_env_filter(EnvFilter::new("debug,tokio=trace,runtime=trace"))
    // .with_thread_ids(true)
    // .init();
    
    // console_subscriber::init();

    use tracing_subscriber::prelude::*;

    // let console_layer = console_subscriber::ConsoleLayer::builder().spawn();
    
    tracing_subscriber::registry()
        // .with(console_layer)
        .with(
            tracing_subscriber::fmt::layer()
            // .with_env_filter(EnvFilter::new("debug,tokio=trace,runtime=trace"))
            .with_target(false)
            .with_thread_ids(true)
            .with_filter(EnvFilter::new("debug,goblin=warn"))
        )
        .init();
}

fn build_some_threads() {
    for _ in 0..5 {
        std::thread::spawn(|| loop {
            let _ = (0..9_000).into_iter().sum::<i128>();
        });
    }
}
mod cpu_util {
    // use std::{time::Duration, sync::Arc};

    // use anyhow::{Result, Context};
    // use parking_lot::{Condvar, Mutex};


    

    // fn monitor_cpu_of_threads(pid: u32) -> Result<()> {        

    //     loop {

    //         let process = rcpu::get_process(pid)
    //         .with_context(|| format!("unable to get info of pid [{}]", pid))?;

    //         let r = print_process(&process);
    //         if let Err(e) = r {
    //             println!("{:?}", e);
    //         }
    //     }
    // }

    // fn print_process(process: &RProcess) -> Result<()> {
    //     let mut thread_stats = rcpu::get_process_thread_stats(&process)?;
        
    //     {
    //         const INTERVAL: Duration = Duration::from_secs(5);
    //         std::thread::sleep(INTERVAL.clone());
    //     }

    //     println!("-----[{:?}]-----", rcpu::current_thread()?);
    //     for (thd, stat_t) in &mut thread_stats {
    //         let usage_t = stat_t.cpu().with_context(||format!("unable to get cpu of thread [{:?}]", thd))?;
    //         if usage_t > 0.96 {
    //             println!(
    //                 "[CPU] thread usage: [{:?}] -> [{:.2}%]",
    //                 thd, usage_t * 100f64
    //             );
    //         }
    //     }

    //     Ok(())
    // }

    

    // fn run_monitor() -> Result<()> {
    //     let core_num = processor_numbers().with_context(||"unable to get processor num")?;
    //     let mut stat_p = ProcessStat::cur().with_context(||"unable to get process state")?;
    //     let mut stat_t = ThreadStat::cur().with_context(||"unable to get thread state")?;

    //     let mut last_loop = Instant::now();
    //     loop {
    //         if last_loop.elapsed() > Duration::from_secs(1) {
    //             last_loop = Instant::now();
    //         } else {
    //             std::thread::sleep(Duration::from_micros(100));
    //             continue;
    //         }
    //         println!("----------");

    //         // cpu
    //         let _ = (0..1_000).into_iter().sum::<i128>();

    //         let usage_p = stat_p.cpu().with_context(||"unable to get process cpu")? * 100f64;
    //         let usage_t = stat_t.cpu().with_context(||"unable to get thread cpu")? * 100f64;

    //         println!(
    //             "[CPU] core Number: {}, process usage: {:.2}%, current thread usage: {:.2}%",
    //             core_num, usage_p, usage_t
    //         );

    //         // mem
    //         let mem_info = get_process_memory_info().unwrap();

    //         println!(
    //             "[Memory] memory used: {} bytes, virtural memory used: {} bytes ",
    //             mem_info.resident_set_size, mem_info.virtual_memory_size
    //         );

    //         // fd
    //         let fd_num = fd_count_cur().unwrap();

    //         println!("[FD] fd number: {}", fd_num);

    //         // io
    //         let io_stat = get_process_io_stats().unwrap();

    //         println!(
    //             "[IO] io-in: {} bytes, io-out: {} bytes",
    //             io_stat.read_bytes, io_stat.write_bytes
    //         );
    //     }

    //     // Result::<()>::Ok(())
    // }


}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    use anyhow::{Result, Context};
    use minidump_writer::{
        minidump_writer::MinidumpWriter,
        thread_info::Pid,
        crash_context::CrashContext,
    };
    use nix::errno::Errno;

    pub fn dump_pid(pid: Pid, with_ctx: bool, filename: &str) -> Result<()> {
        do_dump_pid(filename, pid, with_ctx)
        .with_context(||format!("fail to dump pid [{}], with_ctx [{}], file [{}], ", pid, with_ctx, filename))?;
        let mut tmpfile = std::fs::File::create(filename)
        .with_context(||format!("failed to create mdump file [{}]", filename))?;
    
        let mut tmp = MinidumpWriter::new(pid, pid);
        #[cfg(not(any(target_arch = "mips", target_arch = "arm")))]
        if with_ctx {
            let crash_context = get_crash_context(pid);
            tmp.set_crash_context(crash_context);
        }

        let _in_memory_buffer = tmp.dump(&mut tmpfile)
        .with_context(||format!("failed to write mdump file [{}]", filename))?;

        Ok(())
    }

    fn do_dump_pid(filename: &str, pid: Pid, with_ctx: bool) -> Result<()> {
    
        let mut tmpfile = std::fs::File::create(filename)
        .with_context(||"failed to create mini dump file")?;
    
        let mut tmp = MinidumpWriter::new(pid, pid);
        #[cfg(not(any(target_arch = "mips", target_arch = "arm")))]
        if with_ctx {
            let crash_context = get_crash_context(pid);
            tmp.set_crash_context(crash_context);
        }

        let _in_memory_buffer = tmp.dump(&mut tmpfile)
        .with_context(||format!("failed to write mini dump file"))?;

        Ok(())
    }

    #[cfg(not(any(target_arch = "mips", target_arch = "arm")))]
    fn get_ucontext() -> Result<crash_context::ucontext_t> {
        let mut context = std::mem::MaybeUninit::uninit();
        unsafe {
            let res = crash_context::crash_context_getcontext(context.as_mut_ptr());
            Errno::result(res)?;

            Ok(context.assume_init())
        }
    }

    #[cfg(not(any(target_arch = "mips", target_arch = "arm")))]
    fn get_crash_context(tid: Pid) -> CrashContext {
        let siginfo: libc::signalfd_siginfo = unsafe { std::mem::zeroed() };
        let context = get_ucontext().expect("Failed to get ucontext");
        let float_state = unsafe { std::mem::zeroed() };
        CrashContext {
            inner: crash_context::CrashContext {
                siginfo,
                pid: std::process::id() as _,
                tid,
                context,
                float_state,
            },
        }
    }

}

// fn test_crash() -> Result<()> {
//     let filename = "/tmp/demo.dmp";
//     make_minidump_macos(filename)?;
//     println!("wrote minidump [{}]", filename);

//     // #[cfg(any(target_os = "linux", target_os = "android"))]
//     // unsafe {
//     //     let mut context = std::mem::zeroed();
//     //     crash_context::crash_context_getcontext(&mut context);
//     // }

//     Ok(())
// }

// fn make_minidump_macos(filename: &str) -> Result<()> {
//     let cc = unsafe {
//         crash_context::CrashContext {
//             task: libc::mach_task_self(),
//             thread: libc::mach_thread_self(),
//             handler_thread: 0, 
//             // handler_thread: libc::mach_thread_self(),
//             // handler_thread: mach2::port::MACH_PORT_NULL,
//             exception: None,
//         }
//     };

//     let mut writer = minidump_writer::minidump_writer::MinidumpWriter::new(cc);

//     let mut minidump_file = std::fs::File::create(filename)
//     .with_context(|| format!("failed to create file [{}]",filename))?;

//     writer.dump(&mut minidump_file)
//     .with_context(|| format!("failed to write file [{}]",filename))?;

//     Ok(())
// }



// fn write_minidump_linux(crash_context: crash_context::CrashContext) {
//     minidump_writer::minidump_writer::MinidumpWriter::new(crash_context)
//     // At a minimum, the crashdump writer needs to know the process and thread that the crash occurred in
//     let mut writer = minidump_writer::minidump_writer::MinidumpWriter::new(crash_context.pid, crash_context.tid);

//     // If provided with a full [crash_context::CrashContext](https://docs.rs/crash-context/latest/crash_context/struct.CrashContext.html),
//     // the crash will contain more info on the crash cause, such as the signal
//     writer.set_crash_context(minidump_writer::crash_context::CrashContext { inner: crash_context });

//     // Here we could add more context or modify how the minidump is written, eg
//     // Add application specific memory blocks to the minidump
//     //writer.set_app_memory()
//     // Sanitize stack memory before it is written to the minidump by replacing
//     // non-pointer values with a sentinel value
//     //writer.sanitize_stack();

//     let mut minidump_file = std::fs::File::create("example_dump.mdmp").expect("failed to create file");
//     writer.dump(&mut minidump_file).expect("failed to write minidump");
// }

// fn write_minidump_macos(crash_context: crash_context::CrashContext) -> Result<()>{
//     let mut writer = minidump_writer::minidump_writer::MinidumpWriter::new(crash_context);

//     let mut minidump_file = std::fs::File::create("/tmp/example_dump.mdmp").expect("failed to create file");
//     writer.dump(&mut minidump_file).expect("failed to write minidump");
//     Ok(())
// }


