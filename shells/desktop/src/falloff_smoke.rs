//! **A cena pronta para o smoke do FALLOFF** (a ideia do Cavalry) — `PH2D_BUILD_SMOKE=28`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `fx_smoke`/`warp_smoke`.
//!
//! Um Falloff não deforma nada sozinho: ele produz um campo escalar de FORÇA e modula o
//! deformador SEGUINTE na pilha. Três formas, e cada uma prova uma coisa:
//!
//! - **Esquerda (controle):** um retângulo com Zig Zag UNIFORME — as cristas têm a mesma altura
//!   em toda a volta. É o "antes".
//! - **Meio (HERÓI, já selecionado):** o MESMO Zig Zag, precedido de um **Falloff Linear** — as
//!   cristas nascem planas de um lado e crescem até cheias do outro. É o falloff a esculpir a
//!   força ao longo do eixo. A pilha aparece na seção **Effects**: o card do Falloff diz
//!   *"modulates the effect below"* — desligue o olho dele para ver as cristas voltarem a uniformes.
//! - **Direita:** o MESMO Zig Zag precedido de um **Falloff Radial** — as cristas nascem altas no
//!   miolo e planas nas pontas. É a MESMA força, outra FORMA de campo: as três formas (sem campo,
//!   Linear, Radial) ensinam que o falloff é uma força ESPACIAL, e a diferença entre elas é só o
//!   campo. (Que o campo também modula um Warp/Bloat está provado nos gates do motor.)

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_falloff::{FalloffShape, FalloffSpec};
use ph2d_vec_scene::fx_zigzag::ZigZagSpec;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPathId};

use crate::build_smoke::shape;

/// Largura do traço, em unidades de MUNDO (a cena vive numa caixa de ~±3.5).
const STROKE_W: f64 = 0.05;

/// A amplitude do Zig Zag, em PERCENTAGEM da forma (100 = média das dimensões). Generosa para as
/// cristas serem óbvias.
const ZZ_AMPLITUDE: f64 = 16.0;
const ZZ_RIDGES: f64 = 12.0;

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => select_hero(app),
        _ => {}
    }
}

fn zigzag() -> PathEffect {
    PathEffect::ZigZag(ZigZagSpec {
        amplitude: ZZ_AMPLITUDE,
        ridges: ZZ_RIDGES,
        smooth: true,
        rough_seed: None,
    })
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;

    // ── Esquerda: Zig Zag UNIFORME (o controle) ──────────────────────────────
    let mut left = shape(
        ShapeKind::Rectangle,
        [-3.4, -1.1],
        [-1.4, 1.1],
        &[],
        [70, 150, 220],
    );
    left.fill = None;
    left.stroke = Some(StrokeSpec::new(Rgba8::new(70, 150, 220, 255), STROKE_W));
    left.effects = vec![FxEntry::new(zigzag())];
    scene.push_path(left);

    // ── Meio: Zig Zag + Falloff LINEAR (as cristas crescem ao longo do eixo) ──
    let mut mid = shape(
        ShapeKind::Rectangle,
        [-1.0, -1.1],
        [1.0, 1.1],
        &[],
        [110, 200, 130],
    );
    mid.fill = None;
    mid.stroke = Some(StrokeSpec::new(Rgba8::new(110, 200, 130, 255), STROKE_W));
    // O Falloff vem ANTES do Zig Zag: ele modula a AMPLITUDE amostra a amostra. `angle 0` (+x),
    // `off_x 0` (linha média no centro), `softness 0.9` (a rampa cobre a largura da forma).
    let linear = FalloffSpec {
        shape: FalloffShape::Linear,
        amount: 1.0,
        size: 0.9,
        off_x: 0.0,
        ..FalloffSpec::new(FalloffShape::Linear)
    };
    mid.effects = vec![
        FxEntry::new(PathEffect::Falloff(linear)),
        FxEntry::new(zigzag()),
    ];
    scene.push_path(mid);

    // ── Direita: Zig Zag + Falloff RADIAL (cristas altas no miolo, planas nas pontas) ──
    let mut right = shape(
        ShapeKind::Rectangle,
        [1.4, -1.1],
        [3.4, 1.1],
        &[],
        [220, 150, 90],
    );
    right.fill = None;
    right.stroke = Some(StrokeSpec::new(Rgba8::new(220, 150, 90, 255), STROKE_W));
    // Radial forte no centro, `size 0.5` (raio ~= metade da forma): a onda cheia no miolo,
    // apagando-se para as pontas. Mesma força do meio, outra FORMA de campo.
    let radial = FalloffSpec {
        shape: FalloffShape::Radial,
        amount: 1.0,
        size: 0.5,
        ..FalloffSpec::new(FalloffShape::Radial)
    };
    right.effects = vec![
        FxEntry::new(PathEffect::Falloff(radial)),
        FxEntry::new(zigzag()),
    ];
    scene.push_path(right);
}

/// Seleciona o retângulo do MEIO (o Zig Zag modulado) — assim a seção **Effects** abre com a
/// pilha `[Falloff Linear, Zig Zag]` e o card do Falloff mostra a dica *"modulates the effect
/// below"*. Frame 4: depois do `sync`, para o alvo já ter entidade.
fn select_hero(app: &mut crate::App) {
    let mid: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().get(1).map(|p| p.id));
    if let Some(id) = mid {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] falloff: 3 retangulos, o MESMO Zig Zag, campos diferentes. ESQUERDA = sem campo \
         (cristas uniformes, controle). MEIO (selecionado) = **Falloff Linear** -> as cristas \
         crescem ao longo do eixo; na secao **Effects** o card do Falloff diz 'modulates the \
         effect below', e desligar o olho dele devolve as cristas uniformes. DIREITA = **Falloff \
         Radial** -> cristas altas no miolo, planas nas pontas. Ajuste Amount/Radius/Center/Curve \
         no card do Falloff. (Warp/Bloat modulados: provado nos gates do motor.)"
    );
}
