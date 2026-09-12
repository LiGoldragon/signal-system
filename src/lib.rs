//! Signal contract — `system` → `router`.
//!
//! Read this file as the public interface of the OS-facts channel between the
//! system observer and the routing actor. The channel carries:
//!
//! - **Focus watch requests** from the router (`WatchFocus` carrying a
//!   `FocusSubscription`).
//! - **Focus unwatch requests** from the router (`UnwatchFocus` carrying a
//!   `FocusSubscriptionToken`). Close follows the Path A discipline: the
//!   system acks the close with a typed `SubscriptionRetracted` response
//!   carrying the closed token, never a fire-and-forget op.
//! - **One-shot observation requests** from the router (`QueryFocus` — the
//!   current focus state right now, no subscription established).
//! - **Component status queries** from the router (`QueryStatus`).
//! - **Observation events** from `system` — focus changes and target
//!   lifecycle — pushed as [`SystemStreamEvent`] frames once a subscription
//!   is accepted.
//!
//! The channel is **bidirectional**: the router initiates subscriptions; the
//! system pushes observation events back over the same channel after
//! subscriptions are accepted. The system pushes; the router never polls.
//!
//! The privileged plane — the daemon's typed startup configuration and the
//! `SystemBackend` it selects — lives in `meta-signal-system`. This contract
//! imports `SystemBackend` from there rather than declaring a second one.
//!
//! See `ARCHITECTURE.md` for the channel's role and boundaries.

pub mod generated;
pub use generated::signal::*;

pub const ETHOS: &str = include_str!("../ethos/signal.ethos");

/// The portable rkyv Signal frame is one shared type across the estate.
/// A second copy here would be a different Rust type, forking the wire.
pub use signal::{ByteViewable, Restorable, Signal, Signalizable};
