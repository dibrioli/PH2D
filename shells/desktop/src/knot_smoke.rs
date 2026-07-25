//! **A cena pronta para o smoke do KNOT** (o entrelace celta) — `PH2D_BUILD_SMOKE=30`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `twist_smoke`/`falloff_smoke`.
//!
//! Onde o caminho se cruza, a fita de baixo ganha um VÃO e a de cima passa inteira — o nó celta.
//! Três pentagramas (5 travessias cada, a forma canônica), traçados SEM preenchimento para lerem
//! como fita:
//!
//! - **Esquerda:** Knot Gap **6%** — o entrelace sutil.
//! - **Meio (HERÓI, já selecionado):** Knot Gap **10%** — o entrelace claro. A pilha aparece na
//!   seção **Effects** com o card do Knot: o slider **Gap** (a espessura aparente) e o toggle
//!   **Swap** (quem passa por cima). Seguindo UMA fita, ela alterna cima/baixo/cima — o "nó sem fim".
//! - **Direita:** o MESMO Gap 10% com **Swap** ligado — em cada travessia inverte quem mergulha.

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_knot::KnotSpec;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex};

/// Largura do traço, em unidades de MUNDO. O vão do Knot (~10% de ~2,8 = 0,28) fica ~3x a largura,
/// então a passagem de baixo se abre visivelmente sob a de cima.
const STROKE_W: f64 = 0.09;

/// Um pentagrama `{5/2}` de raio `r` centrado em `c` — 5 pontos ligados de dois em dois.
fn pentagram(c: [f64; 2], r: f64, rgb: [u8; 3]) -> VecPath {
    let verts = (0..5)
        .map(|k| {
            let a = (90.0 + f64::from(k) * 144.0).to_radians();
            VecVertex::corner([c[0] + r * a.cos(), c[1] + r * a.sin()])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        fill: None,
        stroke: Some(StrokeSpec::new(
            Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            STROKE_W,
        )),
        ..VecPath::default()
    }
}

fn knot(gap: f64, swap: bool) -> PathEffect {
    PathEffect::Knot(KnotSpec { gap, swap })
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => select_hero(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;

    let mut left = pentagram([-2.6, 0.0], 1.4, [70, 150, 220]);
    left.effects = vec![FxEntry::new(knot(6.0, false))];
    scene.push_path(left);

    let mut mid = pentagram([0.0, 0.0], 1.4, [110, 200, 130]);
    mid.effects = vec![FxEntry::new(knot(10.0, false))];
    scene.push_path(mid);

    let mut right = pentagram([2.6, 0.0], 1.4, [220, 150, 90]);
    right.effects = vec![FxEntry::new(knot(10.0, true))];
    scene.push_path(right);
}

/// Seleciona o pentagrama do MEIO — a seção **Effects** abre com o card do Knot (Gap + Swap).
/// Frame 4: depois do `sync`, para o alvo já ter entidade.
fn select_hero(app: &mut crate::App) {
    let mid: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().get(1).map(|p| p.id));
    if let Some(id) = mid {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] knot: 3 pentagramas (5 travessias cada), tracados como FITA. ESQUERDA = Gap 6% \
         (entrelace sutil). MEIO (selecionado) = Gap 10% -> na secao **Effects** o card do Knot tem \
         o slider Gap e o toggle Swap; seguindo uma fita ela alterna cima/baixo (o no' sem fim). \
         DIREITA = MESMO Gap 10% com **Swap** -> inverte quem passa por cima em toda travessia."
    );
}
