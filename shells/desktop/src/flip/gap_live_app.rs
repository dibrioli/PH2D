//! **A metade que precisa da `App`** do `gap_live` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::gap_live`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::gap_live::*;

impl crate::App {
    /// Roda no prólogo do frame (ao lado do ajuste ao vivo do Colorize): mantém os
    /// helpers do Gap Closure sincronizados com o desenho NA TELA e o alcance atual.
    pub(crate) fn flip_gap_helpers_tick(&mut self) {
        if !wants_gap_helpers(self.flip_state.active, self.flip_state.style) {
            self.flip_state.gap.clear();
            return;
        }
        let Some(style) = self.flip_state.style else {
            self.flip_state.gap.clear();
            return;
        };
        // **A MESMA régua do clique** (`fill_click`): o Gap é em unidades de MUNDO, a
        // geometria é LOCAL, então só a escala do objeto atravessa (mundo→local) — SEM
        // `px_to_world`. Helper e clique têm de usar a mesma fórmula, senão a tela mostra
        // um vão que o clique não fecha. É a régua zoom-invariante (Enio 2026-07-25).
        let w2l = self.flip_active_world_to_local();
        let obj_scale = w2l.mean_scale() as f32;
        let Some(gfx) = self.gfx.as_ref() else {
            self.flip_state.gap.clear();
            return;
        };
        let reach = (style.gap as f32) * obj_scale;

        // O desenho NA TELA, read-only — nunca o `flip_autokey` (que CRIA chave; um
        // overlay que autora seria o gesto acontecendo sem ninguém gesticular).
        let Some((oid, lid)) =
            ph2d_app_flip::strip_resolve::target(&gfx.flip, self.flip_state.active_layer)
        else {
            self.flip_state.gap.clear();
            return;
        };
        let Some(obj) = gfx.flip.object(oid) else {
            self.flip_state.gap.clear();
            return;
        };
        let frame = obj.frame_at(&self.playhead);
        let Some(drawing) = obj
            .layer(lid)
            .and_then(|l| l.drawing_at_cycled(frame))
            .and_then(|did| obj.drawing(did))
        else {
            self.flip_state.gap.clear();
            return;
        };
        if self.flip_state.gap.drive(reach, drawing) {
            // Sem repaint o resultado espera o próximo input para aparecer.
            self.title_dirty = true;
        }
    }
}
