//! **A reescrita da gravação na imagem do artista** (W8 item 2b, `docs/Macros/05_plano_W8.md`): editar uma
//! etapa muda a imagem — cada texel de cada camada que ainda é igual ao do fim antigo da gravação passa ao
//! do fim novo (cor, relevo, cobertura, material); o resto é do artista. Um passo de undo, marcado com o
//! número da transação (o passo global da edição desfaz-se com ele).

use super::PainterTool;
use crate::compositor::LayerImage;
use crate::layers::LayerId as RtLayerId;
use crate::macro_track::etapas::ModeloDoFim;
use crate::undo::ModelSnapshot;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Os píxeis da camada `id` de `s` (a ativa vive na tela viva).
fn pixeis(s: &ModelSnapshot, id: RtLayerId) -> Option<&[u8]> {
    if s.layers.active() == Some(id) {
        Some(s.canvas_rgba.as_slice())
    } else {
        s.images.get(&id).map(|i| i.rgba8.as_slice())
    }
}

/// O valor do plano preguiçoso `m` no texel `i` da camada `id` (`None` = nunca escrito).
fn em<T: Copy>(m: &BTreeMap<RtLayerId, Arc<Vec<T>>>, id: RtLayerId, i: usize) -> Option<T> {
    m.get(&id).and_then(|v| v.get(i)).copied()
}

/// Põe `x` (`None` = o de fábrica) no texel `i` de `n` da camada `id` do plano `m`.
fn poe<T: Copy + Default>(
    m: &mut BTreeMap<RtLayerId, Arc<Vec<T>>>,
    id: RtLayerId,
    i: usize,
    n: usize,
    x: Option<T>,
) {
    if x.is_none() && m.get(&id).is_none_or(|v| v.len() <= i) {
        return;
    }
    let v = Arc::make_mut(m.entry(id).or_default());
    if v.len() < n {
        v.resize(n, T::default());
    }
    v[i] = x.unwrap_or_default();
}

/// Os planos de um texel: a cor, o relevo (os bits), a cobertura, o material.
type Texel = ([u8; 4], Option<u32>, Option<u8>, Option<[u8; 7]>);

fn texel(s: &ModelSnapshot, px: &[u8], id: RtLayerId, i: usize) -> Texel {
    (
        [px[i * 4], px[i * 4 + 1], px[i * 4 + 2], px[i * 4 + 3]],
        em(&s.heights, id, i).map(f32::to_bits),
        em(&s.covers, id, i),
        em(&s.mats, id, i),
    )
}

/// Reescreve em `alvo` os texels iguais aos de `de` com os de `para`, camada a camada (as que existem nos
/// três com o mesmo tamanho). Devolve quantos mudaram.
fn reescreve(alvo: &mut ModelSnapshot, de: &ModelSnapshot, para: &ModelSnapshot) -> usize {
    let (w, h) = alvo.canvas_size;
    let n = w as usize * h as usize;
    if de.canvas_size != (w, h) || para.canvas_size != (w, h) {
        return 0;
    }
    let ids: std::collections::BTreeSet<RtLayerId> = alvo
        .images
        .keys()
        .copied()
        .chain(alvo.layers.active())
        .collect();
    let mut mudados = 0;
    for id in ids {
        let (Some(a), Some(d), Some(p)) = (pixeis(alvo, id), pixeis(de, id), pixeis(para, id)) else {
            continue;
        };
        if a.len() != n * 4 || d.len() != n * 4 || p.len() != n * 4 {
            continue;
        }
        let trocar: Vec<usize> = (0..n)
            .filter(|&i| texel(alvo, a, id, i) == texel(de, d, id, i))
            .filter(|&i| texel(para, p, id, i) != texel(de, d, id, i))
            .collect();
        if trocar.is_empty() {
            continue;
        }
        let mut novos = a.to_vec();
        for &i in &trocar {
            novos[i * 4..i * 4 + 4].copy_from_slice(&p[i * 4..i * 4 + 4]);
            poe(&mut alvo.heights, id, i, n, em(&para.heights, id, i));
            poe(&mut alvo.covers, id, i, n, em(&para.covers, id, i));
            poe(&mut alvo.mats, id, i, n, em(&para.mats, id, i));
        }
        if alvo.layers.active() == Some(id) {
            alvo.canvas_rgba = Arc::new(novos);
        } else {
            alvo.images.insert(
                id,
                Arc::new(LayerImage {
                    width: w,
                    height: h,
                    rgba8: novos,
                }),
            );
        }
        mudados += trocar.len();
    }
    mudados
}

impl PainterTool {
    /// **A edição de uma etapa chega à imagem** (W8 item 2b): os texels do documento que ainda são os do
    /// fim antigo da gravação (`de`) passam aos do fim novo (`para`) — um passo de undo marcado com
    /// `transacao`. Devolve quantos texels mudaram (`0` = nenhum passo).
    pub fn reescreve_a_gravacao(
        &mut self,
        de: &ModeloDoFim,
        para: &ModeloDoFim,
        transacao: u64,
    ) -> usize {
        let mut depois = self.snapshot_model();
        let n = reescreve(&mut depois, &de.0, &para.0);
        if n == 0 {
            return 0;
        }
        self.wetpaint_end_session();
        let before = self.snapshot_model();
        self.restore_model(depois);
        self.edited_since_bind = true;
        self.undo.transacao_seguinte = Some(transacao);
        self.commit_structural_edit(before);
        self.undo.transacao_seguinte = None;
        self.invalidate_composite();
        n
    }

    /// A transação do passo que um desfazer (`redo == false`) ou um refazer aplicaria agora.
    #[must_use]
    pub fn transacao_do_topo(&self, redo: bool) -> Option<u64> {
        self.undo.transacao_do_topo(redo)
    }

    /// **A água da tela parou** (ou não há): o documento já não muda sozinho.
    pub fn agua_parada(&mut self) -> bool {
        self.wet_assentada()
    }
}
