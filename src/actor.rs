const GRACEFUL: isize = 1;
const HARD: isize = 2;

use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::{AbortHandle, JoinSet};

/// An error returned from the [`ActorContext::spawn`] or [`ActorContext::spawn_child`]
/// functions.
///
/// A **spawn** or **spawn_child** operation can only fail if [`ActorHandle::graceful_shutdown`] or
/// has [`ActorHandle::hard_shutdown`] has been called for this actor (or for an actor in
/// its parent chain).
///
#[derive(Error, Debug)]
pub enum ActorError {
    #[error("actor is shut down")]
    ShutDown,
}

/// Handles a message.
///
/// The `on_message` method gives &mut access to self.
///
/// In addition, a reference to this actor's [`ActorContext`] is provided.
/// This allows the actor to shut itself down, or spawn child actors or associated tasks.
///
/// [`ActorContext`]: ActorContext
pub trait Actor<T> {
    fn on_message(&mut self, ctx: &ActorContext, message: T) -> impl Future<Output = ()> + Send;
}

/// Spawns an Actor, returning an [`ActorHandle`]
/// that can be used to send messages, or shutdown the actor.
///
/// The actor will start running in the background immediately
/// when this method is called.
///
/// # Panics
///
/// This method panics if called outside of a Tokio runtime.
///
/// [`ActorHandle`]: ActorHandle
pub fn spawn<A, T>(queue_size: usize, mut actor: A) -> ActorHandle<T>
where
    A: Actor<T>,
    A: Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    assert!(queue_size > 0, "spawning actor requires queue_size > 0");

    let context = ActorContext::new();
    let (queue, mut rx) = mpsc::channel(queue_size);

    {
        let mut join_set = context.inner.join_set.lock().unwrap();
        let js = join_set.as_mut().unwrap();

        let c = ActorContext::from(&context);
        js.spawn(async move {
            while let Some(message) = rx.recv().await {
                actor.on_message(&c, message).await;
                match c.inner.cancelled.load(Ordering::Acquire) {
                    0 => {}
                    GRACEFUL => rx.close(),
                    _ => {
                        rx.close();
                        break;
                    }
                }
            }
        });
    }

    ActorHandle {
        inner: Arc::new(InnerActorHandle { queue, context }),
    }
}

/// A handle to an Actor, that can be used to send messages, or shutdown the actor.
///
/// This handle is cheap to clone.
///
/// Messages may be sent by calling [`ActorHandle::send`].
///
/// The actor may be shutdown by calling the [`ActorHandle::graceful_shutdown`] or
/// [`ActorHandle::hard_shutdown`] methods.
///
/// Dropping the last `ActorHandle` for an actor will cause it to finish and be dropped.
///
/// [`ActorHandle`]: ActorHandle
#[derive(Clone)]
pub struct ActorHandle<T> {
    inner: Arc<InnerActorHandle<T>>,
}

struct InnerActorHandle<T> {
    queue: mpsc::Sender<T>,
    context: ActorContext,
}

impl<T> Debug for ActorHandle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorHandle")
            .field("capacity", &self.inner.queue.capacity())
            .field("cancelled", &self.inner.context.is_shutdown())
            .finish()
    }
}

/// A context actor, that can be used by the actor to spawn associated tasks, child actors, or
/// shutdown the actor.
///
/// The actor may be shutdown by calling the [`ActorContext::graceful_shutdown`] or
/// [`ActorContext::hard_shutdown`] methods.
///
/// To spawn an associated task, call [`ActorContext::spawn`].
///
/// To spawn a child of this actor, call [`ActorContext::spawn_child`].
///
pub struct ActorContext {
    inner: Arc<InnerContext>,
}

struct InnerContext {
    join_set: Mutex<Option<JoinSet<()>>>,
    cancelled: AtomicIsize,
    children: Mutex<Option<Vec<ActorContext>>>,
    shutting_down_children: Mutex<Option<Vec<ActorContext>>>,
}

impl ActorContext {
    fn new() -> Self {
        ActorContext {
            inner: Arc::new(InnerContext {
                join_set: Mutex::new(Some(JoinSet::new())),
                cancelled: AtomicIsize::new(0),
                children: Mutex::new(Some(vec![])),
                shutting_down_children: Mutex::new(Some(vec![])),
            }),
        }
    }

    fn from(other: &ActorContext) -> Self {
        ActorContext {
            inner: other.inner.clone(),
        }
    }

    /// Spawns an associated task for this actor.
    ///
    /// An associated task is a standard tokio task that is tied to the lifetime of the actor.
    /// The task will run as long as the actor is not **hard** shutdown, and the last actor handle
    /// has not been dropped.
    ///
    /// Note: calling [`ActorHandle::graceful_shutdown`] will not abort this associated task.
    ///
    /// This function will return [`Err(ActorError::ShutDown)`] if this actor has been shutdown
    /// (either graceful or hard) already.
    ///
    pub fn spawn<F>(&self, task: F) -> Result<AbortHandle, ActorError>
    where
        F: Future<Output = ()>,
        F: Send + 'static,
    {
        let mut js = self.inner.join_set.lock().unwrap();
        if let Some(js) = js.as_mut() {
            Ok(js.spawn(task))
        } else {
            Err(ActorError::ShutDown)
        }
    }

    /// Spawns a child actor.
    ///
    /// A child actor is a self-contained actor, but has the actor for this [`ActorContext`]
    /// as its parent. If the parent actor is shutdown, the child actor is also shutdown in the
    /// same manner.
    ///
    /// This function will return [`Err(ActorError::ShutDown)`] if this actor has been shutdown
    /// (either graceful or hard) already.
    ///
    pub fn spawn_child<A, T>(
        &self,
        queue_size: usize,
        actor: A,
    ) -> Result<ActorHandle<T>, ActorError>
    where
        A: Actor<T>,
        A: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let mut children = self.inner.children.lock().unwrap();
        if let Some(kids) = children.as_mut() {
            let handle = spawn(queue_size, actor);
            kids.push(ActorContext::from(&handle.inner.context));
            Ok(handle)
        } else {
            Err(ActorError::ShutDown)
        }
    }

    /// Indicates if this actor has been shutdown.
    ///
    pub fn is_shutdown(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire) > 0
    }

    /// Gracefully shuts down this actor and any children.
    ///
    /// Any future [`ActorHandle::send`] calls will return an error.
    /// Any messages already in the actor's queue will be processed, and the actor will then finish.
    ///
    /// Graceful shutdown is propagated to all child actors.
    ///
    /// Note that any associated tasks for this actor are not aborted when graceful shutdown is used.
    /// See [`ActorContext::hard_shutdown`].
    ///
    pub fn graceful_shutdown(&self) {
        let inner = &self.inner;
        if inner
            .cancelled
            .compare_exchange(0, GRACEFUL, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            let shutting_down_children = {
                let mut children = inner.children.lock().unwrap();
                let mut shutting_down_children = inner.shutting_down_children.lock().unwrap();
                *shutting_down_children = children.take();
                shutting_down_children
            };

            if let Some(children) = shutting_down_children.as_ref() {
                for child in children {
                    child.graceful_shutdown();
                }
            }
        }
    }

    /// Forcibly shuts down this actor and any children.
    ///
    /// Any future [`ActorHandle::send`] calls will return an error.
    /// Any messages already in the actor's queue will **not** be processed.
    ///
    /// Hard shutdown is propagated to all child actors.
    ///
    /// All associated tasks for this actor are aborted.
    ///
    /// It is fine to call [`ActorContext::graceful_shutdown`] and later
    /// [`ActorContext::hard_shutdown`]. This may be useful to allow processing of messages to
    /// continue for some time.
    ///
    pub fn hard_shutdown(&self) {
        let inner = &self.inner;
        if inner
            .cancelled
            .compare_exchange(0, HARD, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            inner.join_set.lock().unwrap().take();
            hard_shutdown_children(&inner.children);
        } else if inner
            .cancelled
            .compare_exchange(GRACEFUL, HARD, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            inner.join_set.lock().unwrap().take();
            hard_shutdown_children(&inner.children);
            hard_shutdown_children(&inner.shutting_down_children);
        }
    }
}

fn hard_shutdown_children(children: &Mutex<Option<Vec<ActorContext>>>) {
    let children = children.lock().unwrap().take();
    if let Some(children) = children {
        for child in children {
            child.hard_shutdown();
        }
    }
}

impl<T: Send + Sync + 'static> ActorHandle<T> {
    /// Gracefully shuts down this actor and any children.
    ///
    /// Any future [`ActorHandle::send`] calls will return an error.
    /// Any messages already in the actor's queue will be processed, and the actor will then finish.
    ///
    /// Graceful shutdown is propagated to all child actors.
    ///
    /// Note that any associated tasks for this actor are not aborted when graceful shutdown is used.
    /// See [`ActorHandle::hard_shutdown`].
    ///
    pub fn graceful_shutdown(&self) {
        self.inner.context.graceful_shutdown();
    }

    /// Forcibly shuts down this actor and any children.
    ///
    /// Any future [`ActorHandle::send`] calls will return an error.
    /// Any messages already in the actor's queue will **not** be processed.
    ///
    /// Hard shutdown is propagated to all child actors.
    ///
    /// All associated tasks for this actor are aborted.
    ///
    /// It is fine to call [`ActorHandle::graceful_shutdown`] and later
    /// [`ActorHandle::hard_shutdown`]. This may be useful to allow processing of messages to
    /// continue for some time.
    ///
    pub fn hard_shutdown(&self) {
        self.inner.context.hard_shutdown();
    }

    /// Indicates if this actor has been shutdown.
    ///
    pub fn is_shutdown(&self) -> bool {
        self.inner.context.is_shutdown()
    }

    /// Sends a message to the actor, waiting until there is capacity.
    ///
    /// This function will return [`Err(ActorError::ShutDown)`] if this actor has been shutdown
    /// (either graceful or hard) already.
    ///
    pub async fn send(&self, message: T) -> Result<(), ActorError> {
        self.inner
            .queue
            .send(message)
            .await
            .map_err(|_| ActorError::ShutDown)
    }
}
