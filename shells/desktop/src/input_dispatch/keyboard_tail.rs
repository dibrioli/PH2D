//! ⭐ **O QUE UMA TECLA FAZ DEPOIS DE TODA A CADEIA** — os controlos de demonstração e a soltura do
//! `P`.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::keyboard`] responde *«quem fica com esta tecla»* — uma cadeia de donos, do input map
//! ao widget. Este responde *«e o que sobra no fim»*, que é outra pergunta: nenhum destes dois
//! **consome** nada, os dois correm depois de todos os ramos, e um deles age na **soltura**.
//!
//! ⚠️ O arquivo estava em `607` linhas contra o tecto de `600` do shell — ⚠️ e **já estava, antes
//! desta wave**: o gate vive em `shells/desktop/tests/` e o `cargo test --bins` não lhe toca.
//! ⛔ *Split, nunca allowlist.*

use super::*;

impl App {
    /// Os dois ramos que correm **depois** de toda a cadeia de donos. Ver a nota do módulo.
    pub(super) fn key_tail(
        &mut self,
        state: ElementState,
        repeat: bool,
        physical_key: PhysicalKey,
    ) {
        // M12 demo controls (only on key Down, no repeat).
        if matches!((state, repeat), (ElementState::Pressed, false))
            && let PhysicalKey::Code(code) = physical_key
        {
            // ⭐ **Estação 2 do Ctrl+Z: sobreviveu aos ~20 `return` da cadeia.** Ver
            // [`crate::App::diag_undo_chord`]. Sem esta linha, «engolido a meio» e «engolido pelo
            // campo de texto do `handle_editor_key`» leem-se iguais — e são troços diferentes.
            if code == KeyCode::KeyZ && (self.modifiers.control_key() || self.modifiers.super_key())
            {
                self.diag_undo_chord("SOBREVIVEU A CADEIA");
            }
            self.handle_editor_key(code);
        }
        // ⭐ **SOLTAR `P` ESCOLHE** (estudo de UI viva, E4) — e é a única tecla deste app cuja
        // SOLTURA significa alguma coisa.
        //
        // ⚠️ **Fechar e escolher são a MESMA operação** (`close_radial` devolve o sector aceso), e
        // separá-las daria a este sítio a chance de fechar sem ler — que é como um menu perde a
        // escolha do artista em silêncio.
        if matches!(
            (state, PhysicalKey::Code(KeyCode::KeyP) == physical_key),
            (ElementState::Released, true)
        ) {
            self.radial_commit();
        }
    }
}
