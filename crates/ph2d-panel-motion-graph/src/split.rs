//! ⭐⭐⭐ **A LEI DO DIVISOR** — a fracção que o arrasto do topo do canvas de nós escreve.
//!
//! ⚠️ **Cortado do `interact.rs` em 2026-08-31 pelo tecto de LOC (622/600), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro **despacha** gestos (qual máquina recebe qual `HitKind`) e
//! isto é **uma lei**, a única deste painel que tem de ser a inversa exacta de código noutra crate.
//! É também a única cujo gate atravessa as duas — ver
//! `tests/the_divider_lands_where_the_pointer_is.rs`.

use ph2d_editor_core::zones::Rect;

/// ⭐⭐⭐ **A fracção que o ponteiro pede ao divisor** — a lei do arrasto, pura e gateável.
///
/// `band` é a região que o divisor parte ([`ph2d_editor_core::screens::layout::HeroLayout::split_band`]),
/// `rect` o sub-rectângulo do grafo (só para saber a ORIENTAÇÃO) e `pointer` o cursor.
///
/// ⛔⛔ **O denominador é a BANDA, e nunca `center + rect`.** Reconstruir a banda somando as duas
/// metades era verdade até a timeline docar DENTRO do split e comer o fundo do grafo — a partir
/// daí o arrasto media contra `chrome_h − altura_da_timeline` e o layout aplicava sobre `chrome_h`.
/// Isso é um **offset** (~1,32 no alvo de referência) e um **tremor**, porque a altura da timeline
/// é ela própria clampada pela do grafo: mover o divisor mudava o denominador, que movia o divisor.
///
/// ⚠️ Sem clamp — quem o faz é o shell (`CenterSplit::clamp_t`), e uma segunda cerca aqui
/// esconderia o extremo de quem a lê.
#[must_use]
pub fn split_fraction(band: Rect, rect: Rect, pointer: (f32, f32)) -> f32 {
    let vertical = rect.x > band.x + 0.5;
    if vertical {
        (pointer.0 - band.x) / band.w.max(1.0)
    } else {
        (pointer.1 - band.y) / band.h.max(1.0)
    }
}
