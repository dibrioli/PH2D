//! **Edit in Pixel**: a composição entre a imagem da sprite e o Pixel Art, duas famílias que não se conhecem. A lei mora
//! em `ph2d_sprite_edit` e na porta `ph2d_app_pixel::imagem`.

use ph2d_ecs::Entity;
use ph2d_i18n::tr;
use ph2d_sprite_edit::{imagem, no_pixel};

/// Abre a imagem de `bits` no Pixel Art e selecciona o objecto de trabalho.
pub(crate) fn abre_no_pixel(
    gfx: &mut crate::AppGfx,
    pixel: &mut ph2d_app_pixel::PixelApp,
    bits: u64,
) {
    let origem = Entity::from_bits(bits);
    if no_pixel::aberta().is_some() {
        fecha_agora(gfx, pixel);
    }
    let Some((w, h, rgba)) =
        imagem::imagem_de(gfx.sim.world(), origem, &gfx.asset_db, &gfx.atlas_asset_map)
    else {
        gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
            "shell.edit_in_pixel.no_pixels",
        )));
        return;
    };
    let Some(key) = pixel.abre_imagem(w, h, rgba) else {
        gfx.toasts
            .push(ph2d_editor_core::Toast::warning(tr("object_add.not_born")));
        return;
    };
    let nome = ph2d_app_pixel::object_add::PIXEL_SPRITE.key.tr();
    let trabalho = ph2d_app_components::object_add::spawn_empty_root(&mut gfx.sim, nome);
    no_pixel::abre(
        gfx.sim.world_mut(),
        origem,
        Entity::from_bits(trabalho),
        key,
    );
    pixel.edita_ao_nascer(trabalho);
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.replace_selection(Some(trabalho));
    }
}

/// Uma vez por quadro: se o artista saiu do Draw, os píxeis voltam a todo objecto que usa a imagem.
pub(crate) fn fecha_se_terminou(gfx: &mut crate::AppGfx, pixel: &mut ph2d_app_pixel::PixelApp) {
    let em_draw = gfx
        .hero_screen
        .as_ref()
        .and_then(|hero| ph2d_app_pixel::mode::editing(gfx.sim.world(), hero));
    if let Some(a) = no_pixel::terminou(gfx.sim.world_mut(), em_draw) {
        devolve(gfx, pixel, a);
    }
}

/// Antes de gravar: o objecto de trabalho não é do artista, e a sprite não pode ficar escondida no ficheiro.
pub(crate) fn fecha_agora(gfx: &mut crate::AppGfx, pixel: &mut ph2d_app_pixel::PixelApp) {
    if let Some(a) = no_pixel::encerra(gfx.sim.world_mut()) {
        devolve(gfx, pixel, a);
    }
}

fn devolve(gfx: &mut crate::AppGfx, pixel: &mut ph2d_app_pixel::PixelApp, a: no_pixel::Aberta) {
    let Some(img) = pixel.fecha_imagem(a.key) else {
        return;
    };
    let renderer = &mut gfx.renderer;
    let r = imagem::troca_os_pixeis(
        gfx.sim.world_mut(),
        &mut |w, h, px| {
            renderer
                .acquire_individual(w, h, px)
                .map_err(|e| e.to_string())
        },
        &gfx.asset_db,
        &gfx.atlas_asset_map,
        a.origem,
        img,
    );
    if let Err(e) = r {
        gfx.toasts.push(ph2d_editor_core::Toast::warning(e));
    }
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.replace_selection(Some(a.origem.to_bits()));
    }
}
