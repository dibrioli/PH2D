//! ⭐ **As alças que entortam o arco entre chaves de posição** (dono 10/11: curvas espaciais editam-se
//! no CANVAS, com «Edit curves on canvas» ligada na Timeline e a aba Keys).
//!
//! Num objeto ainda em X/Y separados, **onde a alça está** lê-se do MESMO amostrador do caminho:
//! `C1 = P(ta) + P'(ta⁺)·Δ/3`, `C2 = P(tb) − P'(tb⁻)·Δ/3`, com a derivada lateral de 4 pontos
//! igualmente espaçados dentro do segmento — exata para polinómios até grau 3.
//! **Arrastar** converte o objeto em caminho (dono 12/11, item 13: a forma mora no caminho, o easing
//! da linha «Position» é só o ritmo) e move a tangente da âncora; daí em diante o objeto é desenhado
//! e agarrado pelo overlay do caminho ([`crate::motion_path_overlay`]).

use ph2d_anim::RationalTime;
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

/// Leva a alça do segmento `[ta, tb]` (lado `out`) ao ponto de mundo `p`. `true` = o documento mudou.
///
/// ⭐ Dono 12/11 (item 13, o modelo After Effects): a forma do arco mora no CAMINHO, nunca no
/// easing de X e de Y. Um objeto ainda em X/Y separados vira caminho no primeiro arrasto (a forma
/// atravessa, [`TimelineDoc::separate_to_path`]) e a alça passa a ser a tangente da âncora — o easing
/// da linha «Position» fica só com o ritmo ao longo dele. Tudo no mesmo passo de undo do gesto.
pub fn drag_alca(
    state: &mut TimelineState,
    world: &World,
    entity: u64,
    ta: RationalTime,
    tb: RationalTime,
    out: bool,
    p: [f32; 2],
) -> bool {
    // O eixo que nunca foi animado vale o que o objeto tem agora (o repouso dele).
    let repouso = pose_at(world, &state.doc, entity, ta.to_seconds())
        .map_or([0.0, 0.0], |x| [x.translation.x, x.translation.y]);
    let convertido = !has_path(&state.doc, entity)
        && state
            .doc
            .separate_to_path_with_rest(entity, repouso)
            .is_some();
    let t = if out { ta } else { tb };
    let Some(target) = state
        .doc
        .binding_for(entity, PropKind::Position)
        .map(|b| b.target)
    else {
        return convertido;
    };
    let Some(local) = local_for_world(world, &state.doc, entity, t.to_seconds(), p) else {
        return convertido;
    };
    let Some((i, id)) = state.doc.active_clip().track(target).and_then(|tr| {
        let i = tr.keys().iter().position(|k| k.t == t)?;
        Some((i, tr.key_id(i)?))
    }) else {
        return convertido;
    };
    let antes = state.doc.path_anchor(target, i);
    state.doc.move_path_tangent(target, i, out, local);
    state.selection.set_single(SelectedKey { target, key: id });
    convertido || state.doc.path_anchor(target, i) != antes
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

/// `PH2D_ANIMACAO_SMOKE=7` (item 13): o pulo do Herói (clip `pular`) com o arco da subida ENTORTADO
/// pela alça (o gesto do canvas, que converte X/Y numa linha «Position») e um easing Cubic In-Out
/// escolhido depois, como o dono faz à mão; a caixa «Edit curves on canvas» marcada, a tocar.
/// A foto mostra que a forma ficou e só o ritmo mudou. No-op nos outros níveis.
pub fn smoke_7(
    state: &mut TimelineState,
    playhead: &mut ph2d_core::Playhead,
    world: &World,
    heroi: u64,
    pular: usize,
) {
    use ph2d_anim::{Easing, EasingFamily, EasingMode, Interp};
    if std::env::var("PH2D_ANIMACAO_SMOKE").as_deref() != Ok("7") {
        return;
    }
    state.doc.set_active(pular);
    let (t0, t1) = (
        RationalTime::from_seconds(0.0),
        RationalTime::from_seconds(0.5),
    );
    if drag_alca(state, world, heroi, t0, t1, true, [-3.4, 1.3]) {
        let interp = Interp::Eased(Easing {
            family: EasingFamily::Cubic,
            mode: EasingMode::InOut,
        });
        let intent = ph2d_timeline::TimelineIntent::SetSelectedInterp { interp };
        ph2d_timeline::apply_intent(state, playhead, intent);
    }
    ph2d_panel_timeline::state::request_curves_on_canvas();
}
