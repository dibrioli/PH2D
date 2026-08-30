//! O estado retido do navegador — e a **publicação** do índice, que vem do shell.
//!
//! ⚠️ O índice **não** vive aqui: ele é derivado do mundo + do `AssetDb` a cada quadro
//! (`shells/desktop/src/asset_index_build.rs`) e publicado por [`set_current_index`], que é o mesmo
//! idioma do `ph2d_panel_inspector::set_current_inspector_instance`. *Guardá-lo no painel faria do
//! painel a segunda fonte de verdade sobre o que existe.*

use ph2d_asset_index::{AssetIndex, AssetKind, SortBy};
use ph2d_editor_core::zones::Rect;
use std::cell::RefCell;

thread_local! {
    /// O índice do quadro corrente. Vazio = ainda ninguém publicou (o painel diz isso em vez de
    /// desenhar uma grade vazia que se lê como *«não tenho assets»*).
    static CURRENT: RefCell<AssetIndex> = RefCell::new(AssetIndex::new());
    /// ⚠️ Publicado ou não — a distinção existe porque um índice **vazio** e um **por publicar**
    /// desenham mensagens diferentes, e lê-los iguais é a armadilha do balde vazio.
    static PUBLISHED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Altura do conteúdo da grade no último `paint` (para a barra de rolagem).
    static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Altura visível do corpo no último `paint`.
    static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Os endereços que a grade pintou, na ordem em que os pintou. É o que o `apply_event`
    /// consulta para saber **qual asset** é a célula clicada.
    ///
    /// ⚠️ **Pintar e despachar leem a MESMA lista** (memória
    /// `feedback_paint_and_dispatch_must_read_the_same_source`): recalcular a consulta no
    /// `apply_event` daria duas respostas sempre que o filtro mudasse entre o quadro que pintou e
    /// o clique que chegou.
    static PAINTED: RefCell<Vec<ph2d_asset_index::AssetRef>> = const { RefCell::new(Vec::new()) };
}

/// **O shell publica o índice do quadro.** Chamado uma vez por quadro, antes do paint.
pub fn set_current_index(index: AssetIndex) {
    CURRENT.with(|c| *c.borrow_mut() = index);
    PUBLISHED.with(|c| c.set(true));
}

/// Lê o índice publicado.
pub(crate) fn with_index<R>(f: impl FnOnce(&AssetIndex) -> R) -> R {
    CURRENT.with(|c| f(&c.borrow()))
}

/// Alguém já publicou um índice nesta sessão?
pub(crate) fn is_published() -> bool {
    PUBLISHED.with(std::cell::Cell::get)
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

/// Altura do conteúdo medida no último paint.
#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(std::cell::Cell::get)
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

/// Altura visível medida no último paint.
#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(std::cell::Cell::get)
}

pub(crate) fn set_painted(keys: Vec<ph2d_asset_index::AssetRef>) {
    PAINTED.with(|c| *c.borrow_mut() = keys);
}

/// **Só para os gates:** semear o que a grade teria pintado, sem correr um `paint` com GPU.
///
/// ⚠️ Ela existe porque a costura que interessa — *o duplo-clique chega ao barramento com o id
/// certo?* — vive DEPOIS do desenho, e um teste que precisasse de uma superfície de janela real
/// nunca correria. É o mesmo idioma do `probe_*` dos irmãos.
pub fn probe_set_painted(keys: Vec<ph2d_asset_index::AssetRef>) {
    set_painted(keys);
}

/// ⭐ **A carga que o cartão `index` arrasta** — a ponte entre o endereço da biblioteca
/// ([`ph2d_asset_index::AssetRef`]) e o que atravessa o painel
/// ([`ph2d_editor_core::interaction::drag_payload::DragPayload`]).
///
/// ⚠️ **Os dois tipos existem de propósito e esta é a única conversão.** O `AssetRef` é o endereço
/// da BIBLIOTECA; o `DragPayload` é fundação de chrome e não pode conhecer o modelo de assets
/// (senão a UI passa a depender dele). Aqui — na crate que conhece os dois — eles encontram-se uma
/// vez.
#[must_use]
pub fn payload_at(
    index: usize,
) -> Option<ph2d_editor_core::interaction::drag_payload::DragPayload> {
    use ph2d_editor_core::interaction::drag_payload::DragPayload;
    match painted_at(index)? {
        ph2d_asset_index::AssetRef::Component { stable_id } => {
            Some(DragPayload::Prefab { stable_id })
        }
        ph2d_asset_index::AssetRef::Texture { asset } => Some(DragPayload::Image { asset }),
    }
}

/// O asset que a célula `index` desenhou neste quadro.
pub(crate) fn painted_at(index: usize) -> Option<ph2d_asset_index::AssetRef> {
    PAINTED.with(|c| c.borrow().get(index).copied())
}

/// Estado retido do painel — **a VISTA**, e só ela.
///
/// ⚠️ Nada aqui é documento: fechar o app esquece a busca, a ordenação e o tamanho do cartão, e
/// isso é a decisão. O que o artista organizou (catálogos, etiquetas) é documento e vive no
/// índice — e ainda não existe (wave A3).
#[derive(Clone, Debug)]
pub struct AssetBrowserState {
    /// O rectângulo flutuante. `None` = ainda não foi aberto (o paint semeia).
    pub rect: Option<Rect>,
    /// A ordenação da grade.
    pub sort: SortBy,
    /// O filtro de família. `None` = todas.
    pub kind: Option<AssetKind>,
    /// O lado do cartão, em px. O slider escreve-o.
    pub cell_px: f32,
}

/// Lado mínimo de um cartão — abaixo disto o nome deixa de caber numa linha, e um cartão sem nome
/// não é um asset, é um quadrado.
pub const CELL_MIN_PX: f32 = 56.0; // LITERAL-PX-OK: piso do cartão, domínio da grade
/// Lado máximo — acima disto cabe **uma** coluna no painel flutuante, e uma grade de uma coluna
/// é uma lista com desperdício.
pub const CELL_MAX_PX: f32 = 160.0; // LITERAL-PX-OK: teto do cartão, domínio da grade
/// O lado com que a grade nasce.
pub const CELL_DEFAULT_PX: f32 = 84.0; // LITERAL-PX-OK: default do cartão, domínio da grade

impl Default for AssetBrowserState {
    fn default() -> Self {
        Self {
            rect: None,
            sort: SortBy::Name,
            kind: None,
            cell_px: CELL_DEFAULT_PX,
        }
    }
}

impl AssetBrowserState {
    /// A posição normalizada do slider de tamanho (`0..1`) — a lei em UM sentido.
    #[must_use]
    pub fn size_slider_value(&self) -> f32 {
        ((self.cell_px - CELL_MIN_PX) / (CELL_MAX_PX - CELL_MIN_PX)).clamp(0.0, 1.0)
    }

    /// E no outro. ⚠️ **As duas metades vivem juntas de propósito** — a régua da barra de frames
    /// do Sprite estava em três cópias e uma mutação sobreviveu a mudar só a do pintor.
    pub fn set_size_from_slider(&mut self, t: f32) {
        self.cell_px = CELL_MIN_PX + t.clamp(0.0, 1.0) * (CELL_MAX_PX - CELL_MIN_PX);
    }

    /// O filtro que o chip `i` representa. ⚠️ **Derivado de `AssetKind::ALL`**, não de uma lista
    /// escrita à mão: uma família nova aparece na fileira sozinha.
    #[must_use]
    pub fn kind_for_chip(i: usize) -> Option<AssetKind> {
        if i == 0 {
            None
        } else {
            AssetKind::ALL.get(i - 1).copied()
        }
    }

    /// O rótulo do chip `i`.
    #[must_use]
    pub fn kind_chip_label(i: usize) -> &'static str {
        Self::kind_for_chip(i).map_or("All", AssetKind::label)
    }
}

/// **Só para os roteiros de smoke:** quantos assets o índice publicado tem, e de que famílias.
///
/// ⚠️ Ela existe porque a pergunta do report do Enio — *«as imagens aparecem no painel?»* — não é
/// alcançável de um `#[test]` (o índice é publicado por um quadro real, com `AppGfx` na mão) nem
/// legível de um pixel. *Uma cena que não sabe dizer o que a grade tem só sabe dizer que não
/// estourou.*
#[must_use]
pub fn probe_index_summary() -> (usize, String) {
    with_index(|ix| {
        let counts = ix.counts();
        let text = counts
            .iter()
            .map(|(k, n)| format!("{} x{n}", k.label()))
            .collect::<Vec<_>>()
            .join(", ");
        (
            ix.len(),
            if text.is_empty() {
                "vazio".into()
            } else {
                text
            },
        )
    })
}
