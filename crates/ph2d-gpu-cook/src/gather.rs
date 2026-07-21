//! The **identity** questions a dispatch asks — split from `lib.rs` at the HR-18
//! LOC cap, and a coherent seam rather than a slice: which port keys an id
//! gather, whether a column is readable at this dispatch length, and how long
//! the prior state was. All three exist because ADR-0130 decoupled a state
//! port's length from the dispatch's.

use crate::stream::GpuStream;
use ph2d_nodegraph::gpu::ColumnBinding;

/// The base port of an ACTIVE id-gather (ADR-0130) for this input set, or `None`
/// when there is none: the kernel declares a [`ColumnBinding`] whose access
/// [`is_gather_key`](ph2d_nodegraph::gpu::ColumnAccess::is_gather_key) AND that
/// port's stream carries the key column at the dispatch length (a dense window —
/// the plan already refused a non-dense one). The state ports (any other) are
/// then paired by id rather than by position, so their length is decoupled from
/// the dispatch.
pub(crate) fn gather_key_port(
    bindings: &[ColumnBinding],
    inputs: &[GpuStream],
    count: u32,
) -> Option<usize> {
    let key = bindings.iter().find(|b| b.access.is_gather_key())?;
    let active = inputs
        .get(key.port)
        .is_some_and(|s| s.count == count && s.cols.contains_key(key.column));
    active.then_some(key.port)
}

/// Is `b`'s column readable off its port for a `count`-element dispatch? A
/// column is "present" only if its port carries it AND is a length this dispatch
/// can PAIR with:
///
/// - **Positional** (the default): the port must be the DISPATCH length. A state
///   whose count no longer matches the live set was rebuilt, so pair nothing and
///   re-seed — the CPU's `pairing` `_ if sn == n` arm, expressed once.
/// - **Under an active id-gather** (`gather_port`): the STATE ports (any but the
///   key's) are paired by id, not position, so `prev_n ≠ n` is NORMAL. Presence
///   is simply "carries the column" (any non-empty length) — the length-decouple
///   ADR-0130 D4 names. The gather's OWN base-port columns stay dispatch-length.
/// - **Broadcast** ([`ph2d_nodegraph::gpu::ColumnAccess::ReadBroadcast`]): the
///   dispatch length OR
///   exactly one, which element `i` then reads row `0` of. A length-1 port judged
///   ABSENT here would read its identity, which is the difference between a flock
///   facing the point the artist animated and a flock facing the origin.
pub(crate) fn column_present(
    gather_port: Option<usize>,
    count: u32,
    inputs: &[GpuStream],
    b: &ColumnBinding,
) -> bool {
    match inputs.get(b.port) {
        None => false,
        Some(s) if gather_port.is_some_and(|kp| b.port != kp) => {
            s.count > 0 && s.cols.contains_key(b.column)
        }
        Some(s) if b.access.broadcasts() => {
            (s.count == count || s.count == 1) && s.cols.contains_key(b.column)
        }
        Some(s) => s.count == count && s.cols.contains_key(b.column),
    }
}

/// The prior state's element count (`prev_n`) an active gather reads through the
/// `gather_prev_n` uniform: the length of the STATE port (the first input that is
/// not the gather's base). `0` at tick 0 (the `pre` is Empty), where nothing
/// pairs and every element seeds.
pub(crate) fn gather_prev_n(inputs: &[GpuStream], key_port: usize) -> u32 {
    inputs
        .iter()
        .enumerate()
        .find(|(p, _)| *p != key_port)
        .map(|(_, s)| s.count)
        .unwrap_or(0)
}

/// A broadcast port the dispatch cannot PAIR — `Some((port, len))` when a
/// [`ph2d_nodegraph::gpu::ColumnAccess::ReadBroadcast`] binding's port carries
/// the column at a length that is neither the dispatch length, nor exactly one,
/// nor empty.
///
/// [`column_present`] would judge such a port ABSENT, so the kernel would read
/// the declared identity at EVERY index — while the CPU (`target_at`'s `_` arm)
/// reads the real rows it has and only falls back past them. Same document, two
/// different fields: a SHAPE divergence, not an ε. The plan cannot refuse it
/// (lengths are a cook-time fact, `applicable` only sees params), so the cook
/// does — the caller falls back to the CPU, which is canonical (the same door
/// as `TooManyBindings`).
///
/// `lookup` answers `(port element count, does the port carry the column?)` for
/// a binding — a closure so the decision is testable without a device (a
/// `GpuStream` column cannot be built without one). Ports an active id-gather
/// length-decouples (any but the key's) are exempt, mirroring the precedence
/// [`column_present`] gives them.
pub(crate) fn broadcast_length_mismatch(
    gather_port: Option<usize>,
    count: u32,
    bindings: &[ColumnBinding],
    lookup: impl Fn(&ColumnBinding) -> Option<(u32, bool)>,
) -> Option<(usize, u32)> {
    bindings.iter().find_map(|b| {
        if !b.access.broadcasts() {
            return None;
        }
        if gather_port.is_some_and(|kp| b.port != kp) {
            return None;
        }
        let (len, carries) = lookup(b)?;
        let bad = carries && len != 0 && len != 1 && len != count;
        bad.then_some((b.port, len))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::gpu::ColumnAccess;
    use ph2d_nodegraph::port::Dim;

    fn bcast(port: usize) -> ColumnBinding {
        ColumnBinding {
            column: "v",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port,
        }
    }

    fn plain_read(port: usize) -> ColumnBinding {
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port,
        }
    }

    /// The divergent case the check exists for: a broadcast port carrying the
    /// column at a length the dispatch can neither pair per-element nor pin to
    /// row 0. Length 3 against a dispatch of 5 — the CPU serves rows 0..3 and
    /// falls back past them; the GPU would have served the identity everywhere.
    #[test]
    fn a_mixed_length_broadcast_port_is_a_mismatch() {
        let bindings = [plain_read(0), bcast(1)];
        let got = broadcast_length_mismatch(None, 5, &bindings, |b| match b.port {
            1 => Some((3, true)),
            _ => Some((5, true)),
        });
        assert_eq!(got, Some((1, 3)), "3 rows against a 5-wide dispatch");
    }

    /// The three lengths broadcast CAN pair — dispatch-length (per element),
    /// exactly one (row 0 pinned) and empty (identity, the CPU's `0 =>` arm) —
    /// and an absent column, must all pass.
    #[test]
    fn pairable_lengths_and_absence_are_not_mismatches() {
        let bindings = [bcast(1)];
        for (len, carries) in [(5, true), (1, true), (0, true), (3, false)] {
            assert_eq!(
                broadcast_length_mismatch(None, 5, &bindings, |_| Some((len, carries))),
                None,
                "len {len}, carries {carries}"
            );
        }
    }

    /// Only `ReadBroadcast` bindings are judged: a plain `Read` port at a
    /// mismatched length is `column_present`'s ordinary absent-column fallback,
    /// which both sides already agree on.
    #[test]
    fn a_plain_read_binding_is_never_a_mismatch() {
        let bindings = [plain_read(0)];
        assert_eq!(
            broadcast_length_mismatch(None, 5, &bindings, |_| Some((3, true))),
            None
        );
    }

    /// Under an active id-gather the non-key ports are length-decoupled (paired
    /// by id, ADR-0130 D4) — the same precedence `column_present` gives them, so
    /// the mismatch check must not fire there.
    #[test]
    fn a_gather_decoupled_port_is_exempt() {
        let bindings = [bcast(1)];
        assert_eq!(
            broadcast_length_mismatch(Some(0), 5, &bindings, |_| Some((3, true))),
            None,
            "port 1 is not the gather key's port 0 - length-decoupled"
        );
        assert_eq!(
            broadcast_length_mismatch(Some(1), 5, &bindings, |_| Some((3, true))),
            Some((1, 3)),
            "the key's own port keeps the dispatch-length rule"
        );
    }
}
