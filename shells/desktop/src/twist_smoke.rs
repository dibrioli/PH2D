//! **A cena pronta para o smoke do TWIST** (o remoinho) — `PH2D_BUILD_SMOKE=29`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `falloff_smoke`/`warp_smoke`.
//!
//! O Twist gira cada ponto em torno do centro por um ângulo que cresce com a distância: o centro
//! fica parado, a borda gira, as quinas enrolam — o *pinwheel*. Três quadrados, e cada um prova
//! uma coisa:
//!
//! - **Esquerda (controle):** um quadrado com Twist **90°** — um remoinho suave, quinas ainda
//!   reconhecíveis. É o "pouco".
//! - **Meio (HERÓI, já selecionado):** o mesmo quadrado com Twist **200°** — o remoinho forte. A
//!   pilha aparece na seção **Effects** com o card do Twist e o slider **Angle**: arraste-o e veja
//!   o giro apertar e afrouxar ao vivo.
//! - **Direita:** um Twist **200°** IDÊNTICO, mas precedido de um **Falloff Radial** — só o miolo
//!   gira, as quinas ficam onde estavam. É o Falloff (a wave de ontem) a MODULAR o Twist (a de
//!   hoje): o card do Falloff diz *"modulates the effect below"*, e desligar o olho dele devolve o
//!   remoinho cheio da direita ao do meio.

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_falloff::{FalloffShape, FalloffSpec};
use ph2d_vec_scene::fx_twist::TwistSpec;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPathId};

use crate::build_smoke::shape;

/// Largura do traço, em unidades de MUNDO (a cena vive numa caixa de ~±3.5).
const STROKE_W: f64 = 0.05;

fn twist(angle: f64) -> PathEffect {
    PathEffect::Twist(TwistSpec { angle })
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

    // ── Esquerda: Twist 90° (o remoinho suave, o controle) ───────────────────
    let mut left = shape(
        ShapeKind::Rectangle,
        [-3.4, -1.0],
        [-1.4, 1.0],
        &[],
        [70, 150, 220],
    );
    left.fill = None;
    left.stroke = Some(StrokeSpec::new(Rgba8::new(70, 150, 220, 255), STROKE_W));
    left.effects = vec![FxEntry::new(twist(90.0))];
    scene.push_path(left);

    // ── Meio: Twist 200° (o remoinho forte — o herói) ────────────────────────
    let mut mid = shape(
        ShapeKind::Rectangle,
        [-1.0, -1.0],
        [1.0, 1.0],
        &[],
        [110, 200, 130],
    );
    mid.fill = None;
    mid.stroke = Some(StrokeSpec::new(Rgba8::new(110, 200, 130, 255), STROKE_W));
    mid.effects = vec![FxEntry::new(twist(200.0))];
    scene.push_path(mid);

    // ── Direita: o MESMO Twist 200°, mas o Falloff Radial só deixa o miolo girar ──
    let mut right = shape(
        ShapeKind::Rectangle,
        [1.4, -1.0],
        [3.4, 1.0],
        &[],
        [220, 150, 90],
    );
    right.fill = None;
    right.stroke = Some(StrokeSpec::new(Rgba8::new(220, 150, 90, 255), STROKE_W));
    // Radial forte no centro, `size 0.5` (raio ~= metade da forma): o Twist cheio no miolo,
    // apagando-se para as quinas — o campo de ontem a modular o giro de hoje.
    let radial = FalloffSpec {
        shape: FalloffShape::Radial,
        amount: 1.0,
        size: 0.5,
        ..FalloffSpec::new(FalloffShape::Radial)
    };
    right.effects = vec![
        FxEntry::new(PathEffect::Falloff(radial)),
        FxEntry::new(twist(200.0)),
    ];
    scene.push_path(right);
}

/// Seleciona o quadrado do MEIO (o Twist 200°) — a seção **Effects** abre com o card do Twist e o
/// slider **Angle**. Frame 4: depois do `sync`, para o alvo já ter entidade.
fn select_hero(app: &mut crate::App) {
    let mid: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().get(1).map(|p| p.id));
    if let Some(id) = mid {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] twist: 3 quadrados. ESQUERDA = Twist 90 (remoinho suave). MEIO (selecionado) = \
         Twist 200 -> na secao **Effects** arraste o slider Angle e veja o giro apertar/afrouxar. \
         DIREITA = MESMO Twist 200, mas o **Falloff Radial** deixa so o miolo girar (as quinas \
         ficam) -- o campo de ontem modulando o giro de hoje; desligue o olho do Falloff e a \
         direita vira o remoinho cheio do meio."
    );
}
