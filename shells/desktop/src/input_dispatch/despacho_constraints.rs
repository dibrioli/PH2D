//! **O gizmo das constraints no palco** ([`super`]) — a decisão mora em `ph2d_app_constraints::gizmo`; aqui só a
//! projecção tela→mundo e a selecção.

impl crate::App {
    /// O gizmo das constraints no palco: premir uma pega, arrastá-la, largá-la.
    pub(super) fn constraint_gizmo(
        &mut self,
        g: ph2d_app_constraints::gizmo::Gesto,
        tela: (f32, f32),
    ) -> bool {
        let mundo = self.vec_world_at(tela);
        let Some(x) = self.gfx.as_mut() else {
            return false;
        };
        let sel = x.hero_screen.as_ref().and_then(|h| h.gizmo.selection);
        let w = x.scene_window();
        ph2d_app_constraints::gizmo::gesto(&mut x.sim, g, sel, tela, mundo, &x.camera, w)
    }
}
