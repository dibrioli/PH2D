//! O estado do **FILTERS** (a pilha de FX raster, plano 24) publicado pela shell — irmão de
//! `state.rs` pelo teto de 600 LOC, e coeso pelo mesmo critério que já separou os Effects /
//! Contour / Pattern.
//!
//! Três fatos, e só a shell os sabe:
//!
//! - **`can_add`** — *"a seleção tem forma para receber um filtro?"*. Sem isso a seção não sobe.
//! - **a PILHA VIVA** da seleção, linha a linha — para os controles nascerem no lugar certo e para
//!   selecionar OUTRA forma mostrar a pilha DELA.
//! - **os nomes dos tipos** que o "Add" oferece, publicados a partir do motor (`ph2d_ecs::FxOp`),
//!   que este painel não alcança — o padrão da seção Effects.
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

/// Um degrau da pilha, como o painel o desenha. Espelha o `ph2d_ecs::FxOp` — o painel **não
/// alcança** o `ph2d-ecs` (ele vive de snapshots), e é a shell que traduz na fronteira.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterRowView {
    /// O nome do efeito ("Blur", "Glow", "Drop Shadow") — vem do motor.
    pub label: &'static str,
    /// `0` Blur · `1` Glow · `2` Drop Shadow. Decide QUAIS controles a linha oferece: só o Drop
    /// Shadow tem offset, e o Blur não tem cor (seriam knobs mortos).
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
    /// Os tipos que o menu "Add" oferece — publicados a partir do motor.
    static KINDS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
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

/// Publica os tipos que o "Add" oferece, na ordem dos códigos de `kind`.
pub fn set_filter_kind_names(names: Vec<&'static str>) {
    KINDS.with(|k| *k.borrow_mut() = names);
}

pub(crate) fn kinds() -> Vec<&'static str> {
    KINDS.with(|k| k.borrow().clone())
}
