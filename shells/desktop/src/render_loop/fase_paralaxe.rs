//! **Fase do quadro: A PARALAXE** (plano 24, W1) — um objecto guarda uma fracção do movimento do
//! mundo, e o fundo fica para trás.
//!
//! ⚠️ **Aqui, imediatamente depois da [`super::fase_hud`] e ANTES do extract**, e as três metades
//! são load-bearing:
//!
//! * **depois da câmera**, porque a vista deste quadro é a que ela acabou de calcular — um passe
//!   antes deslocaria contra o enquadramento do quadro anterior, e o fundo leria-se como *«atrasado
//!   um quadro»* em cima de uma câmera que segue;
//! * **depois do HUD**, e isso é ordem entre irmãos e não necessidade: as duas populações são
//!   disjuntas por construção (a ponte exclui quem tem `UiCanvas`), e mantê-las juntas põe as duas
//!   leis da mesma família uma ao lado da outra;
//! * **antes do extract**, porque é ele quem propaga as poses e desenha, e os filhos de um fundo
//!   herdam a dele.
//!
//! ⚠️ A lei e a ponte vivem na [`ph2d_app_components::parallax_bridge`] — aqui fica só a
//! composição: ler o centro que a fase da câmera devolveu e entregá-lo, com o ledger.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo. `camera_rect` é o que a [`super::fase_game_camera`] devolveu:
    /// centro e meia-janela em metros, ou `None` quando não há câmera de jogo na cena.
    pub(super) fn fase_paralaxe(&mut self, camera_rect: Option<([f32; 2], [f32; 2])>) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        // ⚠️ Só o CENTRO atravessa: a paralaxe é um deslocamento, e o tamanho da vista não entra na
        // lei. *Passar a meia-janela seria oferecer à ponte um dado que ela não pode usar.*
        let centro = camera_rect.map(|(center, _half)| center);
        let n = ph2d_app_components::parallax_bridge::drive_parallax(
            sim,
            centro,
            &mut self.preview_drive,
        );
        // ⭐ **O diagnóstico é a única forma de ver um deslocamento** — ele não deixa rasto na tela.
        // `PH2D_PARALLAX_LOG=1` imprime o centro e quantos foram conduzidos, uma vez por mudança.
        if n > 0 && std::env::var_os("PH2D_PARALLAX_LOG").is_some() {
            let agora = format!("{centro:?}");
            if self.components.parallax_log.as_deref() != Some(agora.as_str()) {
                eprintln!("[parallax] centro={agora} objectos conduzidos={n}");
                self.components.parallax_log = Some(agora);
            }
        }
    }
}
