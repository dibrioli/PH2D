//! **O cartão Performance** (n.º 8, dono 13/11: «Ambas SIM, mas como opções a mais e não a substituição do que já
//! existe»): o palpite da `assenta` do motor Wet, desligado de fábrica — mais rápido, os pincéis pintam um pouco
//! diferente. Chave do motor (`ph2d_wet_paint`) lida no fio da simulação; nada vai ao documento. O AVX2, a segunda caixa
//! pedida, foi MEDIDO e recusado (`N8_NOTAS.md`: o tecto dele é ~4 % do passo).

use crate::ids::PAINTER_PERFORMANCE_PALPITE;
use ph2d_editor_core::tool::PanelEvent;

/// O palpite do motor Wet está ligado?
#[must_use]
pub fn palpite_ligado() -> bool {
    ph2d_wet_paint::assenta::palpite_ligado()
}

/// Instala a escolha guardada do artista (`~/.ph2d/prefs.txt`, antes do primeiro quadro).
pub fn instala_o_palpite(sim: bool) {
    ph2d_wet_paint::assenta::liga_o_palpite(sim);
}

/// O clique da caixa: troca o palpite.
pub(crate) fn route_desempenho(event: &PanelEvent) -> bool {
    match event {
        PanelEvent::Click(id) if *id == PAINTER_PERFORMANCE_PALPITE => {
            instala_o_palpite(!palpite_ligado());
            true
        }
        _ => false,
    }
}
