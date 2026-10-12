//! **Fase do quadro: «SEND TO BOARD…»** (MiroClone W15) — a composição: o pedido da linha abre o modal
//! (`ph2d_app_board::send_to_board::ask`), e a escolha lê o conteúdo do objecto NA FAMÍLIA DELE — os
//! píxeis da sprite, o SVG do vector (`ph2d_app_vec`), o quadro actual do Flip pela placa
//! (`ph2d_app_flip::pass::raster`) — e a família do quadro entrega-o (`deliver`).
//!
//! ⭐ A escala é a régua do editor em píxeis (`pixels_per_meter`): o vector e o Flip entram no quadro
//! com o tamanho que a régua lhes dá; a sprite com os píxeis dela.

use super::*;
use ph2d_app_board::send_to_board::{Content, ask, deliver};
use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::screens::hero::send_to_board::goes_to_board;
use ph2d_i18n::{tr, tr_with};

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_send_to_board(
        &mut self,
        asked: Option<NodeId>,
        chosen: Option<(NodeId, Option<u64>)>,
    ) {
        if asked.is_none() && chosen.is_none() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        let world = gfx.sim.world();
        let entity = |row: NodeId| {
            let bits = gfx.hero_live.as_ref()?.bridge.entity_for(row)?;
            let e = ph2d_ecs::Entity::from_bits(bits);
            world.get_entity(e).is_ok().then_some(e)
        };
        let kind = |e| ph2d_app_components::component_attach::kind_of(world, e);
        if let Some(row) = asked {
            let goes = entity(row).is_some_and(|e| goes_to_board(kind(e)));
            if let Some(t) = ask(hero, row, goes) {
                gfx.toasts.push(t);
            }
        }
        let Some((row, board)) = chosen else {
            return;
        };
        let Some(e) = entity(row) else {
            gfx.toasts.push(Toast::warning(tr("board.send.gone")));
            return;
        };
        let name = world
            .get::<ph2d_ecs::Name>(e)
            .map_or_else(String::new, |n| n.0.clone());
        let ppm = hero.project.pixels_per_meter;
        let unreadable = || Toast::warning(tr_with("board.send.unreadable", &[("name", &name)]));
        let content = match kind(e) {
            ObjectKind::Image => {
                // PRECISION-READONLY: LÊ os pixels e manda uma CÓPIA ao quadro (8 bits, o formato das
                // imagens do quadro); a sprite da cena nunca é escrita de volta.
                let src = crate::hero_intents::texture_edit::read_sprite_source(
                    e,
                    &gfx.sim,
                    &mut gfx.renderer,
                    &gfx.asset_db,
                    &gfx.atlas_asset_map,
                );
                let Some(src) = src else {
                    gfx.toasts.push(unreadable());
                    return;
                };
                let img = src.image.into_straight();
                Some(Content::rgba([img.width, img.height], img.pixels))
            }
            ObjectKind::Vector => world.get::<ph2d_ecs::VecPathRef>(e).and_then(|r| {
                let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                let vista = ph2d_vec_entities::entities::view_state(&gfx.sim, &self.vec.entities);
                ph2d_app_vec::svg_export::svg_of_path(
                    &gfx.vec_scene,
                    &xf,
                    r.0,
                    &|id| vista.is_derived(id),
                    f64::from(ppm),
                )
                .map(Content::Svg)
            }),
            ObjectKind::Flip => {
                let Some(r) = world.get::<ph2d_ecs::FlipObjectRef>(e) else {
                    gfx.toasts.push(unreadable());
                    return;
                };
                match ph2d_app_flip::pass::raster::current_frame(
                    &gfx.flip,
                    ph2d_flip::FlipObjectId(r.0),
                    ph2d_flip_entities::transform::object_xform(&gfx.sim, e),
                    &self.playhead,
                    &mut gfx.flip_render,
                    &mut gfx.flip_compose,
                    ppm,
                    gfx.surface.gpu(),
                ) {
                    Ok(f) => Some(Content::rgba(f.size, f.rgba)),
                    Err(ph2d_app_flip::pass::raster::RasterError::Empty) => None,
                    Err(ph2d_app_flip::pass::raster::RasterError::Gpu) => {
                        gfx.toasts.push(unreadable());
                        return;
                    }
                }
            }
            _ => {
                gfx.toasts.push(Toast::warning(tr("board.send.wrong_kind")));
                return;
            }
        };
        if let Err(t) = deliver(hero, board, &name, content) {
            gfx.toasts.push(t);
        }
    }
}
