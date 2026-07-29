//! The **Position resolver** — the half of [`crate::apply`] that knows a track's
//! value can be a distance rather than a coordinate ([ADR-0141]).
//!
//! Every other property is a number that goes into a field. Position is a number
//! that goes into a *trajectory*, which then answers with a point — so it needs the
//! binding (where the path lives) and not just the [`crate::PropKind`]. It lives in
//! its own module for exactly that reason: `apply.rs` resolves scalars, this
//! resolves the one channel that is a curve.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_ecs::{Entity, Transform, World};

use crate::binding::TargetBinding;

/// **Where along the path the object currently sits** — a Position binding read back
/// out of the world, in the TRACK's units.
///
/// ⚠️ It was called `read_rest` while `rest` capture was its only caller. It is now
/// the Position arm of [`crate::apply::read_prop`], so it also answers a
/// `Nome.position` prop-link — and a name that describes one of two callers is a
/// comment that lies (ADR-0146 C4). What it computes never changed.
///
/// For rest capture it is the base a partially-covering clip lane fades in FROM
/// (ADR-0115 R5), and it has to be in the *track's* units, which for this kind is
/// distance. Reading a coordinate here, or defaulting to `0.0`, would put the base at
/// the START of the trajectory and hurl the sprite there on the first frame of a fade
/// — the very failure `rest` was introduced to prevent, one channel over.
///
/// `None` while the binding has no path yet (a Position track before its first key),
/// which leaves `rest` uncaptured and lets the next frame try again — exactly what
/// an unresolvable scalar does.
pub(crate) fn read_distance(world: &World, entity: Entity, b: &TargetBinding) -> Option<f32> {
    let path = b.path.as_ref()?;
    let xf = world.get::<Transform>(entity)?;
    let s = path.project([xf.translation.x, xf.translation.y])?;
    Some(s as f32)
}

// A escrita da distância no `Transform` (Position → ponto + auto-orient) mudou-se para
// `crate::pose::set_transform_field`, a porta única que o apply e o `pose_at` do onion
// compartilham (ADR-0142). Aqui fica só o `read_rest` (a captura da distância autorada).
