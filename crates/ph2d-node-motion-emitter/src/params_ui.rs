//! The emitter's **param UI metadata** — labels, ranges, widgets. Split from
//! `lib.rs` at the HR-18 LOC cap; it is a clean seam, because none of this is
//! behaviour: the node computes the same particles whatever a slider looks like.
//!
//! The one thing here that is NOT free-standing is the `max` row's ceiling: it
//! and [`super::MAX_ALIVE`] answer the same question, so it is DERIVED, never
//! re-typed ([[feedback_two_doors_to_the_same_question_diverge]]).

use super::MAX_ALIVE;
use ph2d_node_registry::{
    ParamGroup, ParamHardMax, ParamHardMin, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. The panel resolves the face; a node that could pin `px` or `m` would be
/// overriding the artist's `ProjectSettings::display_unit`.
///
/// The three `Length`s are not a guess, they are traced to the columns this node
/// emits: `x`/`y` become the `P` column, which `lower_to_instances` reads into
/// `RenderInstance::world_pos` (world metres), and `size` becomes the `size`
/// column, which lands in `RenderInstance::size` — documented as *local meters*.
///
/// ⚠️ **`speed` is deliberately absent.** It is metres per SECOND, and this
/// vocabulary has no velocity: `Length` would be a lie about the denominator, and
/// a unit that is wrong is worse than a unit that is missing — the artist can
/// still read a bare number, but `12 px` on a rate would teach them the wrong
/// thing. It gets a unit when a `Velocity` is declared, not before.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "size",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "life",
        unit: ParamUnit::Seconds,
    },
];

/// The typed FLOOR — the twin of [`PARAM_HARD_MAX`], and the half that did not
/// exist before doc 88.
///
/// `life`'s slider starts at `0.1 s` because that is where a *fountain* lives,
/// and it is the wrong scale for the other thing this node does: a muzzle flash
/// or a spark is **tens of milliseconds**. The drag range stays where the common
/// case is and the box reaches the uncommon one — Blender's soft/hard split,
/// which the ceiling already had and the floor did not.
pub(crate) static PARAM_HARD_MIN: &[ParamHardMin] = &[ParamHardMin {
    param: "life",
    min: 0.001,
}];

/// Params whose typed entry reaches past their slider (Blender's hard limits).
///
/// A `rate` in the millions is not a mis-click: paired with a millisecond `life`
/// it is a one-frame burst, and `MAX_ALIVE` bounds what actually gets BUILT
/// regardless — `rate` drives no allocation at all, only how fast spawn indices
/// advance.
///
/// ⚠️ **This number does not protect anything, and the thing that needs
/// protecting is not a rate.** The emitter's real ceiling is the product
/// `rate × playhead < 2²⁴`, because ids are `f32`: past 16.777.216 two particles
/// share an id and both the CPU pairing and the GPU gather hand them the same
/// state, silently (gated: `the_id_space_is_exact_only_below_two_to_the_24`).
/// At 4.000.000/s that is **4,2 seconds** of playback; at the slider's 12.000/s
/// it is 23 minutes. No STATIC cap on `rate` can express a bound on a product,
/// which is why this one is an ergonomic stop and not a safety one. The fix is
/// exact ids (wrapped indices, or a `u32`/`f64` id column) — a change to the
/// stream data model, not to this table.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rate",
        max: 4_000_000.0,
    },
    // ⚠️ **DERIVED, never re-typed** — this entry and `MAX_ALIVE` answer the same
    // question ("how many particles may be alive?"), and when the constant went
    // 4096 → 16384 the literal that used to live in the *slider* stayed behind and
    // silently became the real ceiling the artist could reach
    // ([[feedback_two_doors_to_the_same_question_diverge]]). The derivation did not
    // go away with the soft/hard split; it moved to the field that means *ceiling*.
    ParamHardMax {
        param: "max",
        max: MAX_ALIVE as f32,
    },
];

/// Param UI hints (M1.P1).
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "Rate",
        min: 0.0,
        // ⚠️ The old comment here read *"12.000/s is a dense fountain at a 1 s
        // life"* — and `life` defaults to **3 s**, where 12.000/s means 36.000
        // alive: seventy times the `max` default of 512. **The two sliders were
        // describing different scenes.** They now describe the same one: a `rate`
        // dragged to its ceiling at the default life (1.200 × 3 = 3.600) fits
        // inside a `max` dragged to its ceiling (4.096). The one-frame BURST the
        // old comment gestured at is precisely what the hard max at 4.000.000
        // serves — you type it.
        max: 1_200.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "life",
        label: "Life",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⚠️ A FRACTION of `speed`, so `0 .. 1` is the whole meaningful range and the row carries
    // no ceiling entry: above 1 the multiplier turns negative and the particle launches into
    // the opposite half of the cone, which is `spread`'s job and better said there.
    ParamUiHint {
        param: "speed_random",
        label: "Speed Random",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "spread",
        label: "Spread",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "x",
        label: "Origin X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "y",
        label: "Origin Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "max",
        label: "Max Particles",
        min: 1.0,
        // The SLIDER's range — where a HAND works, not where the machine stops.
        // `MAX_ALIVE` is still the ceiling and is still DERIVED (never re-typed);
        // it just moved to `PARAM_HARD_MAX`, where a ceiling belongs. Dragging to
        // 4.194.304 was never authoring: at a 154 px track one pixel moved 27.000
        // particles, so the default of 512 sat in the first fiftieth of a pixel
        // and could not be expressed at all — the resource ruler had become the
        // artist's ruler (CLAUDE.md §0, mirrored into the UI).
        max: 4_096.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "size",
        label: "Size",
        min: 0.01,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// As SEÇÕES deste nó (doc 88 B3). Dez controles numa lista plana são uma parede; a pergunta
/// que cada seção responde é uma só.
///
/// ⚠️ O `seed` está em **Velocity** por MEDIÇÃO, não por hábito: os lanes que ele alimenta são
/// `LANE_ANGLE` e `LANE_SPEED`, ou seja ele é a aleatoriedade do `spread` **e** do
/// `speed_random` — os dois moram aqui, e pô-lo numa seção "Random" o separaria justamente dos
/// números que ele randomiza. *(A frase anterior dizia "um único lane"; o segundo chegou com o
/// `speed_random` e a razão sobreviveu porque o lane novo caiu na mesma seção — o dia em que um
/// lane nascer noutra, esta nota deixa de valer.)*
///
/// ⚠️ E ficam SOLTOS `rate`, `life`, `size` e `max`: os três primeiros são o que um emissor
/// É (com que frequência, por quanto tempo, de que tamanho) e o quarto é o orçamento. Param
/// sem grupo pinta antes de toda seção, e é ali que os essenciais moram.
pub static PARAM_GROUPS: &[ParamGroup] = &[
    // Como a partícula é lançada.
    ParamGroup::new("speed", "Velocity"),
    ParamGroup::new("speed_random", "Velocity"),
    ParamGroup::new("angle", "Velocity"),
    ParamGroup::new("spread", "Velocity"),
    ParamGroup::new("seed", "Velocity"),
    // De onde.
    ParamGroup::new("x", "Origin"),
    ParamGroup::new("y", "Origin"),
];
