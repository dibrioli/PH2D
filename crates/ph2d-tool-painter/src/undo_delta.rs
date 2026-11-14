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
//! **descartada**, e a razão é uma armadilha real deste módulo: encadeie deltas sobre ele e o segundo
//! undo escreve a janela certa sobre um fundo errado — sem erro, sem warning, e nada no sistema pisca.
//!
//! ⚠️ **Quanto disso é verdade foi MEDIDO em 2026-07-27, e a frase que morava aqui era larga demais.**
//! Ela dizia que o `restore_shape_overlay` re-carimba *"a figura inteira"*, e portanto que o vivo nunca
//! serve. O censo (`PH2D_UNDO_AUDIT=1`, a suíte inteira desta crate) diz **81 undos/redos · 79 em que o
//! vivo É o cursor, pelo MESMO buffer · 2 em que não é**: o re-stamp de um shape com preview vivo (o
//! caso que a frase descrevia) e o **escorrido do Wet Paint** (a sim compositando depois do pen-up, sem
//! entrada). Duas exceções com NOME valem mais que uma proibição geral — são exatamente elas que o S3
//! (doc 28 §7) tem de fechar. `paint::undo_live_base_tests` as pina.
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

/// Abaixo de tantos elementos a varredura é sub-milissegundo e a escolha não pode importar, então o
/// limiar só existe para manter o fork do rayon longe de plano pequeno.
///
/// Em ELEMENTOS de propósito: é a varredura de comparação (custo por elemento), não uma cópia — as
/// cópias decidem em BYTES por [`crate::plane_copy::worth_parallel`] (B17.1, IN-115).
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

    /// A janela em ELEMENTOS que corresponde a uma região do canvas em PIXELS.
    ///
    /// `stride` é a largura do plano em elementos e `canvas_w` a do canvas em pixels, então
    /// `k = stride / canvas_w` é quantos elementos cada pixel carrega (4 no RGBA, 1 nos escalares).
    /// `None` quando a conta não fecha — e aí o chamador VARRE, que é sempre correto.
    pub(crate) fn from_region(
        r: crate::compositor::Region,
        canvas_w: u32,
        stride: usize,
    ) -> Option<Self> {
        let cw = canvas_w as usize;
        if cw == 0 || stride == 0 || r.w == 0 || r.h == 0 || !stride.is_multiple_of(cw) {
            return None;
        }
        let k = stride / cw;
        Some(Self {
            row: r.y as usize,
            rows: r.h as usize,
            col: (r.x as usize) * k,
            cols: (r.w as usize) * k,
            stride,
            plane_len: 0, // ancorado por `fit_to`, que é quem conhece o plano
        })
    }

    /// Ancora esta janela num plano concreto — `None` se ela não couber nele.
    fn fit_to(mut self, plane_len: usize) -> Option<Self> {
        if !fits(plane_len, self.stride) || self.rows == 0 || self.cols == 0 {
            return None;
        }
        let rows_total = plane_len / self.stride;
        if self.row >= rows_total || self.col >= self.stride {
            return None;
        }
        self.rows = self.rows.min(rows_total - self.row);
        self.cols = self.cols.min(self.stride - self.col);
        self.plane_len = plane_len;
        Some(self)
    }

    /// Esta janela contém `other`? A pergunta que o `debug_assert` do [`StoredPlane::split`] faz.
    #[cfg(debug_assertions)]
    fn contains(&self, other: &Self) -> bool {
        self.row <= other.row
            && self.row + self.rows >= other.row + other.rows
            && self.col <= other.col
            && self.col + self.cols >= other.col + other.cols
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
    /// Diferem só nestas caixas (os tiles que o passo tocou, recortados a `win`); os DOIS lados delas,
    /// concatenados na ordem de `rects` (B17.5, [`tiles`]).
    Tiles {
        win: PlaneWindow,
        rects: Box<[PlaneWindow]>,
        before: Box<[T]>,
        after: Box<[T]>,
    },
    /// Diferem em forma, ou a janela não compensa: os dois buffers inteiros.
    Whole {
        before: Arc<Vec<T>>,
        after: Arc<Vec<T>>,
    },
    /// **A camada nova do «+»** (n.º 7 item 3): o `before` inteiro (a tela da camada que saiu) sem diff e o
    /// `after` todo neutro (`T::default()`), que não se segura: o refazer recria-o. Pesa no tecto como o
    /// `Whole` de um lado só.
    Nova { before: Arc<Vec<T>> },
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

impl<T: Copy + Default + PartialEq + Send + Sync> StoredPlane<T> {
    /// Extrai o delta dos dois endpoints e **ESVAZIA os dois** — depois disto os pixels vivem aqui e só
    /// aqui, e o `ModelSnapshot` guardado carrega apenas metadados.
    pub(crate) fn split(
        before: &mut Arc<Vec<T>>,
        after: &mut Arc<Vec<T>>,
        stride: usize,
        hint: Option<PlaneWindow>,
    ) -> Self {
        Self::split_tiled(before, after, stride, hint, None)
    }

    /// [`Self::split`] com os tiles que o passo marcou (`px`, em pixels de uma tela de `canvas_w`): uma
    /// janela declarada grande guarda só eles ([`tiles`]).
    pub(crate) fn split_tiled(
        before: &mut Arc<Vec<T>>,
        after: &mut Arc<Vec<T>>,
        stride: usize,
        hint: Option<PlaneWindow>,
        px: Option<(&[crate::compositor::Region], u32)>,
    ) -> Self {
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
        // ⚠️ **A JANELA DECLARADA poupa a varredura** (`crate::undo::window`): o `split` custava ~470 MB de
        // comparação por traço a 4096² para DERIVAR o que quem escreveu já sabia. Em build de DEBUG a
        // derivação roda mesmo assim e AFIRMA que a janela verdadeira cabe na declarada — a suíte desta
        // crate roda em debug em ~4 s e exercita todos os gestos, então o invariante é **conferido a cada
        // rodada** em vez de assumido. Foi essa rede que reprovou o atalho da §5.17 na primeira rodada.
        if let Some(win) = hint.and_then(|h| h.fit_to(before.len())) {
            #[cfg(debug_assertions)]
            if let Some(real) = diff_window(before, after, stride) {
                debug_assert!(
                    win.contains(&real),
                    "a janela declarada nao contem a verdadeira: declarada {win:?}, real {real:?} — o \
                     undo perderia esses texels (ver crate::undo::window)"
                );
            }
            if let Some(rects) = px.and_then(|(px, cw)| win.rects_within(px, cw))
                && let Some(out) = Self::changed_tiles(
                    win,
                    rects,
                    before.len(),
                    |r| !r.same_in(before, after),
                    |r| r.extract(before),
                    after,
                )
            {
                #[cfg(debug_assertions)]
                debug_assert!(
                    out.side(after, true).is_some_and(|b| *b == **before),
                    "os tiles marcados nao cobrem o que mudou — o undo perderia texels (B17.5)"
                );
                *before = drained();
                *after = drained();
                return out;
            }
            return Self::from_window(win, before, after);
        }
        match diff_window(before, after, stride) {
            None => {
                // Ponteiros diferentes, conteúdo igual — acontece sempre que um passe reconstrói um plano
                // sem mudá-lo. O cursor serve os dois lados.
                *before = drained();
                *after = drained();
                Self::Unchanged
            }
            Some(w) => Self::from_window(w, before, after),
        }
    }

    /// Guarda os dois lados de `win` (ou o plano inteiro, se a janela não compensar) e ESVAZIA os
    /// endpoints. Porta única: o caminho da janela declarada e o da derivada terminam aqui, então eles
    /// não podem divergir sobre o que é guardado.
    fn from_window(win: PlaneWindow, before: &mut Arc<Vec<T>>, after: &mut Arc<Vec<T>>) -> Self {
        // O delta guarda DOIS lados, então ele só compensa abaixo de metade do plano. Acima disso o
        // `Whole` é mais barato E não paga a cópia do cursor na materialização.
        if 2 * win.elems() >= before.len() {
            return Self::Whole {
                before: std::mem::replace(before, drained()),
                after: std::mem::replace(after, drained()),
            };
        }
        let out = Self::Patch {
            win,
            before: win.extract(before),
            after: win.extract(after),
        };
        *before = drained();
        *after = drained();
        out
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
            Self::Nova { before } => Some(if want_before {
                Arc::clone(before)
            } else {
                Arc::new(vec![T::default(); before.len()])
            }),
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
            Self::Tiles {
                win,
                rects,
                before,
                after,
            } => {
                if cursor.len() != win.plane_len {
                    return None;
                }
                let mut v = crate::plane_copy::par_clone(cursor);
                Self::blit_tiles(rects, if want_before { before } else { after }, &mut v);
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
            Self::Tiles {
                rects,
                before,
                after,
                ..
            } => {
                (before.len() + after.len()) * size_of::<T>()
                    + rects.len() * size_of::<PlaneWindow>()
            }
            Self::Whole { before, after } => (before.len() + after.len()) * size_of::<T>(),
            Self::Nova { before } => before.len() * size_of::<T>(),
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
    /// Um `OnlyAfter` que guarda só a janela fora do `neutro`, sem segurar o plano vivo: o refazer recria-o
    /// ([`aparece`], ADR-0205; o relevo no F1p). `win = None` = nasceu todo neutro.
    Aparece {
        len: usize,
        win: Option<PlaneWindow>,
        after: Box<[T]>,
        neutro: T,
    },
}

/// Um mapa `LayerId -> plano` guardado como delta (os três planos de impasto: `heights`, `covers`,
/// `mats`).
#[derive(Clone, Debug, Default)]
pub(crate) struct StoredMap<T> {
    entries: BTreeMap<RtLayerId, StoredEntry<T>>,
    /// A janela fora do neutro de cada plano que APARECE, achada no commit ([`aparece`]).
    aparece: BTreeMap<RtLayerId, confine::PlaneReach>,
}

impl<T: Copy + Default + PartialEq + Send + Sync> StoredMap<T> {
    /// Extrai o delta dos dois mapas e **esvazia os dois**. `stride` é em elementos por linha do plano
    /// (um por pixel nos três mapas de impasto).
    pub(crate) fn split(
        before: &mut BTreeMap<RtLayerId, Arc<Vec<T>>>,
        after: &mut BTreeMap<RtLayerId, Arc<Vec<T>>>,
        stride: usize,
        hint: Option<PlaneWindow>,
    ) -> Self {
        let mut entries = BTreeMap::new();
        let keys: Vec<RtLayerId> = before.keys().chain(after.keys()).copied().collect();
        for k in keys {
            if entries.contains_key(&k) {
                continue;
            }
            let e = match (before.get(&k).cloned(), after.get(&k).cloned()) {
                (Some(mut b), Some(mut a)) => {
                    StoredEntry::Both(StoredPlane::split(&mut b, &mut a, stride, hint))
                }
                (Some(b), None) => StoredEntry::OnlyBefore(b),
                (None, Some(a)) => StoredEntry::OnlyAfter(a),
                (None, None) => continue,
            };
            entries.insert(k, e);
        }
        before.clear();
        after.clear();
        Self {
            entries,
            aparece: BTreeMap::new(),
        }
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
                borda => {
                    if let Some(v) = Self::borda(borda, want_before) {
                        out.insert(*k, v);
                    }
                }
            }
        }
        Some(out)
    }

    /// O lado de uma chave que existe num lado só (não lê a base); `None` = a chave não existe nesse lado.
    pub(super) fn borda(e: &StoredEntry<T>, want_before: bool) -> Option<Arc<Vec<T>>> {
        match e {
            StoredEntry::Both(_) => None,
            StoredEntry::OnlyBefore(v) => want_before.then(|| Arc::clone(v)),
            StoredEntry::OnlyAfter(v) => (!want_before).then(|| Arc::clone(v)),
            StoredEntry::Aparece {
                len,
                win,
                after,
                neutro,
            } => (!want_before).then(|| Arc::new(aparece::recria(*len, *win, after, *neutro))),
        }
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
                    let mut rgba8 = Arc::clone(&c.rgba8);
                    let px: &mut Vec<u8> = Arc::make_mut(&mut rgba8);
                    win.blit(if want_before { before } else { after }, px);
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

/// **A rota do JOURNAL** — o mesmo motor, com o lado `before` vindo dos bytes que a escrita capturou
/// em vez de um segundo snapshot (doc 28 §5.58.2, degrau 2 do S3).
///
/// Filho por `#[path]`, e não irmão: ele mexe nos campos privados de [`PlaneWindow`], que são privados
/// ao MÓDULO — um irmão teria de os abrir para o resto da crate, e a janela deixaria de ser um tipo com
/// um dono. O corte é o mesmo do [`tests`]: *o que um plano guardado É* fica aqui, *de onde o `before`
/// vem quando não há segundo snapshot* fica lá.
#[path = "undo_delta_diff.rs"]
mod diff; // a régua exacta: onde dois planos diferem
use diff::diff_window;

#[path = "undo_delta_journal.rs"]
mod journal_route;

/// **Até onde um passo ALCANÇA** — filho pelo mesmo motivo (ele lê os campos privados da janela para
/// a converter de volta em região de canvas). O corte: *o que um plano guardado É* fica aqui, *quanto
/// da figura ele pode ter mudado* fica lá.
#[path = "undo_delta_confine.rs"]
pub(crate) mod confine;

/// **Re-partir um plano do topo sem materializar a tela** — filho pelo mesmo motivo (a janela).
#[path = "undo_delta_aparece.rs"]
pub(crate) mod aparece;

#[path = "undo_delta_absorb.rs"]
mod absorb_route;

#[path = "undo_delta_tiles.rs"]
pub(crate) mod tiles;

#[path = "undo_delta_copiado.rs"]
mod copiado;

#[path = "undo_delta_composto.rs"]
pub(crate) mod composto; // o topo seguido do journal do vidro (a absorção do escorrido, ADR-0205)

#[cfg(test)]
#[path = "undo_delta_tests.rs"]
mod tests;
