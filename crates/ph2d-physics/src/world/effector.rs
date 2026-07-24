//! **Force zones** — an area that pushes whatever is inside it (ADR-0131 W-Area).
//!
//! Wind, an updraft, a conveyor, a current: a sensor collider carrying a force
//! vector (and, since W-AreaTorque, a rotational `torque` — a whirlpool, a
//! turntable), applied each substep to every dynamic body overlapping it. Unity's
//! `AreaEffector2D` and Godot's `Area2D` overrides are the same shape.
//!
//! ## A force, never an acceleration — and why that decides the model
//!
//! The impulse `F·dt` is resisted by the body's mass, so a leaf is carried by a
//! wind a crate barely feels. That asymmetry IS the feature: it is the intuition
//! every artist brings, and it is the half of "make this thing move" that cannot
//! be authored per body today. An *acceleration* zone would be mass-independent,
//! and it would also be a second answer to a question `gravity_scale` (W8) already
//! answers per body — two doors to one quantity, the recurring bug of this line.
//!
//! ## The sensor coupling
//!
//! A force zone must be a **sensor**. Two independent reasons, and they agree:
//! the narrow phase only records an *intersection* when one side is a sensor (a
//! solid pair produces a contact instead), so a solid zone would see nobody; and
//! a solid collider pushes bodies OUT rather than letting them in, so an area you
//! cannot enter is not an area. [`zone_force`] is the single door that answers
//! "is this body a force zone" — the world registers through it and the §11 rows
//! are offered under the same condition, so the two cannot drift.
//!
//! ## Impulse, and awake
//!
//! `apply_impulse`, never `add_force` — the lesson `drag` paid for: rapier's
//! `add_force` persists until `reset_forces`, which the pipeline never calls, so
//! it accumulates over every substep of every tick.
//!
//! ⚠️ **The impulse WAKES the body** (unlike drag, which passes `false`). A zone
//! that cannot start a body already resting on the floor is broken in its main
//! use — the updraft under a crate, the conveyor under a box. The price is that a
//! body inside an active zone does not sleep, which is honest: it is being pushed.
//! A zone with a ZERO force is never registered at all, precisely so it cannot
//! wake anything — that is what keeps an inert zone byte-identical (there is a
//! gate on it).

use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::geometry::{ColliderSet, NarrowPhase};
use rapier2d::na::{Isometry2, Point2, Vector2};

use super::buoyancy;
use super::desc::{AreaEffect, BodyDesc};
use super::form_drag;
use super::shape::ShapeDesc;

/// **The authored force, in world axes** — the single door for the zone's FRAME
/// (W-AreaFrame).
///
/// A zone's force is authored in the zone's own frame by default, so **rotating the
/// sensor rotates the wind** (a diagonal conveyor is a conveyor you turned). Pass
/// `world_axes` to pin the direction to the world instead — the zone turns, the blow
/// does not (Unity's `AreaEffector2D::useGlobalAngle`).
///
/// ⚠️ **It takes `(sin, cos)`, never an angle, and that is a determinism decision.**
/// rapier stores a body's rotation as a `UnitComplex`, whose `im`/`re` ARE the sine and
/// cosine — already exact. Asking it for `.angle()` would call `atan2`, and `atan2` is
/// std's, not `libm`'s: it is not pinned across OSes, and this result feeds impulses that
/// feed the `physics_ecs_c9` hash the CI compares between Linux, macOS and Windows (law 6
/// — 1 ulp is a cross-OS bug). Callers holding an ANGLE use [`zone_force_world_at`], which
/// crosses that bridge through `libm` exactly once.
///
/// ⚠️ **Byte-identical for an unrotated zone in BOTH modes:** `sin 0` is exactly `0.0` and
/// `cos 0` exactly `1.0`, so the rotated branch reduces to the identity on the f32 bits.
/// That is what let the default change without moving a single existing scene.
///
/// This is the door the SOLVER and the OVERLAY both ask, for the reason `scaled_shape`
/// exists: an arrow drawn from a second answer would describe a wind that does not blow
/// there, and nobody reads a number off a screenshot to catch it.
///
/// ⚠️ **O `mirror` entra ANTES da rotação, e a ordem é a do `Transform`.** A pose de mundo
/// compõe como `R · S` (a escala age no espaço local, a rotação depois), então um vetor
/// local vira `R · S · v` — espelhar e só então rodar. Trocar a ordem dá a reflexão sobre
/// o eixo errado em toda zona que seja espelhada *e* rotacionada, que é o caso de uso
/// inteiro (a metade espelhada de um nível).
///
/// ⚠️ **`world_axes` desliga o espelho junto com a rotação, e isso é o significado do
/// toggle:** ele diz *"esta força é um vetor de MUNDO"*, e um vetor de mundo não é tocado
/// por nada que o frame da zona faça. Sair antes das duas é a leitura literal disso.
#[must_use]
pub fn zone_force_world(
    force: [f32; 2],
    world_axes: bool,
    sin_r: f32,
    cos_r: f32,
    mirror: [f32; 2],
) -> [f32; 2] {
    if world_axes {
        return force;
    }
    let (fx, fy) = (force[0] * mirror[0], force[1] * mirror[1]);
    [fx * cos_r - fy * sin_r, fx * sin_r + fy * cos_r]
}

/// **O fator de handedness do torque** — `+1` num frame normal, `−1` num espelhado
/// (W-AreaMirror). É `det(S)`, e é a única coisa que uma reflexão faz a um pseudoescalar.
///
/// Porta própria porque a pergunta é UMA (*este frame trocou de mão?*) e os dois
/// consumidores que a fazem — o solver e o glifo violeta do overlay — divergiriam calados:
/// ninguém lê o sentido de um giro num screenshot, exatamente como ninguém lê a direção de
/// uma seta ([`zone_force_world`] existe pelo mesmo motivo).
///
/// ⚠️ Espelhar os DOIS eixos **não** é uma reflexão — é uma rotação de 180°, e o produto
/// devolve `+1` sozinho. A forma fechada apaga o caso especial que uma paridade
/// por-eixo teria de enumerar.
#[must_use]
pub fn zone_spin_sign(mirror: [f32; 2]) -> f32 {
    mirror[0] * mirror[1]
}

/// [`zone_force_world`] for a caller that holds the rotation as an ANGLE (radians) —
/// the ECS side, whose `Transform.rotation` is one.
///
/// `libm::sincosf`, never `f32::sin_cos`: the module's transcendentals are pinned (law 6),
/// and this is the one place an angle becomes a pair. The rule itself lives in
/// [`zone_force_world`] — this only crosses the bridge to it.
#[must_use]
pub fn zone_force_world_at(
    force: [f32; 2],
    world_axes: bool,
    rotation: f32,
    mirror: [f32; 2],
) -> [f32; 2] {
    let (sin_r, cos_r) = libm::sincosf(rotation);
    zone_force_world(force, world_axes, sin_r, cos_r, mirror)
}

/// **Quanto desta zona chega a um corpo neste ponto** — a porta única do falloff
/// (W-AreaFalloff). `1.0` no centro, `1 − falloff` na borda, e nunca menos que zero.
///
/// `zone_iso` é a pose de MUNDO do collider da zona (a do collider, não a do corpo, para
/// que uma zona com offset meça a partir de onde a forma de fato está) e `point` é o
/// ponto do mundo perguntado — o centro do corpo empurrado.
///
/// ⚠️ **Capado em `t ≤ 1`, e o cap é load-bearing:** a sobreposição que registra o par é
/// forma-contra-forma, então o CENTRO de um corpo grande pode estar do lado de fora
/// enquanto ele ainda encosta. Sem o cap, `1 − falloff·t` fica negativo e a zona passa a
/// **puxar para trás** exatamente na borda onde deveria soltar — um sinal invertido que
/// nenhum ledger acusa porque a soma continua fechando.
///
/// ⚠️ **`falloff == 0` devolve `1.0` e isso é byte-neutro por DUAS vias**: o chamador nem
/// entra aqui (o early-out compra o CUSTO — nada de transformação inversa nem `sqrt` numa
/// cena que não pediu falloff), e multiplicar por `1.0` exato é a identidade em IEEE-754
/// (isso compra a IDENTIDADE, e é o que torna a cena antiga bit a bit a mesma). São duas
/// camadas, cada uma comprando uma coisa diferente
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[must_use]
pub fn zone_falloff_scale(
    shape: ShapeDesc,
    zone_iso: &Isometry2<f32>,
    point: Vector2<f32>,
    falloff: f32,
) -> f32 {
    if falloff <= 0.0 {
        return 1.0;
    }
    let local = zone_iso.inverse_transform_point(&Point2::from(point));
    let t = shape.radial_fraction([local.x, local.y]).min(1.0);
    (1.0 - falloff * t).max(0.0)
}

/// **Is this body a force zone, and with what effect?** The single door.
///
/// `None` for a body with no effector, for a zero force, and for a **solid**
/// collider.
///
/// ⚠️ **Neither refusal is load-bearing for the simulation, and both were measured
/// to find that out.** rapier's own `apply_impulse` opens with `if !impulse.is_zero()`,
/// so a zero force could not wake anything even if it were registered; and the
/// narrow phase records an *intersection* only when one side is a sensor, so a solid
/// zone would see nobody. Deleting either line leaves every wrapper gate green, and
/// that is expected — the same shape as `drag`'s early-out and the impasto light's
/// flat-paint branch ([[feedback_layered_defenses_need_per_layer_gates]]: ask what
/// each layer protects ALONE, and be willing to answer "nothing").
///
/// What they DO buy: an inert zone never enters `effectors`, so the substep walk is
/// skipped entirely rather than iterated to no effect; and the sensor half is the
/// single door the §11 rows mirror — the panel offers Force only for a sensor, and
/// its seam gate is where that rule IS observable.
#[must_use]
pub(crate) fn zone_effect(desc: &BodyDesc) -> Option<AreaEffect> {
    let e = desc.effector?;
    // ⚠️ `falloff` is deliberately NOT in this list: it is a MODIFIER, not an effect. A
    // zone carrying nothing but a falloff attenuates nothing, so it is exactly as inert
    // as one carrying nothing at all — and registering it would cost the substep walk and
    // wake bodies for a push of zero (the same reasoning as the two refusals below).
    let inert = e.force[0] == 0.0
        && e.force[1] == 0.0
        && e.torque == 0.0
        && e.drag <= 0.0
        && e.density <= 0.0
        && e.form_drag <= 0.0;
    if !desc.is_sensor || inert {
        return None;
    }
    Some(e)
}

/// Apply one substep of every registered zone's force to the dynamic bodies
/// overlapping it.
///
/// Called from inside [`super::PhysicsWorld::step`]'s substep loop, beside
/// `drag::apply` and for the same reason: the integrator sees smaller slices, and
/// an impulse is exactly "this force, for this slice of time".
///
/// ⚠️ The overlap read is the narrow phase's state from the PREVIOUS pipeline
/// step, so a body entering a zone is pushed one substep later than it geometrically
/// arrives. That is inherent to reading a solver's own broad/narrow phase rather
/// than re-deriving overlap here (which would be a second, disagreeing answer to
/// "who is inside"), and at 240 Hz of substeps it is not a visible delay.
///
/// `zones` is kept sorted by handle by the world, so a body standing in TWO
/// overlapping zones accumulates their impulses in a fixed order — float addition
/// is not associative, and the ORDER of a sum is exactly the kind of thing that
/// makes a cross-OS hash drift (HR-5). Within one zone the order cannot matter:
/// each overlapping body is a distinct entry, so each gets exactly one impulse.
pub(crate) fn apply(
    bodies: &mut RigidBodySet,
    colliders: &ColliderSet,
    narrow_phase: &NarrowPhase,
    zones: &[(RigidBodyHandle, AreaEffect, ShapeDesc)],
    gravity: Vector2<f32>,
    dt: f32,
) {
    // Fast path AND contract: a world with no zone never touches a body, so it is
    // byte-identical to one built before this module existed.
    if zones.is_empty() {
        return;
    }
    let mut to_float: Vec<RigidBodyHandle> = Vec::new();
    for &(zone_body, effect, zone_shape) in zones {
        to_float.clear();
        // The zone's own collider handle AND its pose, copied out so the immutable
        // borrow of `bodies` ends before the impulses below need it mutably.
        //
        // ⚠️ The rotation is the LIVE one, read here every substep, not the one baked
        // into the spawn `BodyDesc`. For the common static zone the two are the same
        // number; they part company for a KINEMATIC zone a curve is turning — a fan
        // sweeping the room — and there the live read is the whole behaviour: the blow
        // sweeps with the housing. Baking it at spawn would forbid that in silence.
        //
        // `rot.im`/`rot.re` are the sine and cosine (rapier keeps rotations as a unit
        // complex), handed to the door exactly as they are — see `zone_force_world` for
        // why an angle must not appear on this path.
        let Some((zone_collider, sin_r, cos_r)) = bodies.get(zone_body).and_then(|b| {
            let rot = *b.rotation();
            b.colliders().first().map(|&c| (c, rot.im, rot.re))
        }) else {
            continue;
        };
        // The authored push, turned into world axes by the zone's own frame (or left
        // alone, when the artist pinned it to the world).
        let f = zone_force_world(effect.force, effect.world_axes, sin_r, cos_r, effect.mirror);
        // O sentido do giro num frame espelhado (pseudoescalar — ver `AreaEffect::mirror`).
        let spin = zone_spin_sign(effect.mirror);
        let force = Vector2::new(f[0], f[1]);
        for (c1, c2, intersecting) in narrow_phase.intersection_pairs_with(zone_collider) {
            // `intersecting` is the real shape overlap; the pair exists as soon as
            // the bounding volumes touch, which is a bigger region than the area.
            if !intersecting {
                continue;
            }
            let other = if c1 == zone_collider { c2 } else { c1 };
            // ONE body, ONE collider: `spawn_body` inserts exactly one, so "the
            // other collider of the pair" is always a different BODY, and there is
            // no self-push to guard against. A guard for it was written, no gate
            // could reach it, and code no gate can reach is a claim that the model
            // is looser than it is. If a multi-collider body ever arrives, the
            // guard comes back — with a fixture that contains the case.
            let Some(parent) = colliders.get(other).and_then(|c| c.parent()) else {
                continue;
            };
            let Some(b) = bodies.get_mut(parent) else {
                continue;
            };
            // Only a dynamic body has a velocity the solver owns. rapier would
            // ignore the impulse on the others anyway, but asking here keeps the
            // wake-up (which is NOT a no-op) off a body that cannot move.
            if !b.is_dynamic() {
                continue;
            }
            // **Quanto desta zona chega AQUI** (W-AreaFalloff): 1 sem falloff (e então o
            // early-out da porta nem calcula nada), menos que 1 conforme o corpo se afasta
            // do centro. Perguntado por corpo porque é uma função da POSIÇÃO dele, e
            // contra a pose do COLLIDER da zona (não a do corpo dela), que é onde a forma
            // de fato está quando há offset.
            let scale = if effect.falloff > 0.0 {
                let here = *b.translation();
                colliders.get(zone_collider).map_or(1.0, |c| {
                    zone_falloff_scale(zone_shape, c.position(), here, effect.falloff)
                })
            } else {
                1.0
            };
            b.apply_impulse(force * dt * scale, true);
            // The rotational push — a whirlpool, a turntable. `apply_torque_impulse`
            // (NOT set_angular_velocity) so it is resisted by the moment of inertia,
            // the exact mirror of the force above being resisted by mass: a long bar
            // spins up less than a compact one of the same area, which is the feature.
            // Guarded on non-zero so a pure-force / pure-drag zone never wakes a body
            // for a torque it does not carry (`zone_effect` already refuses the wholly
            // inert zone; this is the per-body echo of that).
            // O falloff pesa o giro pelo MESMO fator, e essa é a metade que a nota aberta
            // do W-AreaTorque pedia: um redemoinho real perde força longe do olho. Os dois
            // são EMPURRÕES da zona; o arrasto e o empuxo abaixo não são, e por isso não
            // recebem o fator (há gate nessa fronteira).
            if effect.torque != 0.0 {
                b.apply_torque_impulse(effect.torque * spin * dt * scale, true);
            }
            // The medium's resistance. ⚠️ Written as `v /= 1 + d·dt`, which is
            // *exactly* what rapier's own `linear_damping` integrator does — so the
            // word "drag" means one thing across the world default, the per-body
            // override and this. Applying it as another impulse would be a second law
            // for the same word, and the two would disagree at large `d` (an impulse
            // can overshoot through zero and push the body BACKWARDS; this cannot).
            if effect.drag > 0.0 {
                let k = 1.0 / (1.0 + effect.drag * dt);
                let v = *b.linvel();
                let w = b.angvel();
                b.set_linvel(v * k, true);
                b.set_angvel(w * k, true);
            }
            // Arquimedes é anotado aqui e aplicado FORA deste laço: ele precisa ler o
            // collider da zona E o do corpo enquanto escreve no corpo, o que não cabe
            // dentro do empréstimo que este laço já segura. A lista é reusada por zona
            // (a capacidade sobrevive ao `clear`), então uma cena sem empuxo não aloca.
            if effect.density > 0.0 || effect.form_drag > 0.0 {
                to_float.push(parent);
            }
        }
        for &body in &to_float {
            if effect.density > 0.0 {
                buoyancy::apply(
                    bodies,
                    colliders,
                    body,
                    zone_collider,
                    gravity,
                    effect.density,
                    dt,
                );
            }
            // O arrasto de forma vive no mesmo passe pelo mesmo motivo: ele precisa do
            // collider do corpo enquanto escreve nele.
            if effect.form_drag > 0.0 {
                form_drag::apply(bodies, colliders, body, effect.form_drag, dt);
            }
        }
    }
}
