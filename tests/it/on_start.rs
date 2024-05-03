use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::Instant;
use toktor::{Actor, ActorContext};

pub struct StartingActor {
    started: Arc<AtomicBool>,
}

impl StartingActor {
    pub fn new(started: Arc<AtomicBool>) -> Self {
        Self { started }
    }
}

impl Actor<usize> for StartingActor {
    async fn on_start(&mut self, ctx: &ActorContext) {
        assert!(!ctx.is_shutdown());
        self.started.store(true, Ordering::Relaxed)
    }

    async fn on_message(&mut self, _ctx: &ActorContext, _message: usize) {
        tokio::time::sleep(Duration::from_secs(u64::MAX)).await
    }
}


#[tokio::test]
async fn actors_are_informed_of_starting() {
    let started = Arc::new(AtomicBool::new(false));

    let _actor = toktor::spawn(10, StartingActor::new(started.clone()));

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if started.load(Ordering::Relaxed) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    assert!(started.load(Ordering::Relaxed))
}
