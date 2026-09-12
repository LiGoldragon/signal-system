# Upgrades

## 3.0.0 — portable Signal frame un-forked, current producer front

`signal-system` no longer declares its own `Signal<T>` frame or its
`Signalizable`, `ByteViewable`, `Restorable` traits. It now re-exports all
four from the new `signal` 5.0.0 crate, which owns the one portable rkyv
Signal frame shared across the estate — a second local copy was a different
Rust type forking the wire. Every consumer that named
`signal_system::Signal` (or the three traits) as a type still gets the same
paths through the re-export, but the concrete type identity changed: a
`Signal<T>` built against the old local definition is not the same type as
one built against `signal` 5.0.0, so anything that pinned an older
`signal-system` alongside a fresh one, or cached a build artifact, must
rebuild against 3.0.0 uniformly.

Repinned to `datom-codec` 0.27.0, `ethos-zero` 9.0.0 (build-dependency
only), `meta-signal-system` 3.0.0, and added `signal` 5.0.0 as a direct
dependency (`default-features = false`; enabled by this crate's own `datom`
feature via `signal/datom`). `protos` is unchanged at 0.30.1
(rev `171b21f65337983ab624b7b906397a4f1f92c5a3`).

`datom-codec` 0.27.0 renames the kind a datom composes into from
`Compositional` to `Composing` (ethos-zero 9.0.0 emits
`datom_codec::Composing` where 8.0.1 emitted `datom_codec::Compositional`).
`src/generated/signal.rs` is regenerated against the new ethos-zero output;
any consumer binding `datom_codec::Compositional` on a type from this crate
must move to `datom_codec::Composing`. No compatibility path is kept for the
old derive name.

Consumers: bump to `signal-system` 3.0.0, and if you also depend on `signal`
directly, pin it to
`{ git = "https://github.com/LiGoldragon/signal", rev = "7bcb0949cdaa4501651ed519625eba48e283fac0" }`
(no `.git` suffix) so cargo resolves a single copy — `signal` declares
`links = "signal"`, so a differing URL spelling for the same crate fails at
resolution rather than silently duplicating it.
