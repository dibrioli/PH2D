//! ⭐⭐⭐ **A APARÊNCIA DO OBJECTO publicada pela shell** — irmão do [`crate::state`] pelo teto de
//! 600 LOC.
//!
//! O que a forma SELECIONADA é, nas duas propriedades que descrevem o objecto inteiro e não uma
//! tinta dele: a **opacidade** e o **modo de mistura** (estudo 42 item 2, v19 do schema).
//!
//! ⚠️ **`None` esconde a seção**, e é a mesma lei das outras seções de forma: sem uma forma
//! selecionada não há sujeito, e uma seção com sliders que não descrevem nada é pior que uma seção
//! ausente — o artista arrasta e nada acontece.

use ph2d_editor_core::zones::Rect;
use ph2d_vec_scene::BlendMode;
use std::cell::{Cell, RefCell};

/// ⭐⭐⭐ **UMA CAMADA da pilha de aparência, como o painel a mostra** (v20).
///
/// ⚠️ É uma VISTA, e não o [`ph2d_vec_scene::PaintEntry`]: o painel precisa do que se PINTA numa
/// linha (um nome, uma cor, uma largura), e publicar o tipo do documento obrigaria o painel a saber
/// desempacotar um `Paint` — que é exactamente a segunda transcrição de *"de que cor é isto?"* que
/// o `swatch_color` existe para não haver.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PaintRow {
    /// `true` = preenchimento, `false` = contorno. É o que decide o rótulo e se há largura.
    pub is_fill: bool,
    /// A cor da swatch, **em bytes** — o formato que o `ColorSwatch` desta casa pinta.
    ///
    /// ⚠️ Ela era `[f32; 4]` e o painel dividia por 255. Isso é uma SEGUNDA régua de *"que cor é
    /// esta?"* no sítio errado: o documento guarda bytes, a swatch pinta bytes, e a fracção só
    /// existia entre os dois. (O `no_magic_numeric` apanhou-a — o `255.0` num ficheiro de painel.)
    pub color: [u8; 4],
    /// A largura, num contorno.
    pub width: f64,
    /// O olho: desligada, a camada não desenha e os parâmetros ficam.
    pub enabled: bool,
    /// `0..=1`.
    pub opacity: f32,
    pub blend: BlendMode,
    /// ⭐ **ONDE esta camada desenha**, relativo à forma, em unidades de mundo (v21).
    ///
    /// ⚠️ Publicado como o `width` é: o painel MOSTRA o que o documento tem, e o campo é semeado
    /// dele no passe de sementes — sem isso ele mostraria o que a última edição deixou no store, que
    /// é o defeito que a wave anterior pagou no campo de largura.
    pub offset: [f64; 2],
}

/// O que a forma selecionada tem, hoje.
#[derive(Clone, Debug, PartialEq)]
pub struct Appearance {
    /// `0..=1`, `1` = opaca.
    pub opacity: f32,
    /// O modo de mistura do documento.
    ///
    /// ⚠️ **É o MODO, não o índice na lista do dropdown.** A lista é derivada da tradução para o
    /// Vello (`ph2d_vec_render::blend::offered`) e os dois lados — o painel que a pinta e a shell
    /// que recebe o clique — chamam a MESMA função. Publicar um índice faria a shell ter de
    /// reconstruir a lista para o traduzir, e uma segunda cópia dela é como as duas passam a
    /// discordar sobre o que a linha 7 significa.
    pub blend: BlendMode,
    /// ⭐ **A PILHA**, do CHÃO para o TOPO — a mesma ordem do documento.
    ///
    /// ⚠️ **O painel pinta-a ao contrário** (o topo em cima, como o Illustrator), e essa inversão
    /// vive num sítio só: senão o artista arrasta uma camada para cima e ela desce.
    pub layers: Vec<PaintRow>,
}

thread_local! {
    static CURRENT: RefCell<Option<Appearance>> = const { RefCell::new(None) };
    /// O rect do chip de mistura quando o popover está aberto — o passe diferido do `paint.rs`
    /// consome-o e pinta a lista POR CIMA de todas as seções (mesma lei dos outros quatro slots).
    static PENDING_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// A camada ABERTA no painel — vista, nunca documento.
    static OPEN: Cell<Option<usize>> = const { Cell::new(None) };
    /// O rect do chip de mistura da CAMADA, para o passe diferido.
    static PENDING_PAINT_DD: Cell<Option<Rect>> = const { Cell::new(None) };
}

/// Publica a aparência da forma selecionada; `None` esconde a seção.
pub fn set_current_appearance(a: Option<Appearance>) {
    CURRENT.with(|c| *c.borrow_mut() = a);
}

pub(crate) fn current_appearance() -> Option<Appearance> {
    CURRENT.with(|c| c.borrow().clone())
}

/// **Qual camada está ABERTA** (as propriedades dela aparecem por baixo da linha).
///
/// ⚠️ **É estado de VISTA**: vive só aqui, não viaja no documento, não entra no undo e não se
/// grava — é a mesma lei da caixa «Show sheet on canvas» do Inspector. O índice é o do DOCUMENTO
/// (chão → topo), e não o da linha pintada.
pub fn open_layer_index() -> Option<usize> {
    open_layer()
}

pub(crate) fn open_layer() -> Option<usize> {
    OPEN.with(Cell::get)
}

/// Abre uma camada, ou fecha a que estava aberta se for a mesma.
pub fn toggle_open_layer(i: usize) {
    OPEN.with(|c| c.set(if c.get() == Some(i) { None } else { Some(i) }));
}

/// Fecha a camada aberta — o que um gesto que muda a PILHA tem de fazer.
///
/// ⚠️ Sem isto, apagar a camada 2 deixaria a 2 aberta e o painel mostraria as propriedades da que
/// tomou o lugar dela: *um índice guardado sobrevive à lista que o explicava*.
pub fn close_open_layer() {
    OPEN.with(|c| c.set(None));
}

/// Que LINHA da lista de modos da CAMADA tem este id.
pub(crate) fn paint_blend_option_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..usize::from(ph2d_vec_scene::MAX_BLEND_MODES))
        .find(|&i| crate::ids::vector_paint_blend_option_id(i) == id)
}

pub(crate) fn set_pending_paint_blend_dd(rect: Option<Rect>) {
    PENDING_PAINT_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_paint_blend_dd() -> Option<Rect> {
    PENDING_PAINT_DD.with(Cell::take)
}

/// Que LINHA da lista de modos tem este id (`None` se não for uma opção do popover).
///
/// ⚠️ Varre o espaço FIXO de ids (`MAX_BLEND_MODES`), como as outras fábricas deste painel: a
/// resolução não pode depender de quantos modos a lista oferece HOJE, senão um modo novo traria
/// um id que ninguém reconhece.
pub(crate) fn blend_option_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..usize::from(ph2d_vec_scene::MAX_BLEND_MODES))
        .find(|&i| crate::ids::vector_obj_blend_option_id(i) == id)
}

pub(crate) fn set_pending_obj_blend_dd(rect: Option<Rect>) {
    PENDING_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_obj_blend_dd() -> Option<Rect> {
    PENDING_DD.with(Cell::take)
}
