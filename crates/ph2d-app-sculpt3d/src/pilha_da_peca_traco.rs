//! ⭐⭐⭐⭐ **O TRAÇO SOBRE A PILHA** (`docs/3D/30` §11 — a W2) — filho
//! (`#[path]`) de [`super`]: lá *o que a pilha É*, aqui *como um traço a muda*.
//!
//! O traço pinta numa CÓPIA DE TRABALHO da camada activa (um `Tinta` com o
//! canal de opacidade: cor pré-multiplicada, `ph2d_mesh_colors::alfa`) em
//! `f32`. A cada quadro as amostras que ele sujou descem à camada em RGBA8
//! ([`PilhaDaPeca::recebe_do_traco`]) e só elas são recompostas no plano da
//! peça ([`PilhaDaPeca::compoe_amostras`]), que é o que a placa lê. Dentro do
//! traço a base de cada amostra é a cópia `f32` (`base` congelada no 1.º
//! toque): quantizar ao byte a cada quadro não pára um traço de força
//! pequena.
//!
//! ⚠️ **Até à W4 o relevo é da PEÇA e mora na camada de BASE** — o traço lê e
//! escreve o relevo do plano da peça, e ele desce à base.

use super::*;

/// ⭐ **Um píxel RGBA8 de camada como o traço o lê**: `(cor pré-multiplicada,
/// opacidade)` na unidade do plano (o byte sRGB sobre `255`).
#[must_use]
pub(crate) fn de_bytes(px: [u8; 4]) -> ([f32; 3], f32) {
    let a = f32::from(px[3]) / 255.0;
    ([px[0], px[1], px[2]].map(|b| f32::from(b) / 255.0 * a), a)
}

/// ⭐ **O inverso de [`de_bytes`]** — a ida e volta é a IDENTIDADE em todo
/// píxel que esta casa escreve (gate `a_ida_e_volta_ao_byte_e_a_identidade`).
/// ⚠️ Opacidade zero é `[0; 4]`: um transparente não guarda cor.
#[must_use]
pub(crate) fn para_bytes(c: [f32; 3], a: f32) -> [u8; 4] {
    let ab = byte_de(a);
    if ab == 0 {
        return [0; 4];
    }
    let a = a.clamp(0.0, 1.0);
    [byte_de(c[0] / a), byte_de(c[1] / a), byte_de(c[2] / a), ab]
}

/// As corridas de índices consecutivos de uma lista ordenada e sem repetidos,
/// como `(início, fim)`.
fn corridas(ord: &[u32]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for &i in ord {
        let i = i as usize;
        match out.last_mut() {
            Some((_, fim)) if *fim == i => *fim += 1,
            _ => out.push((i, i + 1)),
        }
    }
    out
}

impl PilhaDaPeca {
    /// A camada de BASE — a de baixo de tudo (onde o relevo mora até à W4).
    #[must_use]
    pub(crate) fn base(&self) -> Option<LayerId> {
        self.pilha.root().last().copied()
    }

    /// ⭐⭐⭐ **A cópia de trabalho da camada ACTIVA** — o plano da peça
    /// (topologia, relevo) com a cor e a opacidade da camada. `None` se a
    /// activa não é um raster com plano (um ajuste, um grupo — a W3 diz porquê).
    /// Regista a camada emprestada: o pen-up pergunta-a a
    /// [`Self::fim_do_traco`], e um painel que mude a activa a meio do traço não
    /// muda onde ele aterra.
    pub(crate) fn trabalho_da_activa(&mut self, peca: &Tinta) -> Option<(LayerId, Tinta)> {
        let id = self.pilha.active()?;
        if !matches!(
            self.pilha.get(id).map(|c| &c.kind),
            Some(LayerKind::Raster(_))
        ) {
            return None;
        }
        let plano = self.planos.get(&id)?;
        let n = self.amostras;
        if peca.amostras().len() != n {
            return None;
        }
        let mut w = peca.clone();
        let mut alfa = Vec::with_capacity(n);
        for (c, px) in w
            .amostras_mut()
            .iter_mut()
            .zip(plano.rgba8[..n * 4].as_chunks::<4>().0)
        {
            let (cor, a) = de_bytes(*px);
            *c = cor;
            alfa.push(a);
        }
        w.com_alfa(Some(alfa));
        self.em_traco = Some(id);
        Some((id, w))
    }

    /// A camada que o traço em curso pinta, se há um.
    #[must_use]
    pub(crate) fn em_traco(&self) -> Option<LayerId> {
        self.em_traco
    }

    /// A camada que o traço em curso pinta — e esquece-a (o pen-up).
    pub(crate) fn fim_do_traco(&mut self) -> Option<LayerId> {
        self.em_traco.take()
    }

    /// ⭐⭐⭐ **As amostras `idx` da cópia de trabalho descem à camada `id`** em
    /// RGBA8, e o relevo delas à BASE.
    pub(crate) fn recebe_do_traco(&mut self, id: LayerId, w: &Tinta, idx: &[u32]) {
        let n = self.amostras;
        if let Some(plano) = self.planos.get_mut(&id) {
            for &i in idx {
                let i = i as usize;
                if i < n {
                    let px = para_bytes(w.amostras()[i], w.opacidade(i));
                    plano.rgba8[i * 4..i * 4 + 4].copy_from_slice(&px);
                    plano.mudou_amostra(i);
                }
            }
        }
        if let Some(r) = w.relevo()
            && let Some(base) = self.base().and_then(|b| self.planos.get_mut(&b))
        {
            let alvo = base.relevo.get_or_insert_with(|| vec![[0.0; 2]; n]);
            for &i in idx {
                if (i as usize) < n {
                    alvo[i as usize] = r[i as usize];
                }
            }
        }
    }

    /// ⭐⭐⭐ **Recompõe só as amostras `idx` no plano da peça** — a cor (pelas
    /// corridas de índices consecutivos: as amostras de uma face são
    /// contíguas) e o relevo da base. Ao bit igual à [`Self::pinta_tinta`]
    /// nessas amostras (gate `recompor_amostras_e_o_pedaco_da_peca_inteira`).
    pub(crate) fn compoe_amostras(
        &self,
        idx: &[u32],
        peca: &mut Tinta,
        fundo: impl FnOnce() -> Vec<[f32; 3]>,
    ) {
        let n = self.amostras.min(peca.amostras().len());
        let mut ord: Vec<u32> = idx.iter().copied().filter(|&i| (i as usize) < n).collect();
        ord.sort_unstable();
        ord.dedup();
        let mut fundo = Some(fundo);
        let mut semente: Vec<[f32; 3]> = Vec::new();
        for (a, b) in corridas(&ord) {
            let composto = self.compor_faixa(a, b);
            if precisa_de_fundo(&composto)
                && let Some(f) = fundo.take()
            {
                semente = f();
            }
            achata(
                &composto,
                |i| {
                    semente
                        .get(a + i)
                        .copied()
                        .unwrap_or(ph2d_mesh_colors::BRANCO)
                },
                &mut peca.amostras_mut()[a..b],
            );
        }
        if let Some(r) = self
            .base()
            .and_then(|b| self.planos.get(&b))
            .and_then(|p| p.relevo.as_ref())
        {
            let alvo = peca.relevo_mut();
            for &i in &ord {
                alvo[i as usize] = r[i as usize];
            }
        }
    }

    /// ⭐⭐ **Troca uma janela de píxeis da camada `id`** (o desfazer de um
    /// traço) — devolve o que lá estava, ou `None` se a camada já não existe
    /// ou a janela não cabe.
    pub(crate) fn troca_janela(
        &mut self,
        id: LayerId,
        idx: &[u32],
        px: &[[u8; 4]],
    ) -> Option<Vec<[u8; 4]>> {
        let n = self.amostras;
        if idx.len() != px.len() || idx.iter().any(|&i| i as usize >= n) {
            return None;
        }
        let plano = self.planos.get_mut(&id)?;
        Some(
            idx.iter()
                .zip(px)
                .map(|(&i, novo)| {
                    let o = i as usize * 4;
                    let antes = [
                        plano.rgba8[o],
                        plano.rgba8[o + 1],
                        plano.rgba8[o + 2],
                        plano.rgba8[o + 3],
                    ];
                    plano.rgba8[o..o + 4].copy_from_slice(novo);
                    plano.mudou_amostra(i as usize);
                    antes
                })
                .collect(),
        )
    }

    /// ⭐⭐ **Troca uma janela do relevo da BASE** (o desfazer) — devolve o que
    /// lá estava.
    pub(crate) fn troca_relevo(&mut self, idx: &[u32], r: &[[f32; 2]]) -> Option<Vec<[f32; 2]>> {
        let n = self.amostras;
        if idx.len() != r.len() || idx.iter().any(|&i| i as usize >= n) {
            return None;
        }
        let base = self.base().and_then(|b| self.planos.get_mut(&b))?;
        let alvo = base.relevo.get_or_insert_with(|| vec![[0.0; 2]; n]);
        Some(
            idx.iter()
                .zip(r)
                .map(|(&i, novo)| std::mem::replace(&mut alvo[i as usize], *novo))
                .collect(),
        )
    }

    /// ⭐⭐ **Troca o plano INTEIRO da camada `id`** (o desfazer do balde) —
    /// devolve o que lá estava.
    pub(crate) fn troca_plano(&mut self, id: LayerId, mut rgba8: Vec<u8>) -> Option<Vec<u8>> {
        let plano = self.planos.get_mut(&id)?;
        if rgba8.len() != plano.rgba8.len() {
            return None;
        }
        std::mem::swap(&mut plano.rgba8, &mut rgba8);
        plano.mudou_toda();
        Some(rgba8)
    }

    /// Uma cópia do plano da camada `id` (dobrado), para o desfazer do balde.
    #[must_use]
    pub(crate) fn copia_do_plano(&self, id: LayerId) -> Option<Vec<u8>> {
        self.planos.get(&id).map(|p| p.rgba8.clone())
    }
}

#[cfg(test)]
#[path = "pilha_da_peca_traco_tests.rs"]
mod tests;
