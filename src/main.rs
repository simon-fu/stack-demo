
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
    // test_crash()?;
    // if test_crash().is_ok() {
    //     return Ok(())
    // }

    // let r = std::backtrace::Backtrace::capture();
    init_log();

    




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

    
    // let console_layer = console_subscriber::spawn();
    let console_layer = console_subscriber::ConsoleLayer::builder().spawn();
    
    tracing_subscriber::registry()
        .with(console_layer)
        .with(
            tracing_subscriber::fmt::layer()
            // .with_env_filter(EnvFilter::new("debug,tokio=trace,runtime=trace"))
            .with_thread_ids(true)
            .with_filter(EnvFilter::new("debug"))
        )
        .init();

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


