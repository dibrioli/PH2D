//! ⭐⭐⭐ **O RELEVO da tinta** — a espessura de cada amostra do plano (a etapa
//! 3b do Painter na peça, [`docs/3D/29`]).
//!
//! O impasto do Painter dá à tinta uma ESPESSURA, e na peça ela é um relevo de
//! LUZ (decisão do dono, 24/09): a normal de sombreado inclina-se com ela e a
//! FORMA não muda — nem a silhueta, nem o ficheiro exportado.
//!
//! # A unidade e o endereço
//!
//! Uma altura por amostra, em **unidades de objecto ao longo da normal**, no
//! MESMO endereço e com os MESMOS pesos da cor ([`Tinta::pesos_tri`] ·
//! [`Tinta::pesos_quad`]) — uma lei de interpolação, dois canais. ⚠️ Por isso
//! tudo o que já transporta o plano (o empréstimo ao traço, a ranhura por
//! degrau, a cerca de identidade) transporta o relevo sem uma linha nova.
//!
//! ⚠️ **`None` até à primeira escrita** ([`Tinta::alturas_mut`]): a `256x` um
//! plano pesa dezenas de MB, e uma peça que nunca levou impasto não paga um
//! byte — nem na memória, nem no documento, nem no device.
//!
//! [`docs/3D/29`]: ../../../docs/3D/29_plano_o_relevo_do_impasto_na_peca.md

use crate::Tinta;

impl Tinta {
    /// Este plano tem relevo? — `false` até à 1.ª escrita.
    #[must_use]
    pub fn tem_relevo(&self) -> bool {
        self.alturas.is_some()
    }

    /// As alturas, na disposição das amostras — `None` sem relevo.
    #[must_use]
    pub fn alturas(&self) -> Option<&[f32]> {
        self.alturas.as_deref()
    }

    /// ⭐ **As alturas, para escrever** — criadas a ZERO na 1.ª vez (uma altura
    /// zero é a superfície sem tinta espessa, e é o que a peça era antes).
    pub fn alturas_mut(&mut self) -> &mut [f32] {
        let n = self.amostras().len();
        self.alturas.get_or_insert_with(|| vec![0.0; n])
    }

    /// A altura da amostra `idx` — zero sem relevo.
    #[must_use]
    pub fn altura(&self, idx: usize) -> f32 {
        self.alturas.as_ref().map_or(0.0, |a| a[idx])
    }

    /// ⭐ **Instala (ou retira) o relevo inteiro** — a porta do documento e do
    /// desfazer do plano inteiro.
    ///
    /// ⛔ **RECUSA** (`false`, e nada muda) um vector com outro tamanho que o das
    /// amostras: *um relevo com o tamanho errado é espessura no sítio errado*,
    /// a mesma razão de todas as cercas deste plano.
    pub fn com_alturas(&mut self, alturas: Option<Vec<f32>>) -> bool {
        if alturas
            .as_ref()
            .is_some_and(|a| a.len() != self.amostras().len())
        {
            return false;
        }
        self.alturas = alturas;
        true
    }

    /// ⭐⭐ **A ALTURA num ponto baricêntrico de um triângulo** — a irmã da
    /// [`Self::cor_tri`], pelos mesmos pesos. Zero sem relevo.
    #[must_use]
    pub fn altura_tri(&self, face: usize, cantos: &[u32], bar: [f32; 3]) -> f32 {
        let Some(a) = self.alturas.as_deref() else {
            return 0.0;
        };
        self.pesos_tri(face, cantos, bar)
            .map(|(idx, peso)| a[idx] * peso)
            .sum()
    }

    /// A irmã para QUADS — a da [`Self::cor_quad`].
    #[must_use]
    pub fn altura_quad(&self, face: usize, cantos: &[u32], uv: [f32; 2]) -> f32 {
        let Some(a) = self.alturas.as_deref() else {
            return 0.0;
        };
        self.pesos_quad(face, cantos, uv)
            .map(|(idx, peso)| a[idx] * peso)
            .sum()
    }
}
