//! ⭐⭐⭐⭐ **O RELEVO POR CAMADA** (`docs/3D/30` §15 — a W4) — filho (`#[path]`)
//! de [`super`] (`pilha_da_peca.rs`): lá *a cor da peça*, aqui *a espessura*.
//!
//! Cada camada de pintura leva o seu relevo `[altura, corpo]`; o da peça é a
//! DOBRA do Painter 2D sobre a pilha — as mesmas camadas
//! ([`LayerStack::relief_layers_bottom_up`]: visíveis, de baixo para cima) e o
//! mesmo passo ([`fold_relief_step`]: `altura · profundidade`, `Add` soma,
//! `Level` enterra pelo CORPO da camada). O corpo da peça é o máximo dos corpos,
//! como a cobertura do 2D (uma presença, não uma quantidade).
//!
//! ⚠️ Nem a opacidade, nem a máscara, nem os ajustes (incluídos os de
//! vizinhança) mexem no relevo: no 2D também não — a «opacidade da espessura» é
//! a profundidade.

use ph2d_tool_painter::{RELIEF_FOLD_SEED, ReliefComposite, fold_relief_step};

use super::*;

/// Uma camada que entra na dobra: o relevo dela, a profundidade e o modo.
type CamadaDoRelevo<'a> = (&'a [[f32; 2]], f32, ReliefComposite);

/// ⭐ **O que decide a forma da dobra** — as camadas que entram (com relevo), a
/// profundidade e o modo de cada uma. Igual = o relevo da peça não mudou com o
/// metadado (o arrasto da opacidade não paga a dobra).
pub(crate) type AssinaturaDoRelevo = Vec<(LayerId, u32, ReliefComposite)>;

/// ⭐⭐⭐ **O relevo da amostra `i`** pela dobra das `camadas`.
#[inline]
fn relevo_em(camadas: &[CamadaDoRelevo<'_>], i: usize) -> [f32; 2] {
    let (mut h, mut corpo) = (RELIEF_FOLD_SEED, f32::NEG_INFINITY);
    for &(r, depth, modo) in camadas {
        let [a, c] = r[i];
        h = fold_relief_step(h, a, depth, modo, || c.clamp(0.0, 1.0));
        corpo = corpo.max(c);
    }
    [h, corpo]
}

impl PilhaDaPeca {
    /// As camadas que entram na dobra, de baixo para cima — só as que têm relevo.
    fn camadas_do_relevo(&self) -> Vec<CamadaDoRelevo<'_>> {
        self.pilha
            .relief_layers_bottom_up()
            .into_iter()
            .filter_map(|id| {
                let c = self.pilha.get(id)?;
                let r = self.planos.get(&id)?.relevo.as_deref()?;
                Some((r, c.impasto_depth, c.impasto_composite))
            })
            .collect()
    }

    /// ⭐ A [`AssinaturaDoRelevo`] de agora.
    #[must_use]
    pub(crate) fn assinatura_do_relevo(&self) -> AssinaturaDoRelevo {
        self.pilha
            .relief_layers_bottom_up()
            .into_iter()
            .filter(|id| self.planos.get(id).is_some_and(|p| p.relevo.is_some()))
            .filter_map(|id| {
                let c = self.pilha.get(id)?;
                Some((id, c.impasto_depth.to_bits(), c.impasto_composite))
            })
            .collect()
    }

    /// ⭐⭐⭐⭐ **O relevo da peça: a dobra da pilha** — `None` se nenhuma camada
    /// que entra tem relevo.
    #[must_use]
    pub(crate) fn relevo_composto(&self) -> Option<Vec<[f32; 2]>> {
        use rayon::prelude::*;
        let camadas = self.camadas_do_relevo();
        if camadas.is_empty() {
            return None;
        }
        // Ponto a ponto: em paralelo dá o mesmo ao bit (`5,2 → 1,1 ms` a `64x`, doc 30 §15).
        Some(
            (0..self.amostras)
                .into_par_iter()
                .map(|i| relevo_em(&camadas, i))
                .collect(),
        )
    }

    /// ⭐⭐⭐ **O relevo da peça só nas amostras `ord`** — o pedaço da
    /// [`Self::relevo_composto`], ao bit. Sem camada nenhuma a dobrar, a peça
    /// que tem relevo lê zero ali (o relevo dela fica, a camada saiu).
    pub(crate) fn relevo_nas(&self, ord: &[u32], peca: &mut Tinta) {
        let camadas = self.camadas_do_relevo();
        if camadas.is_empty() && !peca.tem_relevo() {
            return;
        }
        let n = self.amostras;
        let alvo = peca.relevo_mut();
        for &i in ord {
            let i = i as usize;
            if i < n && i < alvo.len() {
                alvo[i] = if camadas.is_empty() {
                    [0.0; 2]
                } else {
                    relevo_em(&camadas, i)
                };
            }
        }
    }

    /// ⭐⭐ **O relevo da peça volta a ser a dobra da pilha, se a FORMA dela
    /// mudou** desde a última vez (a profundidade, o modo, uma camada com relevo
    /// que entrou, saiu ou se escondeu). Devolve se reescreveu.
    pub(crate) fn redobra_o_relevo(&mut self, peca: &mut Tinta) -> bool {
        let agora = self.assinatura_do_relevo();
        if self.relevo_dobrado.0.as_ref() == Some(&agora) {
            return false;
        }
        peca.com_relevo(self.relevo_composto());
        self.relevo_dobrado = Dobrado(Some(agora));
        true
    }

    /// ⭐ **Uma camada pintada de uma vez, cor e relevo** — a fixtura das cenas
    /// (uma peça já pintada, como um ficheiro aberto). `false` se a camada não
    /// existe ou as listas não têm `N` amostras.
    pub(crate) fn pinta_camada(
        &mut self,
        id: LayerId,
        px: &[[u8; 4]],
        relevo: Option<Vec<[f32; 2]>>,
    ) -> bool {
        let n = self.amostras;
        if px.len() != n || relevo.as_ref().is_some_and(|r| r.len() != n) {
            return false;
        }
        let Some(plano) = self.planos.get_mut(&id) else {
            return false;
        };
        plano.escreve(px, relevo);
        self.marca_relevo(id);
        true
    }

    /// ⭐ A camada `id` passou a ter relevo — a projecção que o painel lê
    /// (`Layer::has_relief`, a linha da profundidade).
    pub(super) fn marca_relevo(&mut self, id: LayerId) {
        let tem = self.planos.get(&id).is_some_and(|p| p.relevo.is_some());
        if let Some(c) = self.pilha.get_mut(id) {
            c.has_relief = tem;
        }
    }

    /// ⭐⭐ **Troca uma janela do relevo da camada `id`** (o desfazer de um
    /// traço) — devolve o que lá estava.
    pub(crate) fn troca_relevo(
        &mut self,
        id: LayerId,
        idx: &[u32],
        r: &[[f32; 2]],
    ) -> Option<Vec<[f32; 2]>> {
        let n = self.amostras;
        if idx.len() != r.len() || idx.iter().any(|&i| i as usize >= n) {
            return None;
        }
        let plano = self.planos.get_mut(&id)?;
        let alvo = plano.relevo.get_or_insert_with(|| vec![[0.0; 2]; n]);
        let antes = idx
            .iter()
            .zip(r)
            .map(|(&i, novo)| std::mem::replace(&mut alvo[i as usize], *novo))
            .collect();
        self.marca_relevo(id);
        Some(antes)
    }
}

/// A assinatura da dobra que está no plano da peça — estado da SESSÃO (`None` =
/// por saber: a próxima recomposição dobra). Duas pilhas iguais são iguais com
/// ou sem ela (`PartialEq`).
#[derive(Clone, Debug, Default)]
pub(crate) struct Dobrado(Option<AssinaturaDoRelevo>);

impl PartialEq for Dobrado {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(test)]
#[path = "pilha_da_peca_relevo_tests.rs"]
mod tests;
