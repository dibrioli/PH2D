//! **Fase do quadro: A RE-ACENDIDA DOS OBJETOS ASSADOS** — a rota A (`docs/3D/02.2`) do lado do quadro
//! (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⛔ **Esta fase NÃO está atrás da `feature` `sculpt3d`, e é a razão de ela existir sozinha**: a irmã
//! `fase_sculpt3d_bake` inteira está, e um objeto assado que voltou de um arquivo tem de acender num
//! binário sem o módulo 3D. O gate `a_baked_object_outlives_the_3d_module` mede as QUATRO portas por
//! onde ela podia cair junto com a feature — a chamada no quadro, o `mod`, a `fn` e o bloco dentro dela.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_relight_baked_forms(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            baked_forms,
            surface,
            renderer,
            baked_light,
            ..
        } = FrameGfx::of(gfx);

        // **A RE-ACENDIDA, e ela NÃO está atrás da feature.** É esta linha que torna a promessa da
        // rota A (`docs/3D/02.2`) verificável em vez de prosa: um objeto assado que voltou de um
        // arquivo acende **sem o módulo 3D no build**. Quase sempre não faz nada — com o rig parado
        // custa um carimbo por objeto, sem tocar a GPU, e num projeto sem nada assado o mapa é vazio.
        ph2d_form_donation::baked_form::relight_stale(
            baked_forms,
            surface.gpu(),
            renderer,
            baked_light,
        );
    }
}
