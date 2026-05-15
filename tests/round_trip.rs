//! Architectural-truth round-trip tests for the
//! `signal-persona-system` channel.
//!
//! Per `~/primary/skills/architectural-truth-tests.md`,
//! each variant of both enums has a witness test that
//! proves the macro-emitted type round-trips through a
//! length-prefixed Frame.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_core::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SignalVerb, StreamEventIdentifier, SubReply, SubscriptionTokenInner,
};
use signal_persona_system::{
    FocusObservation, FocusSnapshot, FocusSubscription, FocusUnsubscription, ObservationGeneration,
    ObservationTargetMissing, SubscriptionAccepted, SubscriptionKind, SystemBackend, SystemEvent,
    SystemFrame, SystemFrameBody, SystemHealth, SystemOperationKind, SystemReadiness, SystemReply,
    SystemRequest, SystemRequestUnimplemented, SystemStatus, SystemStatusQuery, SystemTarget,
    SystemUnimplementedReason, WindowClosed,
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
    let expected_verb = request.signal_verb();
    let frame = SystemFrame::new(SystemFrameBody::Request {
        exchange: exchange(),
        request: request.into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::Request { request, .. } => {
            let operation = request.operations().head();
            assert_eq!(operation.verb, expected_verb);
            operation.payload.clone()
        }
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: SystemReply) -> SystemReply {
    let frame = SystemFrame::new(SystemFrameBody::Reply {
        exchange: exchange(),
        reply: Reply::completed(NonEmpty::single(SubReply::Ok {
            verb: SignalVerb::Match,
            payload: reply,
        })),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = SystemFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        SystemFrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok { payload, .. } => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted reply, got {other:?}"),
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

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder).expect("encode nota text");
    let encoded = encoder.into_string();
    assert_eq!(encoded, expected);

    let mut decoder = Decoder::new(&encoded);
    let recovered = T::decode(&mut decoder).expect("decode nota text");
    assert_eq!(recovered, value);
}

#[test]
fn focus_subscription_round_trips() {
    let request = SystemRequest::FocusSubscription(FocusSubscription { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_subscription_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::FocusSubscription(FocusSubscription { target: TARGET }),
        "(FocusSubscription (NiriWindow 223))",
    );
}

#[test]
fn focus_unsubscription_round_trips() {
    let request = SystemRequest::FocusUnsubscription(FocusUnsubscription { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_unsubscription_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::FocusUnsubscription(FocusUnsubscription { target: TARGET }),
        "(FocusUnsubscription (NiriWindow 223))",
    );
}

#[test]
fn focus_snapshot_round_trips() {
    let request = SystemRequest::FocusSnapshot(FocusSnapshot { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_snapshot_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::FocusSnapshot(FocusSnapshot { target: TARGET }),
        "(FocusSnapshot (NiriWindow 223))",
    );
}

#[test]
fn system_status_query_round_trips() {
    let request = SystemRequest::SystemStatusQuery(SystemStatusQuery {
        backend: SystemBackend::Niri,
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn system_status_query_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::SystemStatusQuery(SystemStatusQuery {
            backend: SystemBackend::Niri,
        }),
        "(SystemStatusQuery Niri)",
    );
}

#[test]
fn system_request_exposes_contract_owned_operation_kind() {
    let cases = [
        (
            SystemRequest::FocusSubscription(FocusSubscription { target: TARGET }),
            SystemOperationKind::FocusSubscription,
        ),
        (
            SystemRequest::FocusUnsubscription(FocusUnsubscription { target: TARGET }),
            SystemOperationKind::FocusUnsubscription,
        ),
        (
            SystemRequest::FocusSnapshot(FocusSnapshot { target: TARGET }),
            SystemOperationKind::FocusSnapshot,
        ),
        (
            SystemRequest::SystemStatusQuery(SystemStatusQuery {
                backend: SystemBackend::Niri,
            }),
            SystemOperationKind::SystemStatusQuery,
        ),
    ];

    for (request, operation) in cases {
        assert_eq!(request.operation_kind(), operation);
    }
}

#[test]
fn system_request_variants_declare_expected_signal_root_verbs() {
    let cases = [
        (
            SystemRequest::FocusSubscription(FocusSubscription { target: TARGET }),
            SignalVerb::Subscribe,
        ),
        (
            SystemRequest::FocusUnsubscription(FocusUnsubscription { target: TARGET }),
            SignalVerb::Retract,
        ),
        (
            SystemRequest::FocusSnapshot(FocusSnapshot { target: TARGET }),
            SignalVerb::Match,
        ),
        (
            SystemRequest::SystemStatusQuery(SystemStatusQuery {
                backend: SystemBackend::Niri,
            }),
            SignalVerb::Match,
        ),
    ];

    for (request, verb) in cases {
        assert_eq!(request.signal_verb(), verb);
    }
}

#[test]
fn system_operation_kind_round_trips_through_nota_text() {
    round_trip_nota(SystemOperationKind::FocusSubscription, "FocusSubscription");
    round_trip_nota(
        SystemOperationKind::FocusUnsubscription,
        "FocusUnsubscription",
    );
    round_trip_nota(SystemOperationKind::FocusSnapshot, "FocusSnapshot");
    round_trip_nota(SystemOperationKind::SystemStatusQuery, "SystemStatusQuery");
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

#[test]
fn focus_observation_event_round_trips_through_nota_text() {
    round_trip_nota(
        SystemEvent::FocusObservation(FocusObservation {
            target: TARGET,
            focused: true,
            generation: ObservationGeneration::new(42),
        }),
        "(FocusObservation (NiriWindow 223) true 42)",
    );
}

#[test]
fn window_closed_round_trips() {
    let event = SystemEvent::WindowClosed(WindowClosed { target: TARGET });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn window_closed_event_round_trips_through_nota_text() {
    round_trip_nota(
        SystemEvent::WindowClosed(WindowClosed { target: TARGET }),
        "(WindowClosed (NiriWindow 223))",
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

#[test]
fn subscription_accepted_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SubscriptionAccepted(SubscriptionAccepted {
            target: TARGET,
            kind: SubscriptionKind::Focus,
        }),
        "(SubscriptionAccepted (NiriWindow 223) Focus)",
    );
}

#[test]
fn observation_target_missing_round_trips() {
    let reply = SystemReply::ObservationTargetMissing(ObservationTargetMissing { target: TARGET });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn observation_target_missing_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::ObservationTargetMissing(ObservationTargetMissing { target: TARGET }),
        "(ObservationTargetMissing (NiriWindow 223))",
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

#[test]
fn system_status_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SystemStatus(SystemStatus {
            backend: SystemBackend::Niri,
            health: SystemHealth::Running,
            readiness: SystemReadiness::Ready,
        }),
        "(SystemStatus Niri Running Ready)",
    );
}

#[test]
fn system_request_unimplemented_reply_round_trips() {
    let reply = SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
        operation: SystemOperationKind::FocusSubscription,
        reason: SystemUnimplementedReason::NotBuiltYet,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn focus_snapshot_reply_round_trips() {
    let reply = SystemReply::FocusSnapshotReply(FocusObservation {
        target: TARGET,
        focused: true,
        generation: ObservationGeneration::new(44),
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn focus_snapshot_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::FocusSnapshotReply(FocusObservation {
            target: TARGET,
            focused: true,
            generation: ObservationGeneration::new(44),
        }),
        "(FocusObservation (NiriWindow 223) true 44)",
    );
}

#[test]
fn system_request_unimplemented_reply_round_trips_through_nota_text() {
    round_trip_nota(
        SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
            operation: SystemOperationKind::FocusSubscription,
            reason: SystemUnimplementedReason::NotBuiltYet,
        }),
        "(SystemRequestUnimplemented FocusSubscription NotBuiltYet)",
    );
}

#[test]
fn explicit_variant_lifts_focus_subscription_into_request() {
    let payload = FocusSubscription { target: TARGET };
    let request = SystemRequest::FocusSubscription(payload.clone());
    assert_eq!(request, SystemRequest::FocusSubscription(payload));
}

#[test]
fn explicit_variant_lifts_system_status_query_into_request() {
    let payload = SystemStatusQuery {
        backend: SystemBackend::Niri,
    };
    let request = SystemRequest::SystemStatusQuery(payload);
    assert_eq!(request, SystemRequest::SystemStatusQuery(payload));
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
            "terminal prompt-gate records belong to signal-persona-terminal:\n{}",
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
