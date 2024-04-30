use std::sync::Arc;
use std::time::Duration;
use toktor::{Actor, ActorContext};

pub struct AwaitingActor {
    _refs: Arc<()>,
}

impl AwaitingActor {
    pub fn new(refs: Arc<()>) -> Self {
        Self { _refs: refs }
    }
}

impl Actor<usize> for AwaitingActor {
    async fn on_message(&mut self, _ctx: &ActorContext, _message: usize) {
        tokio::time::sleep(Duration::from_secs(u64::MAX)).await
    }
}
