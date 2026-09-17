//! **Fase do quadro: O HUD** (TOP-20 #20) — a raiz de um placar cola-se à vista da câmera do jogo.
//!
//! ⚠️ **Aqui, imediatamente depois da [`super::fase_game_camera`] e ANTES do extract**, e as duas
//! metades são load-bearing pela mesma razão que a fase da câmera escreve: depois, porque a vista
//! deste quadro é a que a câmera acabou de calcular (um passe antes enquadraria o quadro anterior,
//! e o HUD leria-se como *«atrasado um quadro»* em cima de uma câmera que segue); antes, porque o
//! extract é quem propaga as poses e desenha, e os filhos do canvas herdam a dele.
//!
//! ⚠️ A lei e a ponte vivem na [`ph2d_app_components::hud_bridge`] — aqui fica só a composição:
//! ler a vista que a fase anterior devolveu e entregá-la, com o ledger.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo. `camera_rect` é o que a [`super::fase_game_camera`] devolveu:
    /// centro e meia-janela em metros, ou `None` quando não há câmera de jogo na cena.
    pub(super) fn fase_hud(&mut self, camera_rect: Option<([f32; 2], [f32; 2])>) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        // ⚠️ A conversão para a `View` é feita AQUI e uma só vez: a `half` atravessa a fronteira
        // como meia-janela porque é assim que a câmera a produz — converter para largura inteira
        // na ponte seria a segunda resposta a *«que rectângulo é este?»*.
        let vista = camera_rect
            .map(|(center, half)| ph2d_app_components::hud_bridge::View { center, half });
        let n =
            ph2d_app_components::hud_bridge::drive_canvases(sim, vista, &mut self.preview_drive);
        // ⭐ **O diagnóstico é a única forma de ver um rectângulo** — ele não deixa rasto na tela.
        // `PH2D_HUD_LOG=1` imprime a vista e a pose conduzida, uma vez por mudança.
        if n > 0 && std::env::var_os("PH2D_HUD_LOG").is_some() {
            let agora = format!("{vista:?}");
            if self.components.hud.log.as_deref() != Some(agora.as_str()) {
                eprintln!("[hud-smoke] vista={agora} canvas conduzidos={n}");
                self.components.hud.log = Some(agora);
            }
        }
    }
}
