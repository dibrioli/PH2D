//! **O registo dos widgets das DUAS secções do ABANÃO** (suplente #25).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique **em silêncio**. *Esta crate
//! já o pagou dez vezes, e as três últimas foram nesta linha.*
//!
//! # ⚠️ As faixas, e de que recurso cada uma é
//!
//! - **amplitude** `0..5 m` — metros de deslocamento da VISTA; a de fábrica é `0,25` (~`2 %` da
//!   altura da vista de fábrica), e `5` é uma vista inteira a saltar: acima disso não há imagem;
//! - **frequência** `1..60 Hz` — o tecto é a **taxa de amostragem do ecrã**: a `60` há **um** quadro
//!   por período e o abanão passa a ser ruído branco, que é onde ele deixa de ser lido como
//!   movimento (o mesmo argumento que o `frequencia: 20` de fábrica declara por escrito);
//! - **decaimento** `0,1..20` — trauma por segundo ⇒ a duração vai de `10 s` a `50 ms`; abaixo de
//!   `0,1` o abanão dura mais de dez segundos, que é uma câmera que nunca assenta;
//! - **semente** `0..999 999` — ⭐ **ela não tem valor CERTO, só tem de ser DIFERENTE**, e o tecto é
//!   a exactidão de um `f64` no campo, não uma propriedade da lei;
//! - **força** `0..1` — é uma **fracção do trauma**, e a lei satura em `1`: deixar o painel produzir
//!   `3` entregaria três pontos do campo com a mesma saída;
//! - **raios** `0..200 m` — a escala de uma cena deste app, a mesma do `lado` do seguidor.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{TextInputState, format_number};

use super::populate::register_button_ids;

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do `CameraShake::default()` e do `ShakeSource::default()`**, e
/// não zeros: um `0` de partida na amplitude faria a secção abrir sobre uma câmera que não treme.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 7] = [
    (ids::INSP_SHAKE_AMPLITUDE, 0.25, 0.0, 5.0, 0.01), // LITERAL-PX-OK: metros de vista
    (ids::INSP_SHAKE_FREQUENCIA, 20.0, 1.0, 60.0, 0.5), // LITERAL-PX-OK: Hz, tecto = a taxa do ecrã
    (ids::INSP_SHAKE_DECAIMENTO, 2.0, 0.1, 20.0, 0.1), // LITERAL-PX-OK: trauma/s
    (ids::INSP_SHAKE_SEMENTE, 24301.0, 0.0, 999_999.0, 1.0), // LITERAL-PX-OK: `0x5EED` em decimal
    (ids::INSP_EMITTER_FORCA, 0.6, 0.0, 1.0, 0.05),    // LITERAL-PX-OK: fracção do trauma
    (ids::INSP_EMITTER_DENTRO, 3.0, 0.0, 200.0, 0.5),  // LITERAL-PX-OK: metros
    (ids::INSP_EMITTER_FORA, 12.0, 0.0, 200.0, 0.5),   // LITERAL-PX-OK: metros
];

pub(crate) fn populate_shake(store: &mut WidgetStore) {
    // ⚠️ **Os chips são BOTÕES** — sem registo eles pintam e morrem sob o dedo.
    register_button_ids(store, &ids::INSP_SHAKE_PERFIL);
    register_button_ids(store, &ids::INSP_SHAKE_EXPOENTE);
    register_button_ids(store, &ids::INSP_EMITTER_DE);
    register_button_ids(store, &ids::INSP_EMITTER_ROW);
    register_button_ids(store, &[ids::INSP_EMITTER_ADD, ids::INSP_EMITTER_REMOVE]);

    // ⚠️ **O nome do sinal é TEXTO** — vazio cala a fonte, e quem o casa é a ponte.
    store.register(
        ids::INSP_EMITTER_ON,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );

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
