use std::{time::Duration, collections::HashMap};
use anyhow::{Result, Context};
use remoteprocess::Tid;
use tracing::error;
use crate::monitor::rcpu;
use super::{rcpu::SThreadCpuSnapshot, cond_pair::{CondFlag, CondFlagGuard}};

#[macro_export]
macro_rules! dbgm {
    ($($arg:tt)* ) => (
        // tracing::debug!($($arg)*)
    );
}


pub trait CpuHandler {
    fn handle(&mut self, states: &Vec<SThreadCpuSnapshot>) -> Result<(Duration, Duration)>;
}

#[derive(Debug)]
struct BurningThread {
    _tid: Tid,
    num_burning: u8,
}

impl BurningThread {
    pub fn inc_detect(&mut self) -> u8 {
        self.num_burning += 1;
        self.num_burning = self.num_burning.clamp(0, MAX_DETECTS);
        self.num_burning
    }
}

#[derive(Debug)]
struct StateDetecting {
    num: u8,
}

#[derive(Debug)]
struct StateConfirming {
    num: u8,
}

#[derive(Debug)]
struct StateBurning {
    last_units: u32,
}

#[derive(Debug)]
enum DetectorState {
    Idle,
    Detecting(StateDetecting),
    Confirming(StateConfirming),
    Burning(StateBurning),
}

const MAX_DETECTS: u8 = 3;
const MAX_CONFIRMS: u8 = 3;
// const INIT_BURNING_UNITS: u32 = 2;

struct DetectorArgs {
    threshold: f64,
    unit_interval: Duration,
    duration: Duration,
}

pub struct BuringThreadsMonitor<F> {
    func: F,
    args: DetectorArgs,
    state: Option<DetectorState>,
    burnings: HashMap<Tid, BurningThread>,
    curr_max: u8,
    callback: bool,
}

impl<F> BuringThreadsMonitor<F> {
    pub fn new(threshold: f64, unit_interval: Duration, duration: Duration, func: F) -> Self {
        Self { 
            burnings: HashMap::new(),
            curr_max: 0,
            state: None,
            args: DetectorArgs{
                threshold, 
                unit_interval, 
                duration, 
            },
            func, 
            callback: false
        }
    }

    fn handle_ready(&mut self) -> Result<Duration> {
        if self.burnings.len() > 0 {
            return self.handle_detecting(StateDetecting{num: 0})
        }

        self.state = Some(DetectorState::Idle);
        Ok(Duration::ZERO)
    }

    fn handle_idle(&mut self) -> Result<Duration> {
        if self.burnings.len() > 0 {
            return self.handle_detecting(StateDetecting{num: 0})
        }

        self.state = Some(DetectorState::Detecting(StateDetecting{num: 1}));
        Ok(Duration::ZERO)
    }

    fn handle_detecting(&mut self, mut state: StateDetecting ) -> Result<Duration> {
        if self.burnings.len() == 0 {
            return self.change_to_idle();
        }

        state.num += 1;
        if state.num < MAX_DETECTS {
            // continue detecting
            self.state = Some(DetectorState::Detecting(state));
            return Ok(Duration::ZERO)
        } 

        if self.curr_max < MAX_DETECTS {
            return self.change_to_idle();
        }

        // change to confirming
        self.handle_confirming(StateConfirming{ num: 0})
    }

    fn handle_confirming(&mut self, mut state: StateConfirming ) -> Result<Duration> {
        if self.burnings.len() == 0 {
            return self.change_to_idle();
        }

        if self.curr_max < MAX_DETECTS {
            // burning thread gone, and some new threads come
            return self.change_to_idle();
        }

        state.num += 1;
        if state.num < MAX_CONFIRMS {
            // continue confirming
            self.callback = true;
            self.state = Some(DetectorState::Confirming(state));
            return Ok(Duration::ZERO)
        } 

        // change to burning
        self.handle_burning(StateBurning{ last_units: 1})
    }

    fn handle_burning(&mut self, state: StateBurning ) -> Result<Duration> {
        if self.burnings.len() == 0 {
            return self.change_to_idle();
        }

        if self.curr_max < MAX_DETECTS {
            // burning thread gone, and some new threads come
            return self.change_to_idle();
        }

        self.callback = true;

        let last_units = state.last_units * 2;
        let last_units = last_units.clamp(0, u32::MAX/4);
        self.state = Some(DetectorState::Burning(StateBurning{ last_units}));
        return Ok(self.args.unit_interval * last_units) 
    }


    fn change_to_idle(&mut self) -> Result<Duration> {
        self.state = None;
        return Ok(self.args.unit_interval)
    }

    fn check_thread_states(&mut self, threads: &Vec<SThreadCpuSnapshot>) {
        
        let mut max = 0;
        self.burnings.retain(|key, value| {
            for th in threads {
                if th.tid() == *key {
                    if max < value.num_burning {
                        max = value.num_burning;
                    }
                    return true;
                }
            }
            return false;
        });
        
        for tstate in threads {
            let usage_t = tstate.cpu();
            if usage_t >= self.args.threshold {
                let entry = self.burnings.entry(tstate.tid())
                .or_insert_with(||BurningThread{ _tid: tstate.tid(), num_burning: 0 });
                let num_burning = entry.inc_detect();
                if max < num_burning {
                    max = num_burning;
                }
            } else {
                self.burnings.remove(&tstate.tid());
            }
        }
        self.curr_max = max;
    }
}


impl<F> CpuHandler for Box<BuringThreadsMonitor<F>> 
where
    // F: for<'r> FnMut(&'r Vec<RThreadCpu>) -> Result<()> + Send + 'static,
    F: FnMut(&Vec<SThreadCpuSnapshot>) -> Result<()> + Send + 'static,
    // F: for<'r> FnMut<(&'r Vec<RThreadCpu>, )>, 
{
    fn handle(&mut self, threads: &Vec<SThreadCpuSnapshot>) -> Result<(Duration, Duration)> {
        self.check_thread_states(threads);
        dbgm!("handle: state {:?}, burnings {}, max {}", self.state, self.burnings.len(), self.curr_max);
        
        self.callback = false;

        let state = self.state.take();
        let interval = if let Some(state) = state {
            match state {
                DetectorState::Idle => self.handle_idle()?,
                DetectorState::Detecting(state) => self.handle_detecting(state)?,
                DetectorState::Confirming(state) => self.handle_confirming(state)?,
                DetectorState::Burning(state) => self.handle_burning(state)?,
            }
        } else {
            self.handle_ready()?
        };

        if self.callback {
            (self.func)(threads)?;
        }
        
        Ok((interval, self.args.duration))


    }
}



pub fn monitor_process_cpu(pid: u32, handler: (impl CpuHandler + Send + 'static)) -> Result<CondFlagGuard> 
// where
//     F: FnMut(&Vec<RThreadCpu>) -> Result<(Duration, Duration)> + Send + 'static,
{

    let cond = CondFlag::default();
    let guard = cond.guard();

    std::thread::spawn(move|| { 
        let r = do_monitor_process_cpu(pid, handler, cond);
        if let Err(e) = r {
            error!("do_monitor_process_cpu error {:?}", e);
        }
    });

    Ok(guard)
}

fn do_monitor_process_cpu(
    pid: u32, 
    mut handler: (impl CpuHandler + Send + 'static),
    cond: CondFlag,
) -> Result<()> {
    let process = rcpu::get_process(pid)
    .with_context(|| format!("unable to get info of pid [{}]", pid))?;

    let (mut interval, mut duration) = handler.handle(&Vec::new())?;
    loop {
        dbgm!("waiting for interval {:?}", interval);
        if cond.wait_for(interval) {
            dbgm!("detected end");
            break;
        }

        let thread_stats = rcpu::get_process_thread_stats(&process)?;
        dbgm!("get cpu with duration {:?}, threads {}", duration, thread_stats.len());
        if cond.wait_for(duration) {
            dbgm!("detected end");
            break;
        }

        let mut snapshots = Vec::with_capacity(thread_stats.len());
        for mut stat in thread_stats {
            snapshots.push(stat.snapshot()?);
        }
        

        (interval, duration) = handler.handle(&snapshots)?;

        // let mut exceed = false;
        // for tstate in &mut thread_stats {
        //     let usage_t = tstate.cpu().with_context(||format!("unable to get cpu of thread [{:?}]", tstate))? ;
        //     if usage_t > threshold {
        //         exceed = true;
        //         break;
        //     }
        // }
        // if exceed {
        //     (interval, duration) = func(&thread_stats)?;
        // }
    }
    Result::<()>::Ok(())
}

pub fn spawn_burning_monitor<F>(pid: u32, threshold: f64, unit_interval: Duration, duration: Duration, func: F) -> Result<CondFlagGuard> 
where
    F: FnMut(&Vec<SThreadCpuSnapshot>) -> Result<()> + Send + 'static,
{
    let handler = Box::new(BuringThreadsMonitor::new(
        threshold, 
        unit_interval, 
        duration, 
        func,
    ));

    monitor_process_cpu(pid,  handler)
}