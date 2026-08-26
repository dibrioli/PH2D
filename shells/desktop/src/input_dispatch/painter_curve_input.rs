//! ⭐ **O menu de alça de um ponto da CURVA no canvas** (Curve / Free Hand) — botão direito num
//! ponto de controlo abre Free / Aligned / Symmetric / Vector / Auto.
//!
//! ⚠️ Irmão de [`super::painter_canvas_input`] pelo teto de 600 LOC da shell, e o corte é por
//! ASSUNTO: lá fica a entrega do PONTEIRO ao Painter; aqui, o gesto secundário sobre um ponto
//! autorado. ⛔ **Não é o [`super::painter_falloff_input`]**, que é a curva do PAINEL de Falloff —
//! dois gráficos, duas leis, e trocá-los põe o menu no sítio errado.

use super::painter_canvas_input::shape_grab_tol_from_affine;
use crate::App;
use ph2d_tool_painter::PainterTool;

impl App {
    /// Secondary Down on an on-canvas **Curve / Free Hand** editor control point: select it and open the
    /// handle-kind menu (Free / Aligned / Vector / Auto) at the cursor. The chrome handler parks the choice
    /// in `HeroScreen.pending_curve_point_handle`; [`Self::painter_apply_pending_curve_handle`] drains it.
    /// Returns `true` (consuming) iff a point was hit. Mirrors `deliver_canvas_pointer`'s screen→image
    /// geometry so the hit-test lands exactly on the painted control dots.
    pub(crate) fn painter_curve_open_point_menu(&mut self, px: f32, py: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let painter_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
            .unwrap_or(false);
        if !painter_active {
            return false;
        }
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(bits) = hero.gizmo.selection else {
            return false;
        };
        let entity = ph2d_ecs::Entity::from_bits(bits);
        // ⚠️ Pose de MUNDO — vide o doc do `sprite_image_to_screen_affine`.
        let (Some(tr), Some(sprite)) = (
            ph2d_ecs::world_transform(gfx.sim.world(), entity),
            gfx.sim.world().get::<ph2d_render::Sprite>(entity),
        ) else {
            return false;
        };
        // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula.
        let sprite_grid = gfx.sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
        let window_size = gfx.surface.size();
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        let (iw, ih) = painter.canvas_size();
        if iw == 0 || ih == 0 {
            return false;
        }
        let affine = crate::render_loop::bgremoval_preview::sprite_image_to_screen_affine(
            iw,
            ih,
            tr,
            sprite,
            sprite_grid,
            &gfx.camera,
            window_size,
        );
        let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
        let tol = shape_grab_tol_from_affine(&affine);
        let img_pt = [img.x as f32, img.y as f32];
        // Same menu for BOTH curve owners: the stroke Shape curve and the selection Convert-to-Curve editor
        // (they're never both active). The pick drains to `set_curve_handle_kind` / `set_selection_curve_...`.
        if !painter.curve_select_point_at(img_pt, tol)
            && !painter.selection_curve_select_point_at(img_pt, tol)
        {
            return false; // off a control point — fall through (pan / other secondary-click handlers)
        }
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.store
                .open_context_menu(ph2d_editor::interaction::ContextMenuRequest {
                    x: px,
                    y: py,
                    kind: ph2d_editor::interaction::ContextMenuKind::CurvePointHandle,
                });
        }
        true
    }
}
