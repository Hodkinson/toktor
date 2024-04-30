//! # toktor
//!
//! A small Actor framework for use within tokio.
//!
//! Features:
//! * both graceful and hard shutdown options for actors
//! * spawn child actors
//! * spawn associated tasks, that may be shutdown with the actor
//!
//! This was created to cater for a need in a project for allowing creation of complex actor
//! structures that could be easily shutdown when required. For example in UIs, when a user
//! closes a window, in telephony when a call is dropped.
//!
//! # Example
//!
//! ```rust
//! use toktor::{Actor, ActorContext};
//! pub struct CountingActor {
//!     total: usize,
//! }
//!
//! impl Actor<usize> for CountingActor {
//!     async fn on_message(&mut self, _ctx: &ActorContext, number: usize) {
//!         self.total += number
//!     }
//! }
//!
//! # tokio_test::block_on(async {
//! let actor = toktor::spawn(16, CountingActor { total: 0 });
//! assert!(actor.send(1).await.is_ok());
//!
//! # });
//!
//! ```
//!
mod actor;

pub use crate::actor::spawn;
pub use crate::actor::Actor;
pub use crate::actor::ActorContext;
pub use crate::actor::ActorHandle;
