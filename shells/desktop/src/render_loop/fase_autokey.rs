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
        // The pass authors on the clock the APPLY drove this parent: the clip
        // playhead in Keys, the CONTAINER playhead inside one, the timeline's on
        // Arrange — the same three-way pick `timeline_bridge::run` made above, from
        // the same stamped facts (`keys_mode` / `container_open`, which the pass
        // reads to root its scratch). Handing it the wrong clock while a solo drives
        // the pose is how one strip in a lane killed auto-key (2026-07-22).
        let autokey_clock = if self.timeline.keys_mode {
            &self.clip_playhead
        } else if self.timeline.container_open.is_some() {
            &self.container_playhead
        } else {
            &self.playhead
        };
        autokey_pass::run(
            &mut self.timeline,
            autokey_clock,
            &mut self.autokey,
            toasts,
            hero,
            sim.world(),
            &self.preview_drive,
        );
    }
}
