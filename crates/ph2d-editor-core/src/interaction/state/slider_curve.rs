//! ⭐ **A LIGAÇÃO CURVA pista ↔ número** — o mapa afim do `link_slider_number_mapped` com um
//! expoente por cima, para os números cuja faixa útil vai de um a milhares.
//!
//! ```text
//! chip_display   = offset + scale · storage^curva
//! slider_storage = ((chip_display − offset) / scale)^(1/curva)
//! ```
//!
//! # Por que existe (2026-09-16)
//!
//! O raio do pincel da escultura passou a poder cobrir a peça inteira (report do dono: *«O radius
//! máximo permitido é pouco»*), e o tecto dele é a diagonal da vista — milhares de pixels. Uma
//! pista LINEAR até lá põe o pincel de 50 px a 1 % da trilha e cada pixel de arrasto a valer uma
//! dúzia de pixels de raio: o artista perde exactamente o gesto mais comum. Com `curva = 3` o
//! mesmo pincel fica a ~21 % e o pincel do tamanho da peça a ~37 %.
//!
//! # ⚠️ As três leis que isto NÃO muda
//!
//! - `curva = 1` é o caminho de sempre **ao bit** — as duas projecções saltam o `powf`, e a
//!   ligação sem curva continua a não ser guardada;
//! - o **intervalo** do scrub do chip é o mesmo `[offset, scale + offset]`: a curva leva `0..1`
//!   em `0..1`, logo os extremos não se mexem;
//! - a detecção de **saturação** do espelho continua na fracção LINEAR (a régua em espaço de
//!   ecrã do `apply_chip_value_with_mirror`); só a escrita no thumb passa pela curva.
//!
//! ⛔ **`linked_slider_mapping` continua a devolver `(scale, offset)`** — sete crates o leem nos
//! seus gates, e uma assinatura nova seria um diff em todas elas por uma extensão que nenhuma
//! delas usa. A curva lê-se à parte, por [`WidgetStore::linked_slider_curve`].

use ph2d_a11y::NodeId;

use super::WidgetStore;

impl WidgetStore {
    /// Liga uma pista a um chip com o mapa **curvo** — ver o cabeçalho. `curva` tem de ser
    /// finita e positiva; `1.0` é exactamente o [`WidgetStore::link_slider_number_mapped`].
    pub fn link_slider_number_curved(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
        curva: f32,
    ) {
        debug_assert!(
            curva.is_finite() && curva > 0.0,
            "link_slider_number_curved: a curva tem de ser finita e positiva ({curva})"
        );
        self.link_slider_number_mapped_inner(slider, number, scale, offset, false, curva);
    }

    /// O expoente da ligação do chip `number` — `1.0` sem ligação curva.
    #[must_use]
    pub fn linked_slider_curve(&self, number: NodeId) -> f32 {
        self.number_to_slider_mapping
            .get(&number)
            .map_or(1.0, |&(_, _, curva)| curva)
    }
}

/// `true` quando a curva é a identidade — e então as projecções não tocam no número.
fn e_linear(curva: f32) -> bool {
    (curva - 1.0).abs() <= f32::EPSILON
}

/// Pista (`0..=1`) → a fracção LINEAR da faixa do chip. ⭐ **A porta ÚNICA da lei**: o store e
/// os painéis que desenham a pista chamam esta, senão as duas metades divergem no primeiro bit.
pub fn pista_para_fracao(storage: f32, curva: f32) -> f32 {
    if e_linear(curva) {
        storage
    } else {
        storage.max(0.0).powf(curva)
    }
}

/// A fracção LINEAR da faixa do chip (já clampada a `0..=1`) → a pista.
pub fn fracao_para_pista(linear: f32, curva: f32) -> f32 {
    if e_linear(curva) {
        linear
    } else {
        linear.max(0.0).powf(curva.recip())
    }
}

#[cfg(test)]
mod tests {
    use super::{fracao_para_pista, pista_para_fracao};

    /// ⭐ **GATE — as duas projecções são INVERSAS, e a identidade não toca num bit.**
    ///
    /// ⚠️ Uma ida-e-volta que não fecha é um controlo que **anda sozinho enquanto se segura**: o
    /// painel publica a pista a partir do número a cada quadro e lê o número de volta a cada
    /// arrasto.
    #[test]
    fn a_curva_vai_e_volta_e_a_identidade_nao_toca_em_nada() {
        for k in 0..=100u16 {
            let t = f32::from(k) / 100.0;
            assert_eq!(pista_para_fracao(t, 1.0).to_bits(), t.to_bits());
            assert_eq!(fracao_para_pista(t, 1.0).to_bits(), t.to_bits());
            for curva in [2.0f32, 3.0] {
                let volta = fracao_para_pista(pista_para_fracao(t, curva), curva);
                assert!((volta - t).abs() < 2e-6, "curva {curva}: {t} -> {volta}");
            }
        }
        assert_eq!(
            pista_para_fracao(0.5, 3.0),
            0.125,
            "a curva nao e' a lei escrita no cabecalho"
        );
        assert_eq!(
            pista_para_fracao(1.0, 3.0),
            1.0,
            "o topo da pista deixou de ser o topo da faixa"
        );
    }
}
