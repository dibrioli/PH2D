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

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, Transform, World};

use crate::binding::TargetBinding;

/// **Where along the path the authored pose sits** — a Position binding's `rest`.
///
/// The base a partially-covering clip lane fades in FROM (ADR-0115 R5), and it has
/// to be in the *track's* units, which for this kind is distance. Reading a
/// coordinate here, or defaulting to `0.0`, would put the base at the START of the
/// trajectory and hurl the sprite there on the first frame of a fade — the very
/// failure `rest` was introduced to prevent, one channel over.
///
/// `None` while the binding has no path yet (a Position track before its first key),
/// which leaves `rest` uncaptured and lets the next frame try again — exactly what
/// an unresolvable scalar does.
pub(crate) fn read_rest(world: &World, entity: Entity, b: &TargetBinding) -> Option<f32> {
    let path = b.path.as_ref()?;
    let xf = world.get::<Transform>(entity)?;
    let s = path.project([xf.translation.x, xf.translation.y])?;
    Some(s as f32)
}

/// Write a sampled **distance** into the entity, by asking the trajectory where that
/// distance is.
///
/// A binding with no path writes nothing — the same silence a track with no keys
/// gets, and for the same reason: there is no answer yet, and a default would move
/// the object somewhere nobody authored.
pub(crate) fn write_position(
    world: &mut World,
    entity: Entity,
    b: &TargetBinding,
    v: AnimValue,
    orient: bool,
) {
    let (Some(path), AnimValue::Float(s)) = (b.path.as_ref(), v) else {
        return;
    };
    let Some(sample) = path.at(f64::from(s)) else {
        return;
    };
    if let Some(mut xf) = world.get_mut::<Transform>(entity) {
        xf.translation.x = sample.point[0];
        xf.translation.y = sample.point[1];
        // **Auto-orient** (ADR-0141 §6): o objeto encara a tangente do caminho.
        //
        // ⚠️ **Numa CÚSPIDE não se escreve nada, e é assim que o ângulo se SEGURA.**
        // `tangent_at` devolve `None` onde a velocidade da curva é zero — ali não há
        // direção, e inventar uma produz o pico solto. Não escrever deixa a `rotation`
        // exatamente como estava, que É "segurar o último ângulo válido", **sem estado
        // nenhum**: nada a guardar, nada a invalidar num scrub, nada que discorde entre
        // um replay e uma reprodução ao vivo.
        //
        // ⚠️ E este canal não tem o bug publicado do AE (*"flips when stopping
        // motion"*) por CONSTRUÇÃO, não por cuidado: lá o ângulo vem do vetor
        // VELOCIDADE, que desaparece quando o objeto para; aqui vem da GEOMETRIA da
        // curva, que continua lá com o objeto parado em cima dela.
        if orient && let Some(t) = sample.tangent {
            xf.rotation = libm::atan2f(t[1], t[0]);
        }
    }
}
