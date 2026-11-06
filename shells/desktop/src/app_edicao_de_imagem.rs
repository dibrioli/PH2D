//! **Edit in Pixel**: a composição entre a imagem da sprite e o Pixel Art, duas famílias que não se conhecem. A lei mora
//! em `ph2d_sprite_edit` e na porta `ph2d_app_pixel::imagem`.

use ph2d_app_painter::documentos_no_projecto;
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
    let (mundo, atlas) = (gfx.sim.world(), &gfx.atlas_asset_map);
    let Some(((w, h, rgba), id)) = imagem::imagem_de(mundo, origem, &gfx.asset_db, atlas)
        .zip(imagem::shown_id(mundo, origem, atlas))
    else {
        gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
            "shell.edit_in_pixel.no_pixels",
        )));
        return;
    };
    let ppm = gfx
        .hero_screen
        .as_ref()
        .map_or(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER, |h| {
            h.project.pixels_per_meter
        });
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
        (key, id),
        (w, h),
        ppm,
        (gfx.camera.center, gfx.camera.height_world),
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
    (gfx.camera.center, gfx.camera.height_world) = a.vista;
    let viva = gfx.sim.world().get_entity(a.origem).is_ok();
    let mut heir = viva.then_some(a.origem);
    if let Some(img) = pixel.fecha_imagem(a.key) {
        heir = heir.or(leva_a_imagem(gfx, &a, img));
    }
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.replace_selection(heir.map(Entity::to_bits));
    }
}

/// Os píxeis vão a quem usa a imagem. Devolve o primeiro que a mostra, se há.
fn leva_a_imagem(
    gfx: &mut crate::AppGfx,
    a: &no_pixel::Aberta,
    img: (u32, u32, Vec<u8>),
) -> Option<Entity> {
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
        a.imagem,
        img,
    );
    let t = match r {
        Ok(t) => t,
        Err(e) => {
            gfx.toasts.push(ph2d_editor_core::Toast::warning(e));
            return None;
        }
    };
    if t.sem_dono() {
        gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
            "shell.edit_in_pixel.no_user",
        )));
    }
    if t.novo != t.antigo
        && let Some(p) = ph2d_app_painter::painter_bridge_queries::o_painter(&mut gfx.tools)
    {
        let sim = &mut gfx.sim;
        if documentos_no_projecto::achata(sim, p, &t.quem, t.antigo, t.novo) {
            gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.edit_in_pixel.layers_flattened",
            )));
        }
    }
    t.quem.first().copied()
}
