//! **Fase do quadro: O PEDIDO DE ENQUADRAMENTO** (F8) — junta as fontes do quadro (a área onde o
//! conteúdo pode ficar, a ordem do que se desenha, o Flip) para `ph2d_app_vec::view_request`.

use super::*;
use ph2d_app_vec::view_request;
use ph2d_editor_core::ViewFocusKind;

impl crate::App {
    /// Ver o cabeçalho do módulo. Corre depois das faixas: precisa das `VecXforms` do quadro.
    pub(super) fn fase_view_frame(
        &mut self,
        kind: Option<ViewFocusKind>,
        vec_xf: &ph2d_vec_scene::VecXforms,
        viewport: EditorRect,
    ) -> Option<()> {
        let gfx = self.gfx.as_mut()?;
        let kind = gfx.view.defer(kind)?;
        let window = gfx.scene_window();
        let FrameGfx {
            sim,
            present,
            camera,
            view,
            toasts,
            vec_scene,
            flip,
            frame_order,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        let (area, margin) = ph2d_app_host::canvas_area::frame_target(hero, viewport);
        let (sim, flip): (&SimWorld, &ph2d_flip::FlipDoc) = (sim, flip);
        let src = view_request::Sources {
            sim,
            present: present.world_mut(),
            scene: vec_scene,
            xf: vec_xf,
            drawn_paths: frame_order.vector_ranks.iter().map(|&(id, _)| id).collect(),
            others: ph2d_app_flip::gizmo_view::drawn_aabbs(sim, flip),
            other_box: &mut |e| ph2d_app_flip::gizmo_view::world_aabb(sim, flip, e),
            selection: hero.gizmo.iter_selected().collect(),
            area,
            window,
            margin,
        };
        let track = hero.motion.get(crate::canvas_zoom::VIEW_FLIGHT);
        let label = view_request::resolve(kind, src, camera, &mut view.flight, track)?;
        toasts.push(Toast::info(ph2d_i18n::tr(label)));
        self.title_dirty = true;
        Some(())
    }
}
