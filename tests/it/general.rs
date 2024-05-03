use crate::test_actor::TestActor;
use tokio::sync::mpsc;

#[tokio::test]
async fn actors_impl_debug() {
    let (tx, _) = mpsc::channel(10);

    let actor = toktor::spawn(10, TestActor::new(tx));
    assert_eq!(
        format!("{:?}", actor),
        "ActorHandle { capacity: 10, cancelled: false }"
    );
}
