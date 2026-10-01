//! ⭐⭐ **O RELEVO na janela do traço** — o canal do impasto do Painter na
//! peça ([`ph2d_mesh_colors::relevo`], `docs/3D/29`). Filho do [`super`]: a
//! janela é a MESMA da cor (o mesmo slot, a mesma ordem de `tocadas`), e o
//! relevo de antes é capturado no mesmo acto que a cor de antes.
//!
//! ⚠️ **A captura é incondicional** (`8 B` por amostra tocada, `O(pegada)`):
//! o relevo pode NASCER a meio de um traço, e uma amostra tocada antes disso
//! já tem de ter o relevo de antes guardado — que é zero, e é a verdade.

use super::TintaDoTraco;

impl TintaDoTraco {
    /// O relevo `[altura, corpo]` de ANTES do traço, na ordem de
    /// [`Self::tocadas`] — a janela do desfazer do relevo.
    #[must_use]
    pub fn base_relevo(&self) -> &[[f32; 2]] {
        &self.base_alt
    }

    /// ⭐⭐ **Escreve o RELEVO da amostra a partir do de antes do traço** — a
    /// irmã da [`Self::repinta`], pela mesma captura. Devolve se ele mudou.
    ///
    /// ⚠️ Um relevo que não muda não cria o relevo do plano: `rel` recebe o de
    /// antes, e só quando devolve OUTRO é que o plano passa a ter relevo —
    /// senão um traço de impasto que não cobrisse nada pagaria o vector
    /// inteiro. A comparação é por BITS, nos dois números.
    pub fn eleva(&mut self, idx: u32, rel: impl FnOnce([f32; 2]) -> [f32; 2]) -> bool {
        let s = self.slot_de(idx);
        let nova = rel(self.base_alt[s]);
        let agora = self.tinta.espessura(idx as usize);
        if agora[0].to_bits() == nova[0].to_bits() && agora[1].to_bits() == nova[1].to_bits() {
            return false;
        }
        self.tinta.relevo_mut()[idx as usize] = nova;
        self.suja[s] = true;
        true
    }

    /// ⭐ **O relevo mudou neste traço?** — a pergunta do registo do desfazer:
    /// sem ela, toda pincelada de COR sobre uma peça com relevo gravaria um
    /// canal de relevo que não mudou.
    #[must_use]
    pub fn relevo_mudou(&self) -> bool {
        let Some(a) = self.tinta.relevo() else {
            return false;
        };
        self.tocadas.iter().zip(&self.base_alt).any(|(&i, b)| {
            let r = a[i as usize];
            r[0].to_bits() != b[0].to_bits() || r[1].to_bits() != b[1].to_bits()
        })
    }
}
