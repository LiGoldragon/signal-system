//! Architectural-truth round-trip tests for the
//! `signal-system` channel.
//!
//! Per `~/primary/skills/architectural-truth-tests.md`,
//! each variant of both enums has a witness test that
//! proves the macro-emitted type round-trips through a
//! length-prefixed Frame.

#[cfg(feature = "nota-text")]
use nota_next::{NotaDecode, NotaEncode, NotaSource};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SignalOperationHeads, StreamEventIdentifier, SubReply, SubscriptionTokenInner,
};
use signal_system::{
    FocusObservation, FocusSnapshot, FocusSubscription, FocusSubscriptionToken,
    ObservationGeneration, ObservationTargetMissing, SubscriptionAccepted, SubscriptionKind,
    SubscriptionRetracted, SystemBackend, SystemEvent, SystemFrame, SystemFrameBody, SystemHealth,
    SystemOperationKind, SystemReadiness, SystemReply, SystemRequest, SystemRequestUnimplemented,
    SystemStatus, SystemStatusQuery, SystemTarget, SystemUnimplementedReason, WindowClosed,
};

const TARGET: SystemTarget = SystemTarget::niri_window(223);

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn stream_event() -> StreamEventIdentifier {
    StreamEventIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Acceptor,
        LaneSequence::first(),
    )
}

fn round_trip_request(request: SystemRequest) -> SystemRequest {
    let frame = SystemFrame::new(SystemFrameBody::Request {
        exchange: exchange(),
        request: request.into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: SystemReply) -> SystemReply {
    let frame = SystemFrame::new(SystemFrameBody::Reply {
        exchange: exchange(),
        reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected Ok sub-reply, got {other:?}"),
            },
            Reply::Rejected { reason } => panic!("unexpected rejected reply: {reason:?}"),
        },
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_event(event: SystemEvent) -> SystemEvent {
    let frame = SystemFrame::new(SystemFrameBody::SubscriptionEvent {
        event_identifier: stream_event(),
        token: SubscriptionTokenInner::new(1),
        event,
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::SubscriptionEvent { event, .. } => event,
        other => panic!("expected subscription event, got {other:?}"),
    }
}

#[cfg(feature = "nota-text")]
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
fn focus_subscription_round_trips() {
    let request = SystemRequest::WatchFocus(FocusSubscription { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[cfg(feature = "nota-text")]
#[test]
fn focus_subscription_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::WatchFocus(FocusSubscription { target: TARGET }),
        "(WatchFocus ((NiriWindow 223)))",
    );
}

#[test]
fn focus_subscription_retraction_round_trips() {
    let request = SystemRequest::UnwatchFocus(FocusSubscriptionToken { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[cfg(feature = "nota-text")]
#[test]
fn focus_subscription_retraction_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::UnwatchFocus(FocusSubscriptionToken { target: TARGET }),
        "(UnwatchFocus ((NiriWindow 223)))",
    );
}

#[test]
fn subscription_retracted_reply_round_trips() {
    let reply = SystemReply::SubscriptionRetracted(SubscriptionRetracted {
        token: FocusSubscriptionToken { target: TARGET },
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[cfg(feature = "nota-text")]
#[test]
fn subscription_retracted_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SubscriptionRetracted(SubscriptionRetracted {
            token: FocusSubscriptionToken { target: TARGET },
        }),
        "(SubscriptionRetracted (((NiriWindow 223))))",
    );
}

#[test]
fn focus_snapshot_round_trips() {
    let request = SystemRequest::QueryFocus(FocusSnapshot { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[cfg(feature = "nota-text")]
#[test]
fn focus_snapshot_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::QueryFocus(FocusSnapshot { target: TARGET }),
        "(QueryFocus ((NiriWindow 223)))",
    );
}

#[test]
fn system_status_query_round_trips() {
    let request = SystemRequest::QueryStatus(SystemStatusQuery {
        backend: SystemBackend::Niri,
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[cfg(feature = "nota-text")]
#[test]
fn system_status_query_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::QueryStatus(SystemStatusQuery {
            backend: SystemBackend::Niri,
        }),
        "(QueryStatus (Niri))",
    );
}

#[test]
fn system_request_exposes_contract_owned_operation_kind() {
    let cases = [
        (
            SystemRequest::WatchFocus(FocusSubscription { target: TARGET }),
            SystemOperationKind::WatchFocus,
        ),
        (
            SystemRequest::UnwatchFocus(FocusSubscriptionToken { target: TARGET }),
            SystemOperationKind::UnwatchFocus,
        ),
        (
            SystemRequest::QueryFocus(FocusSnapshot { target: TARGET }),
            SystemOperationKind::QueryFocus,
        ),
        (
            SystemRequest::QueryStatus(SystemStatusQuery {
                backend: SystemBackend::Niri,
            }),
            SystemOperationKind::QueryStatus,
        ),
    ];

    for (request, operation) in cases {
        assert_eq!(request.operation_kind(), operation);
    }
}

#[test]
fn system_request_variants_declare_contract_local_operation_heads() {
    assert_eq!(
        <SystemRequest as SignalOperationHeads>::HEADS,
        &["WatchFocus", "UnwatchFocus", "QueryFocus", "QueryStatus"]
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn system_operation_kind_round_trips_through_nota_text() {
    round_trip_nota(SystemOperationKind::WatchFocus, "WatchFocus");
    round_trip_nota(SystemOperationKind::UnwatchFocus, "UnwatchFocus");
    round_trip_nota(SystemOperationKind::QueryFocus, "QueryFocus");
    round_trip_nota(SystemOperationKind::QueryStatus, "QueryStatus");
}

#[test]
fn focus_observation_round_trips_with_focused_true() {
    let event = SystemEvent::FocusObservation(FocusObservation {
        target: TARGET,
        focused: true,
        generation: ObservationGeneration::new(42),
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn focus_observation_round_trips_with_focused_false() {
    let event = SystemEvent::FocusObservation(FocusObservation {
        target: TARGET,
        focused: false,
        generation: ObservationGeneration::new(43),
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[cfg(feature = "nota-text")]
#[test]
fn focus_observation_event_round_trips_through_nota_text() {
    round_trip_nota(
        SystemEvent::FocusObservation(FocusObservation {
            target: TARGET,
            focused: true,
            generation: ObservationGeneration::new(42),
        }),
        "(FocusObservation ((NiriWindow 223) True 42))",
    );
}

#[test]
fn window_closed_round_trips() {
    let event = SystemEvent::WindowClosed(WindowClosed { target: TARGET });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[cfg(feature = "nota-text")]
#[test]
fn window_closed_event_round_trips_through_nota_text() {
    round_trip_nota(
        SystemEvent::WindowClosed(WindowClosed { target: TARGET }),
        "(WindowClosed ((NiriWindow 223)))",
    );
}

#[test]
fn subscription_accepted_round_trips_for_focus_kind() {
    let reply = SystemReply::SubscriptionAccepted(SubscriptionAccepted {
        target: TARGET,
        kind: SubscriptionKind::Focus,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[cfg(feature = "nota-text")]
#[test]
fn subscription_accepted_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SubscriptionAccepted(SubscriptionAccepted {
            target: TARGET,
            kind: SubscriptionKind::Focus,
        }),
        "(SubscriptionAccepted ((NiriWindow 223) Focus))",
    );
}

#[test]
fn observation_target_missing_round_trips() {
    let reply = SystemReply::ObservationTargetMissing(ObservationTargetMissing { target: TARGET });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[cfg(feature = "nota-text")]
#[test]
fn observation_target_missing_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::ObservationTargetMissing(ObservationTargetMissing { target: TARGET }),
        "(ObservationTargetMissing ((NiriWindow 223)))",
    );
}

#[test]
fn system_status_reply_round_trips() {
    let reply = SystemReply::SystemStatus(SystemStatus {
        backend: SystemBackend::Niri,
        health: SystemHealth::Running,
        readiness: SystemReadiness::Ready,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[cfg(feature = "nota-text")]
#[test]
fn system_status_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SystemStatus(SystemStatus {
            backend: SystemBackend::Niri,
            health: SystemHealth::Running,
            readiness: SystemReadiness::Ready,
        }),
        "(SystemStatus (Niri Running Ready))",
    );
}

#[test]
fn system_request_unimplemented_reply_round_trips() {
    let reply = SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
        operation: SystemOperationKind::WatchFocus,
        reason: SystemUnimplementedReason::NotBuiltYet,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn focus_snapshot_reply_round_trips() {
    let reply = SystemReply::QueryFocusReply(FocusObservation {
        target: TARGET,
        focused: true,
        generation: ObservationGeneration::new(44),
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[cfg(feature = "nota-text")]
#[test]
fn focus_snapshot_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::QueryFocusReply(FocusObservation {
            target: TARGET,
            focused: true,
            generation: ObservationGeneration::new(44),
        }),
        "(QueryFocusReply ((NiriWindow 223) True 44))",
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn system_request_unimplemented_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
            operation: SystemOperationKind::WatchFocus,
            reason: SystemUnimplementedReason::NotBuiltYet,
        }),
        "(SystemRequestUnimplemented (WatchFocus NotBuiltYet))",
    );
}

#[test]
fn explicit_variant_lifts_focus_subscription_into_request() {
    let payload = FocusSubscription { target: TARGET };
    let request = SystemRequest::WatchFocus(payload.clone());
    assert_eq!(request, SystemRequest::WatchFocus(payload));
}

#[test]
fn explicit_variant_lifts_system_status_query_into_request() {
    let payload = SystemStatusQuery {
        backend: SystemBackend::Niri,
    };
    let request = SystemRequest::QueryStatus(payload);
    assert_eq!(request, SystemRequest::QueryStatus(payload));
}

#[test]
fn explicit_variant_lifts_focus_observation_into_event() {
    let payload = FocusObservation {
        target: TARGET,
        focused: true,
        generation: ObservationGeneration::new(1),
    };
    let event = SystemEvent::FocusObservation(payload);
    assert_eq!(event, SystemEvent::FocusObservation(payload));
}

#[test]
fn explicit_variant_lifts_system_status_into_reply() {
    let payload = SystemStatus {
        backend: SystemBackend::Niri,
        health: SystemHealth::Running,
        readiness: SystemReadiness::Ready,
    };
    let reply = SystemReply::SystemStatus(payload);
    assert_eq!(reply, SystemReply::SystemStatus(payload));
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

#[cfg(feature = "nota-text")]
#[test]
fn system_daemon_configuration_round_trips_through_nota_text() {
    use nota_next::{NotaEncode, NotaSource};
    use signal_persona::origin::{OwnerIdentity, UnixUserIdentifier};
    use signal_persona::{SocketMode, WirePath};
    use signal_system::{SystemBackend, SystemDaemonConfiguration};

    let configuration = SystemDaemonConfiguration {
        system_socket_path: WirePath::new("/run/persona/X/system.sock"),
        system_socket_mode: SocketMode::new(0o600),
        supervision_socket_path: WirePath::new("/run/persona/X/system-supervision.sock"),
        supervision_socket_mode: SocketMode::new(0o600),
        backend: SystemBackend::Niri,
        owner_identity: OwnerIdentity::UnixUser(UnixUserIdentifier::new(1000)),
    };

    let text = configuration.to_nota();
    let recovered = NotaSource::new(&text)
        .parse::<SystemDaemonConfiguration>()
        .expect("decode configuration");

    assert_eq!(recovered, configuration);
}

#[test]
fn system_daemon_configuration_round_trips_through_rkyv() {
    use signal_persona::origin::{OwnerIdentity, UnixUserIdentifier};
    use signal_persona::{SocketMode, WirePath};
    use signal_system::{SystemBackend, SystemDaemonConfiguration};

    let configuration = SystemDaemonConfiguration {
        system_socket_path: WirePath::new("/run/persona/X/system.sock"),
        system_socket_mode: SocketMode::new(0o600),
        supervision_socket_path: WirePath::new("/run/persona/X/system-supervision.sock"),
        supervision_socket_mode: SocketMode::new(0o600),
        backend: SystemBackend::Niri,
        owner_identity: OwnerIdentity::UnixUser(UnixUserIdentifier::new(1000)),
    };

    let bytes = configuration.to_rkyv_bytes().expect("archive");
    let recovered = SystemDaemonConfiguration::from_rkyv_bytes(&bytes).expect("decode rkyv");
    assert_eq!(recovered, configuration);
}
