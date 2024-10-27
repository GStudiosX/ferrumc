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
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep_until, Duration, Instant};
use tracing::error;

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
    cancel: TaskCancel,
}

impl ScheduledTask {
    pub fn new(
        runnable: Runnable,
        delay: Duration,
        interval: Option<Duration>,
        cancel: TaskCancel,
    ) -> Self {
        Self {
            period: Instant::now() + delay,
            interval,
            runnable,
            cancel,
        }
    }

    pub async fn run(&self, state: Arc<ServerState>) -> anyhow::Result<()> {
        if self.cancel.is_cancelled().await {
            Err(CancelError.into())
        } else {
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
pub struct TaskCancel(Arc<Mutex<bool>>);

impl TaskCancel {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(false)))
    }

    pub async fn cancel(&self) -> bool {
        *self.0.lock().await = true;
        true
    }

    pub async fn is_cancelled(&self) -> bool {
        *self.0.lock().await
    }
}

impl Default for TaskCancel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Scheduler {
    task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    wake: Arc<Notify>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            wake: Arc::new(Notify::new()),
        }
    }

    /// Schedules a async task
    pub async fn schedule_task<F, Fut>(
        &self,
        cb: F,
        delay: Duration,
        interval: Option<Duration>,
    ) -> TaskCancel
    where
        F: Fn(Arc<ServerState>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<()>> + Send + Sync,
    {
        let cancel = TaskCancel::new();
        // Add scheduled task into the queue
        let scheduled_task = ScheduledTask::new(
            Arc::new(move |state| {
                let cb = cb.clone();
                Box::pin(async move { cb(state).await })
            }),
            delay,
            interval,
            cancel.clone(),
        );
        {
            let mut queue = self.task_queue.lock().await;
            queue.push(scheduled_task);
        }
        // Wake up the scheduler
        self.wake.notify_one();
        //trace!("Task scheduled");
        cancel
    }

    // this should be safe as I drop the queue mutex guard
    // before an .await call. https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_lock
    //
    // note: this may not be needed anymore since I use a await aware Mutex.
    // tokio::sync::Mutex
    #[allow(clippy::await_holding_lock)] 
    pub async fn run(self: Arc<Self>, state: Arc<ServerState>) {
        loop {
            let mut queue = self.task_queue.lock().await;
            if let Some(task) = queue.peek() {
                if task.period <= Instant::now() {
                    let task = queue.pop().unwrap();
                    let state = Arc::clone(&state);
                    drop(queue);
                    tokio::spawn({
                        let scheduler = Arc::clone(&self);
                        async move {
                            if let Err(e) = task.run(state).await {
                                if e.is::<CancelError>() {
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
                                    task.cancel,
                                );
                                {
                                    let mut queue = scheduler.task_queue.lock().await;
                                    queue.push(scheduled_task);
                                }
                                // Wake up the scheduler
                                scheduler.wake.notify_one();
                            }
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
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
