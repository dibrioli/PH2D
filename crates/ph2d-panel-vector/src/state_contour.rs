//! O estado do **Contour** publicado pela shell — irmão de `state.rs` pelo teto de 600 LOC, e
//! coeso pelo mesmo critério que já separou os Effects, o Envelope, o Text on Path e o Pattern on
//! Path.
//!
//! Dois fatos, e só a shell os sabe:
//!
//! - **`can_add`** — *"a seleção tem forma para receber um contour?"*. É uma pergunta sobre a
//!   SELEÇÃO, feita uma vez, com o pintor a decidir se oferece e o arm a honrar. O painel não vê a
//!   cena (é a lei do `Join Selected Bodies`).
//! - **os valores VIVOS** — para os controles nascerem no lugar certo em vez de saltarem no
//!   primeiro frame, e para que selecionar OUTRA forma com contour mostre o contour DELA.
//!
//! ⚠️ `present` é o que decide qual das duas caras a seção mostra: sem contour, só o botão que o
//! cria; com contour, os controles. É o que impede a swatch de cor de existir sem ter para onde
//! escrever — um controle de cor sem alvo é o knob morto na sua forma mais cara, porque abre um
//! picker inteiro para descartar o resultado.

use std::cell::Cell;

thread_local! {
    static CAN_ADD: Cell<bool> = const { Cell::new(false) };
    static PRESENT: Cell<bool> = const { Cell::new(false) };
    static STEPS: Cell<f64> = const { Cell::new(crate::contour_params::CONTOUR_STEPS_DEFAULT) };
    /// O offset por passo em FRAÇÃO do tamanho da forma (o slider fala fração; o componente
    /// guarda mundo — a conversão é da shell, com a mesma escala que o Expand usa).
    static D_FRAC: Cell<f64> = const { Cell::new(0.0) };
    static ACCEL: Cell<f64> = const { Cell::new(1.0) };
    static JOIN: Cell<u8> = const { Cell::new(1) };
    static SIDE: Cell<u8> = const { Cell::new(0) };
    /// A cor do ÚLTIMO anel, em sRGB. É o que a swatch mostra; a de PARTIDA é a da forma e não
    /// é autorada (nem publicada): mostrá-la seria um segundo controle para uma cor que o
    /// artista já escolheu no Fill.
    static TO: Cell<[u8; 4]> = const { Cell::new([255, 255, 255, 255]) };
}

/// Publica se a seleção corrente permite **criar** um contour (há forma selecionada e ela ainda
/// não tem um).
pub fn set_current_contour_can_add(v: bool) {
    CAN_ADD.with(|c| c.set(v));
}

pub(crate) fn can_add() -> bool {
    CAN_ADD.with(Cell::get)
}

/// Publica o contour da seleção: se existe, e com que valores.
#[allow(clippy::too_many_arguments)]
pub fn set_current_contour(
    present: bool,
    steps: f64,
    d_frac: f64,
    accel: f64,
    join: u8,
    side: u8,
    to: [u8; 4],
) {
    PRESENT.with(|c| c.set(present));
    STEPS.with(|c| c.set(steps));
    D_FRAC.with(|c| c.set(d_frac));
    ACCEL.with(|c| c.set(accel));
    JOIN.with(|c| c.set(join));
    SIDE.with(|c| c.set(side));
    TO.with(|c| c.set(to));
}

pub(crate) fn present() -> bool {
    PRESENT.with(Cell::get)
}

pub(crate) fn steps() -> f64 {
    STEPS.with(Cell::get)
}

pub(crate) fn d_frac() -> f64 {
    D_FRAC.with(Cell::get)
}

pub(crate) fn accel() -> f64 {
    ACCEL.with(Cell::get)
}

pub(crate) fn join() -> u8 {
    JOIN.with(Cell::get)
}

pub(crate) fn side() -> u8 {
    SIDE.with(Cell::get)
}

pub(crate) fn to() -> [u8; 4] {
    TO.with(Cell::get)
}
