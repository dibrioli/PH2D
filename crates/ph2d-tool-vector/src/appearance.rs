//! **A aparência da forma selecionada** (opacidade, mistura e a PILHA de camadas Fill/Stroke) —
//! desceu do `ph2d-panel-vector` (CartoesUI C4): o Inspector em cartões também a lê, e painel não
//! depende de painel. A shell (`ph2d-app-vec`) publica; os painéis leem.

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
    /// As pontas do degradê (`None` = tinta sólida): a amostra da linha pinta-as em rampa.
    pub ramp: Option<[[u8; 4]; 2]>,
    /// A cor da swatch, **em bytes** — o formato que o `ColorSwatch` desta casa pinta.
    ///
    /// ⚠️ Ela era `[f32; 4]` e o painel dividia por 255. Isso é uma SEGUNDA régua de *"que cor é
    /// esta?"* no sítio errado: o documento guarda bytes, a swatch pinta bytes, e a fracção só
    /// existia entre os dois. (O `no_magic_numeric` apanhou-a — o `255.0` num ficheiro de painel.)
    pub color: [u8; 4],
    /// A largura, num contorno.
    pub width: f64,
    /// A Position, num contorno (F44b).
    pub align: ph2d_vec_scene::StrokeAlign,
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
    /// ⭐ **O OFFSET DE CAD** (v22): a silhueta cresce (`>0`) ou encolhe (`<0`).
    pub dilate: f64,
    /// A quina dele (`0` Miter · `1` Round · `2` Bevel) — o painel pinta a fileira só com o offset
    /// armado.
    pub dilate_join: u8,
    /// Os campos em que as formas selecionadas DIFEREM nesta camada (plano 32).
    pub mixed: RowMixed,
}

/// **Em que campos a camada `i` difere entre as selecionadas** — o painel pinta-os em «misto»
/// (amostra «—», número em branco, nenhum segmento aceso), a lei do F20 levada à pilha.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct RowMixed {
    pub color: bool,
    pub width: bool,
    pub opacity: bool,
    pub blend: bool,
    pub offset: bool,
    pub dilate: bool,
    pub dilate_join: bool,
    pub align: bool,
    /// O olho: a camada visível numas e escondida noutras (F22).
    pub enabled: bool,
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
    /// ⭐ F45 — **a pilha À VISTA, do chão ao topo**: onde a tinta de base fica entre as camadas, só as
    /// bases que a forma tem (vistas ou escondidas). Vazia = a ordem neutra ([`Self::stack`]).
    pub order: Vec<ph2d_vec_scene::StackItem>,
    /// A seleção múltipla tem opacidades / modos DIFERENTES: o número fica em branco e o chip do
    /// modo diz «misto» (plano 31; revoga o «mostra o primário» do `appearance.rs`).
    pub mixed_opacity: bool,
    pub mixed_blend: bool,
    /// ⭐ **As pilhas das selecionadas têm ESTRUTURAS diferentes** (outro número de camadas, ou
    /// outra espécie no mesmo índice): o painel não lista linhas — a linha `2` de uma não é a `2`
    /// da outra — e diz «Camadas mistas»; *+ Fill / + Stroke* continuam a valer para todas.
    pub layers_differ: bool,
}

impl Appearance {
    /// A pilha inteira, do chão ao topo — a publicada, ou a neutra (base fill, base stroke, camadas).
    #[must_use]
    pub fn stack(&self) -> Vec<ph2d_vec_scene::StackItem> {
        use ph2d_vec_scene::StackItem;
        let camadas = self
            .order
            .iter()
            .filter(|it| matches!(it, StackItem::Entry(_)))
            .count();
        if !self.order.is_empty() && camadas == self.layers.len() {
            return self.order.clone();
        }
        [StackItem::BaseFill, StackItem::BaseStroke]
            .into_iter()
            .chain((0..self.layers.len()).map(StackItem::Entry))
            .collect()
    }
}

thread_local! {
    /// ⭐ F45 — o arrasto pelo ⚙ largou a linha: (linha, lugar), para o verbo `Move` da shell.
    static PAINT_MOVE: Cell<Option<(ph2d_vec_scene::StackItem, ph2d_vec_scene::Place)>> =
        const { Cell::new(None) };
    /// O estilo da ferramenta publicado pela shell (cap, join, alinhamento, largura…).
    static STYLE: RefCell<Option<crate::VectorStyleSnapshot>> = const { RefCell::new(None) };
    /// A camada ABERTA da pilha (a que mostra as propriedades) — partilhada pelo painel Vector e
    /// pelo Inspector em cartões.
    static OPEN: Cell<Option<usize>> = const { Cell::new(None) };
    static CURRENT: RefCell<Option<Appearance>> = const { RefCell::new(None) };
}

/// ⭐ **O estado de uma tinta de BASE** (F44b): o que o olho e o «−» da linha dela mostram.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BasePaint {
    /// Não há tinta (o «+» do título dá-a).
    Absent,
    Shown,
    /// Escondida pelo olho: a tinta fica guardada e a linha a cinzento.
    Hidden,
}

thread_local! {
    static BASE_PAINTS: Cell<Option<[BasePaint; 2]>> = const { Cell::new(None) };
    static BASE_FILL_RAMP: Cell<Option<[[u8; 4]; 2]>> = const { Cell::new(None) };
}

/// Publica `[preenchimento, traço]` da forma selecionada (`None` sem resposta).
pub fn set_base_paints(v: Option<[BasePaint; 2]>) {
    BASE_PAINTS.with(|c| c.set(v));
}

/// `[preenchimento, traço]` da forma selecionada.
#[must_use]
pub fn base_paints() -> Option<[BasePaint; 2]> {
    BASE_PAINTS.with(Cell::get)
}

/// As duas pontas do degradê do preenchimento de base (`None` = tinta sólida) — a amostra da linha
/// pinta-as em rampa, como o Figma.
pub fn set_base_fill_ramp(v: Option<[[u8; 4]; 2]>) {
    BASE_FILL_RAMP.with(|c| c.set(v));
}

#[must_use]
pub fn base_fill_ramp() -> Option<[[u8; 4]; 2]> {
    BASE_FILL_RAMP.with(Cell::get)
}

/// Publica a aparência da forma selecionada; `None` esconde a seção.
pub fn set_current_appearance(a: Option<Appearance>) {
    CURRENT.with(|c| *c.borrow_mut() = a);
}

pub fn current_appearance() -> Option<Appearance> {
    CURRENT.with(|c| c.borrow().clone())
}

pub fn open_layer_index() -> Option<usize> {
    OPEN.with(Cell::get)
}

/// ⭐ F44 — a camada aberta é DERIVADA do `Dropdown` da linha dela (`paint_side`, uma vez por
/// quadro), e um campo da LINHA de outra camada aponta-a por um gesto (`event_paint_stack`).
pub fn set_open_layer(i: Option<usize>) {
    OPEN.with(|c| c.set(i));
}

thread_local! {
    /// C2: a linha de um cartão PRESO que o gesto aponta (o `OPEN` é da seleção e fica intocado).
    static PINNED_ROW: Cell<Option<usize>> = const { Cell::new(None) };
}

/// O campo da linha `i` de um cartão preso foi editado: a escrita do quadro é nela, no objecto dele.
pub fn set_pinned_row(i: usize) {
    PINNED_ROW.with(|c| c.set(Some(i)));
}

#[must_use]
pub fn pinned_row() -> Option<usize> {
    PINNED_ROW.with(Cell::get)
}

/// ⭐ F45 — guarda a queda do arrasto pelo ⚙ (o painel), para o verbo `Move` a ler (a shell).
pub fn set_paint_move(m: Option<(ph2d_vec_scene::StackItem, ph2d_vec_scene::Place)>) {
    PAINT_MOVE.with(|c| c.set(m));
}

/// Tira a queda guardada (uma vez).
pub fn take_paint_move() -> Option<(ph2d_vec_scene::StackItem, ph2d_vec_scene::Place)> {
    PAINT_MOVE.with(Cell::take)
}

pub fn toggle_open_layer(i: usize) {
    OPEN.with(|c| c.set(if c.get() == Some(i) { None } else { Some(i) }));
}

pub fn close_open_layer() {
    OPEN.with(|c| c.set(None));
}

pub fn set_current_vector_style(snapshot: Option<crate::VectorStyleSnapshot>) {
    STYLE.with(|c| *c.borrow_mut() = snapshot);
}

#[must_use]
pub fn current_vector_style() -> Option<crate::VectorStyleSnapshot> {
    STYLE.with(|c| *c.borrow())
}
