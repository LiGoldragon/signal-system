//! Schema-derived Signal contract for `system` focus observations.
//!
//! This crate is the public wire interface between `router` (request side) and
//! `system` (reply/event side). Router opens focus subscriptions with
//! `WatchFocus`; system acknowledges the subscription, pushes focus events on
//! `FocusEventStream`, and acknowledges `UnwatchFocus` retractions with a typed
//! `SubscriptionRetracted` reply.
//!
//! `schema/lib.schema` is the source of truth. The checked-in
//! `src/schema/lib.rs` is a freshness-checked schema-rust artifact, not
//! handwritten vocabulary. See `ARCHITECTURE.md` for the channel role and
//! boundaries.

#[rustfmt::skip]
#[allow(dead_code, private_interfaces)]
pub mod schema;

pub use schema::lib::*;

pub type SystemRequest = Input;
pub type SystemReply = Output;
pub type SystemFrame = Frame;
pub type SystemFrameBody = FrameBody;
pub type SystemReplyEnvelope = ReplyEnvelope;
pub type SystemRequestBuilder = RequestBuilder;
pub type OperationKind = SystemOperationKind;
pub type StreamKind = SystemStreamKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemStreamKind {
    FocusEventStream,
}

impl Copy for NiriWindowId {}
impl Copy for ObservationGeneration {}
impl Copy for Target {}
impl Copy for Focused {}
impl Copy for Generation {}
impl Copy for Backend {}
impl Copy for Kind {}
impl Copy for Health {}
impl Copy for Readiness {}
impl Copy for Operation {}
impl Copy for Reason {}
impl Copy for SystemTarget {}
impl Copy for FocusSubscription {}
impl Copy for FocusSubscriptionToken {}
impl Copy for FocusSnapshot {}
impl Copy for SystemStatusQuery {}
impl Copy for FocusObservation {}
impl Copy for WindowClosed {}
impl Copy for SubscriptionAccepted {}
impl Copy for ObservationTargetMissing {}
impl Copy for SystemStatus {}
impl Copy for SystemRequestUnimplemented {}

impl WirePath {
    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl std::fmt::Display for WirePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.payload().fmt(formatter)
    }
}

impl AsRef<str> for WirePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for WirePath {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl SocketMode {
    pub fn into_u32(self) -> u32 {
        self.into_payload() as u32
    }
}

impl SystemSocketPath {
    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl SupervisionSocketPath {
    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl SystemSocketMode {
    pub fn into_u32(self) -> u32 {
        self.into_payload().into_u32()
    }
}

impl SupervisionSocketMode {
    pub fn into_u32(self) -> u32 {
        self.into_payload().into_u32()
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

impl UnixUserIdentifier {
    pub fn as_u32(&self) -> u32 {
        *self.payload() as u32
    }
}

impl SystemPrincipal {
    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl NiriWindowId {
    pub fn value(self) -> u64 {
        self.into_payload()
    }
}

impl SystemTarget {
    pub fn niri_window_id(self) -> Option<NiriWindowId> {
        match self {
            Self::NiriWindow(window_id) => Some(window_id),
        }
    }
}

impl ObservationGeneration {
    pub fn into_u64(self) -> u64 {
        self.into_payload()
    }
}

impl Target {
    pub fn system_target(&self) -> SystemTarget {
        *self.payload()
    }
}

impl Focused {
    pub fn as_bool(&self) -> bool {
        *self.payload()
    }

    pub fn into_bool(self) -> bool {
        self.into_payload()
    }
}

impl Generation {
    pub fn into_u64(self) -> u64 {
        self.into_payload().into_u64()
    }
}

impl Backend {
    pub fn system_backend(&self) -> SystemBackend {
        *self.payload()
    }
}

impl Kind {
    pub fn subscription_kind(&self) -> SubscriptionKind {
        *self.payload()
    }
}

impl Health {
    pub fn system_health(&self) -> SystemHealth {
        *self.payload()
    }
}

impl Readiness {
    pub fn system_readiness(&self) -> SystemReadiness {
        *self.payload()
    }
}

impl Operation {
    pub fn system_operation_kind(&self) -> SystemOperationKind {
        *self.payload()
    }
}

impl Reason {
    pub fn system_unimplemented_reason(&self) -> SystemUnimplementedReason {
        *self.payload()
    }
}

impl FocusSubscription {
    pub fn from_target(target: SystemTarget) -> Self {
        Self::new(Target::new(target))
    }
}

impl From<SystemTarget> for FocusSubscription {
    fn from(target: SystemTarget) -> Self {
        Self::from_target(target)
    }
}

impl FocusSubscriptionToken {
    pub fn from_target(target: SystemTarget) -> Self {
        Self::new(Target::new(target))
    }
}

impl From<SystemTarget> for FocusSubscriptionToken {
    fn from(target: SystemTarget) -> Self {
        Self::from_target(target)
    }
}

impl FocusSnapshot {
    pub fn from_target(target: SystemTarget) -> Self {
        Self::new(Target::new(target))
    }
}

impl From<SystemTarget> for FocusSnapshot {
    fn from(target: SystemTarget) -> Self {
        Self::from_target(target)
    }
}

impl SystemStatusQuery {
    pub fn from_backend(backend: SystemBackend) -> Self {
        Self::new(Backend::new(backend))
    }
}

impl From<SystemBackend> for SystemStatusQuery {
    fn from(backend: SystemBackend) -> Self {
        Self::from_backend(backend)
    }
}

impl FocusObservation {
    pub fn new(target: SystemTarget, focused: bool, generation: u64) -> Self {
        Self {
            target: Target::new(target),
            focused: Focused::new(focused),
            generation: Generation::new(ObservationGeneration::new(generation)),
        }
    }
}

impl WindowClosed {
    pub fn from_target(target: SystemTarget) -> Self {
        Self::new(Target::new(target))
    }
}

impl From<SystemTarget> for WindowClosed {
    fn from(target: SystemTarget) -> Self {
        Self::from_target(target)
    }
}

impl ObservationTargetMissing {
    pub fn from_target(target: SystemTarget) -> Self {
        Self::new(Target::new(target))
    }
}

impl From<SystemTarget> for ObservationTargetMissing {
    fn from(target: SystemTarget) -> Self {
        Self::from_target(target)
    }
}

impl SubscriptionRetracted {
    pub fn from_token(token: FocusSubscriptionToken) -> Self {
        Self::new(Token::new(token))
    }
}

impl SystemRequestUnimplemented {
    pub fn from_parts(operation: SystemOperationKind, reason: SystemUnimplementedReason) -> Self {
        Self {
            operation: Operation::new(operation),
            reason: Reason::new(reason),
        }
    }
}

impl Input {
    pub fn operation_kind(&self) -> SystemOperationKind {
        match self {
            Self::WatchFocus(_) => SystemOperationKind::WatchFocus,
            Self::UnwatchFocus(_) => SystemOperationKind::UnwatchFocus,
            Self::QueryFocus(_) => SystemOperationKind::QueryFocus,
            Self::QueryStatus(_) => SystemOperationKind::QueryStatus,
        }
    }

    pub fn kind(&self) -> SystemOperationKind {
        self.operation_kind()
    }

    pub fn opened_stream(&self) -> Option<SystemStreamKind> {
        match self {
            Self::WatchFocus(_) => Some(SystemStreamKind::FocusEventStream),
            Self::UnwatchFocus(_) | Self::QueryFocus(_) | Self::QueryStatus(_) => None,
        }
    }

    pub fn closed_stream(&self) -> Option<SystemStreamKind> {
        match self {
            Self::UnwatchFocus(_) => Some(SystemStreamKind::FocusEventStream),
            Self::WatchFocus(_) | Self::QueryFocus(_) | Self::QueryStatus(_) => None,
        }
    }
}

impl SystemEvent {
    pub fn stream_kind(&self) -> SystemStreamKind {
        match self {
            Self::FocusObservation(_) | Self::WindowClosed(_) => SystemStreamKind::FocusEventStream,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDaemonConfigurationParts {
    pub system_socket_path: WirePath,
    pub system_socket_mode: SocketMode,
    pub supervision_socket_path: WirePath,
    pub supervision_socket_mode: SocketMode,
    pub backend: SystemBackend,
    pub owner_identity: OwnerIdentity,
}

impl From<SystemDaemonConfigurationParts> for SystemDaemonConfiguration {
    fn from(parts: SystemDaemonConfigurationParts) -> Self {
        Self {
            system_socket_path: SystemSocketPath::new(parts.system_socket_path),
            system_socket_mode: SystemSocketMode::new(parts.system_socket_mode),
            supervision_socket_path: SupervisionSocketPath::new(parts.supervision_socket_path),
            supervision_socket_mode: SupervisionSocketMode::new(parts.supervision_socket_mode),
            backend: Backend::new(parts.backend),
            owner_identity: parts.owner_identity,
        }
    }
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
