use meta_signal_system::SystemBackend;
use signal_system::{
    ByteViewable, FocusObservation, FocusSnapshot, FocusSubscription, FocusSubscriptionToken,
    ObservationTargetMissing, Query, Response, Restorable, Signal, Signalizable,
    SubscriptionAccepted, SubscriptionKind, SubscriptionRetracted, SystemHealth,
    SystemOperationKind, SystemReadiness, SystemRequestUnimplemented, SystemStatus,
    SystemStatusQuery, SystemStreamEvent, SystemTarget, SystemUnimplementedReason, WindowClosed,
};

fn token() -> FocusSubscriptionToken {
    FocusSubscriptionToken {
        system_target: SystemTarget::NiriWindow(223),
    }
}

fn queries() -> Vec<Query> {
    vec![
        Query::WatchFocus(FocusSubscription {
            system_target: SystemTarget::NiriWindow(223),
        }),
        Query::UnwatchFocus(token()),
        Query::QueryFocus(FocusSnapshot {
            system_target: SystemTarget::NiriWindow(223),
        }),
        Query::QueryStatus(SystemStatusQuery {
            system_backend: SystemBackend::Niri,
        }),
    ]
}

fn responses() -> Vec<Response> {
    vec![
        Response::SubscriptionAccepted(SubscriptionAccepted {
            system_target: SystemTarget::NiriWindow(223),
            subscription_kind: SubscriptionKind::Focus,
        }),
        Response::SubscriptionRetracted(SubscriptionRetracted {
            focus_subscription_token: token(),
        }),
        Response::ObservationTargetMissing(ObservationTargetMissing {
            system_target: SystemTarget::NiriWindow(999),
        }),
        Response::SystemStatus(SystemStatus {
            system_backend: SystemBackend::Niri,
            system_health: SystemHealth::Running,
            system_readiness: SystemReadiness::Ready,
        }),
        Response::SystemRequestUnimplemented(SystemRequestUnimplemented {
            system_operation_kind: SystemOperationKind::QueryFocus,
            system_unimplemented_reason: SystemUnimplementedReason::NotBuiltYet,
        }),
        Response::FocusObservation(FocusObservation {
            system_target: SystemTarget::NiriWindow(223),
            focused: true,
            observation_generation: 12,
        }),
    ]
}

fn stream_events() -> Vec<SystemStreamEvent> {
    vec![
        SystemStreamEvent::FocusObservation(FocusObservation {
            system_target: SystemTarget::NiriWindow(223),
            focused: true,
            observation_generation: 12,
        }),
        SystemStreamEvent::WindowClosed(WindowClosed {
            system_target: SystemTarget::NiriWindow(223),
        }),
    ]
}

#[test]
fn queries_round_trip_through_received_bytes() {
    for query in queries() {
        let received =
            Signal::<Query>::from(query.signalize().expect("query archives").bytes().to_vec());
        assert_eq!(received.restore().expect("query restores"), query);
    }
}

#[test]
fn responses_round_trip_through_received_bytes() {
    for response in responses() {
        let received = Signal::<Response>::from(
            response
                .signalize()
                .expect("response archives")
                .bytes()
                .to_vec(),
        );
        assert_eq!(received.restore().expect("response restores"), response);
    }
}

#[test]
fn stream_events_round_trip_through_received_bytes() {
    for event in stream_events() {
        let received = Signal::<SystemStreamEvent>::from(
            event.signalize().expect("event archives").bytes().to_vec(),
        );
        assert_eq!(received.restore().expect("event restores"), event);
    }
}

#[test]
fn malformed_archive_is_rejected() {
    assert!(Signal::<Query>::from(vec![1, 2, 3]).restore().is_err());
}

#[cfg(feature = "datom")]
fn budget() -> datom_codec::Budget {
    datom_codec::Budget {
        remaining: 8192,
        reader: protos::ReaderBudget { remaining: 8192 },
        depth: 0,
        maximum_depth: 1024,
    }
}

#[cfg(feature = "datom")]
#[test]
fn queries_round_trip_as_datom_text() {
    use datom_codec::{Actualizing, Datomizable, Potential};
    use protos::{Protosizable, Textualizable};

    for query in queries() {
        let text = query.clone().datomize(vec![]).protosize().textualize();
        let restored = Potential::<Query>::from(text)
            .actualize(&mut budget())
            .expect("Datom restores");
        assert_eq!(restored, query);
    }
}

#[cfg(feature = "datom")]
#[test]
fn every_canonical_datom_line_actualizes_into_a_contract_head() {
    use datom_codec::{Actualizing, Potential};

    let canonical = include_str!("../examples/canonical.datom");
    let mut lines = 0;
    for line in canonical.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        lines += 1;
        let as_query = Potential::<Query>::from(line.to_owned())
            .actualize(&mut budget())
            .is_ok();
        let as_response = Potential::<Response>::from(line.to_owned())
            .actualize(&mut budget())
            .is_ok();
        let as_event = Potential::<SystemStreamEvent>::from(line.to_owned())
            .actualize(&mut budget())
            .is_ok();
        assert!(
            as_query || as_response || as_event,
            "canonical line is no contract head: {line}"
        );
    }
    assert_eq!(
        lines, 11,
        "canonical file should carry eleven contract heads"
    );
}
