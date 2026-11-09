//! A cena 24 (P11): o botão e o boneco de `ph2d_pixel_cenas::slices`, com a Slice Tool na mão.

use ph2d_ecs::SimWorld;
pub use ph2d_pixel_cenas::slices::*;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Sprite};

use crate::{PixelApp, Tool};

pub fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let spec = NewSprite {
        width: SIZE.0,
        height: SIZE.1,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get_mut(&key)?;
    let (arte, hero) = monta(&mut doc.sprite)?;
    (doc.frame, doc.layer) = (doc.sprite.frames()[0].id, arte);
    app.slices.sel.insert(key, vec![hero]);
    app.tool = Tool::Slice;
    Some(bits)
}

/// A cena 25 (P12): o boneco de `ph2d_pixel_cenas::historia`, os passos pela história e as duas janelas.
pub fn stage_historia(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    use ph2d_editor_core::pixel_panel::PixelEditAction as Edit;
    use ph2d_pixel_cenas::historia as h;
    let bits = crate::object_add::spawn(sim, app, h::SPEC, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    h::monta(&mut app.docs.get_mut(&key)?.sprite)?;
    for i in 0..h::PASSOS.len() {
        let (op, label) = h::passo(&mut app.docs.get_mut(&key)?.sprite, i)?;
        app.commit_labeled(key, op, label).ok()?;
    }
    let (keys, ficheiro) = h::teclas()?;
    ph2d_panel_pixel_keys::canal::usa_ficheiro(Some(ficheiro));
    (app.keys, app.onion_on) = (keys, true);
    for e in [Edit::UndoHistory, Edit::KeyboardShortcuts] {
        app.pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(e));
    }
    Some(bits)
}

/// A cena 26 (P14 §8): o `heroi_p14` no mundo, pelas peças de `ph2d_pixel_cenas::heroi`.
pub fn stage_heroi(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    use ph2d_pixel_cenas::heroi as h;
    let bits = crate::object_add::spawn(sim, app, NewSprite::default(), spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let doc_de = |app: &mut PixelApp, k: u32, s: Sprite| {
        let d = app.docs.get_mut(&k)?;
        (d.frame, d.layer) = (s.frames()[0].id, s.layers()[0].id);
        d.sprite = s;
        Some(())
    };
    doc_de(app, key, s.clone())?;
    let chao = h::bloco(h::CHAO.0, h::CHAO.1, h::COR_CHAO, Some("ground"))?;
    let parede = h::bloco(h::PAREDE.0, h::PAREDE.1, h::COR_PAREDE, None)?;
    let nova = |app: &mut PixelApp, s: Sprite| {
        let k = app.create(NewSprite::default()).ok()?;
        doc_de(app, k, s).map(|()| k)
    };
    let k = h::Chaves {
        chao: nova(app, chao.clone())?,
        parede: nova(app, parede)?,
    };
    h::monta(
        sim.world_mut(),
        ph2d_ecs::Entity::from_bits(bits),
        &s,
        &k,
        &chao,
    )?;
    app.wants_edit = None;
    Some(bits)
}
