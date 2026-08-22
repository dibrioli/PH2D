//! ADR-0154 — the shell half of `source.shape`: **build each shape's `VecPath`
//! from its graph node's params, publish it into the cook under the content key
//! the node reads, and draw the resulting instances as LIVE GPU vector.**
//!
//! A node is handed only its params, its inputs and the playhead — it cannot reach
//! the vector library or the GPU (the property that lets the cook memoize and
//! replay bit-exactly). So the shell is where a shape *descriptor* becomes
//! *geometry*: [`publish`] scans every `source.shape` node, reads its params
//! through the SAME door the node reads ([`ShapeParams::read`] over the ladder the
//! `EvalCtx` uses — driven, else override, else manifest default), builds the
//! `VecPath` once, interns it
//! under [`shape_key`], and sets a one-row instance stream `(P, geometry_id)`
//! external the node's `eval` clones. Identical descriptors share ONE `VecPath`
//! (the store is content-addressed), so a `motion.duplicator` stamping 10k stars
//! interns once. [`encode`] draws each cooked instance into the shared vector
//! scene, so a shape composites behind the chrome and over the sprites (Fase 1).
//!
//! ⚠️ **A camada CONDUZIDA da escada foi medida, não deduzida** (2026-08-21): esta
//! nota dizia *"o nó não tem entradas, então o `ctx.param` dele não tem camada
//! conduzida com que discordar"* — e a frase confundia **porta de entrada** com
//! **param conduzido**, que é um fio para um PARAM e não precisa de porta nenhuma.
//! Com `trim_end` conduzido o cook devolvia contagem **0** e a forma desaparecia em
//! silêncio. O mecanismo e a cura estão em
//! [`super::motion_externals::driven_params`].

use ph2d_eval_motion::VectorInstance;
use ph2d_node_motion_shape::{MANIFEST, ShapeKind, ShapeParams, shape_key};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_vec_scene::{ShapeKind as VecKind, VecPath, cook};
#[cfg(test)]
use ph2d_vec_scene::{ellipse, gear, heart, regular_polygon_rounded, rounded_rect, star_rounded};
use ph2d_vector::{Affine, VectorScene};

use crate::motion_state::MotionState;

/// O cache de geometria, no irmão que o teto de LOC pediu. Re-exportado sob o nome de
/// sempre: `motion_shape_gen::VecPathStore` continua a resolver, então nenhum chamador
/// mudou de endereço.
#[path = "motion_shape_store.rs"]
mod store;
pub(crate) use store::VecPathStore;

/// The manifest default for a param NAME — the fallback the node's `ctx.param`
/// takes when there is no override (and, for `source.shape`, no driven layer).
pub(crate) fn manifest_default(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.default)
        .unwrap_or(0.0)
}

// ⚠️ O `read_params` (o leitor por-`Option<&BTreeMap>`) MORREU aqui: o `publish`
// e os gates passaram a ler pelo mesmo GETTER que a chave usa, e um `pub(crate)`
// sem chamador não é código morto silencioso — é uma SEGUNDA porta esperando
// alguém a chamar, e ela leria os params sem a chave concordar.

/// **The translation** — this node's kind to the vector library's, and the box and
/// values `cook` wants. The ONE place the two orders meet.
///
/// ⚠️ The two enums are ordered DIFFERENTLY (`Circle = 0` here, `Rectangle = 0`
/// there) and this node's index is WIRE FORMAT, so a table is not tidiness: reading
/// one order as the other renames every shape in every saved graph. The `match` is
/// exhaustive by the compiler, which is what makes a dropdown label without
/// geometry behind it impossible to ship.
///
/// The first eight rows reproduce, value for value, what the node built before it
/// was wired to `cook()` — gated byte-for-byte against the frozen builder. The rest
/// take the library's own canonical proportions (`VecKind::defaults`), so a shape
/// arrives looking like itself; the knobs that tune each one are the next wave, and
/// `size`/`aspect` already drive the box every one of them is cut from.
fn vec_recipe(p: &ShapeParams) -> (VecKind, [f64; 2], [f64; 2], Vec<f64>) {
    let s = f64::from(p.size.max(0.01));
    let ry = s * f64::from(p.aspect.clamp(0.05, 20.0));
    let sides = f64::from(p.sides.clamp(3, 32));
    // `corner` is a FRACTION of the size -> a world-unit radius that scales.
    let corner = f64::from(p.corner.clamp(0.0, 1.0)) * s;
    let sq = ([-s, -s], [s, s]);
    let box_ = ([-s, -ry], [s, ry]);
    // ⚠️ **A SENTINELA do `sweep`** (doc 89 folha 14): `0` quer dizer *"como a forma nasce"*,
    // e não *"uma fatia de zero graus"*. Sem ela o default não reduz — o círculo passa `0`
    // (que a biblioteca lê como volta inteira) e a `Pie` passa o ângulo canónico dela.
    let swept = |k: VecKind, i: usize| {
        let v = f64::from(p.sweep);
        if v == 0.0 { k.defaults()[i] } else { v }
    };
    let start = f64::from(p.start);
    let inner = f64::from(p.inner.clamp(0.0, 0.99));
    // Os três desvios por canto, em unidades de MUNDO como o `corner` — a fracção é a
    // mesma escada, então mexer no `size` leva os quatro juntos.
    let off = |k: usize| f64::from(p.corner_offsets[k].clamp(-1.0, 1.0)) * s;
    let smoothing = f64::from(p.smoothing.clamp(0.0, 1.0));
    match p.kind {
        // ——— the eight that shipped, reproduced exactly ———
        // `ellipse_sweep` with sweep/start/inner all zero is the plain ellipse: the
        // sweep field reads "unset" as a full turn, which is what makes the legacy
        // circle survive the move.
        // ⚠️ O `defaults()[0]` da `Ellipse` é a volta INTEIRA, e a receita passava `0` —
        // que a biblioteca lê como o mesmo. A sentinela devolve `0` aqui, então o círculo
        // de todo documento anterior cozinha os MESMOS bits.
        ShapeKind::Circle => (
            VecKind::Ellipse,
            sq.0,
            sq.1,
            vec![
                if p.sweep == 0.0 {
                    0.0
                } else {
                    f64::from(p.sweep)
                },
                start,
                inner,
            ],
        ),
        ShapeKind::Ellipse => (
            VecKind::Ellipse,
            box_.0,
            box_.1,
            vec![
                if p.sweep == 0.0 {
                    0.0
                } else {
                    f64::from(p.sweep)
                },
                start,
                inner,
            ],
        ),
        // The three per-corner offsets and the smoothing were APPENDED to the
        // round-rect's values and are all neutral at zero, so `[r, 0, 0, 0, 0]` is
        // the uniform round-rect the node always built.
        ShapeKind::Square => (
            VecKind::RoundRect,
            sq.0,
            sq.1,
            vec![corner.min(s), off(0), off(1), off(2), smoothing],
        ),
        ShapeKind::Rectangle => (
            VecKind::RoundRect,
            box_.0,
            box_.1,
            vec![corner.min(s.min(ry)), off(0), off(1), off(2), smoothing],
        ),
        ShapeKind::Polygon => (VecKind::Polygon, sq.0, sq.1, vec![sides, corner]),
        ShapeKind::Star => (
            VecKind::Star,
            sq.0,
            sq.1,
            vec![
                sides,
                f64::from(p.star_depth.clamp(0.05, 0.95)),
                corner,
                corner,
            ],
        ),
        ShapeKind::Heart => (
            VecKind::Heart,
            sq.0,
            sq.1,
            vec![f64::from(p.cleft.clamp(0.02, 0.45))],
        ),
        ShapeKind::Gear => (
            VecKind::Gear,
            sq.0,
            sq.1,
            vec![
                sides,
                f64::from(p.tooth_depth.clamp(0.05, 0.6)),
                f64::from(p.hole.clamp(0.0, 1.0)),
            ],
        ),
        // ——— the catalogue, at the library's own proportions ———
        other => {
            let k = match other {
                ShapeKind::Pie => VecKind::Pie,
                ShapeKind::Segment => VecKind::Segment,
                ShapeKind::ArrowRight => VecKind::ArrowRight,
                ShapeKind::ArrowDouble => VecKind::ArrowDouble,
                ShapeKind::ArrowBent => VecKind::ArrowBent,
                ShapeKind::Chevron => VecKind::Chevron,
                ShapeKind::Diamond => VecKind::Diamond,
                ShapeKind::Pill => VecKind::Pill,
                ShapeKind::Parallelogram => VecKind::Parallelogram,
                ShapeKind::Trapezoid => VecKind::Trapezoid,
                ShapeKind::TrapezoidFlip => VecKind::TrapezoidFlip,
                ShapeKind::HexagonFlat => VecKind::HexagonFlat,
                ShapeKind::Cylinder => VecKind::Cylinder,
                ShapeKind::Document => VecKind::Document,
                ShapeKind::Delay => VecKind::Delay,
                ShapeKind::Display => VecKind::Display,
                ShapeKind::PredefinedProcess => VecKind::PredefinedProcess,
                ShapeKind::OffPage => VecKind::OffPage,
                ShapeKind::Junction => VecKind::Junction,
                ShapeKind::SpeechRect => VecKind::SpeechRect,
                ShapeKind::SpeechOval => VecKind::SpeechOval,
                ShapeKind::Thought => VecKind::Thought,
                ShapeKind::Burst => VecKind::Burst,
                ShapeKind::Cloud => VecKind::Cloud,
                ShapeKind::Bolt => VecKind::Bolt,
                ShapeKind::Moon => VecKind::Moon,
                ShapeKind::Drop => VecKind::Drop,
                ShapeKind::Shield => VecKind::Shield,
                ShapeKind::Tag => VecKind::Tag,
                ShapeKind::Cross => VecKind::Cross,
                ShapeKind::Check => VecKind::Check,
                ShapeKind::Banner => VecKind::Banner,
                ShapeKind::IsoCube => VecKind::IsoCube,
                ShapeKind::IsoCone => VecKind::IsoCone,
                ShapeKind::IsoPyramid => VecKind::IsoPyramid,
                // The eight above are handled by name; this arm cannot be reached
                // for them, and the compiler is what keeps that true when a kind is
                // appended.
                _ => unreachable!("as oito primeiras tem braco proprio"),
            };
            // ⚠️ **A pizza e a corda deixam de correr no default.** Elas são a MESMA forma
            // da elipse com outros números (`ellipse_sweep`/`ellipse_chord`), e a folha 14
            // dizia exactamente isto: *"a FORMA chegou, o CONTROLO não"*. A `Segment` não
            // tem miolo — a corda não o computa —, então o `inner` fica de fora dela e é o
            // gate da tabela que o prova, mexendo no número.
            let mut v = k.defaults().to_vec();
            match k {
                VecKind::Pie => {
                    v[0] = swept(k, 0);
                    v[1] = start;
                    v[2] = inner;
                }
                VecKind::Segment => {
                    v[0] = swept(k, 0);
                    v[1] = start;
                }
                // ⚠️ **O raio de quina do balão vem em unidades de MUNDO, e é o ÚNICO do
                // catálogo assim** (medido: `the_unit_geometry_scaled_is_the_geometry_at_the_
                // authored_size` percorre as 43 espécies e só esta divergia, por 23%). Todos
                // os outros `defaults()` que a receita passa verbatim são frações, ângulos ou
                // contagens — livres de escala. Sem esta multiplicação a quina do balão não
                // acompanha o `size`: enorme num balão pequeno, imperceptível num grande. O
                // `0,12` da biblioteca é tomado sobre o semi-eixo unitário, então em `size = 1`
                // isto é byte-idêntico ao que sempre saiu.
                VecKind::SpeechRect => v[0] *= s,
                _ => {}
            }
            (k, box_.0, box_.1, v)
        }
    }
}

/// Build the `VecPath` for a shape descriptor (ADR-0154). World-unit geometry,
/// centred at the origin; the instance transform places, rotates and scales it.
///
/// Everything goes through `ph2d_vec_scene::cook`, which declares itself the single
/// door for parametric geometry — *"o `ShapeTool` e o re-cook da forma viva passam
/// os dois por aqui"* — and until now this node was the one caller that did not.
/// That is why 35 shapes the editor could already draw were unreachable from a
/// graph: not geometry missing, wiring missing.
/// As espécies cuja RECEITA já consome o `corner` (o raio entra nos `values` que o `cook`
/// recebe: os quatro raios do round-rect, o do polígono, a ponta e o vale da estrela).
///
/// ⚠️ **A lista existe para o arredondamento GERAL não passar duas vezes por elas** — o raio
/// já está na geometria quando ela sai do `cook`, e aplicá-lo outra vez comeria a quina que
/// acabou de nascer. Para as outras ~37 o `corner` era **inerte**, e é isso que a rota geral
/// cura (feedback do Enio, 2026-08-19: *"senti falta de controle das quinas de uma rosca
/// cortada e formas similares"*).
const CORNER_IN_THE_RECIPE: &[ShapeKind] = &[
    ShapeKind::Square,
    ShapeKind::Rectangle,
    ShapeKind::Polygon,
    ShapeKind::Star,
];

pub(crate) fn build_shape_path(p: &ShapeParams) -> VecPath {
    let (kind, a, b, v) = vec_recipe(p);
    let mut path = cook(kind, a, b, &v);
    // ⚠️ **O ARREDONDAMENTO GERAL** — a mesma máquina das Live Corners (ADR-0121), aplicada
    // depois do `cook` para as espécies cuja receita não tem onde receber um raio.
    //
    // ⚠️ **O raio é carimbado em TODOS os vértices, e isso é DERIVADO, não descuido:** o
    // `corner_setback` recusa uma quina colinear, e um vértice do meio de um arco é colinear
    // por construção — medido, a rosquinha e a elipse inteira saem com **zero** quinas
    // arredondadas enquanto a seta ganha 7 e a cruz 12. Uma lista de índices por espécie
    // apodreceria na primeira forma nova; esta regra não tem o que envelhecer.
    if !CORNER_IN_THE_RECIPE.contains(&p.kind) {
        let r = f64::from(p.corner.clamp(0.0, 1.0)) * f64::from(p.size.max(0.01));
        if r > 0.0 {
            let mut verts = path.verts.clone();
            for vt in &mut verts {
                vt.corner_radius = r;
            }
            if let Some(rounded) =
                ph2d_vec_scene::corner_live::round_authored_corners(&verts, path.closed)
            {
                path.verts = rounded;
            }
        }
    }
    // ⚠️ **O TRAÇO** (doc 89 folha 14, P0). O renderer já o desenha — o
    // `tessellate_shape_instance` ramifica em `fill.is_some() || stroke.is_some()`
    // desde sempre —, e o que faltava era o shell PÔ-LO aqui: o `cook` devolve o
    // caminho com os dois em `None`. `stroke_width = 0` ⇒ `None` ⇒ a forma que
    // sempre shipou, byte-idêntica.
    if let Some(st) = p.stroke {
        let mut spec = ph2d_vec_scene::StrokeSpec::new(rgba8(st.rgba), f64::from(st.width));
        // ⚠️ **O TRACEJADO já mora no `StrokeSpec`** (doc 89 folha 14) — `Some((dash, gap))`
        // em MÚLTIPLOS da largura, e `dash <= 0` é contínuo. O que faltava era o nó ter
        // onde dizê-lo: nenhuma linha de código nova desenha um tracejado.
        spec.dash = p.dash.map(|(d, g)| (f64::from(d), f64::from(g.max(0.0))));
        path.stroke = Some(spec);
    }
    // ⚠️ **O TRIM entra na PILHA de efeitos do caminho, nunca na geometria** (doc 89 folha
    // 14, a linha do *trim/dash*). O `cooked()` — que é quem o renderer já chama — corre a
    // pilha, então isto alcança de uma vez os contornos compostos (o furo da engrenagem, o
    // anel) sem uma linha de fiação por espécie. E o neutro `{0, 1, 0}` faz `run_stack`
    // devolver `None` ⇒ `Cow::Borrowed` ⇒ a forma de sempre, byte-idêntica.
    //
    // ⚠️ **NÃO é o `marker::trim_path`**, que a célula da folha 14 apontava: aquele recua as
    // pontas em unidades de MUNDO para dar lugar às setas e devolve um caminho FECHADO
    // intocado — inerte em 42 das 47 formas da biblioteca. Este mede por arco exato e ABRE
    // o contorno, que é o que faz um traço correr em torno de um círculo.
    let trim = ph2d_vec_scene::fx_trim::TrimSpec {
        start: f64::from(p.trim[0].clamp(0.0, 1.0)),
        end: f64::from(p.trim[1].clamp(0.0, 1.0)),
        offset: f64::from(p.trim[2]),
    };
    if !trim.is_neutral() {
        path.effects.push(ph2d_vec_scene::effect::FxEntry::new(
            ph2d_vec_scene::effect::PathEffect::Trim(trim),
        ));
    }
    path
}

/// `[f32; 4]` em 0..1 → o `Rgba8` que o `StrokeSpec` fala.
fn rgba8(c: [f32; 4]) -> ph2d_vec_scene::Rgba8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clampado a 0..1 e escalado a 0..255"
    )]
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    ph2d_vec_scene::Rgba8::new(ch(c[0]), ch(c[1]), ch(c[2]), ch(c[3]))
}

/// The builder EXACTLY as it shipped before the catalogue was wired to `cook()`,
/// frozen under `cfg(test)` as the oracle for "the eight that shipped still cook
/// what they cooked". A `pub(crate)` copy without the `cfg` would be a second door
/// waiting for a caller (the `warp_axis` / `serial_side` precedent).
#[cfg(test)]
pub(crate) fn build_shape_path_as_it_shipped(p: &ShapeParams) -> VecPath {
    let s = f64::from(p.size.max(0.01));
    let ry = s * f64::from(p.aspect.clamp(0.05, 20.0));
    let sides = p.sides.clamp(3, 32);
    let corner = f64::from(p.corner.clamp(0.0, 1.0)) * s;
    match p.kind {
        ShapeKind::Circle => ellipse([0.0, 0.0], s, s),
        ShapeKind::Ellipse => ellipse([0.0, 0.0], s, ry),
        ShapeKind::Square => rounded_rect([-s, -s], [s, s], corner.min(s)),
        ShapeKind::Rectangle => rounded_rect([-s, -ry], [s, ry], corner.min(s.min(ry))),
        ShapeKind::Polygon => regular_polygon_rounded([0.0, 0.0], s, s, sides, corner),
        ShapeKind::Star => {
            let ratio = f64::from(p.star_depth.clamp(0.05, 0.95));
            star_rounded([0.0, 0.0], s, s, sides, ratio, corner, corner)
        }
        ShapeKind::Heart => heart([-s, -s], [s, s], f64::from(p.cleft.clamp(0.02, 0.45))),
        ShapeKind::Gear => {
            let depth = f64::from(p.tooth_depth.clamp(0.05, 0.6));
            let hole = f64::from(p.hole.clamp(0.0, 1.0));
            gear([-s, -s], [s, s], f64::from(sides), depth, hole)
        }
        _ => unreachable!("o oraculo so conhece as oito que shipavam"),
    }
}

/// Publish every `source.shape` node's geometry into the cook (ADR-0154).
///
/// ⚠️ **Called from the bridge POST-drain, pre-cook** — after `apply_graph_intents`
/// has applied this frame's param edits and before the pump cooks. Publishing it
/// earlier (before the drain) would set the PRE-edit key while the cook reads the
/// POST-edit key ⇒ the node clones an empty external for one frame ⇒ the shape
/// vanishes = **flicker on edit**. It only ADDS (the drawn-curve publish already
/// ran `clear_externals`), under the `shape:` key namespace the node reads.
pub(crate) fn publish(motion: &mut MotionState, seconds: f64) {
    let ids: Vec<ph2d_nodegraph::graph::NodeId> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == MANIFEST.name)
        .map(|n| n.id)
        .collect();
    // Collect the (key, params) jobs first so the graph borrow drops before we
    // mutate the store and the cook (three disjoint fields of `MotionState`).
    let mut jobs: Vec<(String, ShapeParams, f32)> = Vec::with_capacity(ids.len());
    for id in ids {
        let driven = super::motion_externals::driven_params(motion, id, seconds);
        let graph = &motion.doc.graph;
        // ⚠️ A chave e o descritor leem pelo MESMO getter — é o que faz a
        // chave do shell e a do nó serem os mesmos bits. E os dois passam pela
        // NORMALIZAÇÃO (`unit_value`), então a chave nomeia a geometria em raio 1
        // que o `build_shape_path` de facto constrói.
        let ov = graph.node_param_overrides(id);
        // ⚠️ **A escada é a MESMA do `EvalCtx::param`: conduzido → override → default.**
        // Ver [`driven_params`] para o porquê de o shell ter de resolver o conduzido.
        let get = |name: &str| {
            driven
                .get(name)
                .copied()
                .or_else(|| ov.and_then(|m| m.get(name).copied()))
                .unwrap_or_else(|| manifest_default(name))
        };
        let (unit, scale) = ShapeParams::read_unit(get);
        jobs.push((shape_key(get), unit, scale));
    }
    for (key, p, scale) in jobs {
        let handle = motion.shape_store.intern(&key, || build_shape_path(&p));
        // One-row instance at the origin, carrying the geometry handle. The
        // duplicator/deformers move, rotate, scale and tint it downstream; a bare
        // shape is white at the origin. `rot`/`tint` are absent ⇒ their stream
        // defaults (0 / white).
        //
        // ⚠️ **O `size` É a coluna** (doc 89 folha 14): a geometria interna vive em raio
        // 1 e o tamanho autorado viaja aqui, onde o `lower_to_vector_instances_onto` já o
        // lê (`vec2_at(size, i, [1, 1])`) e a pose da instância o aplica. Duas
        // consequências, e as duas eram o item: animar o `size` deixa de internar um
        // `VecPath` por valor visitado, e um `value.attribute("size")` a jusante passa a
        // VER o tamanho da forma — antes a coluna não existia.
        let stream = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("size", Column::Vec2(vec![[scale, scale]]))
            .with("geometry_id", Column::Scalar(vec![handle as f32]));
        motion.pump.cook.set_external(key, stream);
    }
}

/// Draw each cooked shape instance into the shared vector scene (ADR-0154). The
/// instance's `geometry_id` names a stored `VecPath`; its world pose (`P`, `basis`,
/// `size`) composes with the world→screen `cam` into the draw transform, and its
/// `tint` paints it — so one stored path serves N differently-tinted copies.
///
/// ⚠️ **N instances of ONE geometry pay ONE tessellation, not N** — the 160k-star
/// freeze (report 2026-08-05). This is a thin adapter over the crate's batch door
/// [`ph2d_vec_render::draw_shared_instances`], which caches the tessellated
/// `BezPath` per `geometry_id` for the frame (gated by build-count there). The
/// shell only maps `VectorInstance → (handle, transform, tint)` and hands the store
/// as the resolver.
pub(crate) fn encode(
    insts: &[VectorInstance],
    store: &VecPathStore,
    cam: Affine,
    scene: &mut VectorScene,
) {
    ph2d_vec_render::draw_shared_instances(
        insts.iter().map(|inst| {
            let [b0, b1, b2, b3] = inst.basis;
            let [sx, sy] = inst.size;
            let [px, py] = inst.world_pos;
            // world pose = translate(P) · R(basis) · scale(size); kurbo Affine coeffs
            // [a,b,c,d,e,f] = matrix [[a,c,e],[b,d,f]] ⇒ linear = R·S, translation = P.
            let pose = Affine::new([
                f64::from(b0 * sx),
                f64::from(b1 * sx),
                f64::from(b2 * sy),
                f64::from(b3 * sy),
                f64::from(px),
                f64::from(py),
            ]);
            (inst.geometry_id, cam * pose, inst.tint)
        }),
        |handle| store.get(handle),
        scene,
    );
}

#[cfg(test)]
#[path = "motion_shape_gen_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_shape_catalogue_tests.rs"]
mod catalogue_tests;

#[cfg(test)]
#[path = "motion_shape_style_tests.rs"]
mod style_tests;
