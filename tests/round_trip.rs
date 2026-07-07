//! Architectural-truth round-trip tests for the `signal-system` channel.

use nota::{NotaDecode, NotaEncode, NotaSource};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, SignalOperationHeads, StreamEventIdentifier,
    SubscriptionTokenInner,
};
use signal_system::{
    Backend, FocusObservation, FocusSnapshot, FocusSubscription, FocusSubscriptionToken, Health,
    Kind, ObservationTargetMissing, Operation, OwnerIdentity, Readiness, Reason, SocketMode,
    SubscriptionAccepted, SubscriptionKind, SubscriptionRetracted, SystemBackend,
    SystemDaemonConfiguration, SystemDaemonConfigurationParts, SystemEvent, SystemFrame,
    SystemFrameBody, SystemHealth, SystemOperationKind, SystemReadiness, SystemReply,
    SystemRequest, SystemRequestUnimplemented, SystemStatus, SystemStatusQuery, SystemStreamKind,
    SystemTarget, SystemUnimplementedReason, Target, UnixUserIdentifier, WindowClosed, WirePath,
};

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

use signal_frame::SessionEpoch;

fn stream_event() -> StreamEventIdentifier {
    StreamEventIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Acceptor,
        LaneSequence::first(),
    )
}

fn target() -> SystemTarget {
    SystemTarget::niri_window(223)
}

fn target_field() -> Target {
    Target::new(target())
}

fn token() -> FocusSubscriptionToken {
    FocusSubscriptionToken::from_target(target())
}

fn observation(generation: u64) -> FocusObservation {
    FocusObservation::new(target(), true, generation)
}

fn round_trip_request(request: SystemRequest) -> SystemRequest {
    let bytes = request
        .clone()
        .into_frame(exchange())
        .encode_length_prefixed()
        .expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: SystemReply) -> SystemReply {
    let bytes = reply
        .clone()
        .into_reply_frame(exchange())
        .encode_length_prefixed()
        .expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::Reply { reply, .. } => match reply {
            signal_frame::Reply::Accepted { per_operation, .. } => {
                match per_operation.into_head() {
                    signal_frame::SubReply::Ok(payload) => payload,
                    other => panic!("expected Ok sub-reply, got {other:?}"),
                }
            }
            signal_frame::Reply::Rejected { reason } => {
                panic!("unexpected rejected reply: {reason:?}")
            }
        },
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_event(event: SystemEvent) -> SystemEvent {
    let bytes = event
        .clone()
        .into_subscription_frame(stream_event(), SubscriptionTokenInner::new(1))
        .encode_length_prefixed()
        .expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::SubscriptionEvent { event, .. } => event,
        other => panic!("expected subscription event, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let encoded = value.to_nota();
    assert_eq!(encoded, expected);

    let recovered = NotaSource::new(&encoded)
        .parse::<T>()
        .expect("decode nota text");
    assert_eq!(recovered, value);
}

#[test]
fn every_request_round_trips_through_length_prefixed_frame() {
    let requests = [
        SystemRequest::WatchFocus(FocusSubscription::from_target(target())),
        SystemRequest::UnwatchFocus(token()),
        SystemRequest::QueryFocus(FocusSnapshot::from_target(target())),
        SystemRequest::QueryStatus(SystemStatusQuery::from_backend(SystemBackend::Niri)),
    ];

    for request in requests {
        assert_eq!(round_trip_request(request.clone()), request);
    }
}

#[test]
fn every_reply_round_trips_through_length_prefixed_frame() {
    let replies = [
        SystemReply::SubscriptionAccepted(SubscriptionAccepted {
            target: target_field(),
            kind: Kind::new(SubscriptionKind::Focus),
        }),
        SystemReply::SubscriptionRetracted(SubscriptionRetracted::from_token(token())),
        SystemReply::ObservationTargetMissing(ObservationTargetMissing::from_target(target())),
        SystemReply::SystemStatus(SystemStatus {
            backend: Backend::new(SystemBackend::Niri),
            health: Health::new(SystemHealth::Running),
            readiness: Readiness::new(SystemReadiness::Ready),
        }),
        SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented::from_parts(
            SystemOperationKind::WatchFocus,
            SystemUnimplementedReason::NotBuiltYet,
        )),
        SystemReply::QueryFocusReply(observation(44)),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn every_event_round_trips_through_length_prefixed_frame() {
    let events = [
        SystemEvent::FocusObservation(observation(42)),
        SystemEvent::WindowClosed(WindowClosed::from_target(target())),
    ];

    for event in events {
        assert_eq!(round_trip_event(event.clone()), event);
        assert_eq!(event.stream_kind(), SystemStreamKind::FocusEventStream);
    }
}

#[test]
fn request_stream_lifecycle_metadata_is_available() {
    let watch = SystemRequest::WatchFocus(FocusSubscription::from_target(target()));
    let unwatch = SystemRequest::UnwatchFocus(token());
    let query = SystemRequest::QueryFocus(FocusSnapshot::from_target(target()));

    assert_eq!(
        watch.opened_stream(),
        Some(SystemStreamKind::FocusEventStream)
    );
    assert_eq!(watch.closed_stream(), None);
    assert_eq!(unwatch.opened_stream(), None);
    assert_eq!(
        unwatch.closed_stream(),
        Some(SystemStreamKind::FocusEventStream)
    );
    assert_eq!(query.opened_stream(), None);
    assert_eq!(query.closed_stream(), None);
}

#[test]
fn system_request_exposes_contract_owned_operation_kind() {
    let cases = [
        (
            SystemRequest::WatchFocus(FocusSubscription::from_target(target())),
            SystemOperationKind::WatchFocus,
        ),
        (
            SystemRequest::UnwatchFocus(token()),
            SystemOperationKind::UnwatchFocus,
        ),
        (
            SystemRequest::QueryFocus(FocusSnapshot::from_target(target())),
            SystemOperationKind::QueryFocus,
        ),
        (
            SystemRequest::QueryStatus(SystemStatusQuery::from_backend(SystemBackend::Niri)),
            SystemOperationKind::QueryStatus,
        ),
    ];

    for (request, operation) in cases {
        assert_eq!(request.operation_kind(), operation);
        assert_eq!(request.kind(), operation);
    }
}

#[test]
fn system_request_variants_declare_contract_local_operation_heads() {
    assert_eq!(
        <SystemRequest as SignalOperationHeads>::HEADS,
        &["WatchFocus", "UnwatchFocus", "QueryFocus", "QueryStatus"]
    );
}

#[test]
fn every_root_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::WatchFocus(FocusSubscription::from_target(target())),
        "(WatchFocus (NiriWindow 223))",
    );
    round_trip_nota(
        SystemRequest::UnwatchFocus(token()),
        "(UnwatchFocus (NiriWindow 223))",
    );
    round_trip_nota(
        SystemRequest::QueryFocus(FocusSnapshot::from_target(target())),
        "(QueryFocus (NiriWindow 223))",
    );
    round_trip_nota(
        SystemRequest::QueryStatus(SystemStatusQuery::from_backend(SystemBackend::Niri)),
        "(QueryStatus Niri)",
    );

    round_trip_nota(
        SystemReply::SubscriptionAccepted(SubscriptionAccepted {
            target: target_field(),
            kind: Kind::new(SubscriptionKind::Focus),
        }),
        "(SubscriptionAccepted ((NiriWindow 223) Focus))",
    );
    round_trip_nota(
        SystemReply::SubscriptionRetracted(SubscriptionRetracted::from_token(token())),
        "(SubscriptionRetracted (NiriWindow 223))",
    );
    round_trip_nota(
        SystemReply::ObservationTargetMissing(ObservationTargetMissing::from_target(
            SystemTarget::niri_window(999),
        )),
        "(ObservationTargetMissing (NiriWindow 999))",
    );
    round_trip_nota(
        SystemReply::SystemStatus(SystemStatus {
            backend: Backend::new(SystemBackend::Niri),
            health: Health::new(SystemHealth::Running),
            readiness: Readiness::new(SystemReadiness::Ready),
        }),
        "(SystemStatus (Niri Running Ready))",
    );
    round_trip_nota(
        SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
            operation: Operation::new(SystemOperationKind::QueryFocus),
            reason: Reason::new(SystemUnimplementedReason::NotBuiltYet),
        }),
        "(SystemRequestUnimplemented (QueryFocus NotBuiltYet))",
    );
    round_trip_nota(
        SystemReply::QueryFocusReply(observation(12)),
        "(QueryFocusReply ((NiriWindow 223) True 12))",
    );

    round_trip_nota(
        SystemEvent::FocusObservation(observation(12)),
        "(FocusObservation ((NiriWindow 223) True 12))",
    );
    round_trip_nota(
        SystemEvent::WindowClosed(WindowClosed::from_target(target())),
        "(WindowClosed (NiriWindow 223))",
    );
}

#[test]
fn system_operation_kind_round_trips_through_nota_text() {
    round_trip_nota(SystemOperationKind::WatchFocus, "WatchFocus");
    round_trip_nota(SystemOperationKind::UnwatchFocus, "UnwatchFocus");
    round_trip_nota(SystemOperationKind::QueryFocus, "QueryFocus");
    round_trip_nota(SystemOperationKind::QueryStatus, "QueryStatus");
}

#[test]
fn generated_field_wrappers_expose_payloads() {
    let observation = observation(42);
    assert_eq!(observation.target.system_target(), target());
    assert!(observation.focused.as_bool());
    assert_eq!(observation.generation.into_u64(), 42);

    let status = SystemStatus {
        backend: Backend::new(SystemBackend::Niri),
        health: Health::new(SystemHealth::Running),
        readiness: Readiness::new(SystemReadiness::Ready),
    };
    assert_eq!(status.backend.system_backend(), SystemBackend::Niri);
    assert_eq!(status.health.system_health(), SystemHealth::Running);
    assert_eq!(status.readiness.system_readiness(), SystemReadiness::Ready);
}

#[test]
fn system_contract_cannot_carry_terminal_prompt_gate_records() {
    let scan = DriftScan::new(env!("CARGO_MANIFEST_DIR"));

    scan.assert_absent(&[
        "InputBuffer",
        "input-buffer",
        "prompt buffer",
        "prompt-buffer",
        "gate message delivery",
        "gate deliveries",
    ]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriftScan {
    root: std::path::PathBuf,
}

impl DriftScan {
    fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn assert_absent(&self, forbidden_fragments: &[&str]) {
        let mut violations = Vec::new();
        self.collect_violations("src/lib.rs", forbidden_fragments, &mut violations);
        self.collect_violations("schema/lib.schema", forbidden_fragments, &mut violations);
        assert!(
            violations.is_empty(),
            "terminal prompt-gate records belong to signal-terminal:\n{}",
            violations.join("\n")
        );
    }

    fn collect_violations(
        &self,
        relative_path: &str,
        forbidden_fragments: &[&str],
        violations: &mut Vec<String>,
    ) {
        let path = self.root.join(relative_path);
        let content = std::fs::read_to_string(&path).expect("scan source file");
        for fragment in forbidden_fragments {
            if content.contains(fragment) {
                violations.push(format!("{relative_path} contains {fragment}"));
            }
        }
    }
}

#[test]
fn system_daemon_configuration_round_trips_through_nota_text() {
    let configuration = SystemDaemonConfiguration::from(SystemDaemonConfigurationParts {
        system_socket_path: WirePath::new("/run/persona/X/system.sock"),
        system_socket_mode: SocketMode::new(0o600),
        supervision_socket_path: WirePath::new("/run/persona/X/system-supervision.sock"),
        supervision_socket_mode: SocketMode::new(0o600),
        backend: SystemBackend::Niri,
        owner_identity: OwnerIdentity::UnixUser(UnixUserIdentifier::new(1000)),
    });

    let text = configuration.to_nota();
    let recovered = NotaSource::new(&text)
        .parse::<SystemDaemonConfiguration>()
        .expect("decode configuration");

    assert_eq!(recovered, configuration);
    assert!(text.contains("/run/persona/X/system.sock"));
    assert!(text.contains("Niri"));
}

#[test]
fn system_daemon_configuration_round_trips_through_rkyv() {
    let configuration = SystemDaemonConfiguration::from(SystemDaemonConfigurationParts {
        system_socket_path: WirePath::new("/run/persona/X/system.sock"),
        system_socket_mode: SocketMode::new(0o600),
        supervision_socket_path: WirePath::new("/run/persona/X/system-supervision.sock"),
        supervision_socket_mode: SocketMode::new(0o600),
        backend: SystemBackend::Niri,
        owner_identity: OwnerIdentity::UnixUser(UnixUserIdentifier::new(1000)),
    });

    let bytes = configuration.to_rkyv_bytes().expect("archive");
    let recovered = SystemDaemonConfiguration::from_rkyv_bytes(&bytes).expect("decode rkyv");
    assert_eq!(recovered, configuration);
}
