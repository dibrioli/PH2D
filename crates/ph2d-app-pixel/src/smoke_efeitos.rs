//! **A cena 19 — a P8** (Gradient, Blur/Jumble, Text, Edit ▸ Fill/Stroke, Adjustments, FX). Um
//! Pixel Sprite RGBA 64 × 48 em Draw: a camada Background branca e, por cima, uma camada normal
//! TRANSPARENTE e activa com um boneco de três cores ([`boneco`]) — sem selecção, os ajustes agem no
//! boneco, o Outline contorna-o e a borracha apaga para transparente (na Background pintaria a cor de
//! fundo). A rampa de seis tons ([`RAMP`]) fica no fim da paleta, com a frente na ponta escura e o
//! fundo na clara — o Gradient vê-se e o «Palette ramp» anda nela.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, layers};

use crate::{PixelApp, Tool};

pub use ph2d_pixel_cenas::efeitos::*;

pub(crate) fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::White,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let (fundo, frame, mut colors, transparent) = {
        let d = app.docs.get(&key)?;
        let s = &d.sprite;
        (
            d.layer,
            d.frame,
            s.palette().to_vec(),
            s.transparent_index(),
        )
    };
    colors.extend(RAMP);
    app.commit(
        key,
        Op::Palette {
            colors,
            transparent,
        },
    )
    .ok()?;
    let (op, layer) = layers::new_layer(&mut app.docs.get_mut(&key)?.sprite, fundo)?;
    app.commit(key, op).ok()?;
    let r = FIGURE;
    let pixels = (r.y..r.y + r.h as i32)
        .flat_map(|y| (r.x..r.x + r.w as i32).flat_map(move |x| boneco(x, y).unwrap_or([0; 4])))
        .collect();
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels,
    };
    app.commit(key, Op::Patch(p)).ok()?;
    app.docs.get_mut(&key)?.layer = layer;
    crate::input::set_tool(app, Tool::Gradient);
    app.fg = RAMP[0];
    app.bg = RAMP[RAMP.len() - 1];
    Some(key)
}

/// Só para as fotos (`PH2D_PIXEL_SMOKE_GESTO`, cena 19), uma vez: `balde` = o flyout do balde
/// aberto · `convolucao` = o diálogo com a lista «Kernel» aberta · `hue` = o diálogo aberto ·
/// `texto` = a caixa do Text com «PH2D» · `fonte` = o Text com a lista de fontes aberta · `solto` =
/// «PH2D» depois do 1.º Enter (a moldura solta) — os eventos sintéticos não chegam à janela da tela
/// virtual (`fotografa_cena.sh`); as listas abrem-se no estado do dropdown, como faz o clique.
pub(crate) fn conduz(app: &mut PixelApp, hero: &mut ph2d_editor_core::screens::hero::HeroScreen) {
    use ph2d_editor_core::pixel_panel::{PixelEditAction as E, PixelRequest as R, PixelToolGroup};
    use std::sync::atomic::{AtomicBool, Ordering};
    use winit::keyboard::KeyCode;
    static FEITO: AtomicBool = AtomicBool::new(false);
    if crate::smoke::armed_level() != Some(19) {
        return;
    }
    let gesto = std::env::var("PH2D_PIXEL_SMOKE_GESTO").unwrap_or_default();
    if FEITO.load(Ordering::Relaxed) {
        abre_lista(hero, &gesto);
        return;
    }
    let Some(key) = app.chave else {
        return;
    };
    if gesto.is_empty() || FEITO.swap(true, Ordering::Relaxed) {
        return;
    }
    let m = crate::compose::Mods::default();
    match gesto.as_str() {
        "balde" => app.pede(R::Group(PixelToolGroup::Bucket)),
        "convolucao" => app.pede(R::Edit(E::ConvolutionMatrix)),
        "hue" => app.pede(R::Edit(E::HueSaturation)),
        "fonte" => crate::input::set_tool(app, Tool::Text),
        "texto" | "solto" => {
            app.fg = [0, 0, 0, 255];
            crate::input::set_tool(app, Tool::Text);
            crate::texto::abre(app, key, (26, 34));
            crate::texto::tecla(app, KeyCode::KeyA, m, Some("PH2D"));
            if gesto == "solto" {
                crate::texto::tecla(app, KeyCode::Enter, m, None);
            }
        }
        _ => {}
    }
}

/// Abre, uma vez, a lista do dropdown do gesto assim que ele existe no `WidgetStore` (o da Kernel
/// só se regista quando o diálogo abre, um quadro depois do pedido).
fn abre_lista(hero: &mut ph2d_editor_core::screens::hero::HeroScreen, gesto: &str) {
    use ph2d_editor_core::interaction::InteractiveState;
    use std::sync::atomic::{AtomicBool, Ordering};
    static ABERTA: AtomicBool = AtomicBool::new(false);
    let id = match gesto {
        "convolucao" => ph2d_editor_core::ids::PIXEL_FILTER_KERNEL,
        "fonte" => ph2d_editor_core::ids::PIXEL_TEXT_FONT_DD,
        _ => return,
    };
    if ABERTA.load(Ordering::Relaxed) {
        return;
    }
    if let Some(InteractiveState::Dropdown { open, .. }) = hero.store.get_mut(id) {
        *open = true;
        ABERTA.store(true, Ordering::Relaxed);
    }
}
