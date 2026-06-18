//! Signal contract - `system` -> `router`.
//!
//! Read this file as the public interface of the OS-facts
//! channel. The channel carries:
//!
//! - **Focus watch requests** from the router
//!   (`WatchFocus` carrying `FocusSubscription`).
//! - **Focus unwatch requests** from the router
//!   (`UnwatchFocus` carrying a `FocusSubscriptionToken`). Close follows the
//!   **Path A** discipline per /181: the system acks the
//!   close with a typed `SubscriptionRetracted` reply
//!   carrying the closed token, not a request-only
//!   fire-and-forget op.
//! - **One-shot observation requests** from the router
//!   (`QueryFocus` — current focus state right
//!   now, no subscription established).
//! - **Component status query** from the router
//!   (`QueryStatus`).
//! - **Observation events** from `system` (focus
//!   changes and target lifecycle) emitted on the
//!   `FocusEventStream` after a subscription opens.
//!
//! The channel is **bidirectional**: the router initiates
//! subscriptions; the system pushes observation events back
//! over the same channel after subscriptions are accepted.
//! Per `~/primary/skills/push-not-pull.md`, the system
//! pushes; the router never polls.
//!
//! See `ARCHITECTURE.md` for the channel's role and boundaries.

use nota_next::{Block, Delimiter, NotaBlock, NotaDecode, NotaDecodeError, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;
use signal_persona::OwnerIdentity;

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WirePath(String);

impl WirePath {
    pub fn new(payload: impl Into<String>) -> Self {
        Self(payload.into())
    }

    pub fn payload(&self) -> &String {
        &self.0
    }

    pub fn into_payload(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl From<String> for WirePath {
    fn from(payload: String) -> Self {
        Self::new(payload)
    }
}

impl std::fmt::Display for WirePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.payload().fmt(formatter)
    }
}

impl AsRef<str> for WirePath {
    fn as_ref(&self) -> &str {
        self.payload().as_str()
    }
}

impl PartialEq<&str> for WirePath {
    fn eq(&self, other: &&str) -> bool {
        self.payload() == other
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct SocketMode(u64);

impl SocketMode {
    pub const fn new(payload: u64) -> Self {
        Self(payload)
    }

    pub const fn payload(&self) -> &u64 {
        &self.0
    }

    pub const fn into_payload(self) -> u64 {
        self.0
    }

    pub const fn into_u32(self) -> u32 {
        self.0 as u32
    }
}

impl From<u64> for SocketMode {
    fn from(payload: u64) -> Self {
        Self::new(payload)
    }
}

impl std::fmt::Display for SocketMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.payload().fmt(formatter)
    }
}

impl PartialEq<u64> for SocketMode {
    fn eq(&self, other: &u64) -> bool {
        self.payload() == other
    }
}

impl PartialOrd<u64> for SocketMode {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        self.payload().partial_cmp(other)
    }
}

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
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    fn to_nota(&self) -> String {
        match self {
            Self::NiriWindow(window_id) => format!("(NiriWindow {})", window_id.to_nota()),
        }
    }
}

impl NotaDecode for SystemTarget {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let fields =
            NotaBlock::new(block).expect_children(Delimiter::Parenthesis, "NiriWindow", 2)?;
        let window_id = NiriWindowId::from_nota_block(&fields[1])?;
        Ok(Self::NiriWindow(window_id))
    }
}

impl NotaDecode for NiriWindowId {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        Ok(Self(NotaBlock::new(block).parse_integer()?))
    }
}

impl NotaEncode for NiriWindowId {
    fn to_nota(&self) -> String {
        self.0.to_string()
    }
}

// ─── Subscription requests (router → system) ──────────────

/// Monotonic observation counter minted by `system`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationGeneration(u64);

impl ObservationGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

impl NotaDecode for ObservationGeneration {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        Ok(Self(NotaBlock::new(block).parse_integer()?))
    }
}

impl NotaEncode for ObservationGeneration {
    fn to_nota(&self) -> String {
        self.0.to_string()
    }
}

/// Subscribe to focus events for `target`. The system
/// replies with an `Accepted` event and then pushes
/// `FocusObservation` events whenever focus changes.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct FocusSubscription {
    pub target: SystemTarget,
}

/// Per-subscription identity for the focus event stream. Carried in the
/// `FocusSubscription`-shaped retract request and echoed back in the
/// `SubscriptionRetracted` reply so callers can match the ack to the
/// request they sent. Matches the structural shape of
/// `<Channel>SubscriptionToken` newtypes per /176 §1 stream-block
/// grammar (`signal-terminal::TerminalWorkerLifecycleToken` is the
/// worked example).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct FocusSubscriptionToken {
    pub target: SystemTarget,
}

/// One-shot: what is the focus state for `target` *right
/// now*? Reply is a single `FocusObservation` event; no
/// subscription established.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct FocusSnapshot {
    pub target: SystemTarget,
}

/// Component-level health/readiness request for the system
/// boundary. The backend is named so a future multi-backend
/// system daemon can answer one backend without implying every
/// backend is healthy.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct SystemStatusQuery {
    pub backend: SystemBackend,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum SystemBackend {
    Niri,
}

// ─── Observation events (system → router) ─────────────────

/// Focus changed (or current state, for a one-shot `FocusSnapshot`).
/// `generation` is a monotonic counter the system mints; the
/// router uses it to discard stale events when subscriptions
/// race.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
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
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct WindowClosed {
    pub target: SystemTarget,
}

/// Subscription was accepted; events of the named kind will
/// follow.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct SubscriptionAccepted {
    pub target: SystemTarget,
    pub kind: SubscriptionKind,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum SubscriptionKind {
    Focus,
}

/// The system can't observe the named target — it doesn't
/// exist (yet, or any more), or the system has no backend
/// for it.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct ObservationTargetMissing {
    pub target: SystemTarget,
}

/// The system daemon's current health and readiness for one
/// backend.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct SystemStatus {
    pub backend: SystemBackend,
    pub health: SystemHealth,
    pub readiness: SystemReadiness,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum SystemHealth {
    Running,
    Degraded,
    Stopped,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum SystemReadiness {
    Ready,
    Starting,
    Unavailable,
}

/// Typed acknowledgement that a focus-event subscription has been
/// retracted. Returned in reply to an `UnwatchFocus` request.
/// Carries the retracted token so callers can match the ack to the
/// request they sent. This is the Path A reply variant per /181;
/// retraction is a closed reply event signaling the stream is over,
/// not a request-only fire-and-forget op.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct SubscriptionRetracted {
    pub token: FocusSubscriptionToken,
}

/// A recognized request reached the system daemon, but that
/// operation is not implemented by this daemon skeleton yet.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "nota-text", derive(NotaEncode, NotaDecode))]
pub struct SystemRequestUnimplemented {
    pub operation: SystemOperationKind,
    pub reason: SystemUnimplementedReason,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum SystemUnimplementedReason {
    NotBuiltYet,
    BackendUnavailable,
}

// ─── Channel declaration ───────────────────────────────────

signal_channel! {
    channel System {
        operation WatchFocus(FocusSubscription) opens FocusEventStream,
        operation UnwatchFocus(FocusSubscriptionToken),
        operation QueryFocus(FocusSnapshot),
        operation QueryStatus(SystemStatusQuery),
    }
    reply SystemReply {
        SubscriptionAccepted(SubscriptionAccepted),
        SubscriptionRetracted(SubscriptionRetracted),
        ObservationTargetMissing(ObservationTargetMissing),
        SystemStatus(SystemStatus),
        SystemRequestUnimplemented(SystemRequestUnimplemented),
        QueryFocusReply(FocusObservation),
    }
    event SystemEvent {
        FocusObservation(FocusObservation) belongs FocusEventStream,
        WindowClosed(WindowClosed) belongs FocusEventStream,
    }
    stream FocusEventStream {
        token FocusSubscriptionToken;
        opened SubscriptionAccepted;
        event FocusObservation;
        close UnwatchFocus;
    }
}

pub type SystemRequest = Operation;
pub type SystemFrame = Frame;
pub type SystemFrameBody = FrameBody;
pub type SystemReplyEnvelope = ReplyEnvelope;
pub type SystemRequestBuilder = RequestBuilder;
pub type SystemOperationKind = OperationKind;
pub type SystemStreamKind = StreamKind;

impl SystemRequest {
    pub fn operation_kind(&self) -> SystemOperationKind {
        self.kind()
    }
}

// ─── Daemon configuration ──────────────────────────────────
//
// Typed startup configuration for `system-daemon`. Human tooling may
// author this record through NOTA, but the live daemon consumes a
// signal-encoded rkyv archive path. The daemon does not parse NOTA.

/// Startup configuration for `system-daemon`.
///
/// Replaces the previous positional `<socket>` argv plus
/// `PERSONA_SOCKET_MODE`, `PERSONA_SUPERVISION_SOCKET_PATH`, and
/// `PERSONA_SUPERVISION_SOCKET_MODE` argv/environment-variable
/// surface.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct SystemDaemonConfiguration {
    /// Where the daemon binds its system Unix socket.
    pub system_socket_path: WirePath,
    /// chmod applied to the system socket after bind.
    pub system_socket_mode: SocketMode,
    /// Where the daemon binds its supervision Unix socket.
    pub supervision_socket_path: WirePath,
    /// chmod applied to the supervision socket after bind.
    pub supervision_socket_mode: SocketMode,
    /// The compositor/system backend the daemon presents.
    pub backend: SystemBackend,
    /// The engine owner identity passed to the system daemon.
    pub owner_identity: OwnerIdentity,
}

impl SystemDaemonConfiguration {
    pub fn from_rkyv_bytes(bytes: &[u8]) -> Result<Self, SystemDaemonConfigurationArchiveError> {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)
            .map_err(|_| SystemDaemonConfigurationArchiveError::Decode)
    }

    pub fn to_rkyv_bytes(&self) -> Result<Vec<u8>, SystemDaemonConfigurationArchiveError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|_| SystemDaemonConfigurationArchiveError::Encode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SystemDaemonConfigurationArchiveError {
    #[error("failed to encode system daemon configuration archive")]
    Encode,
    #[error("failed to decode system daemon configuration archive")]
    Decode,
}
