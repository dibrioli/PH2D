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

/// **O que um tipo de degrau é**, como o painel precisa de saber. Espelha o `ph2d_ecs::FxKindSpec`
/// — o painel **não alcança** o `ph2d-ecs` (vive de snapshots), e é a shell que traduz na
/// fronteira. Indexada pelo `kind`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterKindView {
    /// O nome do efeito ("Blur", "Outline", "Color Overlay"…) — o rótulo do card e do "Add".
    pub name: &'static str,
    /// O rótulo do raio, ou `None` se o tipo não tem raio (o Color Overlay é pontual).
    pub radius_label: Option<&'static str>,
    /// Oferece Offset X/Y?
    pub has_offset: bool,
    /// Oferece a cor do halo?
    pub has_color: bool,
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
}

thread_local! {
    static CAN_ADD: Cell<bool> = const { Cell::new(false) };
    /// A pilha da forma selecionada, NA ORDEM em que se aplica.
    static STACK: RefCell<Vec<FilterRowView>> = const { RefCell::new(Vec::new()) };
    /// A tabela dos tipos, indexada pelo `kind` — publicada a partir do motor.
    static KINDS: RefCell<Vec<FilterKindView>> = const { RefCell::new(Vec::new()) };
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
