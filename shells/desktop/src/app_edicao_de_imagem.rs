//! **Edit in Pixel**: a composição entre a imagem da sprite e o Pixel Art, duas famílias que não se conhecem. A lei mora
//! em `ph2d_app_pixel::imagem` e `ph2d_app_sprite_anim::sprite_card`.

use ph2d_ecs::Entity;
use ph2d_i18n::tr;

/// Abre a imagem de `bits` no Pixel Art e selecciona o objecto de trabalho.
pub(crate) fn abre_no_pixel(gfx: &mut crate::Gfx, pixel: &mut ph2d_app_pixel::PixelApp, bits: u64) {
    let e = Entity::from_bits(bits);
    let imagem = ph2d_app_sprite_anim::sprite_card::imagem_de(
        gfx.sim.world(),
        e,
        &gfx.asset_db,
        &gfx.atlas_asset_map,
    );
    let aberta = imagem.ok_or(tr("shell.edit_in_pixel.no_pixels")).and_then(|im| {
        ph2d_app_pixel::imagem::abre(
            &mut gfx.sim,
            pixel,
            bits,
            im,
            ph2d_app_components::object_add::spawn_empty_root,
        )
    });
    match aberta {
        Ok(trabalho) => {
            if let Some(hero) = gfx.hero_screen.as_mut() {
                hero.gizmo.replace_selection(Some(trabalho));
            }
        }
        Err(e) => gfx.toasts.push(ph2d_editor_core::Toast::warning(e)),
    }
}

/// Uma vez por quadro: se o artista saiu do Draw, os píxeis voltam a todo objecto que usa a imagem.
pub(crate) fn fecha_se_terminou(gfx: &mut crate::Gfx, pixel: &mut ph2d_app_pixel::PixelApp) {
    let em_draw = gfx
        .hero_screen
        .as_ref()
        .and_then(|hero| ph2d_app_pixel::mode::editing(gfx.sim.world(), hero));
    if let Some(d) = ph2d_app_pixel::imagem::terminou(gfx.sim.world_mut(), pixel, em_draw) {
        devolve(gfx, d);
    }
}

/// Antes de gravar: o objecto de trabalho não é do artista, e a sprite não pode ficar escondida no ficheiro.
pub(crate) fn fecha_agora(gfx: &mut crate::Gfx, pixel: &mut ph2d_app_pixel::PixelApp) {
    if let Some(d) = ph2d_app_pixel::imagem::encerra(gfx.sim.world_mut(), pixel) {
        devolve(gfx, d);
    }
}

fn devolve(gfx: &mut crate::Gfx, d: ph2d_app_pixel::imagem::Devolvida) {
    let renderer = &mut gfx.renderer;
    let r = ph2d_app_sprite_anim::sprite_card::troca_os_pixeis(
        gfx.sim.world_mut(),
        &mut |w, h, px| {
            renderer
                .acquire_individual(w, h, px)
                .map_err(|e| e.to_string())
        },
        &gfx.asset_db,
        &gfx.atlas_asset_map,
        Entity::from_bits(d.origem),
        (d.w, d.h, d.rgba),
    );
    if let Err(e) = r {
        gfx.toasts.push(ph2d_editor_core::Toast::warning(e));
    }
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.replace_selection(Some(d.origem));
    }
}
