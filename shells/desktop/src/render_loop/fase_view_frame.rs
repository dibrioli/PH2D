//! **Fase do quadro: O PEDIDO DE ENQUADRAMENTO** (F8) — junta as fontes do quadro (a área onde o
//! conteúdo pode ficar, a ordem do que se desenha, o Flip) para `ph2d_app_vec::view_request`.

use super::*;
use ph2d_app_vec::view_request;
use ph2d_editor_core::ViewFocusKind;
use ph2d_editor_core::screens::hero::float_bar;
use ph2d_render::camera_frame::Area;

impl crate::App {
    /// Ver o cabeçalho do módulo. Corre depois das faixas: precisa das `VecXforms` do quadro.
    pub(super) fn fase_view_frame(
        &mut self,
        kind: Option<ViewFocusKind>,
        vec_xf: &ph2d_vec_scene::VecXforms,
        viewport: EditorRect,
    ) -> Option<()> {
        let kind = std::mem::replace(&mut self.view_request, kind)?;
        let gfx = self.gfx.as_mut()?;
        let window = gfx.scene_window();
        let FrameGfx {
            sim,
            present,
            camera,
            view_flight,
            toasts,
            vec_scene,
            flip,
            frame_order,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        let (sim, flip): (&SimWorld, &ph2d_flip::FlipDoc) = (sim, flip);
        let a = ph2d_app_host::canvas_area::frameable(hero, viewport);
        let sel = hero
            .gizmo
            .selection
            .iter()
            .chain(&hero.gizmo.extra_selection);
        let src = view_request::Sources {
            sim,
            present: present.world_mut(),
            scene: vec_scene,
            xf: vec_xf,
            drawn_paths: frame_order.vector_ranks.iter().map(|&(id, _)| id).collect(),
            others: ph2d_app_flip::gizmo_view::drawn_aabbs(sim, flip),
            other_box: &mut |e| ph2d_app_flip::gizmo_view::world_aabb(sim, flip, e),
            selection: sel.copied().collect(),
            area: Area {
                x: a.x,
                y: a.y,
                w: a.w,
                h: a.h,
            },
            window,
            margin: float_bar::frame_margin() * hero.escala().factor(),
        };
        let track = hero.motion.get(crate::canvas_zoom::VIEW_FLIGHT);
        let label = view_request::resolve(kind, src, camera, view_flight, track)?;
        toasts.push(Toast::info(ph2d_i18n::tr(label)));
        self.title_dirty = true;
        Some(())
    }
}
