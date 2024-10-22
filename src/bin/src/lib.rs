// Security
#![forbid(unsafe_code)]

use std::sync::Arc;
use ferrumc_scheduler::Scheduler;
use lazy_static::lazy_static;

pub mod events;

lazy_static! {
    static ref SCHEDULER: Arc<Scheduler> = Arc::new(Scheduler::new());
}

pub fn get_scheduler() -> Arc<Scheduler> {
    SCHEDULER.clone()
}

/*pub fn get_scheduler() -> &'static Scheduler {
    static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();
    SCHEDULER.get_or_init(|| Scheduler::new())
}*/
