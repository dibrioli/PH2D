//! **A cena pronta para o smoke da PILHA de efeitos** (ADR-0132) — `PH2D_BUILD_SMOKE=13`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `envelope_smoke`.
//!
//! Duas formas, e cada uma prova uma coisa diferente:
//!
//! - **A elipse DESENHA-SE SOZINHA.** O `end` do Trim sobe de 0 a 1 ao longo de ~3 s e
//!   fica. É o *draw-on*, e é o que torna o efeito impossível de confundir com um recorte
//!   estático. Ele também é a prova de que a medida é por ARCO: a ponta anda a **velocidade
//!   constante**, e não acelera nas curvas — que é exatamente o que a versão ingênua (fatiar
//!   por `t`) faria numa forma cujos segmentos têm curvatura desigual.
//! - **A estrela mostra a JANELA.** Trim fixo de um quarto do caminho, com `offset` a girar —
//!   o traço corre à volta da forma e **atravessa a emenda** sem tropeçar.
//!
//! O que NÃO há ainda é a seção "Effects" no painel: a pilha é dado de documento e esta cena
//! escreve-a por código. Ver o handoff — a UI é o passo seguinte, e a costura de painel é a
//! parte que a DIRETIVA manda não improvisar.

use ph2d_vec_scene::effect::PathEffect;
use ph2d_vec_scene::fx_trim::TrimSpec;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPathId};

use crate::build_smoke::shape;

/// Quantos frames a elipse leva a desenhar-se. ~3 s a 60 fps — devagar o bastante para se
/// ver a ponta a andar, e não tão devagar que o smoke pareça travado.
const DRAW_ON_FRAMES: f64 = 180.0;

/// Largura do traço, em unidades de MUNDO (a cena vive numa caixa de ~±3.5).
const STROKE_W: f64 = 0.06;

/// Os dois paths da cena, guardados entre frames para o ramp poder editá-los.
static IDS: std::sync::Mutex<Vec<VecPathId>> = std::sync::Mutex::new(Vec::new());

pub(crate) fn frame(app: &mut crate::App, f: u32, level: u32) {
    if level != 13 {
        return;
    }
    match f {
        3 => build(app),
        _ if f > 3 => animate(app, f - 3),
        _ => {}
    }
}

/// Monta a cena: elipse + estrela, ambas **só com traço** — um Trim revela o TRAÇO, e um
/// preenchimento por cima esconderia justamente o que se quer ver.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;

    let mut ids = Vec::new();
    for (kind, a, b, v, rgb) in [
        (
            ShapeKind::Ellipse,
            [-3.4, -1.2],
            [-0.6, 1.2],
            &[][..],
            [70, 150, 220],
        ),
        (
            ShapeKind::Star,
            [0.6, -1.2],
            [3.4, 1.2],
            &[5.0, 0.45, 0.0][..],
            [220, 150, 70],
        ),
    ] {
        let mut p = shape(kind, a, b, v, rgb);
        p.fill = None; // o Trim revela o TRAÇO
        p.stroke = Some(StrokeSpec::new(
            Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            STROKE_W,
        ));
        p.effects = vec![PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: 0.0,
            offset: 0.0,
        })];
        ids.push(scene.push_path(p));
    }
    // A estrela abre já com a janela de um quarto — ela não desenha, ela CORRE.
    if let Some(p) = ids.get(1).and_then(|&i| scene.path_mut(i)) {
        p.effects = vec![PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: 0.25,
            offset: 0.0,
        })];
    }
    *IDS.lock().expect("ids") = ids;
}

/// Anima: a elipse cresce o `end`, a estrela gira o `offset`.
fn animate(app: &mut crate::App, t: u32) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids = IDS.lock().expect("ids").clone();
    let phase = f64::from(t) / DRAW_ON_FRAMES;
    let scene = &mut gfx.vec_scene;

    if let Some(p) = ids.first().and_then(|&i| scene.path_mut(i)) {
        p.effects = vec![PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: phase.clamp(0.0, 1.0),
            offset: 0.0,
        })];
    }
    if let Some(p) = ids.get(1).and_then(|&i| scene.path_mut(i)) {
        p.effects = vec![PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: 0.25,
            offset: phase.rem_euclid(1.0),
        })];
    }
}
