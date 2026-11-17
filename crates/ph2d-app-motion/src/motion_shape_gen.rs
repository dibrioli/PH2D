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

#[path = "motion_shape_mistura.rs"]
pub(crate) mod mistura;
pub use store::VecPathStore;

/// Os dois raios de colisão de uma geometria (doc 109), no irmão que o tecto de LOC pede.
#[path = "motion_shape_collider.rs"]
pub mod collider;

/// The manifest default for a param NAME — the fallback the node's `ctx.param`
/// takes when there is no override (and, for `source.shape`, no driven layer).
/// ⚠️ **Só os TESTES o chamam desde 2026-08-28** — o produto lê a escada inteira pela porta
/// (`motion_externals::resolved_params`), que já resolve o default. Ele fica porque duas suítes
/// de forma perguntam *«qual é o default deste knob?»* sem quererem a escada, e sai do binário
/// para não ser código morto lá dentro.
#[cfg(test)]
pub fn manifest_default(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.default)
        .unwrap_or(0.0)
}

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
    let corner = f64::from(p.corner.clamp(0.0, 1.0)) * s;
    let sq = ([-s, -s], [s, s]);
    let box_ = ([-s, -ry], [s, ry]);
    let rig_box = ([0.0, -ry], [2.0 * s, ry]);
    let swept = |k: VecKind, i: usize| {
        let v = f64::from(p.sweep);
        if v == 0.0 { k.defaults()[i] } else { v }
    };
    let start = f64::from(p.start);
    let inner = f64::from(p.inner.clamp(0.0, 0.99));
    let off = |k: usize| f64::from(p.corner_offsets[k].clamp(-1.0, 1.0)) * s;
    let smoothing = f64::from(p.smoothing.clamp(0.0, 1.0));
    let (k, a, b, v) = match p.kind {
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
                ShapeKind::Bone => VecKind::Bone,
                ShapeKind::RopeSegment => VecKind::RopeSegment,
                _ => unreachable!("as oito primeiras tem braco proprio"),
            };
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
                VecKind::SpeechRect => v[0] *= s,
                VecKind::Bone => v[0] = f64::from(p.hole.clamp(0.0, 1.0)),
                VecKind::RopeSegment if p.inner != 0.0 => v[0] = inner,
                _ => {}
            }
            let caixa = match k {
                VecKind::Bone | VecKind::RopeSegment => rig_box,
                _ => box_,
            };
            (k, caixa.0, caixa.1, v)
        }
    };
    let desloca = |f: f32, lo: f64, hi: f64| f64::from(f) * (hi - lo) * 0.5;
    let (dx, dy) = (
        desloca(p.pivot[0], a[0], b[0]),
        desloca(p.pivot[1], a[1], b[1]),
    );
    (k, [a[0] + dx, a[1] + dy], [b[0] + dx, b[1] + dy], v)
}

/// Build the `VecPath` for a shape descriptor (ADR-0154). World-unit geometry; the
/// instance transform places, rotates and scales it.
///
/// ⚠️ **Nem toda forma é centrada na origem, e a excepção é uma LEI:** os dois símbolos de rig
/// (`Bone`, `RopeSegment`) são cortados de `[0, 2s]` para que a origem local seja a CABEÇA — ver
/// o `rig_box` do [`vec_recipe`]. *Escrever «centrada na origem» aqui era a promessa que o
/// report do dono de 19/09 desmentiu.*
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

pub fn build_shape_path(p: &ShapeParams) -> VecPath {
    let (kind, a, b, v) = vec_recipe(p);
    let mut path = cook(kind, a, b, &v);
    if !CORNER_IN_THE_RECIPE.contains(&p.kind) {
        let r = f64::from(p.corner.clamp(0.0, 1.0)) * f64::from(p.size.max(0.01));
        path = ph2d_vec_scene::round_shape_corners(path, r);
    }
    if let Some(st) = p.stroke {
        let mut spec = ph2d_vec_scene::StrokeSpec::new(rgba8(st.rgba), f64::from(st.width));
        spec.dash = p
            .dash
            .map(|(d, g)| ph2d_vec_scene::DashPattern::single(f64::from(d), f64::from(g.max(0.0))));
        (spec.cap, spec.join) = mistura::traco::ponta_e_junta(st);
        path.stroke = Some(spec);
    }
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
/// what they cooked". A `pub` copy without the `cfg` would be a second door
/// waiting for a caller (the `warp_axis` / `serial_side` precedent).
#[cfg(test)]
pub fn build_shape_path_as_it_shipped(p: &ShapeParams) -> VecPath {
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
pub fn publish(motion: &mut MotionState, seconds: f64) {
    let ids: Vec<ph2d_nodegraph::graph::NodeId> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == MANIFEST.name)
        .map(|n| n.id)
        .collect();
    let mut jobs: Vec<(String, ShapeParams, f32)> = Vec::with_capacity(ids.len());
    for id in ids {
        let resolved = super::motion_externals::resolved_params(motion, id, seconds, &MANIFEST);
        let get = |name: &str| resolved.get(name).copied().unwrap_or(0.0);
        let (unit, scale) = ShapeParams::read_unit(get);
        jobs.push((shape_key(get), unit, scale));
    }
    for (key, p, scale) in jobs {
        let handle = motion.shape_store.intern(&key, || build_shape_path(&p));
        let fit = motion.shape_store.collider_fit(handle);
        let stream = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("size", Column::Vec2(vec![[scale, scale]]))
            .with("geometry_id", Column::Scalar(vec![handle as f32]))
            .with(
                ph2d_node_motion_shape::param::COLLIDER_FIT_CENTER_COL,
                Column::Vec2(vec![fit.center]),
            )
            .with(
                ph2d_node_motion_shape::param::COLLIDER_FIT_HALF_COL,
                Column::Vec2(vec![fit.half]),
            );
        motion.pump.cook.set_external(key, stream);
    }
}

/// Draw each cooked shape instance into the shared vector scene (ADR-0154). The
/// instance's `geometry_id` names a stored `VecPath`; its world pose (`P`, `basis`,
/// `size`) composes with the world→screen `cam` into the draw transform, and its
/// `tint` paints it — so one stored path serves N differently-tinted copies.
///
/// ⭐⭐⭐ **E desde 2026-09-21 uma cópia que a câmara não mostra NÃO é entregue à placa** (ordem
/// do dono, depois de ele medir que `72 900` estrelas arredondadas não cabem num quadro). A
/// `janela` é o rectângulo do ALVO DE RENDER, em píxeis — ⛔ **não** a banda do canvas: a cena
/// vectorial é rasterizada ao tamanho da janela inteira e o chrome pinta-se por cima, logo fora
/// do alvo uma forma é invisível **por construção**, e recortar ali não pode estar errado.
/// `None` desliga o recorte.
///
/// ⭐⭐ **E o `filtro` é o do PROJECTO** (doc 118 §8): um quad de IMAGEM cujo sink diz `Inherit`
/// amostra como a sprite amostraria — `PixelArt` nítido, `Smooth` bilinear. Só a imagem o lê.
///
/// ⚠️ **N instances of ONE geometry pay ONE tessellation, not N** — the 160k-star
/// freeze (report 2026-08-05). This is a thin adapter over the crate's batch door
/// [`ph2d_vec_render::draw_shared_instances`], which caches the tessellated
/// `BezPath` per `geometry_id` for the frame (gated by build-count there). The
/// shell only maps `VectorInstance → (handle, transform, tint)` and hands the store
/// as the resolver.
pub fn encode(
    insts: &[VectorInstance],
    store: &VecPathStore,
    art: &mut dyn FnMut(u32, [f32; 4]) -> Option<crate::motion_leaf_images::Art>,
    cam: Affine,
    janela: Option<ph2d_vector::Rect>,
    filtro: ph2d_render::ImageFilterMode,
    scene: &mut VectorScene,
) {
    let mut traco = mistura::traco::TracoDaPlaca::default();
    let mut i = 0;
    while i < insts.len() {
        let chave = mistura::chave_de_mistura(&insts[i].mistura);
        let mut j = i + 1;
        while j < insts.len() && mistura::chave_de_mistura(&insts[j].mistura) == chave {
            j += 1;
        }
        mistura::desenha_corrida(
            &mut traco,
            &insts[i..j],
            chave,
            store,
            art,
            cam,
            janela,
            filtro,
            scene,
        );
        i = j;
    }
}

/// **A pose de mundo de uma instância vectorial**, com a câmera já aplicada.
///
/// ⚠️ Ela é uma função com nome porque a conta vivia DENTRO de um `map` sobre um iterador
/// consumido pelo `draw_shared_instances`, e medi-la de fora exigiria uma cena Vello e um
/// raster. *Uma lei que só se pode medir renderizando não é medida.* O gate do pivô chama
/// esta função — o `encode` chama a MESMA, não há segunda cópia da conta.
pub(crate) fn instance_pose(inst: &VectorInstance, cam: Affine) -> Affine {
    {
        {
            let [b0, b1, b2, b3] = inst.basis;
            let [sx, sy] = inst.size;
            let [px, py] = inst.world_pos;
            let [ax, ay] = inst.anchor;
            let pose = Affine::new([
                f64::from(b0 * sx),
                f64::from(b1 * sx),
                f64::from(b2 * sy),
                f64::from(b3 * sy),
                f64::from(px + b0 * ax + b2 * ay),
                f64::from(py + b1 * ax + b3 * ay),
            ]);
            cam * pose
        }
    }
}

#[cfg(test)]
#[path = "motion_shape_gen_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_shape_gen_pivot_tests.rs"]
mod pivot_tests;

#[cfg(test)]
#[path = "motion_shape_catalogue_tests.rs"]
mod catalogue_tests;

#[cfg(test)]
#[path = "motion_shape_style_tests.rs"]
mod style_tests;
