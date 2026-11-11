//! ⭐ **As alças que entortam o arco entre chaves de posição** (dono 10/11: curvas espaciais editam-se
//! no CANVAS, com «Edit curves on canvas» ligada na Timeline e a aba Keys).
//!
//! A lei: cada chave de X/Y guarda o easing do segmento que SAI dela. Com
//! `Interp::BezierW { x1: 1/3, x2: 2/3 }` o tempo é LINEAR no parâmetro, logo o caminho entre duas chaves
//! é EXATAMENTE a cúbica espacial `K0, K0 + (dy1x, dy1y), K1 + (dy2x, dy2y), K1` (espaço local).
//!
//! **Onde a alça está** lê-se do MESMO amostrador do caminho: `C1 = P(ta) + P'(ta⁺)·Δ/3`,
//! `C2 = P(tb) − P'(tb⁻)·Δ/3`, com a derivada lateral de 4 pontos igualmente espaçados dentro do
//! segmento — exata para polinómios até grau 3 (BezierW 1/3–2/3 e Linear), uma boa leitura nos outros.
//! **Arrastar** garante chave de X e Y em `ta` e `tb` (a que falta nasce no valor amostrado, sem mexer
//! no movimento fora do segmento) e escreve em `ta`, por canal, a BezierW cujo controlo é o ponteiro
//! e cujo OUTRO lado fica onde estava.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::World;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_timeline::{PropKind, SelectedKey, TimelineDoc, TimelineState, pose_at, world_pose_at};
use ph2d_tokens::{ColorToken, StrokeToken, timeline_tinta::REFERENCIA};
use ph2d_vector::{BezPath, Point, VectorScene};

use crate::motion_key_points::{has_path, key_times, local_for_world};
use crate::motion_path_overlay::MotionPathGrab;

/// Raio do círculo da alça: o mesmo das alças do grafo da Timeline (`graph_paint::HANDLE_R`).
const ALCA_R: f64 = 3.5; // LITERAL-PX-OK: chrome de overlay, geometria de tela
/// Raio da alça arrastada: realçada.
const ALCA_VIVA_R: f64 = 5.0; // LITERAL-PX-OK: chrome de overlay, geometria de tela
/// Raio de pega: o mesmo das alças do grafo e dos losangos.
const HIT_R_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay, alvo de mouse
/// Segmentos do círculo.
const CIRC_SEGS: usize = 16;

/// Pesos (÷ 6) da derivada lateral de 4 pontos em `P0..P3`, já somada à âncora: `C1` e `C2`.
const PESOS_C1: [f64; 4] = [-5.0, 18.0, -9.0, 2.0];
const PESOS_C2: [f64; 4] = [2.0, -9.0, 18.0, -5.0];
const DIV_PESOS: f64 = 6.0;

/// Uma alça desenhável: o segmento, o lado, a chave de onde sai e a ponta (mundo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alca {
    /// Início do segmento.
    pub ta: RationalTime,
    /// Fim do segmento.
    pub tb: RationalTime,
    /// `true` = a alça que sai de `ta`; `false` = a que chega a `tb`.
    pub out: bool,
    /// A chave de onde a alça sai (mundo).
    pub key: [f32; 2],
    /// A ponta da alça (mundo).
    pub at: [f32; 2],
}

/// `(K0, C1, C2, K1)` da cúbica pelas 4 amostras `ta, ta+Δ/3, ta+2Δ/3, tb` de `f`.
fn cubica(f: &dyn Fn(f64) -> Option<[f32; 2]>, ta: f64, tb: f64) -> Option<[[f64; 2]; 4]> {
    let mut p = [[0.0; 2]; 4];
    for (k, q) in p.iter_mut().enumerate() {
        let v = f(ta + (tb - ta) * k as f64 / 3.0)?;
        *q = [f64::from(v[0]), f64::from(v[1])];
    }
    let c = |w: [f64; 4]| [0, 1].map(|i| (0..4).map(|k| w[k] * p[k][i]).sum::<f64>() / DIV_PESOS);
    Some([p[0], c(PESOS_C1), c(PESOS_C2), p[3]])
}

fn local_of(world: &World, doc: &TimelineDoc, e: u64) -> impl Fn(f64) -> Option<[f32; 2]> {
    move |t| pose_at(world, doc, e, t).map(|x| [x.translation.x, x.translation.y])
}

fn world_of(world: &World, doc: &TimelineDoc, e: u64) -> impl Fn(f64) -> Option<[f32; 2]> {
    move |t| world_pose_at(world, doc, e, t).map(|x| [x.translation.x, x.translation.y])
}

fn f32x2(p: [f64; 2]) -> [f32; 2] {
    [p[0] as f32, p[1] as f32]
}

/// **A porta única** de onde as alças estão: o desenho e o hit-test leem daqui. Vazio sem
/// `curves` (a checkbox «Edit curves on canvas»), fora da aba Keys, sem seleção ou com caminho Path.
#[must_use]
pub fn alcas(
    keys_tab: bool,
    curves: bool,
    world: &World,
    doc: &TimelineDoc,
    selected: Option<u64>,
) -> Vec<Alca> {
    let Some(e) = selected.filter(|&e| curves && keys_tab && !has_path(doc, e)) else {
        return Vec::new();
    };
    let f = world_of(world, doc, e);
    let mut out = Vec::new();
    for w in key_times(doc, e).windows(2) {
        let (ta, tb) = (w[0], w[1]);
        let Some([k0, c1, c2, k1]) = cubica(&f, ta.to_seconds(), tb.to_seconds()) else {
            continue;
        };
        out.push(Alca {
            ta,
            tb,
            out: true,
            key: f32x2(k0),
            at: f32x2(c1),
        });
        out.push(Alca {
            ta,
            tb,
            out: false,
            key: f32x2(k1),
            at: f32x2(c2),
        });
    }
    out
}

/// O hit-test das alças (o dos losangos vem antes, em [`crate::motion_key_points`]).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn alca_hit(
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
    let entity = selected?;
    let r2 = HIT_R_PX * HIT_R_PX;
    alcas(keys_tab, curves, world, doc, selected)
        .into_iter()
        .filter(|a| longe_da_chave(camera, window, a))
        .map(|a| {
            let (sx, sy) = camera.world_to_screen(a.at, window);
            let d2 =
                (f64::from(sx) - f64::from(x)).powi(2) + (f64::from(sy) - f64::from(y)).powi(2);
            (d2, a)
        })
        .filter(|(d2, _)| *d2 <= r2)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, a)| MotionPathGrab::Alca {
            entity,
            ta: a.ta,
            tb: a.tb,
            out: a.out,
        })
}

/// Uma alça colada à chave (trecho parado) não se oferece: no ecrã ela seria a chave — o mesmo
/// piso das tangentes do caminho Path ([`crate::motion_path_overlay::TANGENT_MIN_PX`]).
fn longe_da_chave(camera: &Camera2d, window: WindowSize, a: &Alca) -> bool {
    let (kx, ky) = camera.world_to_screen(a.key, window);
    let (cx, cy) = camera.world_to_screen(a.at, window);
    f64::from(cx - kx).hypot(f64::from(cy - ky)) >= crate::motion_path_overlay::TANGENT_MIN_PX
}

fn curvo(i: Interp) -> bool {
    matches!(
        i,
        Interp::Eased(_) | Interp::Bezier { .. } | Interp::BezierW { .. }
    )
}

/// A BezierW 1/3–2/3 que reproduz o canal `axis` de `f` em `[a, b]` (exata para cúbica no tempo).
fn bezier_w_de(f: &dyn Fn(f64) -> Option<[f32; 2]>, axis: usize, a: f64, b: f64) -> Option<Interp> {
    let [k0, c1, c2, k1] = cubica(f, a, b)?;
    Some(Interp::bezier_w(
        1.0 / 3.0,
        c1[axis] - k0[axis],
        2.0 / 3.0,
        c2[axis] - k1[axis],
    ))
}

/// Garante a chave do canal `prop` em `t`, no valor amostrado, sem mudar o movimento: num segmento
/// curvo, as duas metades ganham a BezierW que reproduz a original; Linear/Hold copiam-se (exato).
fn garante_chave(
    state: &mut TimelineState,
    world: &World,
    e: u64,
    prop: PropKind,
    t: RationalTime,
) {
    let axis = usize::from(prop == PropKind::TranslationY);
    let clip = state.doc.active_clip();
    let track = state
        .doc
        .binding_for(e, prop)
        .and_then(|b| clip.track(b.target));
    let keys = track.map(|tr| tr.keys().to_vec()).unwrap_or_default();
    if keys.iter().any(|k| k.t == t) {
        return;
    }
    let prev = keys.iter().rev().find(|k| k.t < t).copied();
    let next = keys.iter().find(|k| k.t > t).copied();
    let f = local_of(world, &state.doc, e);
    let Some(v) = f(t.to_seconds()).map(|p| p[axis]) else {
        return;
    };
    let ts = t.to_seconds();
    let (antes, depois) = match prev {
        Some(p) if curvo(p.interp) => (
            bezier_w_de(&f, axis, p.t.to_seconds(), ts),
            next.and_then(|n| bezier_w_de(&f, axis, ts, n.t.to_seconds())),
        ),
        Some(p) => (None, Some(p.interp)),
        None => (None, None),
    };
    drop(f);
    let (target, _) = state.doc.upsert_key(
        e,
        prop,
        t,
        AnimValue::Float(v),
        depois.unwrap_or(Interp::Linear),
    );
    if let (Some(p), Some(i)) = (prev, antes)
        && let Some(tr) = state.doc.active_clip_mut().track_mut(target)
        && let Some(id) = tr
            .keys()
            .iter()
            .position(|k| k.t == p.t)
            .and_then(|ix| tr.key_id(ix))
    {
        tr.set_interp(id, i);
    }
}

/// Leva a alça do segmento `[ta, tb]` (lado `out`) ao ponto de mundo `p`. `true` = o documento mudou.
#[allow(clippy::too_many_arguments)]
pub fn drag_alca(
    state: &mut TimelineState,
    world: &World,
    entity: u64,
    ta: RationalTime,
    tb: RationalTime,
    out: bool,
    p: [f32; 2],
) -> bool {
    let (sa, sb) = (ta.to_seconds(), tb.to_seconds());
    let Some([_, mut c1, mut c2, _]) = cubica(&local_of(world, &state.doc, entity), sa, sb) else {
        return false;
    };
    let Some(alvo) = local_for_world(world, &state.doc, entity, if out { sa } else { sb }, p)
    else {
        return false;
    };
    let alvo = [f64::from(alvo[0]), f64::from(alvo[1])];
    if out {
        c1 = alvo;
    } else {
        c2 = alvo;
    }
    let mut mudou = false;
    for (axis, prop) in [(0, PropKind::TranslationX), (1, PropKind::TranslationY)] {
        garante_chave(state, world, entity, prop, ta);
        garante_chave(state, world, entity, prop, tb);
        let Some(target) = state.doc.binding_for(entity, prop).map(|b| b.target) else {
            continue;
        };
        let Some(tr) = state.doc.active_clip_mut().track_mut(target) else {
            continue;
        };
        let valor = |t: RationalTime| {
            tr.keys()
                .iter()
                .position(|k| k.t == t)
                .and_then(|i| match tr.keys()[i].value {
                    AnimValue::Float(v) => Some((i, f64::from(v))),
                    _ => None,
                })
        };
        let (Some((ia, v0)), Some((_, v1))) = (valor(ta), valor(tb)) else {
            continue;
        };
        let Some(id) = tr.key_id(ia) else {
            continue;
        };
        let novo = Interp::bezier_w(1.0 / 3.0, c1[axis] - v0, 2.0 / 3.0, c2[axis] - v1);
        if tr.keys()[ia].interp != novo {
            tr.set_interp(id, novo);
            mudou = true;
        }
        if axis == 0 {
            state.selection.set_single(SelectedKey { target, key: id });
        }
    }
    mudou
}

fn rgba(token: ColorToken) -> [f32; 4] {
    let c = token.resolve(REFERENCIA);
    let u = |v: u8| f32::from(v) / f32::from(u8::MAX);
    [u(c.r), u(c.g), u(c.b), u(c.a)]
}

fn circulo(c: Point, r: f64) -> BezPath {
    let mut p = BezPath::new();
    for k in 0..=CIRC_SEGS {
        let a = std::f64::consts::TAU * k as f64 / CIRC_SEGS as f64;
        let q = Point::new(c.x + r * a.cos(), c.y + r * a.sin());
        if k == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p.close_path();
    p
}

/// Pinta a linha chave→alça e o círculo de cada alça; a arrastada no azul da chave escolhida.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    keys_tab: bool,
    curves: bool,
    world: &World,
    doc: &TimelineDoc,
    drag: Option<MotionPathGrab>,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Fill, Stroke};
    let s = vector_scene.inner_mut();
    let solid = |c: [f32; 4]| Brush::Solid(Color::new(c));
    let id = Affine::IDENTITY;
    let fino = Stroke::new(f64::from(StrokeToken::Thin.px()));
    let tela = |w: [f32; 2]| {
        let (sx, sy) = camera.world_to_screen(w, window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    for a in alcas(keys_tab, curves, world, doc, selected) {
        let viva = matches!(
            drag,
            Some(MotionPathGrab::Alca { entity, ta, tb, out })
                if Some(entity) == selected && ta == a.ta && tb == a.tb && out == a.out
        );
        if !viva && !longe_da_chave(camera, window, &a) {
            continue;
        }
        let (k, c) = (tela(a.key), tela(a.at));
        let mut linha = BezPath::new();
        linha.move_to(k);
        linha.line_to(c);
        s.stroke(
            &fino,
            id,
            &solid(rgba(ColorToken::TimelineHandleLine)),
            None,
            &linha,
        );
        let (tok, r) = if viva {
            (ColorToken::TimelineKeyActive, ALCA_VIVA_R)
        } else {
            (ColorToken::TimelineHandle, ALCA_R)
        };
        s.fill(Fill::NonZero, id, &solid(rgba(tok)), None, &circulo(c, r));
    }
}

#[cfg(test)]
#[path = "motion_key_handles_tests.rs"]
mod tests;
