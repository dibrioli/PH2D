//! **O histórico guarda um DELTA, nunca um documento** — o motor de janelas que faz uma entrada de
//! undo custar a região que o passo tocou em vez dos quatro planos canvas-shaped inteiros.
//!
//! É o **U1** do [plano 26](../../../docs/Painter/26_plano_performance_procreate.md) §7.3, e o molde é o
//! **ADR-0117** (o mesmo defeito, curado no áudio): *o passo guarda só o lado que não está no documento;
//! o intervalo sai de um diff; o cap é em BYTES, nunca em contagem.* A medição que o abriu: **1.627 MB
//! retidos depois de 24 traços** (`tests/measure_undo_memory.rs`), ou seja **um documento por traço**,
//! com o cap por contagem (`DEFAULT_MAX_DEPTH = 300`) **multiplicando** isso em vez de limitá-lo.
//!
//! # A base de todo delta é o CURSOR, nunca o estado vivo
//!
//! Um delta precisa de um lado completo para partir. A escolha óbvia — *o estado vivo do tool* — foi
//! **descartada**, e a razão é uma armadilha real deste módulo: `restore_model` termina em
//! `restore_shape_overlay`, que **RE-CARIMBA a figura inteira**, então o estado vivo depois de um undo
//! não é byte-a-byte o snapshot que se instalou. Encadeie deltas sobre ele e o segundo undo escreve a
//! janela certa sobre um fundo errado — sem erro, sem warning, e nada no sistema pisca.
//!
//! O [`UndoController`](crate::undo::UndoController) guarda em vez disso **um cursor**: o endpoint
//! adjacente ao topo, materializado. É **UM documento, constante** — e em regime custa **zero bytes**,
//! porque enquanto ninguém escreve ele compartilha os mesmos `Arc`s do tool; só diverge entre o primeiro
//! dab de um traço e o commit dele.
//!
//! # As três formas de um plano guardado
//!
//! [`StoredPlane`] tem exatamente três, e a primeira é o caso comum:
//!
//! - **`Unchanged`** — os dois endpoints compartilham o mesmo `Arc` (ou o mesmo conteúdo). **Zero bytes.**
//!   Um traço toca a camada ativa e mais nada, então os planos das outras camadas, a máscara, a seleção,
//!   as sessões de Deform/Sculpt caem todos aqui por `Arc::ptr_eq`, sem ler um byte.
//! - **`Patch`** — diferem numa janela; guarda **os dois lados dela** (é o que permite andar nas duas
//!   direções a partir do cursor: no undo o cursor é o `after`, no redo é o `before`).
//! - **`Whole`** — o plano mudou de tamanho, ou a janela é grande demais para compensar. É a frase do
//!   ADR-0117 sobre o irredutível: *uma edição de camada inteira é irredutivelmente uma camada por passo.*

use rayon::prelude::*;
use std::collections::BTreeMap;
use std::mem::size_of;
use std::sync::Arc;

use crate::compositor::LayerImage;
use crate::layers::LayerId as RtLayerId;

/// Abaixo de tantos elementos a varredura é sub-milissegundo e a escolha não pode importar, então o
/// limiar só existe para manter o fork do rayon longe de plano pequeno.
///
/// Irmão exato do [`crate::tool::paint::plane_fork`]`::PAR_MIN`, que existe pela mesma razão e sobre os
/// mesmos planos — e o mesmo número, porque a pergunta é a mesma: *este plano é canvas-shaped?*
const PAR_MIN: usize = 1 << 20;

/// A janela em que dois estados de um plano diferem — em **ELEMENTOS do plano**, nunca em pixels do
/// canvas: um plano RGBA carrega quatro elementos por pixel, um de altura carrega um, e um de material
/// carrega um (de sete bytes). O `stride` é o que traduz, e ele vem do chamador porque só ele sabe a
/// forma do plano que está guardando.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlaneWindow {
    /// Primeira linha que difere, e quantas.
    row: usize,
    rows: usize,
    /// Primeira coluna que difere (em elementos, dentro da linha) e quantas.
    col: usize,
    cols: usize,
    /// Elementos por linha do plano INTEIRO.
    stride: usize,
    /// O comprimento total do plano quando a janela foi tirada.
    ///
    /// ⚠️ Não é decoração: a materialização copia o cursor e escreve a janela por cima, então um cursor
    /// de outro tamanho produziria pixels em lugares que ninguém autorou. Com ele, a entrada se **recusa**
    /// (o controller descarta o histórico com um `warn`) em vez de mentir.
    plane_len: usize,
}

impl PlaneWindow {
    /// Quantos elementos a janela carrega (por lado).
    const fn elems(&self) -> usize {
        self.rows * self.cols
    }

    /// Copia a janela para fora de um plano completo.
    fn extract<T: Clone>(&self, src: &[T]) -> Box<[T]> {
        let mut out = Vec::with_capacity(self.elems());
        for r in 0..self.rows {
            let s = (self.row + r) * self.stride + self.col;
            out.extend_from_slice(&src[s..s + self.cols]);
        }
        out.into_boxed_slice()
    }

    /// Escreve a janela de volta num plano completo.
    fn blit<T: Clone>(&self, patch: &[T], dst: &mut [T]) {
        for r in 0..self.rows {
            let s = (self.row + r) * self.stride + self.col;
            let p = r * self.cols;
            dst[s..s + self.cols].clone_from_slice(&patch[p..p + self.cols]);
        }
    }
}

/// O bbox **EXATO** em que dois planos do mesmo tamanho diferem, ou `None` se são idênticos.
///
/// ⚠️ **Ele é calculado, não recebido, e isso é deliberado.** O ADR-0124 diz que quem está a jusante tem
/// de ser *informado* do intervalo em vez de obrigado a redescobri-lo — e aqui o chamador de fato sabe
/// (o traço tem um bbox). Mas há **75 sítios** que commitam uma entrada estrutural, quase nenhum é um
/// traço, e uma janela informada errado não falha: ela deixa fora exatamente os texels que o undo depois
/// não restaura. Derivá-la do conteúdo **estabelece** o invariante de que a materialização depende, em
/// vez de assumi-lo do chamador — a mesma escolha que o motor de blend do vetor teve de fazer quando o
/// invariante *"a origem está na lista"* era esperado de quem chamava.
///
/// ⚠️ **O custo dessa escolha foi MEDIDO e a estimativa que morava aqui estava errada.** Esta doc dizia
/// *"uma varredura por linha, uma vez por commit (user-paced)"*, como se user-paced quisesse dizer
/// barato. Um commit acontece no **pen-up de todo traço**, e a 4096² com impasto são quatro planos a
/// varrer (~256 MB): medido, **91% do pen-up era este scan** — 40,20 ms de pen-up contra 3,49 sem o
/// commit, com o resto plano na tela (`measure_stroke_owners`). A varredura ficou (a decisão acima
/// continua certa: uma janela informada errado não falha, some com texels) e virou **paralela**
/// ([`diff_rows`] / [`diff_cols`]) — 25,03 → 10,96 ms no `record_structural`. Só nos planos que os
/// `Arc`s já não deram como idênticos, como sempre foi.
///
/// ⚠️ **Só é chamada com um stride que divide o plano** ([`fits`]) — porque `None` aqui significa
/// *idênticos*, e devolver `None` para *"não sei medir"* faria o `split` gravar `Unchanged` sobre dois
/// buffers que de fato diferem: o undo perderia a edição, sem erro e sem warning. Os dois casos têm de
/// ser perguntas separadas, e são.
fn diff_window<T: PartialEq + Sync>(a: &[T], b: &[T], stride: usize) -> Option<PlaneWindow> {
    debug_assert_eq!(
        a.len(),
        b.len(),
        "diff_window quer dois planos do mesmo tamanho"
    );
    debug_assert!(
        fits(a.len(), stride),
        "diff_window quer um stride que divide o plano"
    );
    // As LINHAS primeiro: uma comparação de slice por linha (memcmp vetorizado em `u8`), e a maioria
    // das linhas de um traço não difere.
    let (row, last) = diff_rows(a, b, stride)?;
    // …e só então as COLUNAS, dentro das linhas que já sabemos que diferem.
    let (col, end) = diff_cols(a, b, stride, row, last);
    Some(PlaneWindow {
        row,
        rows: last - row + 1,
        col,
        cols: end.saturating_sub(col),
        stride,
        plane_len: a.len(),
    })
}

/// **A varredura de LINHAS, paralela acima de [`PAR_MIN`]** — a primeira e a última linha em que os dois
/// planos diferem.
///
/// Uma linha é comparada contra a linha de MESMO índice do outro plano e não olha nenhuma outra: as
/// linhas são **disjuntas e a leitura é pura**, que é a forma que o [ADR-0109] sanciona e que esta crate
/// já usa no `sculpt_offset`, no `sculpt_close`, no campo da aquarela e no fold da luz. **Byte-idêntica
/// por construção**: muda qual thread avalia qual linha, nunca o que a linha responde — `min`/`max` sobre
/// índices é associativo e comutativo, então a ordem da redução não entra no resultado.
///
/// [ADR-0109]: ../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md
fn diff_rows<T: PartialEq + Sync>(a: &[T], b: &[T], stride: usize) -> Option<(usize, usize)> {
    let rows = a.len() / stride;
    if a.len() < PAR_MIN {
        let mut first = None;
        let mut last = 0usize;
        for r in 0..rows {
            let s = r * stride;
            if a[s..s + stride] != b[s..s + stride] {
                if first.is_none() {
                    first = Some(r);
                }
                last = r;
            }
        }
        return first.map(|f| (f, last));
    }
    a.par_chunks(stride)
        .zip(b.par_chunks(stride))
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(r, _)| (r, r))
        .reduce_with(|(f1, l1), (f2, l2)| (f1.min(f2), l1.max(l2)))
}

/// **A varredura de COLUNAS, dentro da faixa que já se sabe diferente** — paralela pela mesma razão e
/// com a mesma redução associativa.
///
/// ⚠️ Ela é element-a-element (não há memcmp que devolva *onde*), então num traço VERTICAL a faixa é a
/// tela inteira em linhas e esta metade custa tanto quanto a outra. É por isso que ela também é
/// paralela, e não só a de linhas.
fn diff_cols<T: PartialEq + Sync>(
    a: &[T],
    b: &[T],
    stride: usize,
    row: usize,
    last: usize,
) -> (usize, usize) {
    let band = (last - row + 1) * stride;
    let (from, to) = (row * stride, (last + 1) * stride);
    let scan = |x: &[T], y: &[T]| -> (usize, usize) {
        let mut col = stride;
        let mut end = 0usize;
        for c in 0..stride {
            if x[c] != y[c] {
                if c < col {
                    col = c;
                }
                if c + 1 > end {
                    end = c + 1;
                }
            }
        }
        (col, end)
    };
    if band < PAR_MIN {
        let mut col = stride;
        let mut end = 0usize;
        for r in row..=last {
            let s = r * stride;
            let (c, e) = scan(&a[s..s + stride], &b[s..s + stride]);
            col = col.min(c);
            end = end.max(e);
        }
        return (col, end);
    }
    a[from..to]
        .par_chunks(stride)
        .zip(b[from..to].par_chunks(stride))
        .map(|(x, y)| scan(x, y))
        .reduce(
            || (stride, 0),
            |(c1, e1), (c2, e2)| (c1.min(c2), e1.max(e2)),
        )
}

/// Um plano canvas-shaped como o histórico o guarda. Ver o cabeçalho do módulo.
#[derive(Clone, Debug, Default)]
pub(crate) enum StoredPlane<T> {
    /// Os dois endpoints são o mesmo buffer (ou o mesmo conteúdo) — **zero bytes**, e o caso comum.
    #[default]
    Unchanged,
    /// Diferem só nesta janela; guarda os DOIS lados dela.
    Patch {
        win: PlaneWindow,
        before: Box<[T]>,
        after: Box<[T]>,
    },
    /// Diferem em forma, ou a janela não compensa: os dois buffers inteiros.
    Whole {
        before: Arc<Vec<T>>,
        after: Arc<Vec<T>>,
    },
}

/// Um plano vazio — o que fica no `ModelSnapshot` guardado depois que o delta tomou os pixels.
fn drained<T>() -> Arc<Vec<T>> {
    Arc::new(Vec::new())
}

/// O stride serve para medir este plano? Pergunta SEPARADA de *"os dois lados diferem?"* — ver
/// [`diff_window`].
const fn fits(len: usize, stride: usize) -> bool {
    stride != 0 && len != 0 && len.is_multiple_of(stride)
}

impl<T: Copy + PartialEq + Send + Sync> StoredPlane<T> {
    /// Extrai o delta dos dois endpoints e **ESVAZIA os dois** — depois disto os pixels vivem aqui e só
    /// aqui, e o `ModelSnapshot` guardado carrega apenas metadados.
    pub(crate) fn split(before: &mut Arc<Vec<T>>, after: &mut Arc<Vec<T>>, stride: usize) -> Self {
        if Arc::ptr_eq(before, after) {
            *before = drained();
            *after = drained();
            return Self::Unchanged;
        }
        // Forma diferente, ou um stride que não mede este plano: o `Whole` é a resposta honesta. **Não**
        // se pode cair em `Unchanged` aqui — os dois buffers diferem, e o cursor serviria os dois lados.
        if before.len() != after.len() || !fits(before.len(), stride) {
            return Self::Whole {
                before: std::mem::replace(before, drained()),
                after: std::mem::replace(after, drained()),
            };
        }
        match diff_window(before, after, stride) {
            None => {
                // Ponteiros diferentes, conteúdo igual — acontece sempre que um passe reconstrói um plano
                // sem mudá-lo. O cursor serve os dois lados.
                *before = drained();
                *after = drained();
                Self::Unchanged
            }
            // O delta guarda DOIS lados, então ele só compensa abaixo de metade do plano. Acima disso o
            // `Whole` é mais barato E não paga a cópia do cursor na materialização.
            Some(w) if 2 * w.elems() < before.len() => {
                let out = Self::Patch {
                    win: w,
                    before: w.extract(before),
                    after: w.extract(after),
                };
                *before = drained();
                *after = drained();
                out
            }
            Some(_) => Self::Whole {
                before: std::mem::replace(before, drained()),
                after: std::mem::replace(after, drained()),
            },
        }
    }

    /// Materializa um dos lados a partir do **cursor** (o endpoint adjacente).
    ///
    /// `None` = a entrada não pode ser honrada com este cursor (o plano mudou de tamanho debaixo dela).
    /// Quem chama descarta o histórico: um undo que devolve pixels quase-certos é pior que um que se
    /// recusa.
    pub(crate) fn side(&self, cursor: &Arc<Vec<T>>, want_before: bool) -> Option<Arc<Vec<T>>> {
        match self {
            Self::Unchanged => Some(Arc::clone(cursor)),
            Self::Whole { before, after } => {
                Some(Arc::clone(if want_before { before } else { after }))
            }
            Self::Patch { win, before, after } => {
                if cursor.len() != win.plane_len {
                    return None;
                }
                // ⚠️ A materialização começa por uma CÓPIA do plano do cursor — 67 MB a 4096² — e ela
                // era o custo de um Ctrl+Z (13,37 ms medidos). É a mesma cópia que a porta de fork faz,
                // então vai pelo mesmo primitivo (`crate::plane_copy`): paralela acima do limiar,
                // byte-idêntica por construção.
                let mut v = crate::plane_copy::par_clone(cursor);
                win.blit(if want_before { before } else { after }, &mut v);
                Some(Arc::new(v))
            }
        }
    }

    /// Bytes que esta entrada de fato retém — o que o cap conta.
    ///
    /// Conservador de propósito no `Whole`: um `Arc` que o documento vivo ainda compartilha custa zero de
    /// verdade, e não há como saber daqui. Um cap que superestima aperta cedo demais; um que subestima
    /// não é um cap.
    pub(crate) fn heap_bytes(&self) -> usize {
        match self {
            Self::Unchanged => 0,
            Self::Patch { before, after, .. } => (before.len() + after.len()) * size_of::<T>(),
            Self::Whole { before, after } => (before.len() + after.len()) * size_of::<T>(),
        }
    }
}

/// Uma chave de um mapa por-camada, guardada como delta. A chave pode existir num lado só (o passo criou
/// ou apagou o relevo de uma camada), e é isso que os dois variants de borda dizem.
#[derive(Clone, Debug)]
pub(crate) enum StoredEntry<T> {
    Both(StoredPlane<T>),
    OnlyBefore(Arc<Vec<T>>),
    OnlyAfter(Arc<Vec<T>>),
}

impl<T: Copy + PartialEq + Send + Sync> StoredEntry<T> {
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Both(p) => p.heap_bytes(),
            Self::OnlyBefore(v) | Self::OnlyAfter(v) => v.len() * size_of::<T>(),
        }
    }
}

/// Um mapa `LayerId -> plano` guardado como delta (os três planos de impasto: `heights`, `covers`,
/// `mats`).
#[derive(Clone, Debug, Default)]
pub(crate) struct StoredMap<T> {
    entries: BTreeMap<RtLayerId, StoredEntry<T>>,
}

impl<T: Copy + PartialEq + Send + Sync> StoredMap<T> {
    /// Extrai o delta dos dois mapas e **esvazia os dois**. `stride` é em elementos por linha do plano
    /// (um por pixel nos três mapas de impasto).
    pub(crate) fn split(
        before: &mut BTreeMap<RtLayerId, Arc<Vec<T>>>,
        after: &mut BTreeMap<RtLayerId, Arc<Vec<T>>>,
        stride: usize,
    ) -> Self {
        let mut entries = BTreeMap::new();
        let keys: Vec<RtLayerId> = before.keys().chain(after.keys()).copied().collect();
        for k in keys {
            if entries.contains_key(&k) {
                continue;
            }
            let e = match (before.get(&k).cloned(), after.get(&k).cloned()) {
                (Some(mut b), Some(mut a)) => {
                    StoredEntry::Both(StoredPlane::split(&mut b, &mut a, stride))
                }
                (Some(b), None) => StoredEntry::OnlyBefore(b),
                (None, Some(a)) => StoredEntry::OnlyAfter(a),
                (None, None) => continue,
            };
            entries.insert(k, e);
        }
        before.clear();
        after.clear();
        Self { entries }
    }

    /// Reconstrói um dos lados a partir do mapa do cursor.
    pub(crate) fn side(
        &self,
        cursor: &BTreeMap<RtLayerId, Arc<Vec<T>>>,
        want_before: bool,
    ) -> Option<BTreeMap<RtLayerId, Arc<Vec<T>>>> {
        let mut out = BTreeMap::new();
        for (k, e) in &self.entries {
            match e {
                StoredEntry::Both(p) => {
                    // `Both` significa que o lado ADJACENTE (o cursor) tem a chave. Não ter é a mesma
                    // classe de incoerência que um plano de outro tamanho: recusa.
                    let c = cursor.get(k)?;
                    out.insert(*k, p.side(c, want_before)?);
                }
                StoredEntry::OnlyBefore(v) => {
                    if want_before {
                        out.insert(*k, Arc::clone(v));
                    }
                }
                StoredEntry::OnlyAfter(v) => {
                    if !want_before {
                        out.insert(*k, Arc::clone(v));
                    }
                }
            }
        }
        Some(out)
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.entries.values().map(StoredEntry::heap_bytes).sum()
    }
}

/// O mapa de pixels das camadas NÃO-ativas. Mesma lei dos outros, com o stride vindo do próprio
/// [`LayerImage`] (`width * 4`) — um traço nunca o toca (a camada ativa vive em `canvas_rgba`), então na
/// prática todo elemento cai em `Unchanged` por `Arc::ptr_eq`; ele existe para as **operações de camada**,
/// que o tocam de verdade.
#[derive(Clone, Debug, Default)]
pub(crate) struct StoredImages {
    entries: BTreeMap<RtLayerId, ImageEntry>,
}

#[derive(Clone, Debug)]
enum ImageEntry {
    /// Mesmo `Arc` nos dois lados.
    Unchanged,
    /// Mesma forma, difere numa janela do `rgba8`.
    Patch {
        win: PlaneWindow,
        width: u32,
        height: u32,
        before: Box<[u8]>,
        after: Box<[u8]>,
    },
    Both(Arc<LayerImage>, Arc<LayerImage>),
    OnlyBefore(Arc<LayerImage>),
    OnlyAfter(Arc<LayerImage>),
}

impl StoredImages {
    pub(crate) fn split(
        before: &mut BTreeMap<RtLayerId, Arc<LayerImage>>,
        after: &mut BTreeMap<RtLayerId, Arc<LayerImage>>,
    ) -> Self {
        let mut entries = BTreeMap::new();
        let keys: Vec<RtLayerId> = before.keys().chain(after.keys()).copied().collect();
        for k in keys {
            if entries.contains_key(&k) {
                continue;
            }
            let e = match (before.get(&k).cloned(), after.get(&k).cloned()) {
                (Some(b), Some(a)) => {
                    if Arc::ptr_eq(&b, &a) {
                        ImageEntry::Unchanged
                    } else if b.width == a.width
                        && b.rgba8.len() == a.rgba8.len()
                        && fits(b.rgba8.len(), (b.width as usize) * 4)
                    {
                        let stride = (b.width as usize) * 4;
                        match diff_window(&b.rgba8, &a.rgba8, stride) {
                            None => ImageEntry::Unchanged,
                            Some(w) if 2 * w.elems() < b.rgba8.len() => ImageEntry::Patch {
                                win: w,
                                width: b.width,
                                height: b.height,
                                before: w.extract(&b.rgba8),
                                after: w.extract(&a.rgba8),
                            },
                            Some(_) => ImageEntry::Both(b, a),
                        }
                    } else {
                        ImageEntry::Both(b, a)
                    }
                }
                (Some(b), None) => ImageEntry::OnlyBefore(b),
                (None, Some(a)) => ImageEntry::OnlyAfter(a),
                (None, None) => continue,
            };
            entries.insert(k, e);
        }
        before.clear();
        after.clear();
        Self { entries }
    }

    pub(crate) fn side(
        &self,
        cursor: &BTreeMap<RtLayerId, Arc<LayerImage>>,
        want_before: bool,
    ) -> Option<BTreeMap<RtLayerId, Arc<LayerImage>>> {
        let mut out = BTreeMap::new();
        for (k, e) in &self.entries {
            match e {
                ImageEntry::Unchanged => {
                    out.insert(*k, Arc::clone(cursor.get(k)?));
                }
                ImageEntry::Patch {
                    win,
                    width,
                    height,
                    before,
                    after,
                } => {
                    let c = cursor.get(k)?;
                    if c.rgba8.len() != win.plane_len {
                        return None;
                    }
                    let mut rgba8 = c.rgba8.clone();
                    win.blit(if want_before { before } else { after }, &mut rgba8);
                    out.insert(
                        *k,
                        Arc::new(LayerImage {
                            width: *width,
                            height: *height,
                            rgba8,
                        }),
                    );
                }
                ImageEntry::Both(b, a) => {
                    out.insert(*k, Arc::clone(if want_before { b } else { a }));
                }
                ImageEntry::OnlyBefore(v) => {
                    if want_before {
                        out.insert(*k, Arc::clone(v));
                    }
                }
                ImageEntry::OnlyAfter(v) => {
                    if !want_before {
                        out.insert(*k, Arc::clone(v));
                    }
                }
            }
        }
        Some(out)
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.entries
            .values()
            .map(|e| match e {
                ImageEntry::Unchanged => 0,
                ImageEntry::Patch { before, after, .. } => before.len() + after.len(),
                ImageEntry::Both(b, a) => b.rgba8.len() + a.rgba8.len(),
                ImageEntry::OnlyBefore(v) | ImageEntry::OnlyAfter(v) => v.rgba8.len(),
            })
            .sum()
    }
}

/// O stride (elementos por linha) de cada família de plano, derivado da largura do canvas que o
/// `ModelSnapshot` carrega. Uma porta só, porque errar o stride não quebra nada visível — só torna a
/// janela larga demais, e a regressão seria de MEMÓRIA, que ninguém vê sem medir.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Strides {
    /// Um plano RGBA8: quatro elementos por pixel.
    pub(crate) rgba: usize,
    /// Um plano escalar por pixel (`heights`, `covers`, `mats`, as máscaras).
    pub(crate) scalar: usize,
}

impl Strides {
    /// Da largura do canvas em pixels.
    #[must_use]
    pub(crate) const fn of(width: u32) -> Self {
        let w = width as usize;
        Self {
            rgba: w * 4,
            scalar: w,
        }
    }
}

#[cfg(test)]
#[path = "undo_delta_tests.rs"]
mod tests;
