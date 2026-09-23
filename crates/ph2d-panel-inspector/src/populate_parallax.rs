//! **O registo dos widgets da secção PARALLAX** (plano 24, W7).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
//! o pagou sete vezes, e a última foi nesta linha em 19/09.*
//!
//! # ⭐⭐⭐ As quatro faixas, e o RECURSO de cada uma
//!
//! - **O factor vai de `−1` a `2`.** ⛔ Não é `0..1`: `k > 1` é uma camada **à frente** do plano do
//!   mundo (o primeiro plano de um jogo de plataformas, que passa mais depressa que o chão) e `k`
//!   negativo é a camada a andar ao contrário — os dois são leis do modelo, e a lei não os recusa.
//!   *O tecto `2` é de PRODUTO e diz-se assim: acima disso o primeiro plano passa tão depressa que
//!   deixa de se ler como cenário.*
//! - **O ladrilho vai de `0` a `1 000` m.** ⚠️ `0` é a AUSÊNCIA de repetição naquele eixo e tem de
//!   caber na faixa (é a convenção do componente); negativo é recusado pela própria lei
//!   ([`ph2d_ecs::envolve_eixo`] deixa o eixo intocado), e oferecê-lo seria um controlo que não faz
//!   nada.
//! - **A deriva vai de `−50` a `50` m/s.** É a escala de velocidade desta casa (o mover de cima
//!   anda a `6`), e o sinal é o sentido.
//! - **A cerca vai de `−10 000` a `10 000` m** — a mesma faixa do `CameraLimits`, porque é a mesma
//!   grandeza: uma região do mundo em metros. *Duas faixas para a mesma grandeza divergiriam.*

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do componente**: o factor nasce em `1` (o neutro, que é o que
/// faz anexar a paralaxe não mudar um pixel) e os outros seis em `0`. ⛔ Semeá-los a zero faria a
/// primeira pintura mostrar uma camada presa ao ecrã que o mundo não tem.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 10] = [
    (ids::INSP_PARALLAX_K_X, 1.0, -1.0, 2.0, 0.05), // LITERAL-PX-OK: fracção adimensional
    (ids::INSP_PARALLAX_K_Y, 1.0, -1.0, 2.0, 0.05), // LITERAL-PX-OK: fracção adimensional
    (ids::INSP_PARALLAX_TILE_X, 0.0, 0.0, 1_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_PARALLAX_TILE_Y, 0.0, 0.0, 1_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_PARALLAX_VEL_X, 0.0, -50.0, 50.0, 0.05), // LITERAL-PX-OK: metros por segundo
    (ids::INSP_PARALLAX_VEL_Y, 0.0, -50.0, 50.0, 0.05), // LITERAL-PX-OK: metros por segundo
    (ids::INSP_PARALLAX_MIN_X, 0.0, -10_000.0, 10_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_PARALLAX_MIN_Y, 0.0, -10_000.0, 10_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_PARALLAX_MAX_X, 0.0, -10_000.0, 10_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_PARALLAX_MAX_Y, 0.0, -10_000.0, 10_000.0, 0.5), // LITERAL-PX-OK: metros
];

pub(crate) fn populate_parallax(store: &mut WidgetStore) {
    for (id, value, lo, hi, step) in NUMEROS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, lo, hi, step);
    }
}
