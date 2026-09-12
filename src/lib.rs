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

use std::marker::PhantomData;

pub const ETHOS: &str = include_str!("../ethos/signal.ethos");

/// A portable rkyv Signal frame whose target contract is carried in its type.
pub struct Signal<T> {
    bytes: Vec<u8>,
    target: PhantomData<fn() -> T>,
}

/// Data that can form a portable Signal frame.
pub trait Signalizable: Sized {
    fn signalize(&self) -> Result<Signal<Self>, rkyv::rancor::Error>;
}

/// A frame exposes its peer-wire bytes for transport framing.
pub trait ByteViewable {
    fn bytes(&self) -> &[u8];
}

/// A typed portable Signal can restore the contract value it carries.
pub trait Restorable<T> {
    fn restore(&self) -> Result<T, rkyv::rancor::Error>;
}

impl Signalizable for Query {
    fn signalize(&self) -> Result<Signal<Self>, rkyv::rancor::Error> {
        Ok(Signal {
            bytes: rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec(),
            target: PhantomData,
        })
    }
}

impl Signalizable for Response {
    fn signalize(&self) -> Result<Signal<Self>, rkyv::rancor::Error> {
        Ok(Signal {
            bytes: rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec(),
            target: PhantomData,
        })
    }
}

impl Signalizable for SystemStreamEvent {
    fn signalize(&self) -> Result<Signal<Self>, rkyv::rancor::Error> {
        Ok(Signal {
            bytes: rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec(),
            target: PhantomData,
        })
    }
}

impl<T> From<Vec<u8>> for Signal<T> {
    fn from(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            target: PhantomData,
        }
    }
}

impl<T> ByteViewable for Signal<T> {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Restorable<Query> for Signal<Query> {
    fn restore(&self) -> Result<Query, rkyv::rancor::Error> {
        rkyv::from_bytes(self.bytes())
    }
}

impl Restorable<Response> for Signal<Response> {
    fn restore(&self) -> Result<Response, rkyv::rancor::Error> {
        rkyv::from_bytes(self.bytes())
    }
}

impl Restorable<SystemStreamEvent> for Signal<SystemStreamEvent> {
    fn restore(&self) -> Result<SystemStreamEvent, rkyv::rancor::Error> {
        rkyv::from_bytes(self.bytes())
    }
}
