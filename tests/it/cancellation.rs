use crate::awaiting_actor::AwaitingActor;
use crate::common::{read_totals, yield_task};
use crate::test_actor::{Action, TestActor};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn actors_can_be_gracefully_shutdown() {
    let (tx, mut rx) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));
    assert!(!actor.is_shutdown());

    assert!(actor.send(Action::Num(1)).await.is_ok());
    assert!(actor.send(Action::Num(2)).await.is_ok());
    actor.graceful_shutdown();
    assert!(actor.is_shutdown());

    assert_eq!(3, read_totals(&mut rx).await);
}

#[tokio::test]
async fn actors_can_be_hard_shutdown() {
    let (tx, mut rx) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));
    assert!(!actor.is_shutdown());

    assert!(actor.send(Action::Num(1)).await.is_ok());
    assert!(actor.send(Action::Num(2)).await.is_ok());
    actor.hard_shutdown();
    assert!(actor.is_shutdown());

    assert_eq!(0, read_totals(&mut rx).await);
}

#[tokio::test]
async fn actors_can_gracefully_shut_themselves_down() {
    let (tx, mut rx) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));
    assert!(!actor.is_shutdown());

    assert!(actor.send(Action::Num(1)).await.is_ok());
    assert!(actor.send(Action::GracefulShutdown).await.is_ok());
    yield_task().await;
    assert!(actor.is_shutdown());

    assert_eq!(1, read_totals(&mut rx).await);
}

#[tokio::test]
async fn actors_can_hard_shut_themselves_down() {
    let (tx, mut rx) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));
    assert!(!actor.is_shutdown());

    assert!(actor.send(Action::Num(1)).await.is_ok());
    assert!(actor.send(Action::HardShutdown).await.is_ok());
    yield_task().await;
    assert!(actor.is_shutdown());
    assert_eq!(1, read_totals(&mut rx).await);
}

#[tokio::test]
async fn dropping_the_handle_drops_the_actor() {
    let refs = Arc::new(());

    let actor = toktor::spawn(10, AwaitingActor::new(refs.clone()));

    assert_eq!(2, Arc::strong_count(&refs));

    drop(actor);
    yield_task().await;

    assert_eq!(1, Arc::strong_count(&refs));
}
