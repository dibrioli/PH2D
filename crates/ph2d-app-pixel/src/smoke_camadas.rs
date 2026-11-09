//! **A cena 7 — as camadas que SE VÊEM** (P3). Um Pixel Sprite RGBA 190 × 96 em Draw:
//!
//! - em cima, o degradê da cena 4 na **Background** e, por cima dele, o grupo recolhido
//!   **«19 modes»** com dezanove faixas de 10 colunas — a MESMA tinta (laranja em cima, azul em
//!   baixo, com um cinza a meio) em cada uma, cada faixa na sua camada e com o seu modo, na ordem do
//!   menu do Aseprite: a diferença entre os modos lê-se lado a lado (o teste prova que as 19 faixas
//!   compostas são duas a duas diferentes);
//! - em baixo, sobre o branco da Background, da esquerda para a direita: um quadrado azul-escuro a
//!   **50 %** de opacidade, uma estrela
//!   numa camada **escondida** (o olho acende-a), uma camada **trancada** (pintar nela recusa e diz
//!   porquê), uma **referência** (vê-se; o Flatten deixa-a de fora) e um **grupo** aberto com duas
//!   formas (o olho do grupo apaga as duas).
//!
//! A camada activa é a do quadrado a 50 % (o campo Opacity mostra 128).

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{ColorMode, LayerId, LayerKind, NewSprite, Rect, Sprite};

use crate::PixelApp;

pub use ph2d_pixel_cenas::camadas::*;

fn put(
    s: &mut Sprite,
    key: u32,
    app: &mut PixelApp,
    layer: LayerId,
    r: Rect,
    f: impl Fn(i32, i32) -> [u8; 4],
) -> Option<()> {
    let frame = s.frames()[0].id;
    // a caixa do cel cresce: pintar a banda de cima NÃO corta o resto da Background
    let bbox = s.cel((layer, frame)).map_or(r, |c| c.bbox.union(r));
    let v = |x, y| s.value_of_rgba(f(x, y)).to_le_bytes();
    app.commit(key, ph2d_pixel_cenas::patch::op(layer, frame, bbox, r, v))
        .ok()?;
    *s = app.docs.get(&key)?.sprite.clone();
    Some(())
}

/// Uma camada nova acima de `above` com o nome dado; devolve o id.
fn nova(app: &mut PixelApp, key: u32, above: LayerId, name: &str, group: bool) -> Option<LayerId> {
    let doc = app.docs.get_mut(&key)?;
    let (op, id) = if group {
        layers::new_group(&mut doc.sprite, above)?
    } else {
        layers::new_layer(&mut doc.sprite, above)?
    };
    app.commit(key, op).ok()?;
    let s = &app.docs.get(&key)?.sprite;
    let op = layers::edit(s, id, |l| name.clone_into(&mut l.name))?;
    app.commit(key, op).ok()?;
    Some(id)
}

fn edita(
    app: &mut PixelApp,
    key: u32,
    id: LayerId,
    f: impl FnOnce(&mut ph2d_pixel_core::Layer),
) -> Option<()> {
    // sem mudança (o Normal de uma camada que já é Normal) não há passo
    match layers::edit(&app.docs.get(&key)?.sprite, id, f) {
        Some(op) => app.commit(key, op).ok(),
        None => Some(()),
    }
}

/// Põe `id` no topo do grupo `g` (nada a fazer se já é filha dele: nasceu acima de uma irmã).
fn entra(app: &mut PixelApp, key: u32, id: LayerId, g: LayerId) -> Option<()> {
    let s = &app.docs.get(&key)?.sprite;
    if s.layer(id)?.parent == Some(g) {
        return Some(());
    }
    let op = layers::move_to(s, id, Place::Into(g))?;
    app.commit(key, op).ok()
}

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
        background: ph2d_pixel_core::Background::White,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let mut s = app.docs.get(&key)?.sprite.clone();
    let bg = s.layers()[0].id;
    let (gw, gh) = crate::smoke::GRADIENT_SIZE;
    put(&mut s, key, app, bg, Rect::new(0, 0, w, BAND_H), |x, y| {
        crate::smoke::gradient_rgba(x as u32 * gw / w, y as u32 * gh / BAND_H)
    })?;
    let grupo = nova(app, key, bg, "19 modes", true)?;
    let mut abaixo = grupo;
    for (i, m) in BlendMode::ALL.iter().enumerate() {
        let nome = ph2d_i18n::tr(crate::camadas::blend_of(*m).label_key());
        let id = nova(app, key, abaixo, nome, false)?;
        entra(app, key, id, grupo)?;
        s = app.docs.get(&key)?.sprite.clone();
        put(&mut s, key, app, id, stripe_rect(i), |_, y| {
            stripe_rgba(y as u32)
        })?;
        edita(app, key, id, |l| l.blend = *m)?;
        abaixo = id;
    }
    edita(app, key, grupo, |l| {
        l.kind = LayerKind::Group { collapsed: true }
    })?;
    // a banda de baixo
    let opa = nova(app, key, grupo, "Opacity 50%", false)?;
    // azul-escuro a 50 % sobre o branco: a metade lê-se (um branco sobre branco não mostrava nada)
    put(&mut s, key, app, opa, OPACITY_BOX, |_, _| {
        [20, 40, 160, 255]
    })?;
    edita(app, key, opa, |l| l.opacity = 128)?;
    let esc = nova(app, key, opa, "Hidden star", false)?;
    put(&mut s, key, app, esc, HIDDEN_STAR, |x, y| {
        estrela(HIDDEN_STAR, x, y)
    })?;
    edita(app, key, esc, |l| l.visible = false)?;
    let tr = nova(app, key, esc, "Locked", false)?;
    put(&mut s, key, app, tr, LOCKED_BOX, |x, y| {
        if (x + y) % 6 < 3 {
            [200, 60, 60, 255]
        } else {
            [120, 30, 30, 255]
        }
    })?;
    edita(app, key, tr, |l| l.locked = true)?;
    let rf = nova(app, key, tr, "Reference", false)?;
    put(&mut s, key, app, rf, REFERENCE_BOX, |_, y| {
        let v = (y - REFERENCE_BOX.y) as u8 * 5;
        [60 + v, 200 - v, 255, 255]
    })?;
    edita(app, key, rf, |l| l.reference = true)?;
    let g2 = nova(app, key, rf, "Group", true)?;
    let ga = nova(app, key, g2, "Circle", false)?;
    entra(app, key, ga, g2)?;
    put(&mut s, key, app, ga, GROUP_A, |x, y| {
        let (dx, dy) = (x - 165, y - 64);
        if dx * dx + dy * dy <= 49 {
            [40, 200, 90, 255]
        } else {
            [0; 4]
        }
    })?;
    let gb = nova(app, key, ga, "Square", false)?;
    entra(app, key, gb, g2)?;
    put(&mut s, key, app, gb, GROUP_B, |_, _| [240, 140, 30, 255])?;
    let doc = app.docs.get_mut(&key)?;
    doc.layer = opa;
    app.tool = crate::Tool::Pencil;
    app.fg = [0, 0, 0, 255];
    app.bg = [255, 255, 255, 255];
    Some(key)
}
