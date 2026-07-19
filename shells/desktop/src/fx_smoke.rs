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
//! A seção "Effects" no painel JÁ existe (o card por efeito + o menu Add). O nível **14** é a
//! cena do **Apply / Convert** (2026-07-19): uma forma com um efeito ATIVO, ESTÁTICA e
//! selecionada, para se clicar o botão **Apply Effects** (assa a pilha) ou **Convert to Curves**
//! (que agora também assa os efeitos). Sem animação — de propósito: o nível 13 reescreve
//! `effects` a cada frame, e por cima disso um bake nunca persistiria.

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_trim::TrimSpec;
use ph2d_vec_scene::fx_zigzag::ZigZagSpec;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPathId};

use crate::build_smoke::shape;

/// Quantos frames a elipse leva a desenhar-se. ~3 s a 60 fps — devagar o bastante para se
/// ver a ponta a andar, e não tão devagar que o smoke pareça travado.
const DRAW_ON_FRAMES: u32 = 180;

/// Quantos frames a elipse fica CHEIA antes de recomeçar. ~1 s: tempo de ler "acabou" sem
/// que a cena pareça parada.
const HOLD_FRAMES: u32 = 60;

/// O ciclo completo do draw-on.
const CYCLE: u32 = DRAW_ON_FRAMES + HOLD_FRAMES;

/// **A fração revelada da elipse no frame `t`** — desenha, segura cheia, e RECOMEÇA.
///
/// ⚠️ **O laço não é enfeite: é o que torna a cena smokável.** A 1ª versão era one-shot
/// (subia de 0 a 1 e ficava), e o smoke do Enio devolveu *"na elipse não vejo nada
/// acontecendo"* — a rampa tinha acabado antes de ele olhar para a janela, e o que restava
/// era uma elipse inteira e parada. Um exemplo que exige apanhar uma janela de 3 s no
/// arranque não está pronto para smoke.
/// [[feedback_ready_to_smoke_example]]
fn draw_on_phase(t: u32) -> f64 {
    let k = t % CYCLE;
    (f64::from(k) / f64::from(DRAW_ON_FRAMES)).min(1.0)
}

/// **O giro da janela da estrela** — contínuo, sem pausa: ela não desenha, ela CORRE.
fn spin_phase(t: u32) -> f64 {
    (f64::from(t) / f64::from(CYCLE)).rem_euclid(1.0)
}

/// Largura do traço, em unidades de MUNDO (a cena vive numa caixa de ~±3.5).
const STROKE_W: f64 = 0.06;

/// Os dois paths da cena, guardados entre frames para o ramp poder editá-los.
static IDS: std::sync::Mutex<Vec<VecPathId>> = std::sync::Mutex::new(Vec::new());

pub(crate) fn frame(app: &mut crate::App, f: u32, level: u32) {
    match level {
        // A pilha ANIMADA (draw-on + janela a girar).
        13 => match f {
            3 => build(app),
            _ if f > 3 => animate(app, f - 3),
            _ => {}
        },
        // A cena do **Apply / Convert** — estática, e SELECIONA no frame 4 (depois de o `sync`
        // ter dado entidade à forma), para o smoke ser um clique só.
        14 => match f {
            3 => build_apply(app),
            4 => select_the_shape(app),
            _ => {}
        },
        _ => {}
    }
}

/// A cena do **Apply / Convert** (`PH2D_BUILD_SMOKE=14`): UMA elipse com um Zig Zag ATIVO, a
/// borda visivelmente rugosa. Estática de propósito — o bake tem de PERSISTIR, e o nível 13
/// reescreveria os `effects` a cada frame.
fn build_apply(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;
    let p = shape(
        ShapeKind::Ellipse,
        [-1.6, -1.6],
        [1.6, 1.6],
        &[],
        [90, 150, 220],
    );
    let id = scene.push_path(p);
    // Zig Zag ATIVO: amplitude e cristas em número que a borda mostra sem dúvida. `amplitude`
    // é PERCENTAGEM da forma (100 = média de largura/altura), então 12 desenha o mesmo em
    // qualquer escala.
    if let Some(p) = scene.path_mut(id) {
        p.effects = vec![FxEntry::new(PathEffect::ZigZag(ZigZagSpec {
            amplitude: 12.0,
            ridges: 24.0,
            smooth: false,
            rough_seed: None,
        }))];
    }
}

/// Seleciona a forma (a única da cena) — assim a seção **Effects** publica o alvo e oferece o
/// card + **Apply Effects**, e o **Convert to Curves** fica habilitado.
fn select_the_shape(app: &mut crate::App) {
    let ids: Vec<VecPathId> = app
        .gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default();
    app.vec_pen.select_many(&ids);
    eprintln!(
        "[smoke] apply/convert: 1 elipse com Zig Zag ATIVO, selecionada. Na seção **Effects** \
         clique **Apply Effects** (assa a pilha na geometria); ou **Convert to Curves** (agora \
         também assa efeitos). O card some e a borda rugosa vira curva editável — entre no modo \
         **Node** e veja as âncoras reais."
    );
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
        p.effects = vec![FxEntry::new(PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: 0.0,
            offset: 0.0,
        }))];
        ids.push(scene.push_path(p));
    }
    // A estrela abre já com a janela de um quarto — ela não desenha, ela CORRE.
    if let Some(p) = ids.get(1).and_then(|&i| scene.path_mut(i)) {
        p.effects = vec![FxEntry::new(PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: 0.25,
            offset: 0.0,
        }))];
    }
    *IDS.lock().expect("ids") = ids;
}

/// Anima: a elipse cresce o `end`, a estrela gira o `offset`.
fn animate(app: &mut crate::App, t: u32) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids = IDS.lock().expect("ids").clone();
    let scene = &mut gfx.vec_scene;

    if let Some(p) = ids.first().and_then(|&i| scene.path_mut(i)) {
        p.effects = vec![FxEntry::new(PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: draw_on_phase(t),
            offset: 0.0,
        }))];
    }
    if let Some(p) = ids.get(1).and_then(|&i| scene.path_mut(i)) {
        p.effects = vec![FxEntry::new(PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: 0.25,
            offset: spin_phase(t),
        }))];
    }
}

#[cfg(test)]
#[path = "fx_smoke_tests.rs"]
mod tests;
