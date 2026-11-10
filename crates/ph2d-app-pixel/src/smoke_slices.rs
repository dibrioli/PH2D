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

/// As cenas 26 e 27 (P14 §8): o `heroi_da_cena` no mundo no arranjo `a` (`ph2d_pixel_cenas::heroi`).
pub fn stage_heroi(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
    a: &ph2d_pixel_cenas::heroi::Arranjo,
) -> Option<u64> {
    use ph2d_pixel_cenas::heroi as h;
    let bits = crate::object_add::spawn(sim, app, NewSprite::default(), spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let s = h::heroi_da_cena();
    let doc_de = |app: &mut PixelApp, k: u32, s: Sprite| {
        let d = app.docs.get_mut(&k)?;
        (d.frame, d.layer) = (s.frames()[0].id, s.layers()[0].id);
        d.sprite = s;
        Some(())
    };
    doc_de(app, key, s.clone())?;
    let chao = h::bloco(a.chao.0, a.chao.1, h::COR_CHAO, Some("ground"))?;
    let parede = h::bloco(a.parede.0, a.parede.1, h::COR_PAREDE, None)?;
    let nova = |app: &mut PixelApp, s: Sprite| {
        let k = app.create(NewSprite::default()).ok()?;
        doc_de(app, k, s).map(|()| k)
    };
    let k = h::Chaves {
        chao: nova(app, chao.clone())?,
        parede: nova(app, parede.clone())?,
    };
    let e = ph2d_ecs::Entity::from_bits(bits);
    h::monta(sim.world_mut(), e, &s, &k, &chao, a)?;
    if a.segunda_parede {
        let p2 = nova(app, parede)?;
        ph2d_pixel_cenas::heroi27::monta(sim.world_mut(), p2, &s, key)?;
    }
    app.wants_edit = None;
    Some(bits)
}

/// As cenas 26 e 27 abrem enquadradas; `PH2D_PIXEL_SMOKE_PLAY=1` entra em Play
/// (`ph2d_app_host::smoke_drive`). Na 27, o `.ase` do herói vai ao File ▸ Import respondido «Sprite».
pub(crate) fn conduz_o_heroi(hero: &mut ph2d_editor_core::screens::hero::HeroScreen) {
    static Q: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if let Some(p) = ph2d_pixel_cenas::heroi27::pendente() {
        hero.store.import_as_image(vec![p]);
    }
    ph2d_pixel_cenas::p13::conduz(hero, crate::smoke::armed_level());
    if matches!(crate::smoke::armed_level(), Some(26 | 27 | 31 | 34)) {
        let q = Q.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let fisica = ph2d_panel_timeline::ids::TIMELINE_PHYSICS;
        ph2d_app_host::smoke_drive::enquadra_e_joga(hero, q, "PH2D_PIXEL_SMOKE_PLAY", fisica);
    }
}
