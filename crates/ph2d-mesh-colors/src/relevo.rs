//! ⭐⭐⭐ **O RELEVO da tinta** — a espessura de cada amostra do plano e QUANTA
//! tinta está lá (a etapa 3b do Painter na peça, [`docs/3D/29`]).
//!
//! O impasto do Painter dá à tinta uma ESPESSURA, e na peça ela é um relevo de
//! LUZ (decisão do dono, 24/09): a normal de sombreado inclina-se com ela e a
//! FORMA não muda — nem a silhueta, nem o ficheiro exportado.
//!
//! # Os dois números de uma amostra: `[altura, corpo]`
//!
//! - **altura** — em **unidades de objecto ao longo da normal**;
//! - **corpo** — `0..1`, quanta TINTA está ali (a cobertura do Painter).
//!
//! ⛔⛔ **O corpo existe porque a altura sozinha MENTE na borda** (report do
//! dono, 01/10, foto: *«o traço tem um relevo indesejado na borda»*). O impasto
//! assenta a espessura com um alisamento, e ela ESPALHA-SE para fora da tinta —
//! medido numa pincelada de raio `10`: a cor ocupa `22` px de largura e a
//! espessura `33`, com `~5` de `15,5` ainda de pé onde a tinta já acabou. No
//! Painter isso é invisível, porque a luz é pesada pela cobertura
//! (`impasto_light::paint_body`: *«relevo sob cobertura zero não acende»* — o
//! halo que o dono já fotografou em 2D em 2026-07-12). Na peça a altura chegava
//! SEM a cobertura, e a encosta acendia o barro nu: o anel cinzento da foto.
//! ⇒ a luz da peça inclina a normal por `corpo × ∇altura`, a MESMA lei.
//!
//! ⚠️ A altura fora do corpo **fica** (não é apagada na pousada): ela é o que o
//! Painter tem, e é o que a próxima pincelada recebe de volta na tela — um
//! alisar ou uma faca a meio da encosta tem de a encontrar.
//!
//! # O endereço e os pesos
//!
//! No MESMO endereço e com os MESMOS pesos da cor ([`Tinta::pesos_tri`] ·
//! [`Tinta::pesos_quad`]) — uma lei de interpolação, três canais. ⚠️ Por isso
//! tudo o que já transporta o plano (o empréstimo ao traço, a ranhura por
//! degrau, a cerca de identidade) transporta o relevo sem uma linha nova.
//!
//! ⚠️ **`None` até à primeira escrita** ([`Tinta::relevo_mut`]): a `256x` um
//! plano pesa dezenas de MB, e uma peça que nunca levou impasto não paga um
//! byte — nem na memória, nem no documento, nem no device.
//!
//! [`docs/3D/29`]: ../../../docs/3D/29_plano_o_relevo_do_impasto_na_peca.md

use crate::Tinta;

/// O índice da ALTURA num par do relevo.
pub const ALTURA: usize = 0;
/// O índice do CORPO num par do relevo.
pub const CORPO: usize = 1;

impl Tinta {
    /// Este plano tem relevo? — `false` até à 1.ª escrita.
    #[must_use]
    pub fn tem_relevo(&self) -> bool {
        self.relevo.is_some()
    }

    /// O relevo, `[altura, corpo]` na disposição das amostras — `None` sem
    /// relevo.
    #[must_use]
    pub fn relevo(&self) -> Option<&[[f32; 2]]> {
        self.relevo.as_deref()
    }

    /// ⭐ **O relevo, para escrever** — criado a ZERO na 1.ª vez (altura zero e
    /// corpo zero são a superfície sem tinta espessa, e é o que a peça era
    /// antes).
    pub fn relevo_mut(&mut self) -> &mut [[f32; 2]] {
        let n = self.amostras().len();
        self.relevo.get_or_insert_with(|| vec![[0.0; 2]; n])
    }

    /// O par `[altura, corpo]` da amostra `idx` — zeros sem relevo.
    #[must_use]
    pub fn espessura(&self, idx: usize) -> [f32; 2] {
        self.relevo.as_ref().map_or([0.0; 2], |a| a[idx])
    }

    /// A altura da amostra `idx` — zero sem relevo.
    #[must_use]
    pub fn altura(&self, idx: usize) -> f32 {
        self.espessura(idx)[ALTURA]
    }

    /// O corpo da amostra `idx` — zero sem relevo.
    #[must_use]
    pub fn corpo(&self, idx: usize) -> f32 {
        self.espessura(idx)[CORPO]
    }

    /// ⭐ **Instala (ou retira) o relevo inteiro** — a porta do documento e do
    /// desfazer do plano inteiro.
    ///
    /// ⛔ **RECUSA** (`false`, e nada muda) um vector com outro tamanho que o das
    /// amostras: *um relevo com o tamanho errado é espessura no sítio errado*,
    /// a mesma razão de todas as cercas deste plano.
    pub fn com_relevo(&mut self, relevo: Option<Vec<[f32; 2]>>) -> bool {
        if relevo
            .as_ref()
            .is_some_and(|a| a.len() != self.amostras().len())
        {
            return false;
        }
        self.relevo = relevo;
        true
    }

    /// ⭐⭐ **O par `[altura, corpo]` num ponto baricêntrico de um triângulo** —
    /// a irmã da [`Self::cor_tri`], pelos mesmos pesos. Zeros sem relevo.
    #[must_use]
    pub fn espessura_tri(&self, face: usize, cantos: &[u32], bar: [f32; 3]) -> [f32; 2] {
        let Some(a) = self.relevo.as_deref() else {
            return [0.0; 2];
        };
        soma(self.pesos_tri(face, cantos, bar), a)
    }

    /// A irmã para QUADS — a da [`Self::cor_quad`].
    #[must_use]
    pub fn espessura_quad(&self, face: usize, cantos: &[u32], uv: [f32; 2]) -> [f32; 2] {
        let Some(a) = self.relevo.as_deref() else {
            return [0.0; 2];
        };
        soma(self.pesos_quad(face, cantos, uv), a)
    }

    /// A ALTURA num ponto de um triângulo — a metade de altura da
    /// [`Self::espessura_tri`].
    #[must_use]
    pub fn altura_tri(&self, face: usize, cantos: &[u32], bar: [f32; 3]) -> f32 {
        self.espessura_tri(face, cantos, bar)[ALTURA]
    }

    /// A ALTURA num ponto de um quad — a metade de altura da
    /// [`Self::espessura_quad`].
    #[must_use]
    pub fn altura_quad(&self, face: usize, cantos: &[u32], uv: [f32; 2]) -> f32 {
        self.espessura_quad(face, cantos, uv)[ALTURA]
    }
}

/// A soma pesada dos pares — cada canal acumula-se sozinho, pela ordem dos
/// pesos, que é a ordem da cor (ao bit com ela canal a canal).
fn soma(pesos: impl Iterator<Item = (usize, f32)>, a: &[[f32; 2]]) -> [f32; 2] {
    let mut o = [0.0f32; 2];
    for (idx, peso) in pesos {
        o[ALTURA] += a[idx][ALTURA] * peso;
        o[CORPO] += a[idx][CORPO] * peso;
    }
    o
}
