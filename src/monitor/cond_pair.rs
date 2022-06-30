use std::{time::{Duration, Instant}, sync::Arc};
use parking_lot::{Mutex, Condvar};

pub struct CondPair<T> {
    ended: Mutex<T>,
    condvar: Condvar,
}

impl<T: Default> Default for CondPair<T> {
    fn default() -> Self {
        Self{
            ended: Default::default(),
            condvar: Default::default(),
        }
    }
}

impl<T> CondPair<T> {
    pub fn wait_for_with<F>(&self, timeout: Duration, func: F) -> bool 
    where
        F: Fn(&T) -> bool
    {
        let start = Instant::now();
        let mut ended = self.ended.lock();
        while !func(&ended) {
            if start.elapsed() >= timeout {
                return false;
            }

            let r = self.condvar.wait_for(&mut ended, timeout - start.elapsed());
            if r.timed_out() {
                return false;
            }
        }
        true
    } 
}

pub struct CondFlagGuard {
    cond: Arc<CondPair<bool>>,
}

impl Drop for CondFlagGuard {
    fn drop(&mut self) {
        // println!("guard out of scope");
        *self.cond.ended.lock() = true;
        self.cond.condvar.notify_one();
    }
}

#[derive(Default)]
pub struct CondFlag {
    cond: Arc<CondPair<bool>>,
}

// pub type CondFlag = Arc<CondPair<bool>>;

impl CondFlag {
    #[inline]

    pub fn guard(&self) -> CondFlagGuard {
        CondFlagGuard{ cond: self.cond.clone()}
    }

    #[inline]
    pub fn wait_for(&self, timeout: Duration) -> bool {
        self.cond.wait_for_with(timeout, |v| *v)
    }
}


