//! **Bake — the simulated pose becomes editable keys** (ADR-0131 D11, W4).
//!
//! `ph2d_physics_ecs::bake` reads the trajectory out of the simulation; this turns those numbers
//! into tracks, in ONE undo step. The measured history (the fit that was rejected, the two-queue
//! gap) is verbatim in `git show ecca0421ac:crates/ph2d-app-physics/src/bake.rs`.
//!
//! - **One key per tick, linear between them — never a curve-fit.** A fit resamples the motion and
//!   rounds a bounce (tightened, it overshoots it). Linear is exact at 60 fps, cannot overshoot,
//!   and a bounce stays a sharp corner. A channel the sim never moved writes no track
//!   (`BakedTrajectory::channel`). ⛔ Solver-velocity tangents buy nothing at 60 fps and risk
//!   overshoot at contacts — deliberately not built.
//! - **Baking flips the body to `Kinematic`, and that IS the bake.** The physics readback writes
//!   `Transform` after the timeline apply, so a baked `Dynamic` body is overwritten every frame.
//!   The flip goes through `apply_physics_edit`, the same door the §11 chip uses.
//! - ⚠️ **Known gap: one gesture undoes as two.** The keys are in the timeline queue, the kind in
//!   the object queue; one Ctrl+Z leaves a half-undone bake. Closing it is a change to the editor's
//!   undo architecture, not to the bake.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{
    InputTape, PhysicsBridge, PoseChannel, SceneAtTick, bake_trajectories_with_scene_and_tape,
    jointed_group,
};
use ph2d_timeline::{PropKind, TimelineState};

/// Which pose channels a bake writes into the timeline.
///
/// The default is [`All`](BakeChannels::All) — the whole pose. The subsets exist
/// for **layering**: bake the physics tumble onto a hand-animated position
/// ([`Rotation`](BakeChannels::Rotation)), or the physics fall under a
/// hand-animated spin ([`Position`](BakeChannels::Position)). A channel not
/// baked is left alone — for a `Kinematic` body it then follows whatever the
/// scene (the artist's own animation, or the static `Transform`) says.
///
/// This is *narrower* than "a channel that never moved is skipped" (which the
/// bake already does): that protects a channel the sim had nothing to say about;
/// this discards a channel the sim DID move because the artist wants to own it.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum BakeChannels {
    /// X, Y and Rotation — the whole pose.
    #[default]
    All,
    /// Only X and Y — keep the artist's own rotation.
    Position,
    /// Only Rotation — keep the artist's own position.
    Rotation,
}

impl BakeChannels {
    /// The channels this selection writes. `bake_selection` iterates exactly
    /// these, so an unselected channel produces no track.
    pub(crate) fn channels(self) -> &'static [PoseChannel] {
        match self {
            BakeChannels::All => &PoseChannel::ALL,
            BakeChannels::Position => &[PoseChannel::X, PoseChannel::Y],
            BakeChannels::Rotation => &[PoseChannel::Rotation],
        }
    }

    /// Segmented-control tag ↔ selection, both directions in one place so a
    /// mismatch is visible (the §11 selector paints by tag, the edit carries a
    /// tag). Any unknown tag folds to `All` — the safe default.
    pub fn tag(self) -> u8 {
        match self {
            BakeChannels::All => 0,
            BakeChannels::Position => 1,
            BakeChannels::Rotation => 2,
        }
    }

    pub fn from_tag(tag: u8) -> Self {
        match tag {
            1 => BakeChannels::Position,
            2 => BakeChannels::Rotation,
            _ => BakeChannels::All,
        }
    }
}

/// The scene, as the timeline says it is at a given tick — the shell's answer
/// to [`SceneAtTick`], and the only half that can give one.
///
/// A kinematic body's pose comes from its curve, so "where is everything at
/// tick T" is `apply_from_doc` at `T · dt`. Physics never learns what a
/// timeline is; it only asks to be put into a state.
pub(crate) struct TimelineScene<'a> {
    pub doc: &'a mut ph2d_timeline::TimelineDoc,
    pub fixed_dt: f64,
    /// Os livros do quadro, que recebem o autorado que o tique tira à curva.
    pub livro: Option<Livros<'a>>,
}

/// Os livros do quadro que cada tique alimenta: a prévia (as juntas), a forma, e a mão do mundo.
pub(crate) struct Livros<'a> {
    pub drive: &'a mut ph2d_preview_drive::PreviewDrive,
    pub fisica: &'a mut crate::canais_corpo::ConducaoFisica,
    pub mundo: &'a mut crate::canais_mundo::ConducaoMundo,
}

impl SceneAtTick for TimelineScene<'_> {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let t = tick as f64 * self.fixed_dt;
        ph2d_timeline::apply_from_doc(sim.world_mut(), self.doc, t);
        let fisica = self.livro.as_mut().map(|l| &mut *l.fisica);
        crate::canais_corpo::aplicar_do_tique(sim, self.doc, t, fisica);
        // A JUNTA do tique, no ledger do QUADRO (a lição da 8.3b); a ponte lê-a por `drive_joint_params`.
        if let Some(l) = self.livro.as_mut() {
            crate::canais_junta::aplicar(sim, l.drive, self.doc);
        }
        true
    }

    /// O MUNDO do tique ([`crate::canais_mundo::alvo_do_tique`]).
    fn mundo(
        &mut self,
        sim: &mut SimWorld,
        autorado: ph2d_physics_ecs::PhysicsSettings,
    ) -> Option<ph2d_physics_ecs::PhysicsSettings> {
        let l = self.livro.as_ref()?;
        crate::canais_mundo::alvo_do_tique(sim, autorado, l.mundo)
    }
}

/// The `PhysicsFieldEdit::Kind` tag for `BodyKind::Kinematic`. Read from the
/// enum rather than written as `2`, so the two cannot drift.
fn kinematic_tag() -> u8 {
    ph2d_physics_ecs::BodyKind::Kinematic.tag()
}

// O intervalo: a porta é da timeline, partilhada com o bake das constraints.
pub use ph2d_timeline::{DEFAULT_BAKE_SECONDS, bake_range};

/// One track's worth of work: whose, which property, and the dense samples that
/// become its keys.
type BakedTrack = (u64, PropKind, Vec<(f64, f64)>);

/// What a bake did, for the toast and for the gates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BakeOutcome {
    /// Bodies whose motion became curves.
    pub bodies: usize,
    /// Tracks written across all of them. A body that never moved contributes
    /// none — see `BakedTrajectory::channel`.
    pub tracks: usize,
    /// The bake did not run at all (an undo bracket was already open).
    /// Distinct from "ran and found nothing", because the honest thing to tell
    /// the artist is different in each case.
    pub refused: bool,
    /// The entity's clock could not place a key at some sampled instant (the
    /// clip plays zero times or twice there, under a strip stack).
    pub unmappable: bool,
    /// Every body asked for was already handed over — a second bake of the same
    /// selection. `readback` skips a kinematic body, so its pose is constant
    /// through the bake and every channel comes back empty: indistinguishable
    /// from "nothing moved" by the numbers, and a very different sentence to
    /// read while watching the object move on screen.
    pub already_baked: bool,
}

impl BakeOutcome {
    pub fn is_empty(&self) -> bool {
        self.tracks == 0
    }

    fn refused() -> Self {
        Self {
            refused: true,
            ..Self::default()
        }
    }

    fn unmappable() -> Self {
        Self {
            unmappable: true,
            ..Self::default()
        }
    }
}

/// The channel-to-track mapping. Physics has three things to say about a rigid
/// body and the timeline has a property for each, in the same unit — radians on
/// both sides, so nothing converts.
fn prop_for(channel: PoseChannel) -> PropKind {
    match channel {
        PoseChannel::X => PropKind::TranslationX,
        PoseChannel::Y => PropKind::TranslationY,
        PoseChannel::Rotation => PropKind::Rotation,
    }
}

/// How many ticks a bake of `seconds` covers, at the caller's tick length.
///
/// At least one: a range that rounds to zero ticks would write a single key and
/// call it an animation.
#[must_use]
pub(crate) fn ticks_for(seconds: f64, fixed_dt: f64) -> u64 {
    if fixed_dt <= 0.0 {
        return 1;
    }
    ((seconds / fixed_dt).round() as i64).max(1) as u64
}

/// Simulate `entities` over `[0, end]` seconds and write the samples in
/// `[start, end]` as keys.
///
/// One timeline undo step for the whole thing, however many bodies, channels
/// and frames it covers — opened before the first key and committed after the
/// fit, so the dense keys and their cleanup revert together.
///
/// ⚠️ **The simulation runs the whole `[0, end]`; only `[start, end]` becomes
/// keys.** The sim is a function of the tick, so it cannot start in the middle,
/// and the front must be simulated to advance the scene for any body riding a
/// baked platform — but the front's samples are discarded rather than written,
/// which is what makes a `[2s, 5s]` loop bake exactly `[2s, 5s]`.
///
/// The bodies are flipped to `Kinematic` (module docs) via `queue`, which the
/// caller applies with the rest of the frame's ECS edits.
#[allow(clippy::too_many_arguments)]
pub fn bake_selection(
    timeline: &mut TimelineState,
    physics: &mut PhysicsBridge,
    sim: &mut SimWorld,
    entities: &[Entity],
    start: f64,
    end: f64,
    fixed_dt: f64,
    channels: BakeChannels,
    tape: &mut InputTape,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> BakeOutcome {
    // ⚠️ **Baking one link of a jointed rig bakes the WHOLE rig.** A bake flips
    // its bodies to Kinematic; with the Physics toggle off, any un-baked DYNAMIC
    // neighbour freezes (nothing steps the solver) while the baked links play,
    // and the joint stretches. There is no coherent partial bake of a coupled
    // rig, so the selection is expanded to its jointed connected component before
    // anything reads it. Static/Kinematic bodies do not conduct (module docs of
    // `jointed_group`); a body with no joints expands to itself. The toast's body
    // count reflects the expanded set, so a rig pulled in is visible.
    let group = jointed_group(sim.world_mut(), entities);
    let entities: &[Entity] = &group;

    let ticks = ticks_for(end, fixed_dt);
    // ⚠️ The bake advances the SCENE (timeline-driven bodies) and replays the TAPE (W16: without it
    // a player is baked with the finger of the click instant) — a bake that does not reproduce
    // the input bakes a different run.
    let trajectories = {
        let mut scene = TimelineScene {
            doc: &mut timeline.doc,
            fixed_dt,
            livro: None,
        };
        bake_trajectories_with_scene_and_tape(
            physics, sim, entities, ticks, fixed_dt, &mut scene, tape,
        )
    };

    // Collect first, write second: a bake that opened an undo bracket and then
    // found it had nothing to say would leave an empty step on the stack.
    let mut work: Vec<BakedTrack> = Vec::new();
    for traj in &trajectories {
        // Only the SELECTED channels, and only the samples inside the bake
        // window. A channel the artist chose to keep is never written, even
        // where the sim moved it (layering); nor is the front `[0, start)`,
        // which was simulated only to advance the scene.
        for &channel in channels.channels() {
            if let Some(samples) = traj.channel_in(channel, start, end) {
                work.push((traj.entity.to_bits(), prop_for(channel), samples));
            }
        }
    }
    if work.is_empty() {
        // Was there anything the sim COULD have said? A body the scene already
        // drives has no simulated motion to read, by construction.
        let all_kinematic = !trajectories.is_empty()
            && trajectories.iter().all(|t| {
                sim.world()
                    .get::<ph2d_physics_ecs::RigidBody>(t.entity)
                    .is_some_and(|rb| rb.kind == ph2d_physics_ecs::BodyKind::Kinematic)
            });
        return BakeOutcome {
            already_baked: all_kinematic,
            ..BakeOutcome::default()
        };
    }

    // The stack has to be primed before anything asks where a key lands — the
    // same first line `apply_samples` runs.
    timeline.doc.prime_stack(0.0);
    // ⚠️ REFUSE rather than join an open bracket. A record/performing session
    // owns the bracket while it runs, and folding a bake into it would make one
    // Ctrl+Z undo the recording AND the bake together — and the bake would
    // never commit its own step, so the artist could not undo it alone. It also
    // cannot happen by accident today (the button is a click, the bracket is a
    // drag), which is exactly why it would be a bug nobody could reproduce.
    if timeline.history.is_open() {
        return BakeOutcome::refused();
    }
    timeline.history.begin(&timeline.doc);

    // One key per tick, LINEAR between them — no fit (module docs). Sampled at
    // 60 fps the playhead lands on the tick times, so this reproduces the sim
    // verbatim; linear cannot overshoot; a bounce stays a sharp corner.
    let interp = Interp::Linear;
    for (entity_bits, prop, samples) in &work {
        for &(t, v) in samples {
            // ⚠️ The key lands on the ENTITY's clock (`key_time`, the same door as every authoring
            // path): raw playhead seconds missed the pose by 1.618 m under a half-speed remap
            // (`feedback_derived_coordinate_seed_must_match_sample`). Primed per sample.
            timeline.doc.prime_stack(t);
            let Some(t) = ph2d_timeline::key_time(&timeline.doc, *entity_bits, t) else {
                // No single answer — the clip plays zero times or twice at this
                // instant under a strip stack. Refusing the WHOLE bake is the
                // honest move: a curve with a hole where the map failed is
                // wrong in a way that looks like the simulation was wrong.
                timeline.history.cancel();
                return BakeOutcome::unmappable();
            };
            // Real seconds, NOT frame-snapped, written directly (an `AddKey` intent snaps and
            // opens its own undo step).
            timeline.doc.upsert_key(
                *entity_bits,
                *prop,
                RationalTime::from_seconds(t),
                AnimValue::Float(v as f32),
                interp,
            );
        }
    }

    // Every dense key was written inside the one bracket, so the whole bake is
    // still a single undo step (there is no fit to fold in anymore).
    let doc = timeline.doc.clone();
    timeline.history.commit_if_changed(&doc);

    // Hand the pose over — but only to the bodies this bake actually WROTE a
    // curve for. Same door as the §11 chip.
    //
    // ⚠️ Not `&trajectories` (the SELECTION, routinely mixed: a marquee catches the floor, which
    // would silently become Kinematic): only a body whose curve was written.
    let baked: std::collections::BTreeSet<u64> = work.iter().map(|(bits, _, _)| *bits).collect();
    let mut bodies = 0usize;
    for bits in baked {
        bodies += 1;
        crate::inspector::body::apply_physics_edit(
            sim,
            bits,
            ph2d_editor_core::PhysicsFieldEdit::Kind(kinematic_tag()),
            queue,
            registry,
        );
    }

    BakeOutcome {
        bodies,
        tracks: work.len(),
        refused: false,
        already_baked: false,
        unmappable: false,
    }
}

#[cfg(test)]
#[path = "bake_curve_tests.rs"]
mod curve_tests;
#[cfg(test)]
#[path = "physics_bake_joint_tests.rs"]
mod joint_tests;
#[cfg(test)]
#[path = "physics_bake_tests.rs"]
mod tests;
