use crate::*;
use futures::future::join_all;
use std::time::{Duration, Instant};
use std::sync::{OnceLock, Arc};
use ferrumc_net::ServerState;
use tokio::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};

fn get_scheduler() -> Arc<Scheduler> {
    static SCHEDULER: OnceLock<Arc<Scheduler>> = OnceLock::new();
    SCHEDULER.get_or_init(|| {
        let scheduler = Arc::new(Scheduler::new());
        tokio::spawn({
            let scheduler = scheduler.clone();
            async move {
                // have to create a temporary tcp listener & universe
                scheduler.run(Arc::new(ServerState {
                    universe: ferrumc_ecs::Universe::default(),
                    tcp_listener: TcpListener::bind("127.0.0.1:0").await.unwrap()
                })).await
            }
        });
        scheduler
    }).clone()
}

async fn schedule_task_for(duration: Duration, count: Arc<AtomicUsize>) -> anyhow::Result<i64> {
    let handle = get_scheduler().schedule_task(move |_| {
        let count = Arc::clone(&count);
        async move {
            count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }, duration, None).await?;

    let now = Instant::now();
    handle.wait().await;

    Ok(Instant::now()
        .duration_since(now)
        .saturating_sub(duration)
        .as_millis() as i64)
}

#[tokio::test]
#[ignore]
async fn test_accuracy() {
    let duration = Duration::from_secs(5);
    let tasks = vec![100, 1_000, 10_000];
    let mut averages = vec![0f64; tasks.len()];

    for (num_tasks, average) in std::iter::zip(tasks, averages.iter_mut()) {
        let count = Arc::new(AtomicUsize::new(0));
        let handles = (0..num_tasks)
            .map({
                let count = Arc::clone(&count);
                move |_| {
                    let count = Arc::clone(&count);
                    tokio::spawn({
                        async move {
                            schedule_task_for(duration, Arc::clone(&count)).await
                        }
                    })
                }
            })
            .collect::<Vec<_>>();

        let results = join_all(handles)
            .await
            .into_iter()
            .filter_map(|res| res.unwrap().ok())
            .collect::<Vec<_>>();

        let avg_accuracy = results.iter().sum::<i64>() as f64 / results.len() as f64;
        println!("Average accuracy for {num_tasks} tasks of duration {:#?}: {:.2} ms", duration, avg_accuracy);
        println!("{} tasks completed", count.load(Ordering::Relaxed));
        println!("{}\n", "-".repeat(20));
        *average = avg_accuracy;
    }

    let avg_accuracy = averages.iter().sum::<f64>() as f64 / averages.len() as f64;
    println!("Overall average accuracy of tasks of duration {:#?}: {:.2} ms", duration, avg_accuracy);
}

