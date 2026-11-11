//! ⭐ **As chaves de POSIÇÃO no canvas, agarráveis** (doc 17, Onda 2, item 7; maquetes C3/C4).
//!
//! Com eixos separados (`TranslationX`/`Y`) o canvas não desenhava nada: só o modo Path tinha
//! trajetória. Aqui, com a Timeline na aba Keys e o objeto selecionado, desenha-se o caminho que o
//! clip ativo faz (amostrado pela MESMA porta do apply, [`world_pose_at`]: pai incluído) e um
//! losango em cada quadro com chave de X ou Y. Arrastar o losango escreve as chaves desse quadro pela
//! inversa exata da composição com o pai ([`Transform::inverse_compose`]); o eixo sem chave nesse
//! quadro ganha uma se o arrasto o mexer (resposta de produto 3: como o Auto Key).
//!
//! O px/m não entra: o `Transform` já está no espaço do mundo da câmera (os fantasmas do onion põem
//! `world_pos` = a translação de mundo, sem escala). Com caminho Path no clip ativo, quem desenha é o
//! [`crate::motion_path_overlay`] e aqui não sai nada.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_timeline::{
    PropKind, SelectedKey, TimelineDoc, TimelineHistory, TimelineState, pose_at, world_pose_at,
};
use ph2d_tokens::{TimelineTinta, timeline_tinta::REFERENCIA};
use ph2d_vector::{BezPath, Point, VectorScene};

use crate::motion_path_overlay::MotionPathGrab;

/// Meia diagonal do losango (a maquete: 9 px a 45°).
const KEY_HALF: f64 = 4.5; // LITERAL-PX-OK: chrome de overlay, geometria de tela
/// O anel escuro do losango.
const KEY_RING_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// O anel do ponto arrastado: 26 px de diâmetro, 1,5 px, e o brilho de 14 px por baixo.
const DRAG_RING_R: f64 = 13.0; // LITERAL-PX-OK: chrome de overlay, geometria de tela
const DRAG_GLOW_PX: f64 = 14.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// O brilho da maquete é 60 % DESFOCADO; num traço chapado de 14 px isso lê-se a ~15 %.
const DRAG_GLOW_ALPHA: f32 = 0.15;
/// O caminho NOVO: branco 70 %, 1,5 px, traço 3/4. O de ANTES: branco 25 %, 1 px, traço 2/4.
const TRAIL_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela
const TRAIL_ALPHA: f32 = 0.7;
const TRAIL_DASH: [f64; 2] = [3.0, 4.0]; // LITERAL-PX-OK: traço da maquete C4
const BEFORE_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela
const BEFORE_ALPHA: f32 = 0.25;
const BEFORE_DASH: [f64; 2] = [2.0, 4.0]; // LITERAL-PX-OK: traço da maquete C4
/// Raio de pega: o mesmo das âncoras do caminho Path.
const HIT_R_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay, alvo de mouse
/// Amostras do caminho por quadro de exibição. Cada amostra é uma `world_pose_at` (a cadeia do
/// pai inteira); quatro por quadro dão a curva lisa a qualquer zoom de trabalho.
const TRAIL_SAMPLES_PER_FRAME: f64 = 4.0;
const MAX_TRAIL_SAMPLES: usize = 2048;
/// Abaixo disto (unidades do mundo) o arrasto não mexeu o eixo sem chave: ela não nasce.
const AXIS_MOVED_EPS: f32 = 1e-5;
/// Segmentos do anel do arrasto.
const RING_SEGS: usize = 32;

/// Um ponto de chave: o tempo autorado e onde o objeto está nesse tempo (mundo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyPoint {
    /// O instante da chave (o tempo da track).
    pub t: RationalTime,
    /// A posição de mundo do objeto nesse instante.
    pub at: [f32; 2],
}

fn axes(doc: &TimelineDoc, entity: u64) -> impl Iterator<Item = &ph2d_timeline::TargetBinding> {
    doc.bindings().iter().filter(move |b| {
        b.entity == entity
            && !b.missing
            && matches!(b.prop, PropKind::TranslationX | PropKind::TranslationY)
    })
}

pub(crate) fn has_path(doc: &TimelineDoc, entity: u64) -> bool {
    doc.bindings().iter().any(|b| {
        b.entity == entity
            && !b.missing
            && b.prop == PropKind::Position
            && doc.clip_path(doc.active_index(), b.target).is_some()
    })
}

/// Os instantes com chave de X ou Y no clip ativo, ordenados e sem repetição.
pub(crate) fn key_times(doc: &TimelineDoc, entity: u64) -> Vec<RationalTime> {
    let clip = doc.active_clip();
    let mut ts: Vec<RationalTime> = axes(doc, entity)
        .filter_map(|b| clip.track(b.target))
        .flat_map(|tr| tr.keys().iter().map(|k| k.t))
        .collect();
    ts.sort_unstable();
    ts.dedup();
    ts
}

/// **A porta única** de onde os pontos estão: o desenho e o hit-test leem daqui.
#[must_use]
pub fn key_points(
    keys_tab: bool,
    world: &World,
    doc: &TimelineDoc,
    selected: Option<u64>,
) -> Vec<KeyPoint> {
    let Some(e) = selected.filter(|&e| keys_tab && !has_path(doc, e)) else {
        return Vec::new();
    };
    key_times(doc, e)
        .into_iter()
        .filter_map(|t| {
            let p = world_pose_at(world, doc, e, t.to_seconds())?.translation;
            Some(KeyPoint { t, at: [p.x, p.y] })
        })
        .collect()
}

/// O caminho de mundo do objeto entre a primeira e a última chave de posição.
#[must_use]
pub fn trail(world: &World, doc: &TimelineDoc, entity: u64) -> Vec<[f32; 2]> {
    let ts = key_times(doc, entity);
    let (Some(a), Some(b)) = (ts.first(), ts.last()) else {
        return Vec::new();
    };
    let (t0, t1) = (a.to_seconds(), b.to_seconds());
    let n = (((t1 - t0) * doc.fps_display.max(1.0) * TRAIL_SAMPLES_PER_FRAME).ceil() as usize)
        .clamp(1, MAX_TRAIL_SAMPLES);
    (0..=n)
        .filter_map(|k| {
            let t = t0 + (t1 - t0) * k as f64 / n as f64;
            let p = world_pose_at(world, doc, entity, t)?.translation;
            Some([p.x, p.y])
        })
        .collect()
}

/// O valor LOCAL `(x, y)` que põe o objeto no ponto de mundo `p` no instante `t`: a inversa exata
/// da composição com o pai posado nesse instante (rotação, escala e skew do pai incluídos).
#[must_use]
pub fn local_for_world(
    world: &World,
    doc: &TimelineDoc,
    entity: u64,
    t: f64,
    p: [f32; 2],
) -> Option<[f32; 2]> {
    let e = Entity::try_from_bits(entity)?;
    let parent = ph2d_ecs::parent_world_transform_with(world, e, &mut Vec::new(), &|elo| {
        pose_at(world, doc, elo.to_bits(), t)
    });
    let mut alvo = Transform::IDENTITY;
    alvo.translation.x = p[0];
    alvo.translation.y = p[1];
    let l = Transform::inverse_compose(parent, alvo)?.translation;
    Some([l.x, l.y])
}

/// O hit-test do caminho Path (âncoras e alças), depois o dos pontos de chave e, com `curves`
/// («Edit curves on canvas»), o das alças do arco ([`crate::motion_key_handles`]).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn motion_path_hit_with_keys(
    keys_tab: bool,
    curves: bool,
    world: &World,
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    x: f32,
    y: f32,
) -> Option<MotionPathGrab> {
    if let Some(g) =
        crate::motion_path_overlay::motion_path_hit(keys_tab, doc, selected, camera, window, x, y)
    {
        return Some(g);
    }
    let entity = selected?;
    let r2 = HIT_R_PX * HIT_R_PX;
    key_points(keys_tab, world, doc, selected)
        .into_iter()
        .map(|k| {
            let (sx, sy) = camera.world_to_screen(k.at, window);
            let d2 =
                (f64::from(sx) - f64::from(x)).powi(2) + (f64::from(sy) - f64::from(y)).powi(2);
            (d2, k.t)
        })
        .filter(|(d2, _)| *d2 <= r2)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, t)| MotionPathGrab::Key { entity, t })
        .or_else(|| {
            crate::motion_key_handles::alca_hit(
                keys_tab, curves, world, doc, selected, camera, window, x, y,
            )
        })
}

/// **A porta única do arrasto** de um ponto de chave ou de uma alça do arco; as outras pegas
/// (Path) não são daqui (`false`).
pub fn drag_grab(
    state: &mut TimelineState,
    world: &World,
    grab: MotionPathGrab,
    p: [f32; 2],
) -> bool {
    match grab {
        MotionPathGrab::Key { entity, t } => drag_key(state, world, entity, t, p),
        MotionPathGrab::Alca {
            entity,
            ta,
            tb,
            out,
        } => crate::motion_key_handles::drag_alca(state, world, entity, ta, tb, out, p),
        MotionPathGrab::Anchor { .. } | MotionPathGrab::Tangent { .. } => false,
    }
}

/// Leva o ponto do instante `t` ao ponto de mundo `p`: escreve a chave de cada eixo que a tem
/// nesse quadro e CRIA a do eixo que não a tem se o arrasto o mexeu. A agulha não se mexe; a
/// chave escrita vira a escolhida na fita. `true` = o documento mudou.
pub fn drag_key(
    state: &mut TimelineState,
    world: &World,
    entity: u64,
    t: RationalTime,
    p: [f32; 2],
) -> bool {
    let ts = t.to_seconds();
    let (Some(local), Some(cur)) = (
        local_for_world(world, &state.doc, entity, ts, p),
        pose_at(world, &state.doc, entity, ts).map(|x| x.translation),
    ) else {
        return false;
    };
    let clip = state.doc.active_clip();
    let tem = |prop: PropKind| {
        state.doc.binding_for(entity, prop).is_some_and(|b| {
            clip.track(b.target)
                .is_some_and(|tr| tr.keys().iter().any(|k| k.t == t))
        })
    };
    let eixos = [
        (
            PropKind::TranslationX,
            local[0],
            cur.x,
            tem(PropKind::TranslationX),
        ),
        (
            PropKind::TranslationY,
            local[1],
            cur.y,
            tem(PropKind::TranslationY),
        ),
    ];
    let mut mudou = false;
    for (prop, v, antes, tem_chave) in eixos {
        if !tem_chave && (v - antes).abs() <= AXIS_MOVED_EPS {
            continue;
        }
        let (target, key) =
            state
                .doc
                .upsert_key(entity, prop, t, AnimValue::Float(v), Interp::Linear);
        state.selection.set_single(SelectedKey { target, key });
        mudou = true;
    }
    mudou
}

fn rgba(tinta: TimelineTinta, alpha: f32) -> [f32; 4] {
    let c = tinta.resolve(REFERENCIA);
    let u = |v: u8| f32::from(v) / f32::from(u8::MAX);
    [u(c.r), u(c.g), u(c.b), alpha]
}

fn polyline(pts: &[[f32; 2]], camera: &Camera2d, window: WindowSize) -> BezPath {
    let mut p = BezPath::new();
    for (i, w) in pts.iter().enumerate() {
        let (sx, sy) = camera.world_to_screen(*w, window);
        let q = Point::new(f64::from(sx), f64::from(sy));
        if i == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p
}

fn diamond(c: Point, half: f64) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(Point::new(c.x, c.y - half));
    p.line_to(Point::new(c.x + half, c.y));
    p.line_to(Point::new(c.x, c.y + half));
    p.line_to(Point::new(c.x - half, c.y));
    p.close_path();
    p
}

fn ring(c: Point, r: f64) -> BezPath {
    let mut p = BezPath::new();
    for k in 0..=RING_SEGS {
        let a = std::f64::consts::TAU * k as f64 / RING_SEGS as f64;
        let q = Point::new(c.x + r * a.cos(), c.y + r * a.sin());
        if k == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p
}

/// O que está sob arrasto agora: o instante agarrado (nenhum, numa alça) e o documento de ANTES.
fn live_drag(
    drag: Option<MotionPathGrab>,
    history: &TimelineHistory,
    entity: u64,
) -> Option<(Option<RationalTime>, &TimelineDoc)> {
    match drag? {
        MotionPathGrab::Key { entity: e, t } if e == entity => Some((Some(t), history.pending()?)),
        MotionPathGrab::Alca { entity: e, .. } if e == entity => Some((None, history.pending()?)),
        _ => None,
    }
}

/// Pinta as alças do arco (com `curves`), o caminho, o de antes (só durante o arrasto), os
/// losangos e o anel do arrastado.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    keys_tab: bool,
    curves: bool,
    world: &World,
    doc: &TimelineDoc,
    history: &TimelineHistory,
    drag: Option<MotionPathGrab>,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Fill, Stroke};
    let pts = key_points(keys_tab, world, doc, selected);
    let Some(entity) = selected.filter(|_| !pts.is_empty()) else {
        return;
    };
    crate::motion_key_handles::draw(
        keys_tab,
        curves,
        world,
        doc,
        drag,
        selected,
        camera,
        window,
        vector_scene,
    );
    let s = vector_scene.inner_mut();
    let solid = |c: [f32; 4]| Brush::Solid(Color::new(c));
    let id = Affine::IDENTITY;
    let arrasto = live_drag(drag, history, entity);
    if let Some((_, antes)) = arrasto {
        let fraco = polyline(&trail(world, antes, entity), camera, window);
        let traco = Stroke::new(BEFORE_PX).with_dashes(0.0, BEFORE_DASH);
        let cor = solid(rgba(TimelineTinta::TextoCabecalho, BEFORE_ALPHA));
        s.stroke(&traco, id, &cor, None, &fraco);
    }
    let novo = polyline(&trail(world, doc, entity), camera, window);
    let traco = Stroke::new(TRAIL_PX).with_dashes(0.0, TRAIL_DASH);
    let cor = solid(rgba(TimelineTinta::TextoCabecalho, TRAIL_ALPHA));
    s.stroke(&traco, id, &cor, None, &novo);
    for k in &pts {
        let (sx, sy) = camera.world_to_screen(k.at, window);
        let c = Point::new(f64::from(sx), f64::from(sy));
        let vivo = arrasto.is_some_and(|(t, _)| t == Some(k.t));
        if vivo {
            let brilho = solid(rgba(TimelineTinta::ChaveEscolhida, DRAG_GLOW_ALPHA));
            s.stroke(
                &Stroke::new(DRAG_GLOW_PX),
                id,
                &brilho,
                None,
                &ring(c, DRAG_RING_R),
            );
            let anel = solid(rgba(TimelineTinta::ChaveEscolhida, 1.0));
            s.stroke(
                &Stroke::new(KEY_RING_PX),
                id,
                &anel,
                None,
                &ring(c, DRAG_RING_R),
            );
        }
        let tinta = if vivo {
            TimelineTinta::ChaveEscolhida
        } else {
            TimelineTinta::Chave
        };
        let forma = diamond(c, KEY_HALF);
        s.fill(Fill::NonZero, id, &solid(rgba(tinta, 1.0)), None, &forma);
        let escuro = solid(rgba(TimelineTinta::FitaPropriedade, 1.0));
        s.stroke(&Stroke::new(KEY_RING_PX), id, &escuro, None, &forma);
    }
}

#[cfg(test)]
#[path = "motion_key_points_tests.rs"]
mod tests;
