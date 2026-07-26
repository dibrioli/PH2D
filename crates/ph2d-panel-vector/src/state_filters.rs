//! O estado do **FILTERS** (FX raster, plano 24) publicado pela shell — irmão de `state.rs` pelo
//! teto de 600 LOC, e coeso pelo mesmo critério que já separou os Effects / Contour / Pattern.
//!
//! Dois fatos, e só a shell os sabe:
//!
//! - **`can_add`** — *"a seleção tem forma para receber um filtro?"*. Sem isso a seção não sobe.
//! - **os valores VIVOS** do filtro da seleção — para os controles nascerem no lugar certo e para
//!   selecionar OUTRA forma mostrar o filtro DELA.
//!
//! ⚠️ `present` decide qual das duas caras a seção mostra: sem filtro, só o seletor de tipo (com
//! *None* aceso); com filtro, o seletor + os parâmetros. O seletor É a porta de armar/desarmar —
//! diferente do Contour (que tem um botão *Add*), porque um filtro tem TIPO, e o tipo é a escolha.

use std::cell::Cell;

/// Faixa do slider de **Radius**, em unidades de MUNDO. NÃO é um limite físico — o produtor capa a
/// textura em `MAX_FX_SIDE`; é só o alcance do controle. (Fração-do-tamanho seria mais robusto para
/// formas de tamanhos diferentes — refino futuro, a mesma nota que o Contour faz do Offset.)
pub(crate) const FILTER_RADIUS_MAX: f64 = 2.0;
/// Faixa BIPOLAR do slider de **Offset** da Drop Shadow (mundo), `0.5` = zero.
pub(crate) const FILTER_OFFSET_MAX: f64 = 2.0;

thread_local! {
    static CAN_ADD: Cell<bool> = const { Cell::new(false) };
    static PRESENT: Cell<bool> = const { Cell::new(false) };
    /// O `kind` do `VecFilter` quando `present` (0 Blur · 1 Glow · 2 Drop Shadow).
    static KIND: Cell<u8> = const { Cell::new(0) };
    static RADIUS: Cell<f64> = const { Cell::new(0.0) };
    static OFFX: Cell<f64> = const { Cell::new(0.0) };
    static OFFY: Cell<f64> = const { Cell::new(0.0) };
    static COLOR: Cell<[u8; 4]> = const { Cell::new([0, 0, 0, 255]) };
    static OPACITY: Cell<f64> = const { Cell::new(1.0) };
}

/// Publica se a seleção corrente permite um filtro (há forma selecionada).
pub fn set_current_filter_can_add(v: bool) {
    CAN_ADD.with(|c| c.set(v));
}

pub(crate) fn can_add() -> bool {
    CAN_ADD.with(Cell::get)
}

/// Publica o filtro da seleção: se existe, e com que valores.
#[allow(clippy::too_many_arguments)]
pub fn set_current_filter(
    present: bool,
    kind: u8,
    radius: f64,
    offx: f64,
    offy: f64,
    color: [u8; 4],
    opacity: f64,
) {
    PRESENT.with(|c| c.set(present));
    KIND.with(|c| c.set(kind));
    RADIUS.with(|c| c.set(radius));
    OFFX.with(|c| c.set(offx));
    OFFY.with(|c| c.set(offy));
    COLOR.with(|c| c.set(color));
    OPACITY.with(|c| c.set(opacity));
}

pub(crate) fn present() -> bool {
    PRESENT.with(Cell::get)
}

pub(crate) fn kind() -> u8 {
    KIND.with(Cell::get)
}

pub(crate) fn radius() -> f64 {
    RADIUS.with(Cell::get)
}

pub(crate) fn offx() -> f64 {
    OFFX.with(Cell::get)
}

pub(crate) fn offy() -> f64 {
    OFFY.with(Cell::get)
}

pub(crate) fn color() -> [u8; 4] {
    COLOR.with(Cell::get)
}

pub(crate) fn opacity() -> f64 {
    OPACITY.with(Cell::get)
}
