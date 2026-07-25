//! **As cenas de smoke do SKETCH e do HATCH** — `PH2D_BUILD_SMOKE=31` (Sketch) e `=32` (Hatch).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `knot_smoke`/`twist_smoke`.
//!
//! **Sketch (=31):** três estrelas traçadas SEM preenchimento. Esquerda = limpa (Sketch neutro);
//! meio (HERÓI, selecionado) = 2 passadas, tremor 4 % → a seção **Effects** abre com o card do
//! Sketch (Passes/Roughness/Detail/Seed); direita = 3 passadas, tremor 7 %, outra seed. As
//! passadas quase-coincidentes leem como lápis.
//!
//! **Hatch (=32):** três círculos PREENCHIDOS. Esquerda = liso; meio (HERÓI) = hachura a 45 %;
//! direita = cross-hatch. O outline + o fill ficam, as linhas enchem o interior (o fill NÃO é
//! furado pelas linhas abertas — gate `an_open_contour_never_punches_a_hole_in_the_fill`).

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_hatch::HatchSpec;
use ph2d_vec_scene::fx_sketch::SketchSpec;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex};

const STROKE_W: f64 = 0.05;

/// Uma estrela de 5 pontas (raio externo `r`, interno `r*0.42`) centrada em `c`, traçada.
fn star(c: [f64; 2], r: f64, rgb: [u8; 3]) -> VecPath {
    let verts = (0..10)
        .map(|k| {
            let rr = if k % 2 == 0 { r } else { r * 0.42 };
            let a = (90.0 + f64::from(k) * 36.0).to_radians();
            VecVertex::corner([c[0] + rr * a.cos(), c[1] + rr * a.sin()])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(rgb[0], rgb[1], rgb[2], 255), STROKE_W)),
        ..VecPath::default()
    }
}

/// Um círculo PREENCHIDO (quatro cúbicas) de raio `r` em `c`, com outline.
fn disc(c: [f64; 2], r: f64, fill: [u8; 3]) -> VecPath {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
    let t = [[0.0, K * r], [-K * r, 0.0], [0.0, -K * r], [K * r, 0.0]];
    let verts = (0..4)
        .map(|i| VecVertex {
            anchor: [c[0] + p[i][0], c[1] + p[i][1]],
            in_handle: [c[0] + p[i][0] - t[i][0], c[1] + p[i][1] - t[i][1]],
            out_handle: [c[0] + p[i][0] + t[i][0], c[1] + p[i][1] + t[i][1]],
            kind: ph2d_vec_scene::VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(fill[0], fill[1], fill[2], 90))),
        stroke: Some(StrokeSpec::new(Rgba8::new(40, 40, 50, 255), STROKE_W)),
        ..VecPath::default()
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32, level: u32) {
    match f {
        3 => build(app, level),
        4 => select_hero(app, level),
        _ => {}
    }
}

fn build(app: &mut crate::App, level: u32) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;
    if level == 31 {
        // Sketch: limpo | 2 passadas 4% | 3 passadas 7% (seed 3).
        scene.push_path(star([-2.6, 0.0], 1.3, [70, 150, 220]));
        let mut mid = star([0.0, 0.0], 1.3, [110, 200, 130]);
        mid.effects = vec![FxEntry::new(PathEffect::Sketch(SketchSpec {
            passes: 2.0,
            roughness: 4.0,
            detail: 6.0,
            seed: 1,
        }))];
        scene.push_path(mid);
        let mut right = star([2.6, 0.0], 1.3, [220, 150, 90]);
        right.effects = vec![FxEntry::new(PathEffect::Sketch(SketchSpec {
            passes: 3.0,
            roughness: 7.0,
            detail: 8.0,
            seed: 3,
        }))];
        scene.push_path(right);
    } else {
        // Hatch: liso | 45% | cross.
        scene.push_path(disc([-2.6, 0.0], 1.2, [90, 150, 220]));
        let mut mid = disc([0.0, 0.0], 1.2, [130, 200, 150]);
        mid.effects = vec![FxEntry::new(PathEffect::Hatch(HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: false,
        }))];
        scene.push_path(mid);
        let mut right = disc([2.6, 0.0], 1.2, [220, 160, 110]);
        right.effects = vec![FxEntry::new(PathEffect::Hatch(HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: true,
        }))];
        scene.push_path(right);
    }
}

/// Seleciona a forma do MEIO — a seção **Effects** abre com o card do efeito. Frame 4: depois do
/// `sync`, para o alvo já ter entidade.
fn select_hero(app: &mut crate::App, level: u32) {
    let mid: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().get(1).map(|p| p.id));
    if let Some(id) = mid {
        app.vec_pen.select_many(&[id]);
    }
    if level == 31 {
        eprintln!(
            "[smoke] sketch: 3 estrelas. ESQUERDA = limpa (Sketch neutro). MEIO (selecionado) = \
             2 passadas, tremor 4% -> na secao **Effects** o card do Sketch tem Passes/Roughness/\
             Detail/Seed; as 2 linhas quase-coincidem e tremem = lapis. DIREITA = 3 passadas, \
             tremor 7%, outra seed."
        );
    } else {
        eprintln!(
            "[smoke] hatch: 3 circulos PREENCHIDOS. ESQUERDA = liso. MEIO (selecionado) = hachura \
             a 45graus (spacing 8%) -> o card do Hatch tem Angle/Spacing/Cross; o fill e o outline \
             ficam, as linhas enchem o interior. DIREITA = MESMO com **Cross** (2a familia a 90graus)."
        );
    }
}
