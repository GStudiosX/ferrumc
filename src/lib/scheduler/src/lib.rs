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

    pub async fn run(&self, state: Arc<ServerState>) -> anyhow::Result<()> {
        if self.handle.is_cancelled() {
            Err(CancelError.into())
        } else {
            self.handle.notify();
            (self.runnable)(state).await
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

#[derive(Debug, Clone)]
pub struct TaskHandle(Arc<AtomicBool>, Arc<Notify>);

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
    wake: Arc<Notify>,
    shutdown: Arc<AtomicBool>,
    active_tasks: Arc<AtomicUsize>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            wake: Arc::new(Notify::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
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

    /// Shutdown the scheduler this will wait 10 seconds for all active tasks to finish.
    ///
    pub fn shutdown(self: Arc<Self>) {
        self.shutdown.store(true, AtomicOrdering::Relaxed);
        self.wake.notify_one();
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
