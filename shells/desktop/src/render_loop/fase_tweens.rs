//! **Fase do quadro: os TWEENS** (suplente #22) — cada objecto escreve o que a lei dele pede, no
//! relógio dele.
//!
//! # ⚠️ AQUI, e as três metades são load-bearing
//!
//! * **depois do tique dos timers** (os sinais deles já chegaram a esta fase como argumento), senão
//!   o `progresso` seria o do quadro anterior;
//! * **depois da [`super::fase_tabela_de_accoes`]**, senão um `Start Timer` publicado neste quadro
//!   só começaria a mexer no seguinte — e o artista vê isso como *«o sinal falhou»*;
//! * **depois do apply da timeline** (ela corre na `fase_timeline_drain`, mais acima no quadro):
//!   o ledger compõe `autorado → A → B` porque o censo de ANTES desta fase é tirado **depois**
//!   daquele apply, logo o `last_written == before` e o autorado não é trocado. É a mesma frase que
//!   a [`super::fase_sequences`] já escreve para as cutscenes.
//!
//! ⛔ E **antes do `post_frame_undo`**, senão a escrita não seria fotografada nem declarada.
//!
//! ⚠️ **Ela chama-se `fase_*` e isso NÃO é estilo:** o texto emendado do quadro colhe só essas, e
//! com outro nome ela desapareceria do oráculo de **toda** lei de ordem desta shell, em silêncio.

impl crate::App {
    /// Ver o cabeçalho. O trabalho vive na família ([`ph2d_app_components::tween_bridge`]); o que
    /// fica aqui é **quando**.
    pub(super) fn fase_tweens(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⭐⭐⭐ **LER a curva primeiro, ESCREVER depois — e num passe só** (suplente #23). A ponte do
        // caminho não toca no mundo: ela devolve o que os seguidores pedem, já em MUNDO, e quem
        // escreve é o passe do tween, que **já fotografa a pose uma vez por entidade**. ⛔ Um
        // segundo passe leria a pré-visualização do tween como se fosse o documento — a chave do
        // ledger é `(entidade, driver)`, e o cabeçalho do `tween_bridge` descreve o defeito.
        let caminhos =
            ph2d_app_components::path_follow_bridge::a_escrever(&mut gfx.sim, &gfx.vec_scene);
        let n = ph2d_app_components::tween_bridge::drive_tweens(
            &mut gfx.sim,
            &mut self.preview_drive,
            &caminhos,
        );
        if n > 0 && std::env::var_os("PH2D_TWEEN_LOG").is_some() {
            eprintln!(
                "[tween] {n} escrita(s) neste quadro ({} de caminho)",
                caminhos.len()
            );
        }
    }
}
