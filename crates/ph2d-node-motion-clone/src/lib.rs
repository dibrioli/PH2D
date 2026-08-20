#![forbid(unsafe_code)]
//! `motion.clone` — a Motion **cloner**: multiplies its input instance stream
//! into `count` copies laid along a **polar step axis** — each copy offset on
//! `P` by `copy_base · (distance·cos θ, distance·sin θ)`, where θ (`angle`, in
//! **degrees**) frees the row from the X/Y grid so the artist dials any direction
//! with one control, and `copy_base` is the copy's signed rank. With `center` off the
//! ranks are `0,1,2,…` (the row grows away from the original); with `center` on
//! they are balanced about zero (`… −1, 0, 1 …`) so the queue straddles the
//! original instead of trailing from it. Other columns replicate unchanged —
//! **except** `Index`/`Count`, which are renumbered **continuous across copies**
//! so downstream index-driven effects (colour ramps, staggers) flow seamlessly
//! over the whole multiplied set rather than restarting per copy.
//!
//! This is a **stream multiplier** (1 node → N×in instances) — NOT entity
//! spawning; it has no ECS analogue (ADR-0035). Output count = `in_count *
//! count`. Pure; the polar direction is transcendental-free (HR-5, see [`trig`]).
//!
//! Params (read via `ctx.param` — per-instance override else the manifest
//! default shown): `count` (3), `distance` (2.0), `angle` (0° → +X, so the
//! default reproduces the old `step = (2, 0)` row), `center` (0 = off).
//! `count` is read as an element count ([`param_as_count`]) and clamped so the
//! output `in_count * count` never overflows the allocation; the minimum is one
//! copy (a cloner is at least a passthrough).

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, SIZE_IDENTITY, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Degrees per full turn — the exact divisor from the authored angle into the
/// cycle-based trig's unit. IEEE division is correctly rounded → deterministic
/// (HR-5); multiplying by a reciprocal would not be exact.
const DEG_PER_TURN: f32 = 360.0;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.clone"),
    name: "motion.clone",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "count",
            default: 3.0,
        },
        ParamSpec {
            name: "distance",
            default: 2.0,
        },
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
        ParamSpec {
            name: "center",
            default: 0.0,
        },
        // **O TAPER** — ver [`SCALE_TAPER`]. Apendados, e literais no default.
        ParamSpec {
            name: "scale_taper",
            default: 1.0,
        },
        ParamSpec {
            name: "rot_taper",
            default: 0.0,
        },
    ],
    // CPU-only by design (see handoff §9): a cloner *changes the element count*
    // (1 → N×in), which is structural, not a per-element `ph2d-expr` map an
    // `eval_column` could lower; and no Instances-domain WGSL runtime exists.
    lowerings: &[LoweringKind::Cpu],
};

/// **O TAPER da fila** (doc 89 folha 08 — *"taper CUMULATIVO por cópia, escala/rotação
/// incrementais"*): MiniCavalry `cloneLinear` `scaleTaper 1` / `rotTaperDeg 0`; C4D Cloner,
/// aba Transform, *"P/S/R aplicados a TODO clone"*.
///
/// ⚠️ **A lei é LERP DA 1ª À ÚLTIMA CÓPIA, e é a da referência citada, não uma potência.** O
/// dump do `cloneLinear` diz `scaleTaper 1 (lerp final)` e, nos gotchas, *"Taper é lerp do 1º
/// ao último"* — o número que o artista digita é **o que a ÚLTIMA cópia vale**, não o fator
/// que compõe a cada passo. A diferença não é cosmética: com `scaleTaper = 0,5` e 5 cópias, a
/// lei composta (`0,5^c`) dá `1 · 0,5 · 0,25 · 0,125 · 0,0625` — a fila desaparece a meio e o
/// knob deixa de ter curso útil —, enquanto o lerp dá `1 · 0,875 · 0,75 · 0,625 · 0,5`, e
/// *metade* quer dizer metade. ⚠️ E ela é **HR-5 de graça**: um lerp é aritmética, uma
/// potência real seria `powf`.
///
/// ⚠️ **O `t` corre pela ORDINAL da cópia, nunca pelo posto assinado do `center`** — *"do 1º
/// ao último"* é literal. Com `center` ligado a fila passa a montar-se em torno do original e
/// o taper continua a correr da ponta de trás para a da frente; se ele seguisse o posto
/// assinado, ligar o `center` **inverteria o sentido do afunilamento no meio da fila**, e os
/// dois controles deixariam de ser ortogonais.
///
/// ⚠️ **O que ele NÃO faz: o layout.** As cópias continuam numa RETA — o taper descreve como
/// cada cópia difere da anterior, não por onde a fila anda. Uma fila que se ENROLA (o *step
/// cumulativo* que o doc 63 §3 chama de espiral) é outra coisa: pede que o próprio passo seja
/// girado e escalado a cada cópia, o que colide com o `center` (um somatório não tem posto
/// assinado) e não é o que a referência desta célula define.
const SCALE_TAPER: &str = "scale_taper";
/// O taper de rotação, em **graus** — a unidade autorada da casa, a mesma da coluna `rot` e
/// do `angle` deste nó. Ver [`SCALE_TAPER`] para a lei.
const ROT_TAPER: &str = "rot_taper";

/// A fração do taper na cópia `copy` de `k`: `0` na primeira, `1` na última.
///
/// ⚠️ **`k = 1` dá `0`, e não uma divisão por zero** — uma fila de uma cópia é a própria
/// entrada, e o taper de uma lista de um elemento é o começo dela.
fn taper_t(copy: usize, k: usize) -> f32 {
    if k <= 1 {
        return 0.0;
    }
    #[expect(clippy::cast_precision_loss, reason = "contagem de cópias, ≤ 2^24")]
    let t = copy as f32 / (k - 1) as f32;
    t
}

/// Replicate a column `k` times (copy 0, copy 1, ... — element order within a
/// copy preserved), matching the `P` offset loop in [`clone_stream`].
fn replicate(col: &Column, k: usize) -> Column {
    fn rep<T: Clone>(v: &[T], k: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(v.len() * k);
        for _ in 0..k {
            out.extend_from_slice(v);
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(rep(v, k)),
        Column::Vec2(v) => Column::Vec2(rep(v, k)),
        Column::Vec3(v) => Column::Vec3(rep(v, k)),
        Column::Vec4(v) => Column::Vec4(rep(v, k)),
    }
}

/// Clamp the requested copy count so the output `in_count * k` stays within
/// `max` (and so the multiplication can never overflow the allocation). Always
/// at least one copy (a cloner is a passthrough at minimum). With `in_count`
/// already bounded by the same budget upstream, one copy always fits.
fn copies_within_budget(requested: usize, in_count: usize, max: usize) -> usize {
    if in_count == 0 {
        return requested.max(1); // output is empty regardless; avoids /0
    }
    requested.min(max / in_count).max(1)
}

/// The signed rank of copy `copy` (of `k`): `0,1,2,…` when `center` is off (the
/// row trails from the original), or balanced about zero when on
/// (`…, −1, 0, 1, …` for odd `k`; `…, −0.5, 0.5, …` for even) so the queue
/// straddles the original. Multiplying this by the step vector gives the copy's
/// `P` offset (and the offset of copy 0 with `center` off is exactly zero — the
/// cloner is a passthrough of the input at rank 0).
fn copy_rank(copy: usize, k: usize, center: bool) -> f32 {
    let r = copy as f32;
    if center {
        r - (k as f32 - 1.0) * 0.5
    } else {
        r
    }
}

/// Multiply `input` into `k` copies, offsetting each copy's `P` by
/// `copy_rank · (sx, sy)` (see [`copy_rank`] for `center`) and replicating every
/// other column unchanged — **except** `Index`/`Count`, renumbered continuous
/// across copies (copy `c` gets `Index += c·in_count`, `Count = in_count·k`) so
/// index-driven downstream effects span the whole multiplied set. `k` must
/// already be budget-clamped ([`copies_within_budget`]) so `in_count * k` fits
/// the allocation. Pure and isolated so the per-copy offset, the global
/// renumbering, *and* the column-replication alignment are unit-tested directly,
/// alongside the end-to-end cook test that drives the params via overrides.
fn clone_stream(
    input: &Stream,
    k: usize,
    sx: f32,
    sy: f32,
    center: bool,
    scale_taper: f32,
    rot_taper: f32,
) -> Stream {
    // ⚠️ **Os dois knobs são LITERAIS no default, e o teste é sobre o VALOR e não sobre um
    // caminho**: `1 + (1 − 1)·t` é `1` e `0·t` é `0` para todo `t` finito, em IEEE-754. O que
    // as bandeiras decidem não é a aritmética — é se a coluna chega a ser TOCADA, porque
    // `size` e `rot` podem não existir e cunhá-las é que mudaria o que sai daqui.
    let scaling = scale_taper != 1.0;
    let turning = rot_taper != 0.0;
    let factor = |copy: usize| 1.0 + (scale_taper - 1.0) * taper_t(copy, k);
    let turn = |copy: usize| rot_taper * taper_t(copy, k);
    // The port type guarantees `P` is `Vec2`; any other dim is an upstream
    // node-author bug — assert loudly rather than replicate `P` without the
    // per-copy offset (which would stack every copy on top of the original).
    debug_assert!(
        !matches!(input.get("P"), Some(c) if !matches!(c, Column::Vec2(_))),
        "motion.clone expects `P` to be a Vec2 column (port type guarantees it)"
    );
    let in_count = input.count();
    let total = in_count * k;
    let mut out = Stream::new(total);
    for (name, col) in input.columns() {
        match (name.as_str(), col) {
            ("P", Column::Vec2(v)) => {
                let mut nv = Vec::with_capacity(total);
                for copy in 0..k {
                    let rank = copy_rank(copy, k, center);
                    let (dx, dy) = (rank * sx, rank * sy);
                    for p in v {
                        nv.push([p[0] + dx, p[1] + dy]);
                    }
                }
                out.set("P", Column::Vec2(nv));
            }
            // Continuous global index so a colour ramp / stagger reads one
            // uninterrupted `0..total` sequence over the multiplied set.
            ("Index", Column::Scalar(v)) => {
                let mut nv = Vec::with_capacity(total);
                for copy in 0..k {
                    let off = (copy * in_count) as f32;
                    nv.extend(v.iter().map(|&i| i + off));
                }
                out.set("Index", Column::Scalar(nv));
            }
            ("Count", Column::Scalar(_)) => {
                out.set("Count", Column::Scalar(vec![total as f32; total]));
            }
            // O taper multiplica o tamanho que a peça JÁ TEM — ele modula a fonte, nunca a
            // substitui (a mesma lei do `point_scale` do `motion.duplicator`).
            ("size", Column::Vec2(v)) if scaling => {
                let mut nv = Vec::with_capacity(total);
                for copy in 0..k {
                    let f = factor(copy);
                    nv.extend(v.iter().map(|s| [s[0] * f, s[1] * f]));
                }
                out.set("size", Column::Vec2(nv));
            }
            // …e a rotação SOMA à que a peça já tem, em graus.
            ("rot", Column::Scalar(v)) if turning => {
                let mut nv = Vec::with_capacity(total);
                for copy in 0..k {
                    let a = turn(copy);
                    nv.extend(v.iter().map(|r| r + a));
                }
                out.set("rot", Column::Scalar(nv));
            }
            _ => out.set(name.clone(), replicate(col, k)),
        }
    }
    // ⚠️ **A coluna AUSENTE é cunhada — mas só quando o knob está ligado.** Sem isto o taper
    // seria um botão morto no caso mais comum de todos: uma grelha não traz `size` nem `rot`,
    // e a peça é desenhada com `SIZE_IDENTITY`/`0°`. Cunhar sempre seria o oposto — uma
    // coluna a mais viaja, é serializada e muda o que um nó a jusante vê ([`clone_stream`]
    // é a mesma escolha que o `point_scale` do `motion.duplicator` documenta).
    if scaling && input.get("size").is_none() {
        let mut nv = Vec::with_capacity(total);
        for copy in 0..k {
            let f = factor(copy);
            nv.extend((0..in_count).map(|_| [SIZE_IDENTITY[0] * f, SIZE_IDENTITY[1] * f]));
        }
        out.set("size", Column::Vec2(nv));
    }
    if turning && input.get("rot").is_none() {
        let mut nv = Vec::with_capacity(total);
        for copy in 0..k {
            let a = turn(copy);
            nv.extend((0..in_count).map(|_| a));
        }
        out.set("rot", Column::Scalar(nv));
    }
    out
}

struct MotionClone;

impl NodeOp for MotionClone {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Polar step axis: distance along a free direction θ (degrees). θ = 0 → +X,
        // so `distance = 2` reproduces the old `(step_x, step_y) = (2, 0)` row.
        // Degrees→cycles at the trig edge (exact IEEE division; HR-5).
        let distance = ctx.param("distance");
        let (c, s) = cos_sin_cycles(ctx.param("angle") / DEG_PER_TURN);
        let (sx, sy) = (distance * c, distance * s);
        let center = ctx.param("center") >= 0.5; // Toggle: ≥0.5 → centred queue
        let (scale_taper, rot_taper) = (ctx.param(SCALE_TAPER), ctx.param(ROT_TAPER));
        // `count` from an `f32` param: total conversion (non-finite/negative →
        // 0) then clamped so `in_count * k` cannot overflow the allocation; at
        // least one copy (passthrough).
        let requested = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let input = ctx.input(0);
        let k = copies_within_budget(requested, input.count(), RECOMMENDED_MAX_ELEMENTS);
        let out = clone_stream(input, k, sx, sy, center, scale_taper, rot_taper);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionClone))?;
    // M1.R1 — UI metadata for the card (a stream multiplier → muted-green
    // distribute, rounded-rect silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Clone",
            category: ph2d_node_registry::NodeUiCategory::Distribute,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // M1.P1 — param rows: whole-number copy count + signed per-copy step.
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `count` — MEDIDO, e ele conta CÓPIAS, não instâncias** (doc 88 A1 · §0),
/// enquanto o slider fica nas 32 que cobrem a autoria confortável.
///
/// ⚠️ **O custo de um multiplicador não é função só deste param:** ele é `count × entrada`, então
/// a mesma contagem custa mil vezes mais sobre um stream mil vezes maior. O que torna um teto
/// estático seguro aqui é o **orçamento de instâncias já existir a jusante** — `copies_within_budget`
/// corta as cópias contra [`RECOMMENDED_MAX_ELEMENTS`], então pedir 10.000 cópias de um stream
/// grande devolve menos cópias, nunca uma explosão. Medido pela porta do produto
/// (`measure_the_count_ceiling`, fonte de 100 instâncias):
///
/// | cópias | instâncias | cook |
/// |---|---|---|
/// | 100 | 10.000 | 0,010 ms |
/// | 1.000 | 100.000 | 0,102 ms |
/// | **10.000** | **1.000.000** | **4,048 ms** |
///
/// Um milhão de instâncias por clonagem custa **24% de um quadro de 60 fps**, e são 312× o que o
/// slider alcança.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "count",
    max: 10_000.0,
}];

/// Param UI hints (M1.P1) for the clone rows: whole-number copy count, a polar
/// step (distance + free angle in degrees), and a centre toggle.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "distance",
        label: "Distance",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "center",
        label: "Center",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ **A faixa é `0..2`, e o `1` fica no MEIO do curso** — o número é *quanto a última
    // cópia mede*, então metade do curso encolhe e a outra metade cresce, simétricas em torno
    // do literal. Um teto de `1` faria do knob um afunilador só, e a fila que ABRE (o cone, o
    // megafone) é metade do que a referência lista como caso de uso.
    ParamUiHint {
        param: SCALE_TAPER,
        label: "Scale Taper",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // A volta ACUMULADA até a última cópia, em graus. `±360` é o mesmo curso do `angle` deste
    // nó — uma volta inteira para cada lado.
    ParamUiHint {
        param: ROT_TAPER,
        label: "Rot Taper",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "distance",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.clone.test.src"),
        name: "motion.clone.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionClone),
                _ => None,
            }
        }
    }

    /// Cook `motion.clone` on the 1-element source, applying `setup` to its
    /// params, and return the output `P` column.
    fn clone_p(setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId)) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.clone.test.src");
        let clone = g.add_node("motion.clone");
        g.connect(Edge {
            from: (src, 0),
            to: (clone, 0),
            delayed: false,
        })
        .unwrap();
        setup(&mut g, clone);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, clone, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn multiplies_stream_with_per_copy_offset() {
        // 1 instance × default count 3, distance 2 (angle 0 → +X) → x=0,2,4.
        let p = clone_p(|_, _| {});
        assert_eq!(p, vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]]);
    }

    #[test]
    fn per_instance_overrides_drive_clone_through_the_cook() {
        // Authoring path: override count → 2, distance → 5, on a 1-element source
        // → 2 copies at x = 0, 5 (vs the default count 3, distance 2).
        let p = clone_p(|g, clone| {
            g.set_param(clone, "count", 2.0);
            g.set_param(clone, "distance", 5.0);
        });
        assert_eq!(p, vec![[0.0, 0.0], [5.0, 0.0]]);
    }

    #[test]
    fn centered_queue_balances_copies_on_the_original() {
        // count 3, distance 2, centre on → ranks −1,0,1 → x = −2, 0, 2 (the
        // original element sits at rank 0, unmoved, with a copy each side).
        let p = clone_p(|g, clone| {
            g.set_param(clone, "count", 3.0);
            g.set_param(clone, "distance", 2.0);
            g.set_param(clone, "center", 1.0);
        });
        assert_eq!(p, vec![[-2.0, 0.0], [0.0, 0.0], [2.0, 0.0]]);
    }

    #[test]
    fn polar_angle_rotates_the_step_axis() {
        // angle 90° → step direction +Y: count 3, distance 2 → y = 0, 2, 4.
        let p = clone_p(|g, clone| {
            g.set_param(clone, "angle", 90.0);
            g.set_param(clone, "distance", 2.0);
        });
        for (i, expected) in [0.0f32, 2.0, 4.0].into_iter().enumerate() {
            assert!(p[i][0].abs() < 1e-5, "x stays ~0 (pure +Y step)");
            assert!((p[i][1] - expected).abs() < 1e-5, "y = {expected}");
        }
    }

    #[test]
    fn a_360_degree_angle_is_the_plus_x_axis_again() {
        // The degrees→cycles edge is exact: 360° wraps to a whole cycle, so the
        // step axis returns to +X (guards the `deg / 360` divisor).
        let p = clone_p(|g, clone| {
            g.set_param(clone, "angle", 360.0);
            g.set_param(clone, "distance", 2.0);
        });
        assert!(
            (p[1][0] - 2.0).abs() < 1e-5 && p[1][1].abs() < 1e-5,
            "back to +X"
        );
    }

    #[test]
    fn copy_rank_is_balanced_when_centered() {
        // off: 0,1,2 ; on (k=3): −1,0,1 ; on (k=4): −1.5,−0.5,0.5,1.5.
        assert_eq!([0, 1, 2].map(|c| copy_rank(c, 3, false)), [0.0, 1.0, 2.0]);
        assert_eq!([0, 1, 2].map(|c| copy_rank(c, 3, true)), [-1.0, 0.0, 1.0]);
        assert_eq!(
            [0, 1, 2, 3].map(|c| copy_rank(c, 4, true)),
            [-1.5, -0.5, 0.5, 1.5]
        );
    }

    #[test]
    fn index_and_count_are_renumbered_continuous_across_copies() {
        // 2 input elements (Index 0,1 / Count 2) × 3 copies → one uninterrupted
        // Index 0..5 and a Count of 6 everywhere, so a downstream ramp spans the
        // whole set instead of restarting per copy.
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
            .with("Index", Column::Scalar(vec![0.0, 1.0]))
            .with("Count", Column::Scalar(vec![2.0, 2.0]));
        let out = clone_stream(&input, 3, 5.0, 0.0, false, 1.0, 0.0);
        match out.get("Index").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
            _ => panic!("Index"),
        }
        match out.get("Count").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![6.0; 6]),
            _ => panic!("Count"),
        }
    }

    #[test]
    fn clone_stream_aligns_p_offset_with_replicated_columns() {
        // The riskiest invariant: a *second* column (here `tint`) must stay
        // aligned with `P` element-for-element across copies (copy-major order).
        // 2 input elements × 2 copies, step (10, 0), centre off.
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
            .with(
                "tint",
                Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]),
            );
        let out = clone_stream(&input, 2, 10.0, 0.0, false, 1.0, 0.0);
        assert_eq!(out.count(), 4);
        // copy 0: elements at x=0,1 ; copy 1: same elements + (10,0).
        match out.get("P").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v, &vec![[0.0, 0.0], [1.0, 0.0], [10.0, 0.0], [11.0, 0.0]]);
            }
            _ => panic!("P"),
        }
        // tint of element e in copy c sits at index c*in_count + e, with the
        // SAME color it had in the input — proving offset and replicate share
        // copy-major order (a mismatch here is the silent-misalignment bug).
        match out.get("tint").unwrap() {
            Column::Vec4(v) => assert_eq!(
                v,
                &vec![
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0],
                ]
            ),
            _ => panic!("tint"),
        }
    }

    #[test]
    fn copies_within_budget_caps_and_floors() {
        // floors at 1 copy even if 0 requested (cloner is ≥ passthrough).
        assert_eq!(copies_within_budget(0, 10, 1000), 1);
        // honors the request when it fits.
        assert_eq!(copies_within_budget(3, 10, 1000), 3);
        // clamps so in_count * k ≤ max: 10 elements, max 25 → at most 2 copies.
        assert_eq!(copies_within_budget(99, 10, 25), 2);
        // empty input: no division by zero, output will be empty anyway.
        assert_eq!(copies_within_budget(5, 0, 1000), 5);
        // input ALREADY over budget (in_count > max): still ≥ 1 copy
        // (passthrough), never 0 — the cloner does not drop a stream it cannot
        // grow. `in_count * 1` does not overflow (no multiplication grows it).
        assert_eq!(copies_within_budget(5, 1001, 1000), 1);
    }
}

#[cfg(test)]
#[path = "taper_tests.rs"]
mod taper_tests;
