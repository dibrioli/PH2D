//! O estado do **FILTERS** (a pilha de FX raster, plano 24) publicado pela shell — irmão de
//! `state.rs` pelo teto de 600 LOC, e coeso pelo mesmo critério que já separou os Effects /
//! Contour / Pattern.
//!
//! Três fatos, e só a shell os sabe:
//!
//! - **`can_add`** — *"a seleção tem forma para receber um filtro?"*. Sem isso a seção não sobe.
//! - **a PILHA VIVA** da seleção, linha a linha — para os controles nascerem no lugar certo e para
//!   selecionar OUTRA forma mostrar a pilha DELA.
//! - **a TABELA dos tipos** que o "Add" oferece, publicada a partir do motor (`ph2d_ecs::FxOp`),
//!   que este painel não alcança — o padrão da seção Effects. ⚠️ Ela traz o nome **e** que
//!   controles cada tipo usa: com sete tipos, decidir isso por `kind == 2` dentro do `paint` (o
//!   que a W2 fazia com três) apodrece na primeira adição, e o modo de falha é um knob morto que
//!   nenhum gate vê.
//!
//! ⚠️ **Uma seção que é uma LISTA, não um formulário.** A W1 era um filtro só, e o estado era um
//! punhado de `Cell`s escalares; a W2 é uma pilha ordenada (o modelo AE/Photoshop/Figma) e o
//! estado é um `Vec` de linhas. A ORDEM é dado, não apresentação: `Shadow → Blur` e
//! `Blur → Shadow` desenham coisas diferentes.

use std::cell::{Cell, RefCell};

/// Faixa do slider de **Radius**, em unidades de MUNDO. NÃO é um limite físico — o produtor capa a
/// textura em `MAX_FX_SIDE`; é só o alcance do controle. (Fração-do-tamanho seria mais robusto para
/// formas de tamanhos diferentes — refino futuro, a mesma nota que o Contour faz do Offset.)
pub(crate) const FILTER_RADIUS_MAX: f64 = 2.0;
/// Faixa BIPOLAR do slider de **Offset** da Drop Shadow (mundo), `0.5` = zero.
pub(crate) const FILTER_OFFSET_MAX: f64 = 2.0;
/// Faixa do slider de **Size** do ruído, em unidades de MUNDO — a mesma do raio, porque é a mesma
/// grandeza (um comprimento no documento) e duas réguas diferentes para a mesma unidade é o que
/// faz um artista aprender dois números.
pub(crate) const FILTER_SCALE_MAX: f64 = 2.0;
/// Quantas OITAVAS o slider de **Detail** oferece. Espelha o `ph2d_ecs::FxOp::MAX_DETAIL`, que o
/// painel não alcança.
///
/// ⚠️ **`pub` de propósito, ao contrário das irmãs**: é a única faixa desta seção cujo número tem
/// um dono do outro lado da fronteira, e um teto MAIOR aqui deixa o artista pedir oitavas que o
/// produtor clampa em silêncio — o slider anda e o desenho para. O gate que compara os dois mora
/// na shell, o único lugar que alcança as duas crates.
pub const FILTER_DETAIL_MAX: f64 = 6.0; // LITERAL-PX-OK: contagem de oitavas, espelha o FxOp::MAX_DETAIL
/// Quantas realizações o slider de **Seed** oferece — a faixa do `u8` que as guarda. Não é um teto
/// escolhido: é a REPRESENTAÇÃO.
pub(crate) const FILTER_SEED_MAX: f64 = 255.0; // LITERAL-PX-OK: a faixa do u8 que guarda a semente

/// **O que um tipo de degrau é**, como o painel precisa de saber. Espelha o `ph2d_ecs::FxKindSpec`
/// — o painel **não alcança** o `ph2d-ecs` (vive de snapshots), e é a shell que traduz na
/// fronteira. Indexada pelo `kind`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterKindView {
    /// O nome do efeito ("Blur", "Outline", "Color Overlay"…) — o rótulo do card e do "Add".
    pub name: &'static str,
    /// O rótulo do raio, ou `None` se o tipo não tem raio (o Color Overlay é pontual).
    pub radius_label: Option<&'static str>,
    /// Os rótulos do par de offset, ou `None` se o tipo não desloca nada. ⚠️ São NOMES, não um
    /// booleano: o mesmo vetor é *para onde a sombra cai* numa Drop Shadow e *de onde a luz vem*
    /// num Bevel.
    pub offset_labels: Option<(&'static str, &'static str)>,
    /// O rótulo da cor, ou `None` se o tipo não tinge.
    pub color_label: Option<&'static str>,
    /// Os MODOS deste tipo, na ordem dos códigos, ou vazio se ele não tem escolha a fazer.
    pub modes: &'static [&'static str],
    /// **A cor deste degrau pousa sobre conteúdo que já existe?** Espelha o
    /// `ph2d_ecs::FxKindSpec::takes_blend` — é ele que decide se o card oferece a lei de mistura.
    /// Um halo EXTERNO entra por baixo (nada com que se misturar onde ele aparece) e o Blur/Feather
    /// não têm cor própria; oferecer-lhes o controle seria um knob cujo efeito inteiro é uma orla
    /// de 1 px.
    pub takes_blend: bool,
    /// Os rótulos dos três knobs do RUÍDO — `(tamanho, detalhe, semente)` —, ou `None` se este tipo
    /// não lê campo nenhum. Espelha o `ph2d_ecs::FxKindSpec::noise_labels`, e é UM campo para os
    /// três porque eles não existem em separado: descrevem *qual ruído*, em três tempos.
    pub noise_labels: Option<(&'static str, &'static str, &'static str)>,
}

/// Um degrau da pilha, como o painel o desenha. Espelha o `ph2d_ecs::FxOp` — o painel **não
/// alcança** o `ph2d-ecs` (ele vive de snapshots), e é a shell que traduz na fronteira.
///
/// ⚠️ **Sem `label`, de propósito:** o nome vem da tabela dos tipos, indexada por `kind`. Guardá-lo
/// aqui seria uma segunda cópia do mesmo fato, e cópias divergem.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterRowView {
    /// O código do tipo — o índice na tabela publicada, que decide QUAIS controles a linha oferece.
    pub kind: u8,
    /// O modo armado — o índice em `FilterKindView::modes`.
    pub mode: u8,
    /// Ligado? Desligado, a pilha o SALTA e o card é desenhado apagado — mas os parâmetros ficam.
    pub enabled: bool,
    /// O `stdDev` do borrão, em MUNDO.
    pub radius: f64,
    /// O deslocamento em X (mundo) — só o Drop Shadow o lê.
    pub offx: f64,
    /// O deslocamento em Y (mundo) — só o Drop Shadow o lê.
    pub offy: f64,
    /// A cor do halo (RGBA sRGB) — Glow / Drop Shadow.
    pub color: [u8; 4],
    /// A intensidade deste degrau, `0..1`.
    pub opacity: f64,
    /// A LEI DE MISTURA armada — o índice na lista publicada por [`set_filter_blend_names`].
    pub blend: u8,
    /// O TAMANHO das ondulações do ruído, em MUNDO.
    pub scale: f64,
    /// Quantas OITAVAS o ruído soma.
    pub detail: u8,
    /// Qual realização do ruído.
    pub seed: u8,
}

thread_local! {
    static CAN_ADD: Cell<bool> = const { Cell::new(false) };
    /// A pilha da forma selecionada, NA ORDEM em que se aplica.
    static STACK: RefCell<Vec<FilterRowView>> = const { RefCell::new(Vec::new()) };
    /// A tabela dos tipos, indexada pelo `kind` — publicada a partir do motor.
    static KINDS: RefCell<Vec<FilterKindView>> = const { RefCell::new(Vec::new()) };
    /// Os NOMES das leis de mistura, na ordem dos códigos — publicados pela shell, que é quem
    /// alcança o `BlendMode`. O painel vive de snapshots e não conhece o enum.
    static BLENDS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// Publica se a seleção corrente permite um filtro (há forma selecionada).
pub fn set_current_filter_can_add(v: bool) {
    CAN_ADD.with(|c| c.set(v));
}

pub(crate) fn can_add() -> bool {
    CAN_ADD.with(Cell::get)
}

/// Publica a pilha da seleção (vazia = forma nua).
pub fn set_current_filters(rows: Vec<FilterRowView>) {
    STACK.with(|s| *s.borrow_mut() = rows);
}

pub(crate) fn stack() -> Vec<FilterRowView> {
    STACK.with(|s| s.borrow().clone())
}

/// Publica a tabela dos tipos, **na ordem dos códigos de `kind`** (é ela que o `paint` indexa).
pub fn set_filter_kinds(kinds: Vec<FilterKindView>) {
    KINDS.with(|k| *k.borrow_mut() = kinds);
}

pub(crate) fn kinds() -> Vec<FilterKindView> {
    KINDS.with(|k| k.borrow().clone())
}

/// A spec do tipo `kind`, se ela foi publicada. **Porta única** do painel para *"o que este tipo
/// usa?"* — o `paint` nunca decide por aritmética de código.
pub(crate) fn kind_spec(kind: u8) -> Option<FilterKindView> {
    KINDS.with(|k| k.borrow().get(kind as usize).copied())
}

/// Publica os nomes das leis de mistura, **na ordem dos códigos** (é ela que o `paint` indexa).
///
/// ⚠️ Mesmo padrão do [`set_filter_kinds`]: o nome de um código é fato do motor, e o painel não
/// alcança nem o `ph2d-ecs` nem o `ph2d-painter-effects`. Uma tabela escrita à mão aqui derivaria
/// do enum na primeira lei nova, e o modo de falha é um rótulo que nomeia outra coisa.
pub fn set_filter_blend_names(names: Vec<&'static str>) {
    BLENDS.with(|b| *b.borrow_mut() = names);
}

pub(crate) fn blend_names() -> Vec<&'static str> {
    BLENDS.with(|b| b.borrow().clone())
}

/// O nome da lei `code`, ou `"Normal"` se a tabela ainda não foi publicada (o neutro nunca mente
/// sobre o que o degrau faz).
pub(crate) fn blend_name(code: u8) -> &'static str {
    BLENDS.with(|b| {
        b.borrow()
            .get(code as usize)
            .copied()
            .unwrap_or(FALLBACK_BLEND_NAME)
    })
}

/// O rótulo que o chip mostra antes de a shell publicar a tabela.
pub(crate) const FALLBACK_BLEND_NAME: &str = "Normal";
