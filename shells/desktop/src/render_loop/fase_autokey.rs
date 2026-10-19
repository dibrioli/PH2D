//! **Fase do quadro: O AUTOKEY** — o ponto ÚNICO que grava chaves de animação, DEPOIS de toda escrita de
//! Transform/opacidade da UI no quadro (gizmo, Inspector, reset da Hierarquia) (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_autokey(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // AutoKey (W4.T1/T2) — the single choke point. Runs HERE, after every
        // UI Transform/opacity write for the frame (gizmo early, Inspector +
        // Hierarchy reset just above) so it reads the settled pose of each
        // selected sprite and keys only what left its curve. Placed after the
        // apply pass too, so an undo/paste/scrub — which the apply writes back
        // to the world — reads world == curve and keys nothing.
        // The pass authors on the clock the APPLY drove — the same `ViewClock` the drain
        // picked. Handing it the wrong clock while a solo drives the pose is how one strip in
        // a lane killed auto-key (2026-07-22).
        let autokey_clock = self.timeline.view_clock().pick(
            &self.playhead,
            &self.clip_playhead,
            &self.container_playhead,
        );
        autokey_pass::run(
            &mut self.timeline,
            autokey_clock,
            &mut self.autokey,
            toasts,
            hero,
            sim,
            vec_scene,
            &self.preview_drive,
            // ⭐ A SEGUNDA mão: a ferramenta Bone a posar um osso é um arrasto como o do gizmo, e é
            // isso que faz o gesto inteiro ser UM passo de undo (ver `autokey_pass::run`).
            &self.skeleton,
        );
    }
}
