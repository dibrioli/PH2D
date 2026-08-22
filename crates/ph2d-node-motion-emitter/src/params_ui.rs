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
    // The shape's half-extents are world distances by the same trace as `x`/`y`: they are added
    // to them, and the sum becomes the `P` column.
    ParamUnitDecl {
        param: "shape_w",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "shape_h",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "life",
        unit: ParamUnit::Seconds,
    },
    // The two burst clocks are playhead seconds, by the same trace `life` is.
    ParamUnitDecl {
        param: "burst_time",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "burst_period",
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
    // ⚠️ DERIVED for the same reason `max` is: one burst cannot mint more particles than the
    // alive set may hold, and re-typing that number is how the two silently drift apart.
    ParamHardMax {
        param: "burst_count",
        max: MAX_ALIVE as f32,
    },
];

/// Param UI hints (M1.P1).
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ FIRST, because it is the question that decides which of the two rows below the artist
    // even sees: `rate` answers *how fast do they arrive?* and the `burst_*` trio answers *how
    // many, and when?*. They are ALTERNATIVES, never both — which is why they can live apart
    // (`size`/`size_random` could not: those are a base and its variance).
    ParamUiHint {
        param: "emit_mode",
        label: "Emit",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Continuous", "Burst"],
        },
    },
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
        param: "burst_count",
        label: "Burst Count",
        min: 1.0,
        // The SLIDER's range — where a hand works. `MAX_ALIVE` is still the ceiling and is still
        // DERIVED (`PARAM_HARD_MAX`), never re-typed, exactly as `max`'s is.
        max: 2_048.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "burst_time",
        label: "Burst At",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // `0` = a single burst. The slider starts there because "once" is the common case and the
    // heartbeat is the addition, not the default.
    ParamUiHint {
        param: "burst_period",
        label: "Burst Every",
        min: 0.0,
        max: 20.0,
        step: 0.1,
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
    // ⚠️ `shape_mode`, not `shape`: the artist reads the LABEL, and the param name is what the
    // catalogue's `a_mode_wears_words_not_an_index` scans for (`mode` / `*_mode`). Calling it
    // `shape` would have put a choice outside the one gate that exists to stop choices being
    // painted as raw indices — for nothing, since the name is internal.
    ParamUiHint {
        param: "shape_mode",
        label: "Shape",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Point", "Disc", "Ring", "Rect"],
        },
    },
    ParamUiHint {
        param: "shape_w",
        label: "Shape W",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "shape_h",
        label: "Shape H",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // ⚠️ `dir_mode`, not `direction`: the artist reads the LABEL, and the param name is what the
    // catalogue's `a_mode_wears_words_not_an_index` scans for (`mode` / `*_mode`).
    ParamUiHint {
        param: "dir_mode",
        label: "Direction",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Angle", "Outwards", "Inwards"],
        },
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
    // ⚠️ Sits HERE, immediately after the number it varies, and that is the whole placement
    // argument: `speed`/`speed_random` are adjacent inside *Velocity*, and a base separated from
    // its variance is exactly the wall the sections exist to remove. Since `size` is loose, its
    // variance is loose too — putting one in a section would paint them at opposite ends.
    //
    // A FRACTION of `size`, so `0 .. 1` is the whole meaningful range and the row carries no
    // ceiling entry: above 1 the multiplier turns negative and the kernel's floor takes it to
    // zero, i.e. some particles simply vanish, which is a thing to type rather than to drag past.
    ParamUiHint {
        param: "size_random",
        label: "Size Random",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Uma FRACÇÃO, e o curso é `[0, 1]` inteiro** — o `0` (nenhuma nasce) é uma resposta
    // legítima e visível, não um estado inválido a esconder: é como se desliga o emitter sem
    // lhe mexer no `rate`, e é o que um `pulse.*` dirigido liga e desliga.
    ParamUiHint {
        param: "probability",
        label: "Probability",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// As SEÇÕES deste nó (doc 88 B3). Dez controles numa lista plana são uma parede; a pergunta
/// que cada seção responde é uma só.
///
/// ⚠️ **O `seed` ficou SOLTO, e a nota anterior previu o dia.** Ela o punha em *Velocity*
/// argumentando que ele alimentava só o `LANE_ANGLE`, ou seja que era a aleatoriedade do
/// `spread`, e dizia que *"o dia em que um lane nascer noutra seção, esta nota deixa de valer"*.
/// Nasceram dois: `LANE_SHAPE_U`/`LANE_SHAPE_V` randomizam onde a partícula nasce, que é
/// **Origin**. Um `seed` que alimenta quatro lanes em duas seções não é um controle de
/// velocidade — é a aleatoriedade do NÓ, e o lugar de um número que não pertence a nenhuma
/// seção é fora de todas, junto dos essenciais.
///
/// ⚠️ E ficam SOLTOS `rate`, `life`, `size`, `size_random` e `max`: os três primeiros são o que
/// um emissor É (com que frequência, por quanto tempo, de que tamanho) e o último é o orçamento.
/// Param sem grupo pinta antes de toda seção, e é ali que os essenciais moram.
///
/// ⚠️ **O `size_random` é a exceção, e ela é sobre ADJACÊNCIA, não sobre ser essencial.** Ele
/// varia o `size`, e uma variância que não pinta ao lado da própria base é a parede que as seções
/// existem para remover — `speed`/`speed_random` são vizinhos DENTRO de *Velocity* pela mesma
/// razão. Com o `size` solto, a única forma de manter o par junto é deixar os dois soltos; pô-lo
/// numa seção os mandaria para pontas opostas do painel. A regra do conjunto solto passa a ser
/// *"param que não pertence a nenhuma seção"*, e não *"param essencial"*.
pub static PARAM_GROUPS: &[ParamGroup] = &[
    // Como a partícula é lançada.
    ParamGroup::new("speed", "Velocity"),
    ParamGroup::new("speed_random", "Velocity"),
    ParamGroup::new("angle", "Velocity"),
    ParamGroup::new("spread", "Velocity"),
    // De onde.
    ParamGroup::new("x", "Origin"),
    ParamGroup::new("y", "Origin"),
    ParamGroup::new("shape_mode", "Origin"),
    ParamGroup::new("shape_w", "Origin"),
    ParamGroup::new("shape_h", "Origin"),
    // It answers *which way does a particle LEAVE?*, so it lives with the launch and not with
    // the birth place it happens to be derived from.
    ParamGroup::new("dir_mode", "Velocity"),
    // Quando e quantas. O `emit_mode` fica SOLTO, ao lado do `rate`: ele e o `rate` respondem a
    // mesma pergunta (*como as particulas chegam?*), e os tres abaixo sao o detalhe de UMA das
    // duas respostas.
    // ⚠️ **O `probability` fica SOLTO, com o `rate` e o `emit_mode`, e NÃO na secção Burst** —
    // ele vale nos dois modos, e os três da secção estão `ParamGate`-ados ao modo Burst. Um
    // controle vivo dentro de uma secção que desaparece leria como se ele desaparecesse também.
    ParamGroup::new("burst_count", "Burst"),
    ParamGroup::new("burst_time", "Burst"),
    ParamGroup::new("burst_period", "Burst"),
];

/// **`Direction` is only offered once a shape gives a particle a radius.**
///
/// With `Point` every birth is the origin, so `Outwards`/`Inwards` fall back to the cone for
/// EVERY particle — the control would be provably inert, which is the dead knob this
/// side-channel exists to prevent. `angle`/`spread` stay visible in all three modes: the cone is
/// still the cone, it just opens around a different axis.
/// ⚠️ **And `rate` / the `burst_*` trio are shown one set at a time**, because they are two
/// different laws and not two knobs on one: a `rate` in burst mode would be a number the count
/// law never reads, and a `burst_count` in continuous mode the same — the dead knob this
/// side-channel exists to prevent, twice.
pub static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "dir_mode",
        when: "shape_mode",
        values: &[1, 2, 3],
    },
    // ⚠️ **As DUAS linhas abaixo faltavam, e a de cima já dizia a condição exacta** (doc 90 §2,
    // 2026-08-22): `birth_offset` sai cedo com `Shape::Point`, o default, então a largura e a
    // altura da forma de nascimento são inertes ali — e o `dir_mode` ao lado desaparecia
    // correctamente pela MESMA condição.
    //
    // ⚠️ *Um gate que trata um dos params de uma família e esquece os outros dois é pior que
    // nenhum*: ele ENSINA ao artista que este painel esconde o que não serve, e é exactamente
    // por acreditar nisso que ele vai arrastar Shape W à procura de um efeito que não existe.
    ph2d_node_registry::ParamGate {
        param: "shape_w",
        when: "shape_mode",
        values: &[1, 2, 3],
    },
    ph2d_node_registry::ParamGate {
        param: "shape_h",
        when: "shape_mode",
        values: &[1, 2, 3],
    },
    ph2d_node_registry::ParamGate {
        param: "rate",
        when: "emit_mode",
        values: &[0],
    },
    ph2d_node_registry::ParamGate {
        param: "burst_count",
        when: "emit_mode",
        values: &[1],
    },
    ph2d_node_registry::ParamGate {
        param: "burst_time",
        when: "emit_mode",
        values: &[1],
    },
    ph2d_node_registry::ParamGate {
        param: "burst_period",
        when: "emit_mode",
        values: &[1],
    },
];
