#![forbid(unsafe_code)]
//! `source.shape` — **generate a parametric vector shape** (ADR-0154).
//!
//! ⚠️ **The geometry is `ph2d_vec_scene::cook`, whose doc calls itself the single
//! door for parametric shapes** — *"o `ShapeTool` e o re-cook da forma viva passam
//! os dois por aqui, então o que se desenha e o que se guarda nunca divergem"* —
//! and this node was the one caller that built its own. That is why 35 shapes the
//! editor could already draw were unreachable from a graph: the geometry was never
//! missing, the wiring was. The shell owns the translation (this node cannot reach
//! the vector library, by design), and the catalogue is now the fillable half of
//! that crate's 47.
//!
//! The state of the art (Cavalry, After Effects shape layers, Blender Geometry
//! Nodes, Rive) treats a shape as **live vector geometry** that flows through the
//! graph and renders resolution-independent on GPU. This node is that door for
//! Motion: it emits **one instance carrying a `geometry_id`** — a handle to a
//! crisp `VecPath` the shell built from these params and drew through Vello. Cross
//! it with a `motion.grid` through a `motion.duplicator` and the shape is stamped,
//! crisp, at every point — with none of a baked tile's dead pixels (nothing
//! downstream can re-shape a raster; a `VecPath` composes with boolean/trim/deform).
//!
//! ## The same door `source.object` uses, one axis up
//!
//! A node is handed its params, its inputs and the playhead — nothing else (the
//! property that lets the cook memoize and replay bit-exactly), so it **cannot
//! reach the GPU or the vector library**. The shell builds the `VecPath` from
//! these params, registers it in its `VecPathStore`, and publishes a one-row
//! instance stream `(P, geometry_id, size, tint)` into the cook's external channel
//! under this shape's **content key** ([`shape_key`]); this node reads that key —
//! exactly as `source.object` reads an object by name. Identical descriptors share
//! ONE `VecPath` (content-addressed). A shape not yet published (a forward cook) is
//! an **empty external → empty stream**: the node emits nothing, never panics.
//!
//! `Effect::Pure` — the output is a pure function of the named external, whose
//! content-revision rides in the cook fingerprint, so editing a slider re-cooks
//! this node and only what is downstream of it.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **The shape family.** The index the `kind` param stores (`0.0..`) maps here; the
/// labels drive the `ParamWidget::Enum` dropdown.
///
/// ⚠️ **Order is the wire format for the `kind` index — APPEND ONLY.** A saved graph
/// stores the index, so moving one renames every shape an artist already chose. The
/// first eight are exactly the eight that shipped, in exactly their old positions,
/// and the thirty-five after them were appended: every document made before this
/// list grew cooks the shape it always cooked.
///
/// ⚠️ **And the shell's translation table is what makes this list real** — the node
/// is handed params and nothing else (that is what lets the cook memoize and replay
/// bit-exactly), so it cannot name `ph2d_vec_scene::ShapeKind`. The two orders
/// DIFFER (here `Circle = 0`; there `Rectangle = 0`), and a naive *"just use
/// `cook()`"* would silently renumber every saved graph. The mapping lives in
/// `motion_shape_gen`, is exhaustive by the compiler, and is gated.
///
/// The catalogue is the FILLABLE half of `ph2d_vec_scene::ALL_SHAPES`, and which
/// half that is was MEASURED rather than assumed (`which_shapes_close`): 42 of the
/// 47 close. The five that do not — **Spiral · Line · Arc · NoteBracket · Brace** —
/// need a stroke to be visible at all, and the fill-based draw entry has none; they
/// wait for the stroke wave. ⚠️ The fence this replaces named only two of those
/// five, so it was right about the class and short about the membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeKind {
    Circle,
    Square,
    Ellipse,
    Rectangle,
    Polygon,
    Star,
    Heart,
    Gear,
    // ——— appended when the catalogue was wired to `cook()` (doc 89 §14) ———
    Pie,
    Segment,
    ArrowRight,
    ArrowDouble,
    ArrowBent,
    Chevron,
    Diamond,
    Pill,
    Parallelogram,
    Trapezoid,
    TrapezoidFlip,
    HexagonFlat,
    Cylinder,
    Document,
    Delay,
    Display,
    PredefinedProcess,
    OffPage,
    Junction,
    SpeechRect,
    SpeechOval,
    Thought,
    Burst,
    Cloud,
    Bolt,
    Moon,
    Drop,
    Shield,
    Tag,
    Cross,
    Check,
    Banner,
    IsoCube,
    IsoCone,
    IsoPyramid,
}

/// The dropdown labels — index-aligned to [`ShapeKind`] (the `Enum` widget stores
/// the selected INDEX as the `kind` param's f32).
pub static KIND_LABELS: &[&str] = &[
    "Circle",
    "Square",
    "Ellipse",
    "Rectangle",
    "Polygon",
    "Star",
    "Heart",
    "Gear",
    "Pie",
    "Segment",
    "Arrow",
    "Arrow Double",
    "Arrow Bent",
    "Chevron",
    "Diamond",
    "Pill",
    "Parallelogram",
    "Trapezoid",
    "Trapezoid Flip",
    "Hexagon",
    "Cylinder",
    "Document",
    "Delay",
    "Display",
    "Process",
    "Off-Page",
    "Junction",
    "Speech Box",
    "Speech Oval",
    "Thought",
    "Burst",
    "Cloud",
    "Bolt",
    "Moon",
    "Drop",
    "Shield",
    "Tag",
    "Cross",
    "Check",
    "Banner",
    "Iso Cube",
    "Iso Cone",
    "Iso Pyramid",
];

/// Every kind, in WIRE ORDER — the one list `from_index`, the labels and the
/// shell's translation all index into. Index-aligned to [`KIND_LABELS`] by a gate:
/// a label with no kind behind it would be a dropdown row that draws nothing.
pub static ALL_KINDS: &[ShapeKind] = &[
    ShapeKind::Circle,
    ShapeKind::Square,
    ShapeKind::Ellipse,
    ShapeKind::Rectangle,
    ShapeKind::Polygon,
    ShapeKind::Star,
    ShapeKind::Heart,
    ShapeKind::Gear,
    ShapeKind::Pie,
    ShapeKind::Segment,
    ShapeKind::ArrowRight,
    ShapeKind::ArrowDouble,
    ShapeKind::ArrowBent,
    ShapeKind::Chevron,
    ShapeKind::Diamond,
    ShapeKind::Pill,
    ShapeKind::Parallelogram,
    ShapeKind::Trapezoid,
    ShapeKind::TrapezoidFlip,
    ShapeKind::HexagonFlat,
    ShapeKind::Cylinder,
    ShapeKind::Document,
    ShapeKind::Delay,
    ShapeKind::Display,
    ShapeKind::PredefinedProcess,
    ShapeKind::OffPage,
    ShapeKind::Junction,
    ShapeKind::SpeechRect,
    ShapeKind::SpeechOval,
    ShapeKind::Thought,
    ShapeKind::Burst,
    ShapeKind::Cloud,
    ShapeKind::Bolt,
    ShapeKind::Moon,
    ShapeKind::Drop,
    ShapeKind::Shield,
    ShapeKind::Tag,
    ShapeKind::Cross,
    ShapeKind::Check,
    ShapeKind::Banner,
    ShapeKind::IsoCube,
    ShapeKind::IsoCone,
    ShapeKind::IsoPyramid,
];

impl ShapeKind {
    /// Decode the `kind` param (a rounded, clamped index) — the ONE place the f32
    /// index becomes a kind, so the node and the shell can never disagree about
    /// which shape an index names.
    #[must_use]
    pub fn from_index(idx: f32) -> ShapeKind {
        let i = (idx.round() as i64).clamp(0, ALL_KINDS.len() as i64 - 1);
        ALL_KINDS[i as usize]
    }

    /// This kind's wire index — the inverse of [`from_index`], and the only place
    /// that direction is spelled. A `match` written by hand here is how the two
    /// directions drift apart the day a shape is appended.
    #[must_use]
    pub fn index(self) -> usize {
        self as usize
    }
}

/// **The whole shape descriptor** — a pure value the node emits (as a key) and the
/// shell reads (to build the `VecPath`). Each shape shows only the params it uses
/// (the `ParamGate`s below): a circle is just `size`, a gear adds `tooth_depth` +
/// `hole`. A few params are shared where the geometry genuinely is (`aspect` for
/// ellipse/rectangle, `sides` for polygon/star/gear, `corner` rounds boxes/polys/
/// star points); the rest are dedicated so every slider reads honestly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeParams {
    pub kind: ShapeKind,
    /// Primary size in world units (radius for round kinds, half-extent for boxes).
    pub size: f32,
    /// Height ÷ width — ellipse / rectangle stretch (`1.0` = round/square).
    pub aspect: f32,
    /// Sides / points / teeth — polygon / star / gear.
    pub sides: u32,
    /// Corner rounding as a FRACTION of `size` (`0` = sharp) — box/polygon/star.
    pub corner: f32,
    /// Star point depth: the inner-radius ratio `0.05..0.95` (small = spiky).
    pub star_depth: f32,
    /// Heart cleft: how deep the top dip sits (`0.02..0.45`).
    pub cleft: f32,
    /// Gear tooth depth as a fraction of the radius (`0.05..0.6`).
    pub tooth_depth: f32,
    /// Gear centre hole as a fraction of the root circle (`0` = solid).
    pub hole: f32,
    /// **O TRAÇO** — largura em unidades de mundo e cor RGBA. `width = 0` é o
    /// mundo de sempre: sem `StrokeSpec`, e a forma aparece só pelo preenchimento
    /// que o `tint` da instância dá.
    ///
    /// ⚠️ **A cor do traço é PRÓPRIA, e tem de ser.** O preenchimento de uma forma
    /// vem do `tint` da instância (é o que o `motion.tint` a jusante pinta), então
    /// um traço que herdasse essa cor seria **invisível** — a mesma tinta por cima
    /// dela mesma. É o controle que separa *forma* de *silhueta*.
    pub stroke: Option<Stroke>,
    /// A abertura do arco em graus — **`0` é a sentinela *"como a forma nasce"***
    /// (ver [`param::SWEEP`]).
    pub sweep: f32,
    /// Onde o arco começa, em graus (`0` = o default da biblioteca).
    pub start: f32,
    /// O raio interno como fracção do externo (`0` = maciço).
    pub inner: f32,
    /// Os três desvios de raio por canto, somados ao `corner` — `[TR, BR, BL]`.
    pub corner_offsets: [f32; 3],
    /// A suavização do canto (`0..1`, o *corner smoothing*).
    pub smoothing: f32,
    /// **O TRIM** — `(start, end, offset)` em frações do comprimento do contorno.
    /// `(0, 1, 0)` é o neutro, e o neutro é no-op byte-idêntico (ver [`param::TRIM_START`]).
    pub trim: [f32; 3],
    /// **O TRACEJADO** — `Some((dash, gap))` em múltiplos da largura do traço;
    /// `None` (ou `dash <= 0`) = contínuo, que é o traço de sempre.
    pub dash: Option<(f32, f32)>,
}

/// A largura e a cor do traço de uma forma.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub width: f32,
    pub rgba: [f32; 4],
}

/// The names of the f32 params — the ONE list the manifest, the UI hints, the
/// gates, the node's `eval` and the shell's reader all key off (no drifting
/// string literal).
pub mod param;

impl ShapeParams {
    /// Read the descriptor from any param source — the node passes
    /// `|n| ctx.param(n)`, the shell passes a closure over the graph's overrides +
    /// the manifest defaults. **One reader, two callers** ⇒ the node's key and the
    /// shell's key are the same f32 bits through the same code.
    #[must_use]
    pub fn read(get: impl Fn(&str) -> f32) -> ShapeParams {
        ShapeParams {
            kind: ShapeKind::from_index(get(param::KIND)),
            size: get(param::SIZE),
            aspect: get(param::ASPECT),
            // Clamp is deliberate here (not in `from_index`): the UI caps at 32, but
            // the builder decides the floor; keep the int stable so the key is exact.
            sides: (get(param::SIDES).round() as i64).clamp(3, 32) as u32,
            corner: get(param::CORNER),
            star_depth: get(param::STAR_DEPTH),
            cleft: get(param::CLEFT),
            tooth_depth: get(param::TOOTH_DEPTH),
            hole: get(param::HOLE),
            // `width <= 0` ⇒ `None`, e é o que torna o default byte-idêntico: a
            // ausência do `StrokeSpec` é a forma que sempre shipou, não um traço
            // de largura zero (que o tesselador ainda percorreria).
            stroke: (get(param::STROKE_WIDTH) > 0.0).then(|| Stroke {
                width: get(param::STROKE_WIDTH),
                rgba: [
                    get(param::STROKE_R),
                    get(param::STROKE_G),
                    get(param::STROKE_B),
                    get(param::STROKE_A),
                ],
            }),
            sweep: get(param::SWEEP),
            start: get(param::START),
            inner: get(param::INNER),
            corner_offsets: [
                get(param::CORNER_TR),
                get(param::CORNER_BR),
                get(param::CORNER_BL),
            ],
            smoothing: get(param::SMOOTHING),
            trim: [
                get(param::TRIM_START),
                get(param::TRIM_END),
                get(param::TRIM_OFFSET),
            ],
            // `dash <= 0` ⇒ `None`, o mesmo desenho do `stroke`: a AUSÊNCIA do
            // tracejado é o traço contínuo de sempre, não um tracejado de período
            // zero (que o plano de traço ainda percorreria).
            dash: (get(param::DASH) > 0.0).then(|| (get(param::DASH), get(param::DASH_GAP))),
        }
    }

    /// **O descritor em RAIO 1 e a ESCALA que ele deixa para a instância** (doc 89 folha 14,
    /// a linha do *`size` é GEOMETRIA, não coluna*).
    ///
    /// ⚠️ **É o que tira o `size` do cache de geometria.** O `size` entrava na
    /// [`shape_key`], então *"um slider animado re-interna um `VecPath` por valor
    /// visitado"* (o próprio doc do `VecPathStore` o admitia) — e nada a jusante que
    /// lesse a coluna `size` via o tamanho da forma. A receita inteira é **linear no
    /// `size`** (`ry = size·aspect`, e o `corner`/os três desvios são FRAÇÕES dele),
    /// então construí-la em raio 1 e publicar `size = escala` na instância dá a MESMA
    /// imagem — com uma entrada só no cache para toda a excursão do slider.
    ///
    /// ⚠️ **A largura do traço é dividida pela escala, e tem de ser:** ela é a única
    /// grandeza do descritor em unidades de MUNDO, e a pose da instância multiplica
    /// tudo o que ela leva. Sem a divisão, engordar o `size` engordaria o traço.
    #[must_use]
    pub fn read_unit(get: impl Fn(&str) -> f32) -> (ShapeParams, f32) {
        let scale = scale_of(&get);
        (ShapeParams::read(|n| unit_value(&get, n, scale)), scale)
    }
}

/// **A ESCALA que o descritor deixa para a instância** — o `size` clampado exactamente
/// como a receita do shell o lê (`size.max(0.01)`), para que os dois lados nunca dividam
/// por um número que o outro não usou.
#[must_use]
pub fn scale_of(get: impl Fn(&str) -> f32) -> f32 {
    get(param::SIZE).max(0.01)
}

/// O valor do param `name` no descritor **NORMALIZADO** — a UMA porta por onde a
/// [`shape_key`] e a [`ShapeParams::read_unit`] passam, e é isso que garante que a chave e
/// a geometria são o mesmo descritor. Tudo o mais no descritor é fração, ângulo ou
/// contagem: já é livre de escala.
fn unit_value(get: &impl Fn(&str) -> f32, name: &str, scale: f32) -> f32 {
    match name {
        param::SIZE => 1.0,
        param::STROKE_WIDTH => get(name) / scale,
        _ => get(name),
    }
}

/// **The content key** of a shape descriptor (ADR-0154) — the external name the
/// node reads and the shell publishes under. Content-addressed: identical
/// descriptors share ONE `VecPath` in the store. Deterministic and exact — the
/// same f32 bits format the same string, so the node's lookup and the shell's
/// publish cannot diverge (gated). Hashes the WHOLE descriptor: a hidden param
/// stays at its default for any shape an artist authors, so identical shapes still
/// share geometry, and no per-kind branch can drift from the reader.
///
/// ⚠️ **A chave é do descritor em RAIO 1** ([`ShapeParams::read_unit`]), não do descritor
/// cru: a geometria que ela nomeia é a normalizada, e o `size` viaja na coluna da
/// instância. É por isso que animar o `size` deixou de crescer o `VecPathStore` — e é
/// também por isso que duas formas de tamanhos diferentes **partilham** um `VecPath`.
#[must_use]
pub fn shape_key(get: impl Fn(&str) -> f32) -> String {
    let scale = scale_of(&get);
    let mut k = String::from("shape");
    for name in param::ALL {
        k.push(':');
        k.push_str(&unit_value(&get, name, scale).to_bits().to_string());
    }
    k
}

/// The static contract of this node type (ADR-0031). The `kind` param is an
/// enum index; the eight geometry params are f32 sliders, gated per-kind by
/// [`PARAM_GATES`]. Frozen `NodeManifest` (8 fields) untouched — these are
/// `ParamSpec`s (f32), not new fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.shape"),
    name: "source.shape",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: param::KIND,
            default: 0.0,
        },
        ParamSpec {
            name: param::SIZE,
            default: 1.0,
        },
        ParamSpec {
            name: param::ASPECT,
            default: 1.0,
        },
        ParamSpec {
            name: param::SIDES,
            default: 6.0,
        },
        ParamSpec {
            name: param::CORNER,
            default: 0.0,
        },
        ParamSpec {
            name: param::STAR_DEPTH,
            default: 0.45,
        },
        ParamSpec {
            name: param::CLEFT,
            default: 0.2,
        },
        ParamSpec {
            name: param::TOOTH_DEPTH,
            default: 0.35,
        },
        ParamSpec {
            name: param::HOLE,
            default: 0.45,
        },
        // ⚠️ **Apendados** (doc 89 folha 14, P0). `stroke_width = 0` ⇒ sem
        // `StrokeSpec` ⇒ a forma que sempre shipou, byte-idêntica.
        ParamSpec {
            name: param::STROKE_WIDTH,
            default: 0.0,
        },
        ParamSpec {
            name: param::STROKE_R,
            default: 0.0,
        },
        ParamSpec {
            name: param::STROKE_G,
            default: 0.0,
        },
        ParamSpec {
            name: param::STROKE_B,
            default: 0.0,
        },
        ParamSpec {
            name: param::STROKE_A,
            default: 1.0,
        },
        // ⚠️ **Apendados** (doc 89 folha 14, as linhas do *sweep/start/inner* e do *raio
        // por canto*). Todos neutros no default — o `sweep = 0` pela sentinela documentada
        // no [`param::SWEEP`], os outros seis porque `0` já é o valor que a biblioteca usa.
        ParamSpec {
            name: param::SWEEP,
            default: 0.0,
        },
        ParamSpec {
            name: param::START,
            default: 0.0,
        },
        ParamSpec {
            name: param::INNER,
            default: 0.0,
        },
        ParamSpec {
            name: param::CORNER_TR,
            default: 0.0,
        },
        ParamSpec {
            name: param::CORNER_BR,
            default: 0.0,
        },
        ParamSpec {
            name: param::CORNER_BL,
            default: 0.0,
        },
        ParamSpec {
            name: param::SMOOTHING,
            default: 0.0,
        },
        // ⚠️ **Apendados** (doc 89 folha 14, a linha do *trim/dash*). `{0, 1, 0}` é o
        // NEUTRO do `TrimSpec`, e a pilha de efeitos salta um efeito neutro por inteiro
        // ⇒ a forma que sempre shipou, byte-idêntica. O `dash_gap` nasce em `1` porque
        // um vão de zero seria um tracejado contínuo com custo — mas ele só existe
        // quando o `dash` sai do zero.
        ParamSpec {
            name: param::TRIM_START,
            default: 0.0,
        },
        ParamSpec {
            name: param::TRIM_END,
            default: 1.0,
        },
        ParamSpec {
            name: param::TRIM_OFFSET,
            default: 0.0,
        },
        ParamSpec {
            name: param::DASH,
            default: 0.0,
        },
        ParamSpec {
            name: param::DASH_GAP,
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct SourceShape;

impl NodeOp for SourceShape {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // ⚠️ A chave e o descritor leem pela MESMA porta — o `ctx.param` do nó —
        // e é isso que faz a chave do nó e a do shell serem os mesmos bits.
        let key = shape_key(|n| ctx.param(n));
        // The shell built this shape's `VecPath`, stored it, and published a
        // one-row instance stream `(P, geometry_id, size, tint)` under `key`.
        // Clone is refcount (Arc columns); a key with no published shape (a
        // forward cook, before the shell's publish pass ran) is the empty
        // external → an empty stream ⇒ nothing drawn, no panic.
        let stream = ctx.external(&key).clone();
        ctx.emit(stream);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceShape))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Shape",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_gates_above(MANIFEST.id, PARAM_GATES_ABOVE);
    // Its output carries `geometry_id` (a live vector shape) drawn by the vector
    // pass. The GPU-resident cook has no `geometry_id` route, so a document
    // bringing a shape in recuses to the CPU render (ADR-0154/0155). Unlike an
    // OBJECT source (`texture_id`), this has no device render path at all.
    reg.register_live_vector_source(MANIFEST.id);
    Ok(())
}

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
    param: "size",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

mod hints;
use hints::{PARAM_GATES, PARAM_GATES_ABOVE, PARAM_HINTS};
