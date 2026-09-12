//! **O invólucro de shell** do `pose_smoke` (W2/L5 Fase B, 2026-09-11).
//!
//! A cena inteira vive em [`ph2d_app_flip::pose_smoke::arm`]. Aqui fica o que só a `App` tem:
//! destrancar o `AppGfx`, emprestar o barramento dos painéis, e **registar o input autorado** —
//! que a família devolve como valor em vez de escrever num campo alheio.

impl crate::App {
    pub(crate) fn flip_pose_smoke(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else { return };
        let armou = ph2d_app_flip::pose_smoke::arm(
            &mut gfx.flip,
            &mut gfx.tools,
            gfx.hero_screen.as_mut(),
            &mut self.playhead,
        );
        self.any_input_this_frame |= armou;
    }
}
