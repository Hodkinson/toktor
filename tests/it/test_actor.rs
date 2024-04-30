use crate::awaiting_actor::AwaitingActor;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use toktor::{Actor, ActorContext, ActorHandle};

pub enum Action {
    GracefulShutdown,
    HardShutdown,
    SpawnChild(Arc<()>),
    Spawn(Arc<()>),
    Num(usize),
}

pub struct TestActor {
    tx: mpsc::Sender<usize>,
    kids: Vec<ActorHandle<usize>>,
}

impl TestActor {
    pub fn new(tx: mpsc::Sender<usize>) -> Self {
        TestActor { tx, kids: vec![] }
    }
}

impl Actor<Action> for TestActor {
    async fn on_message(&mut self, ctx: &ActorContext, message: Action) {
        match message {
            Action::GracefulShutdown => ctx.graceful_shutdown(),
            Action::HardShutdown => ctx.hard_shutdown(),
            Action::SpawnChild(refs) => {
                let child = AwaitingActor::new(refs);
                self.kids.push(ctx.spawn_child(10, child).expect("child"));
            }
            Action::Spawn(refs) => {
                ctx.spawn(async move {
                    let _refs = refs;
                    tokio::time::sleep(Duration::from_secs(u64::MAX)).await
                })
                .expect("task");
            }
            Action::Num(num) => {
                let _ = self.tx.send(num).await;
            }
        }
    }
}
