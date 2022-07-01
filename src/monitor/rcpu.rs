use std::{io::Read, fmt::Write};

use anyhow::{Result, Context};
use perf_monitor::cpu::{ThreadStat, cur_thread_id};
use remoteprocess::{Tid, Pid};


#[derive(Clone)]
pub struct ROsThread {
    tid: Tid,
    name: Option<String>,
}

impl std::fmt::Debug for ROsThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ROsThread").field("tid", &self.tid).field("name", &self.name).finish()
    }
}

impl  std::fmt::Display for ROsThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;
        write!(f, "tid: {}", self.tid)?;
        write!(f, ", name: {:?}", self.name)?;
        f.write_char('}')
    }
}

pub struct SThreadCpuSnapshot {
    thread: ROsThread,
    cpu: f64,
}

impl SThreadCpuSnapshot {
    pub fn tid(&self) -> Tid {
        self.thread.tid
    }

    pub fn cpu(&self) -> f64 {
        self.cpu
    }
}

pub struct SThreadCpu {
    thread: ROsThread,
    state: ThreadStat,
}

impl SThreadCpu {
    pub fn snapshot(&mut self) -> Result<SThreadCpuSnapshot> {
        let cpu = self.state.cpu()? * 100_f64;
        Ok(SThreadCpuSnapshot {
            thread: self.thread.clone(),
            cpu,
        })
    }
}

impl std::fmt::Debug for SThreadCpuSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SThreadCpuSnapshot").field("thread", &self.thread).field("cpu", &self.cpu).finish()
    }
}

impl  std::fmt::Display for SThreadCpuSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;
        write!(f, "tid: {}", self.thread.tid)?;
        write!(f, ", name: {:?}", self.thread.name)?;
        write!(f, ", cpu: {:04.1}%", self.cpu)?;
        f.write_char('}')
    }
}

#[test]
fn test() {
    println!("{:04.1}", 1.4567);
}

pub struct RProcess {
    inner: remoteprocess::Process
}


pub fn get_process(pid: u32) -> Result<RProcess> {
    let inner = remoteprocess::Process::new(pid as Pid)?;
    Ok(RProcess{inner})
}

pub fn get_process_thread_stats(process: &RProcess) -> Result<Vec<SThreadCpu>> {
    let process = &process.inner;
    let pid = process.pid as u32;
    // let process = remoteprocess::Process::new(pid as Pid)
    // .with_context(|| format!("unable to get info of pid [{}]", pid))?;
    
    let raw_threads = process.threads()
    .with_context(|| format!("unable to get threads of pid [{}]", pid))?;

    // println!("process children [{}] -> {:#?}", pid, process.child_processes()?);
    let mut threads = Vec::with_capacity(raw_threads.len());
    for thread in raw_threads.iter() {
        let tid = thread.id()?;
        let thread = get_thread(pid, tid);
        let state = ThreadStat::build(pid, thread.tid as u32)
        .with_context(||format!("unable to get thread state [{}-{}]", pid, thread.tid))?;
        // println!("thread [{:?}] -> [{}]", thread, state.cpu()? * 100_f64);
        threads.push(SThreadCpu{thread, state});
    }
    Ok(threads)
}

pub fn current_thread() -> Result<ROsThread> {
    Ok(get_thread(std::process::id(), cur_thread_id()? as Tid))
}

fn get_thread(pid: u32, tid: Tid) -> ROsThread{
    ROsThread {
        tid,
        name: get_thread_name(pid, tid),
    }
}

fn get_thread_name(pid: u32, tid: Tid) -> Option<String> {
    let path = format!("/proc/{}/task/{}/comm", pid, tid);
    let mut name = vec![];
    match std::fs::File::open(path).and_then(|mut f| f.read_to_end(&mut name)) {
        Ok(_) => Some(String::from_utf8_lossy(&name).trim().to_string()),
        Err(_e) => {
            // debug!("error getting name for thread {}: {}", self.id, e);
            None
        }
    }
}
