use tokio::sync::mpsc::Receiver;
use tokio::task::yield_now;

pub async fn read_totals(rx: &mut Receiver<usize>) -> usize {
    let mut total = 0;
    while let Some(num) = rx.recv().await {
        total += num;
    }
    total
}

pub async fn yield_task() {
    yield_now().await;
}