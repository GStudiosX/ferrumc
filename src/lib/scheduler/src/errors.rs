use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("This scheduler is shutdown.")]
    Shutdown,
}
