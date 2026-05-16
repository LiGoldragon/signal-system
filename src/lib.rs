//! Signal contract — `persona-system` → `persona-router`.
//!
//! Read this file as the public interface of the OS-facts
//! channel. The channel carries:
//!
//! - **Focus subscription requests** from the router
//!   (`Subscribe FocusSubscription`).
//! - **Focus subscription retraction requests** from the
//!   router (`Retract FocusSubscriptionRetraction` carrying
//!   a `FocusSubscriptionToken`). Retraction follows the
//!   **Path A** discipline per /181: the system acks the
//!   retraction with a typed `SubscriptionRetracted` reply
//!   carrying the closed token, not a request-only
//!   fire-and-forget op.
//! - **One-shot observation requests** from the router
//!   (`Match FocusSnapshot` — current focus state right
//!   now, no subscription established).
//! - **Component status query** from the router
//!   (`Match SystemStatusQuery`).
//! - **Observation events** from `persona-system` (focus
//!   changes and target lifecycle) emitted on the
//!   `FocusEventStream` after a subscription opens.
//!
//! The channel is **bidirectional**: the router initiates
//! subscriptions; the system pushes observation events back
//! over the same channel after subscriptions are accepted.
//! Per `~/primary/skills/push-not-pull.md`, the system
//! pushes; the router never polls.
//!
//! See `ARCHITECTURE.md` for the channel's role and
//! boundaries; `~/primary/reports/designer/72-harmonized-implementation-plan.md`
//! §6 for the contract-creation discipline.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_core::signal_channel;

// ─── Target identity ──────────────────────────────────────

/// A typed identifier for a window-shaped OS surface the
/// router cares about. Currently only Niri windows; future
/// backends (Mac, Hyprland, etc.) add variants through a
/// coordinated schema upgrade because this closed enum rejects
/// unknown variants at decode time.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemTarget {
    NiriWindow(NiriWindowId),
}

/// Niri's typed window id (a u64 newtype).
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaTransparent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub struct NiriWindowId(u64);

impl NiriWindowId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl SystemTarget {
    pub const fn niri_window(window_id: u64) -> Self {
        Self::NiriWindow(NiriWindowId::new(window_id))
    }

    pub const fn niri_window_id(self) -> Option<NiriWindowId> {
        match self {
            Self::NiriWindow(window_id) => Some(window_id),
        }
    }
}

impl NotaEncode for SystemTarget {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::NiriWindow(window_id) => {
                encoder.start_record("NiriWindow")?;
                window_id.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for SystemTarget {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        decoder.expect_record_head("NiriWindow")?;
        let window_id = NiriWindowId::decode(decoder)?;
        decoder.expect_record_end()?;
        Ok(Self::NiriWindow(window_id))
    }
}

// ─── Subscription requests (router → system) ──────────────

/// Monotonic observation counter minted by `persona-system`.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaTransparent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub struct ObservationGeneration(u64);

impl ObservationGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

/// Subscribe to focus events for `target`. The system
/// replies with an `Accepted` event and then pushes
/// `FocusObservation` events whenever focus changes.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct FocusSubscription {
    pub target: SystemTarget,
}

/// Per-subscription identity for the focus event stream. Carried in the
/// `FocusSubscription`-shaped retract request and echoed back in the
/// `SubscriptionRetracted` reply so callers can match the ack to the
/// request they sent. Matches the structural shape of
/// `<Channel>SubscriptionToken` newtypes per /176 §1 stream-block
/// grammar (`signal-persona-terminal::TerminalWorkerLifecycleToken` is
/// the worked example).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct FocusSubscriptionToken {
    pub target: SystemTarget,
}

/// One-shot: what is the focus state for `target` *right
/// now*? Reply is a single `FocusObservation` event; no
/// subscription established.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    pub target: SystemTarget,
}

/// Component-level health/readiness request for the system
/// boundary. The backend is named so a future multi-backend
/// system daemon can answer one backend without implying every
/// backend is healthy.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct SystemStatusQuery {
    pub backend: SystemBackend,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemBackend {
    Niri,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum SystemOperationKind {
    FocusSubscription,
    FocusSubscriptionRetraction,
    FocusSnapshot,
    SystemStatusQuery,
}

// ─── Observation events (system → router) ─────────────────

/// Focus changed (or current state, for a one-shot `FocusSnapshot`).
/// `generation` is a monotonic counter the system mints; the
/// router uses it to discard stale events when subscriptions
/// race.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct FocusObservation {
    pub target: SystemTarget,
    pub focused: bool,
    pub generation: ObservationGeneration,
}

impl FocusObservation {
    pub const fn new(target: SystemTarget, focused: bool, generation: u64) -> Self {
        Self {
            target,
            focused,
            generation: ObservationGeneration::new(generation),
        }
    }
}

/// The target window has gone away (closed by user, killed,
/// etc.). The system stops emitting events for it; existing
/// subscriptions on that target are implicitly cancelled.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct WindowClosed {
    pub target: SystemTarget,
}

/// Subscription was accepted; events of the named kind will
/// follow.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct SubscriptionAccepted {
    pub target: SystemTarget,
    pub kind: SubscriptionKind,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionKind {
    Focus,
}

/// The system can't observe the named target — it doesn't
/// exist (yet, or any more), or the system has no backend
/// for it.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct ObservationTargetMissing {
    pub target: SystemTarget,
}

/// The system daemon's current health and readiness for one
/// backend.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct SystemStatus {
    pub backend: SystemBackend,
    pub health: SystemHealth,
    pub readiness: SystemReadiness,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemHealth {
    Running,
    Degraded,
    Stopped,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemReadiness {
    Ready,
    Starting,
    Unavailable,
}

/// Typed acknowledgement that a focus-event subscription has been
/// retracted. Returned in reply to a `FocusSubscriptionRetraction` request.
/// Carries the retracted token so callers can match the ack to the
/// request they sent. This is the Path A reply variant per /181;
/// retraction is a closed reply event signaling the stream is over,
/// not a request-only fire-and-forget op.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionRetracted {
    pub token: FocusSubscriptionToken,
}

/// A recognized request reached the system daemon, but that
/// operation is not implemented by this daemon skeleton yet.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct SystemRequestUnimplemented {
    pub operation: SystemOperationKind,
    pub reason: SystemUnimplementedReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemUnimplementedReason {
    NotBuiltYet,
    BackendUnavailable,
}

// ─── Channel declaration ───────────────────────────────────

signal_channel! {
    channel System {
        request SystemRequest {
            Subscribe FocusSubscription(FocusSubscription) opens FocusEventStream,
            Retract FocusSubscriptionRetraction(FocusSubscriptionToken),
            Match FocusSnapshot(FocusSnapshot),
            Match SystemStatusQuery(SystemStatusQuery),
        }
        reply SystemReply {
            SubscriptionAccepted(SubscriptionAccepted),
            SubscriptionRetracted(SubscriptionRetracted),
            ObservationTargetMissing(ObservationTargetMissing),
            SystemStatus(SystemStatus),
            SystemRequestUnimplemented(SystemRequestUnimplemented),
            FocusSnapshotReply(FocusObservation),
        }
        event SystemEvent {
            FocusObservation(FocusObservation) belongs FocusEventStream,
            WindowClosed(WindowClosed) belongs FocusEventStream,
        }
        stream FocusEventStream {
            token FocusSubscriptionToken;
            opened SubscriptionAccepted;
            event FocusObservation;
            close FocusSubscriptionRetraction;
        }
    }
}

impl SystemRequest {
    pub fn operation_kind(&self) -> SystemOperationKind {
        match self {
            Self::FocusSubscription(_) => SystemOperationKind::FocusSubscription,
            Self::FocusSubscriptionRetraction(_) => {
                SystemOperationKind::FocusSubscriptionRetraction
            }
            Self::FocusSnapshot(_) => SystemOperationKind::FocusSnapshot,
            Self::SystemStatusQuery(_) => SystemOperationKind::SystemStatusQuery,
        }
    }
}
