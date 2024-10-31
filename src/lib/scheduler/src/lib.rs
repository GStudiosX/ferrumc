#![feature(trait_alias)]

use anyhow::Result;
use ferrumc_net::ServerState;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::Mutex;
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering}};
use tokio::sync::Notify;
use tokio::time::{sleep_until, Duration, Instant};
use tracing::{error, info};
use dashmap::DashMap;

#[cfg(test)]
mod tests;
mod errors;

pub use errors::SchedulerError;

pub trait AsyncCallback = 'static
    + Send
    + Sync
    + Fn(Arc<ServerState>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
    + Send
    + Sync;
pub type Runnable = Arc<dyn AsyncCallback>;

#[derive(Debug)]
pub struct CancelError;

impl Error for CancelError {}

impl fmt::Display for CancelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Clone)]
pub struct ScheduledTask {
    period: Instant,
    interval: Option<Duration>,
    runnable: Runnable,
    handle: TaskHandle,
}

impl ScheduledTask {
    pub fn new(
        runnable: Runnable,
        delay: Duration,
        interval: Option<Duration>,
        handle: TaskHandle,
    ) -> Self {
        Self {
            period: Instant::now() + delay,
            interval,
            runnable,
            handle,
        }
    }

    async fn run(&self, state: Arc<ServerState>) -> anyhow::Result<()> {
        if self.handle.is_cancelled() {
            Err(CancelError.into())
        } else {
            (self.runnable)(state).await?;
            self.handle.notify();
            Ok(())
        }
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.period == other.period
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.period.cmp(&self.period))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.period.cmp(&other.period).reverse()
    }
}

#[derive(Clone)]
pub struct TickTask {
    //delay: usize,
    runnable: Runnable,
    handle: TaskHandle,
}

impl TickTask {
    async fn run(&self, state: Arc<ServerState>) -> anyhow::Result<()> {
        if self.handle.is_cancelled() {
            Err(CancelError.into())
        } else {
            (self.runnable)(state).await?;
            self.handle.notify();
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskHandle(Arc<AtomicBool>, Arc<Notify>);

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) && Arc::ptr_eq(&self.1, &other.1)
    }
}

impl Eq for TaskHandle {}

impl TaskHandle {
    /// INTERNAL ONLY
    ///
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)), Arc::new(Notify::new()))
    }

    pub(crate) fn notify(&self) {
        self.1.notify_one();
    }

    /// Wait for this task to be ran.
    ///
    pub async fn wait(&self) {
        self.1.notified().await;
    }

    /// Cancel this task.
    ///
    pub fn cancel(&self) -> bool {
        self.0.store(true, AtomicOrdering::Relaxed);
        true
    }

    /// Check if the task is currently cancelled.
    ///
    pub fn is_cancelled(&self) -> bool {
        self.0.load(AtomicOrdering::Relaxed)
    }
}

impl Default for TaskHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Scheduler {
    task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    pending_ticks: Arc<DashMap<usize, Vec<TickTask>>>,
    wake: Arc<Notify>,
    shutdown: Arc<AtomicBool>,
    active_tasks: Arc<AtomicUsize>,
    current_tick: Arc<Mutex<usize>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            pending_ticks: Arc::new(DashMap::new()),
            wake: Arc::new(Notify::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
            current_tick: Arc::new(Mutex::new(0)),
        }
    }

    /// INTERNAL
    ///
    pub async fn tick(&self, state: Arc<ServerState>) {
        let current_tick = {
            let mut tick = self.current_tick.lock().await;
            let (new, _) = tick.overflowing_add(1);
            *tick = new;
            new
        };

        if let Some(tasks) = self.pending_ticks.remove(&current_tick) {
            for task in tasks.1 {
                tokio::spawn({
                    let state = Arc::clone(&state);
                    async move {
                        if let Err(e) = task.run(state).await {
                            if e.is::<CancelError>() {
                                return;
                            }
                            error!("Error in scheduled task: {}", e);
                        }
                    }
                });
            }
        }
    }

    /// Schedules a task for delay next ticks
    ///
    /// Delay is in ticks it can not be negative if it's 0 it will be scheduled for next tick.
    pub async fn schedule_tick<F, Fut>(
        &self,
        cb: F,
        delay: usize
    ) -> Result<TaskHandle, SchedulerError>
    where
        F: FnOnce(Arc<ServerState>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<()>> + Send + Sync,
    {
        if self.shutdown.load(AtomicOrdering::Relaxed) {
            return Err(SchedulerError::Shutdown);
        }

        let handle = TaskHandle::new();

        let delay = delay.overflowing_add(1).0;
        let period = {
            let tick = self.current_tick.lock().await;
            let (period, _) = tick.overflowing_add(delay);
            period
        
        };

        let task = TickTask {
            //delay,
            runnable: Arc::new(move |state| {
                let cb = cb.clone();
                Box::pin(async move { cb(state).await })
            }),
            handle: handle.clone(),
        };

        self.pending_ticks.entry(period)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(task);

        Ok(handle)
    }

    /// Schedules a async task
    ///
    pub async fn schedule_task<F, Fut>(
        &self,
        cb: F,
        delay: Duration,
        interval: Option<Duration>,
    ) -> Result<TaskHandle, SchedulerError>
    where
        F: Fn(Arc<ServerState>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<()>> + Send + Sync,
    {
        if self.shutdown.load(AtomicOrdering::Relaxed) {
            return Err(SchedulerError::Shutdown);
        }

        let handle = TaskHandle::new();
        // Add scheduled task into the queue
        let scheduled_task = ScheduledTask::new(
            Arc::new(move |state| {
                let cb = cb.clone();
                Box::pin(async move { cb(state).await })
            }),
            delay,
            interval,
            handle.clone(),
        );
        {
            let mut queue = self.task_queue.lock().await;
            queue.push(scheduled_task);
        }
        // Wake up the scheduler
        self.wake.notify_one();
        //trace!("Task scheduled");
        Ok(handle)
    }

    // this should be safe as I drop the queue mutex guard
    // before an .await call. https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_lock
    //
    // note: this may not be needed anymore since I use a await aware Mutex.
    // tokio::sync::Mutex
    #[allow(clippy::await_holding_lock)] 
    /// Run the scheduler.
    ///
    pub async fn run(self: Arc<Self>, state: Arc<ServerState>) {
        while !self.shutdown.load(AtomicOrdering::Relaxed) {
            let mut queue = self.task_queue.lock().await;
            if let Some(task) = queue.peek() {
                if task.period <= Instant::now() {
                    let task = queue.pop().unwrap();
                    let state = Arc::clone(&state);
                    drop(queue);
                    tokio::spawn({
                        let scheduler = Arc::clone(&self);
                        scheduler.active_tasks.fetch_add(1, AtomicOrdering::Relaxed);
                        async move {
                            if let Err(e) = task.run(state).await {
                                if e.is::<CancelError>() {
                                    scheduler.active_tasks.fetch_sub(1, AtomicOrdering::Relaxed);
                                    //trace!("CancelError recieved");
                                    return;
                                }
                                error!("Error in scheduled task: {}", e);
                            }

                            // probably should really do something else than rescheduling the task
                            if let Some(delay) = task.interval {
                                //trace!("Rescheduling interval task");
                                let scheduled_task = ScheduledTask::new(
                                    task.runnable,
                                    delay,
                                    task.interval,
                                    task.handle,
                                );
                                {
                                    let mut queue = scheduler.task_queue.lock().await;
                                    queue.push(scheduled_task);
                                }
                                // Wake up the scheduler
                                scheduler.wake.notify_one();
                            }

                            scheduler.active_tasks.fetch_sub(1, AtomicOrdering::Relaxed);
                        }
                    });
                } else {
                    let next_period = task.period;
                    drop(queue);
                    tokio::select! {
                        _ = sleep_until(next_period) => {},
                        _ = self.wake.notified() => {},
                    };
                }
            } else {
                drop(queue);
                self.wake.notified().await;
            }
        }

        if !self.wait_for_complete().await {
            info!("Shutdown timeout reached; forcing shutdown.");
        }

        info!("Scheduler was shutdown");
    }

    /// Shutdown the scheduler this will wait 10 seconds for all active tasks to finish.
    ///
    /// Does not wait for ticked tasks as that requires the tick system which may be already shutdown.
    pub fn shutdown(self: Arc<Self>) {
        self.shutdown.store(true, AtomicOrdering::Relaxed);
        self.wake.notify_one();
    }
 
    /// wait for active tasks to complete
    ///
    async fn wait_for_complete(self: Arc<Self>) -> bool {
        let mut elapsed = 0;

        while self.active_tasks.load(AtomicOrdering::Relaxed) > 0 && elapsed < 10 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            elapsed += 1;
        }

        self.active_tasks.load(AtomicOrdering::Relaxed) == 0
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
