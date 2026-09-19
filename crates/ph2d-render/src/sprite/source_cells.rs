//! ⭐⭐⭐ **ONDE, NA FONTE, ESTÃO OS PIXELS QUE O QUAD DESTA SPRITE MOSTRA** — a região recorta a
//! imagem, a grelha recorta a célula, e o que sobra é o rectângulo que o quad de facto desenha.
//!
//! # ⛔ Por que isto é uma PORTA e não uma conta em cada casa
//!
//! A conta já existia **duas vezes** e as duas viviam na shell: o 9-slice (`sub_rect_source_px`,
//! que mede as bordas na imagem que o artista vê) e a emissão (`region_subrect` seguido de
//! [`super::sprite_sheet_subrect`]). O **terceiro** leitor chegou em 2026-09-17 e é de outra crate
//! — quem PRENDE uma imagem ao esqueleto ([`ph2d_skeleton_live::skin_image_bind::bind_image`]) tem de
//! traçar a malha sobre **a mesma** célula, senão ela é traçada sobre a folha inteira.
//!
//! ⚠️⚠️ **E o defeito que isso dava era MUDO** (medido 2026-09-17, sonda `sonda_as_tres_formas`):
//! uma folha `4×1` presa a um osso desenhava `1 277` peças recortadas da folha INTEIRA dentro do
//! quad de UMA célula, com a UV de uma célula esticada por cima — *nenhum aviso, nenhum gate, e a
//! arte de quatro quadros espremida no sítio de um*. O 9-slice, esse, pelo menos avisava.
//!
//! ⇒ *uma LEI que três famílias usam desce para o motor que a executa* (`HOWTO` §2.20).

/// ⭐⭐ **A grelha de células da FONTE desta sprite**, em pixels — a origem da célula `0`, o tamanho
/// de UMA célula, e quantas há.
///
/// ⚠️ **Tudo em `f32` de propósito:** a região é autorada em `f32` (`SpriteRegion::rect`) e a
/// divisão pela grelha raramente dá inteiro. Arredondar aqui poria a borda do 9-slice meio pixel
/// fora do sítio em que o artista a mediu.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SourceCells {
    /// Canto superior esquerdo da célula `0`, em pixels da fonte.
    pub origin: [f32; 2],
    /// Dimensões de **UMA** célula, em pixels da fonte.
    pub cell: [f32; 2],
    pub hframes: u32,
    pub vframes: u32,
}

impl SourceCells {
    /// A grelha desta sprite, ou `None` quando não há fonte medível (dimensões desconhecidas, ou
    /// uma célula de lado nulo).
    ///
    /// `region` é `[x, y, w, h]` em pixels da fonte; um rect de lado `<= 0` é **ignorado**, que é a
    /// mesma porta de saída que a [`super::region_subrect`] usa — *duas respostas diferentes para
    /// «esta região conta?» seriam uma célula no extract e outra no bind*.
    #[must_use]
    pub fn of(
        src_dims: [u32; 2],
        region: Option<[f32; 4]>,
        hframes: u32,
        vframes: u32,
    ) -> Option<Self> {
        let (mut ox, mut oy) = (0.0_f32, 0.0_f32);
        let (mut w, mut h) = (src_dims[0] as f32, src_dims[1] as f32);
        if let Some(r) = region
            && r[2] > 0.0
            && r[3] > 0.0
        {
            ox = r[0];
            oy = r[1];
            w = r[2];
            h = r[3];
        }
        let (hf, vf) = (hframes.max(1), vframes.max(1));
        let cell = [w / hf as f32, h / vf as f32];
        (cell[0] > 0.0 && cell[1] > 0.0 && cell[0].is_finite() && cell[1].is_finite()).then_some(
            Self {
                origin: [ox, oy],
                cell,
                hframes: hf,
                vframes: vf,
            },
        )
    }

    /// Quantas células a grelha tem (nunca zero).
    #[must_use]
    pub fn count(&self) -> u32 {
        self.hframes.saturating_mul(self.vframes).max(1)
    }

    /// O canto superior esquerdo da célula `k`, em pixels da fonte.
    ///
    /// ⚠️ **A ordem é a da [`super::sprite_sheet_subrect`]** — `col = k % hframes`, com a linha a
    /// crescer para BAIXO. Uma segunda convenção aqui poria a malha do quadro 3 sobre a arte do 1.
    #[must_use]
    pub fn cell_origin(&self, k: u32) -> [f32; 2] {
        let k = k.min(self.count() - 1);
        let (col, row) = (k % self.hframes, k / self.hframes);
        [
            self.origin[0] + col as f32 * self.cell[0],
            self.origin[1] + row as f32 * self.cell[1],
        ]
    }

    /// As dimensões de uma célula **arredondadas para o pixel inteiro que a contém** — a régua de
    /// quem precisa de um buffer, e não de um rectângulo.
    ///
    /// ⚠️ `ceil` e não `round`: um lado de `20,5` px tem de caber em `21`, senão a última coluna da
    /// arte fica de fora da malha.
    #[must_use]
    pub fn cell_px(&self) -> [u32; 2] {
        [
            (self.cell[0].ceil() as u32).max(1),
            (self.cell[1].ceil() as u32).max(1),
        ]
    }
}

#[cfg(test)]
#[path = "source_cells_tests.rs"]
mod tests;
