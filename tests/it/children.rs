use crate::common::yield_task;
use crate::test_actor::{Action, TestActor};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn actors_can_spawn_child_actors_and_shut_them_down() {
    let (tx, _) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));

    let refs = Arc::new(());
    assert!(actor.send(Action::SpawnChild(refs.clone())).await.is_ok());
    assert!(actor.send(Action::SpawnChild(refs.clone())).await.is_ok());
    yield_task().await;

    assert_eq!(3, Arc::strong_count(&refs));

    drop(actor);

    yield_task().await;
    assert_eq!(1, Arc::strong_count(&refs));
}

#[tokio::test]
async fn actors_can_spawn_associated_tasks() {
    let (tx, _) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));

    let refs = Arc::new(());
    assert!(actor.send(Action::Spawn(refs.clone())).await.is_ok());
    assert!(actor.send(Action::Spawn(refs.clone())).await.is_ok());
    yield_task().await;

    assert_eq!(3, Arc::strong_count(&refs));

    actor.graceful_shutdown();
    yield_task().await;

    assert_eq!(3, Arc::strong_count(&refs));

    actor.hard_shutdown();
    yield_task().await;

    assert_eq!(1, Arc::strong_count(&refs));
}
