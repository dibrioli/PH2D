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

use std::collections::BTreeMap;
use std::mem::size_of;
use std::sync::Arc;

use crate::compositor::LayerImage;
use crate::layers::LayerId as RtLayerId;

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
/// O custo é uma varredura por linha, uma vez por commit (user-paced), e só nos planos que os `Arc`s já
/// não deram como idênticos.
///
/// ⚠️ **Só é chamada com um stride que divide o plano** ([`fits`]) — porque `None` aqui significa
/// *idênticos*, e devolver `None` para *"não sei medir"* faria o `split` gravar `Unchanged` sobre dois
/// buffers que de fato diferem: o undo perderia a edição, sem erro e sem warning. Os dois casos têm de
/// ser perguntas separadas, e são.
fn diff_window<T: PartialEq>(a: &[T], b: &[T], stride: usize) -> Option<PlaneWindow> {
    debug_assert_eq!(a.len(), b.len(), "diff_window quer dois planos do mesmo tamanho");
    debug_assert!(fits(a.len(), stride), "diff_window quer um stride que divide o plano");
    let rows = a.len() / stride;
    // As LINHAS primeiro: uma comparação de slice por linha (memcmp vetorizado em `u8`), e a maioria
    // das linhas de um traço não difere.
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
    let row = first?;
    // …e só então as COLUNAS, dentro das linhas que já sabemos que diferem.
    let mut col = stride;
    let mut end = 0usize;
    for r in row..=last {
        let s = r * stride;
        for c in 0..stride {
            if a[s + c] != b[s + c] {
                if c < col {
                    col = c;
                }
                if c + 1 > end {
                    end = c + 1;
                }
            }
        }
    }
    Some(PlaneWindow {
        row,
        rows: last - row + 1,
        col,
        cols: end.saturating_sub(col),
        stride,
        plane_len: a.len(),
    })
}

/// Um plano canvas-shaped como o histórico o guarda. Ver o cabeçalho do módulo.
#[derive(Clone, Debug)]
pub(crate) enum StoredPlane<T> {
    /// Os dois endpoints são o mesmo buffer (ou o mesmo conteúdo) — **zero bytes**, e o caso comum.
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
    stride != 0 && len != 0 && len % stride == 0
}

impl<T: Clone + PartialEq> StoredPlane<T> {
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
                let mut v = cursor.as_ref().clone();
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

impl<T: Clone + PartialEq> StoredEntry<T> {
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

impl<T: Clone + PartialEq> StoredMap<T> {
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
mod tests {
    use super::*;

    /// A janela é o bbox EXATO da diferença — nem uma linha a mais.
    #[test]
    fn the_window_is_the_exact_bbox_of_the_difference() {
        let stride = 8;
        let mut a = vec![0u8; stride * 6];
        let b = {
            let mut b = a.clone();
            b[2 * stride + 3] = 9;
            b[4 * stride + 5] = 7;
            b
        };
        let w = diff_window(&a, &b, stride).expect("difere");
        assert_eq!((w.row, w.rows, w.col, w.cols), (2, 3, 3, 3));
        // …e ela reconstrói o outro lado exatamente.
        let patch = w.extract(&b);
        w.blit(&patch, &mut a);
        assert_eq!(a, b);
    }

    /// Planos idênticos em CONTEÚDO (ponteiros diferentes) não custam nada.
    #[test]
    fn identical_content_costs_nothing_even_with_different_pointers() {
        let mut a = Arc::new(vec![3u8; 64]);
        let mut b = Arc::new(vec![3u8; 64]);
        assert!(!Arc::ptr_eq(&a, &b));
        let p = StoredPlane::split(&mut a, &mut b, 8);
        assert!(matches!(p, StoredPlane::Unchanged));
        assert_eq!(p.heap_bytes(), 0);
    }

    /// Uma janela grande demais NÃO vira patch: dois lados de meia-tela custam mais que os dois buffers.
    #[test]
    fn a_window_that_does_not_pay_for_itself_falls_back_to_whole() {
        let stride = 8;
        let mut a = Arc::new(vec![0u8; stride * 8]);
        let mut b = Arc::new({
            let mut v = vec![0u8; stride * 8];
            for (i, x) in v.iter_mut().enumerate() {
                if i % stride < 6 {
                    *x = 1;
                }
            }
            v
        });
        let p = StoredPlane::split(&mut a, &mut b, stride);
        assert!(matches!(p, StoredPlane::Whole { .. }), "esperava Whole");
    }

    /// Um cursor de outro tamanho RECUSA em vez de escrever pixels em lugares que ninguém autorou.
    #[test]
    fn a_cursor_of_the_wrong_size_is_refused_not_patched() {
        let stride = 8;
        let mut a = Arc::new(vec![0u8; stride * 8]);
        let mut b = Arc::new({
            let mut v = vec![0u8; stride * 8];
            v[9] = 5;
            v
        });
        let p = StoredPlane::split(&mut a, &mut b, stride);
        assert!(matches!(p, StoredPlane::Patch { .. }));
        let wrong = Arc::new(vec![0u8; stride * 4]);
        assert!(p.side(&wrong, true).is_none());
    }
}
