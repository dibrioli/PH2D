//! ⭐⭐ **O RELEVO na janela do traço** — o canal do impasto do Painter na
//! peça ([`ph2d_mesh_colors::relevo`], `docs/3D/29`). Filho do [`super`]: a
//! janela é a MESMA da cor (o mesmo slot, a mesma ordem de `tocadas`), e a
//! altura de antes é capturada no mesmo acto que a cor de antes.
//!
//! ⚠️ **A captura é incondicional** (`4 B` por amostra tocada, `O(pegada)`):
//! o relevo pode NASCER a meio de um traço, e uma amostra tocada antes disso
//! já tem de ter a altura de antes guardada — que é zero, e é a verdade.

use super::TintaDoTraco;

impl TintaDoTraco {
    /// A altura de ANTES do traço, na ordem de [`Self::tocadas`] — a janela do
    /// desfazer do relevo.
    #[must_use]
    pub fn base_alturas(&self) -> &[f32] {
        &self.base_alt
    }

    /// ⭐⭐ **Escreve a ALTURA da amostra a partir da de antes do traço** — a
    /// irmã da [`Self::repinta`], pela mesma captura. Devolve se ela mudou.
    ///
    /// ⚠️ Uma altura que não muda não cria o relevo do plano: `alt` recebe a
    /// de antes, e só quando devolve OUTRA é que o plano passa a ter relevo —
    /// senão um traço de impasto que não cobrisse nada pagaria o vector
    /// inteiro.
    pub fn eleva(&mut self, idx: u32, alt: impl FnOnce(f32) -> f32) -> bool {
        let s = self.slot_de(idx);
        let nova = alt(self.base_alt[s]);
        if self.tinta.altura(idx as usize).to_bits() == nova.to_bits() {
            return false;
        }
        self.tinta.alturas_mut()[idx as usize] = nova;
        self.suja[s] = true;
        true
    }

    /// ⭐ **O relevo mudou neste traço?** — a pergunta do registo do desfazer:
    /// sem ela, toda pincelada de COR sobre uma peça com relevo gravaria um
    /// canal de alturas que não mudou.
    #[must_use]
    pub fn relevo_mudou(&self) -> bool {
        let Some(a) = self.tinta.alturas() else {
            return false;
        };
        self.tocadas
            .iter()
            .zip(&self.base_alt)
            .any(|(&i, b)| a[i as usize].to_bits() != b.to_bits())
    }
}
