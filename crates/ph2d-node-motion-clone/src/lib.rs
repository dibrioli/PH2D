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

mod radial;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

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
        // **O LEQUE** — ver [`radial`]. O default é o NOME do modo, não um `0` solto.
        ParamSpec {
            name: "mode",
            default: radial::MODE_LINEAR as f32,
        },
        // O SETOR que as cópias repartem, em graus. `360` ⇒ o círculo inteiro.
        ParamSpec {
            name: "arc",
            default: 360.0,
        },
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
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
/// **O MODO de disposição** — ver [`radial`]. `Linear` (o default) é a fila em recta;
/// `Radial` é o leque em torno de um pivô, a capacidade que a folha 04 pedia ao cloner e que
/// nenhum nó entregava.
const MODE: &str = "mode";
/// **O SETOR** que as `k` cópias repartem, em graus — ver [`radial`]. `360` é o círculo
/// inteiro (e a lei do `motion.kaleidoscope`); um valor menor faz o leque, e negativo
/// inverte-lhe o sentido.
const ARC: &str = "arc";
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
/// A fila em recta com o passo JÁ resolvido (`sx`, `sy`) — a assinatura que os gates desta
/// crate usavam antes do leque existir, mantida para eles.
///
/// ⚠️ `#[cfg(test)]`: ela é uma projecção de [`clone_stream`] e **não** uma segunda lei — mas
/// uma projecção sem chamador de produção é exactamente a forma que vira uma segunda resposta
/// no dia em que alguém pega no nome mais curto. Os gates do modo novo entram pelo
/// [`clone_stream`], como o `eval`.
#[cfg(test)]
fn clone_row(
    input: &Stream,
    k: usize,
    sx: f32,
    sy: f32,
    center: bool,
    scale_taper: f32,
    rot_taper: f32,
) -> Stream {
    let place = |copy: usize| {
        let rank = copy_rank(copy, k, center);
        radial::Placement::Linear {
            dx: rank * sx,
            dy: rank * sy,
        }
    };
    clone_stream(input, k, &place, scale_taper, rot_taper)
}

fn clone_stream(
    input: &Stream,
    k: usize,
    place: &dyn Fn(usize) -> radial::Placement,
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
                    // ⚠️ A colocação é resolvida UMA vez por cópia, fora do laço dos
                    // elementos: em `Radial` ela carrega um `cos`/`sin`, e um por elemento
                    // seria o mesmo número recalculado `in_count` vezes.
                    let pl = place(copy);
                    for p in v {
                        nv.push(pl.apply(*p));
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
        let angle = ctx.param("angle");
        let center = ctx.param("center") >= 0.5; // Toggle: ≥0.5 → centred queue
        let (scale_taper, rot_taper) = (ctx.param(SCALE_TAPER), ctx.param(ROT_TAPER));
        // **O LEQUE** — ver [`radial`]. `Linear` é o default e corre pela expressão de sempre.
        let radial = ctx.param(MODE).round() as i32 == radial::MODE_RADIAL;
        let arc = ctx.param(ARC);
        let pivot = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        // `count` from an `f32` param: total conversion (non-finite/negative →
        // 0) then clamped so `in_count * k` cannot overflow the allocation; at
        // least one copy (passthrough).
        let requested = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let input = ctx.input(0);
        let k = copies_within_budget(requested, input.count(), RECOMMENDED_MAX_ELEMENTS);
        let step = radial::step_deg(arc, k);
        let place = |copy: usize| {
            radial::Placement::of(
                radial,
                copy_rank(copy, k, center),
                step,
                angle,
                distance,
                pivot,
            )
        };
        let out = clone_stream(input, k, &place, scale_taper, rot_taper);
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
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
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
    // ⚠️ **Primeiro, porque ele decide o que os outros QUEREM DIZER** (a fila ou o leque).
    ParamUiHint {
        param: MODE,
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: radial::MODE_LABELS,
        },
    },
    // ⚠️ **Vivo nos DOIS modos**, e é isso que o mantém fora da caça aos knobs mortos: em
    // `Linear` é o passo entre cópias, em `Radial` é o RAIO a que o leque as põe.
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
    // ⚠️ **O setor e o pivô só aparecem em `Radial`** (ver [`PARAM_GATES`]): num modo em que
    // não têm significado eles seriam três knobs a não fazer nada, que é o defeito que o
    // doc 90 mede.
    ParamUiHint {
        param: ARC,
        label: "Arc",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "pivot_x",
        label: "Pivot X",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_y",
        label: "Pivot Y",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
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
/// **Os três controles do leque só existem no modo que os lê** — ver [`radial`].
///
/// ⚠️ **O `distance` NÃO está aqui, de propósito**: ele é o passo em `Linear` e o raio em
/// `Radial`, vivo nos dois. Um gate sobre ele esconderia o knob mais usado do nó no modo em
/// que ele é o mais usado.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: ARC,
        when: MODE,
        values: &[radial::MODE_RADIAL],
    },
    ph2d_node_registry::ParamGate {
        param: "pivot_x",
        when: MODE,
        values: &[radial::MODE_RADIAL],
    },
    ph2d_node_registry::ParamGate {
        param: "pivot_y",
        when: MODE,
        values: &[radial::MODE_RADIAL],
    },
];

static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "distance",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
#[path = "taper_tests.rs"]
mod taper_tests;
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "radial_tests.rs"]
mod radial_tests;
