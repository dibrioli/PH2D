//! **O registo dos widgets da secção RAY SENSOR** (suplente #21).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
//! o pagou sete vezes, e a última foi nesta linha em 19/09.*
//!
//! # ⭐⭐⭐ O tecto do ALCANCE foi MEDIDO, e ele NÃO é de máquina (§0.0)
//!
//! A sonda [`mede_o_que_a_composicao_ja_da_ao_raio`] varreu o alcance sobre uma cena de **2 000**
//! corpos, `10 000` raios por célula, em `--release`:
//!
//! | alcance | raio que ACERTA | raio que ERRA |
//! |---|---|---|
//! | `1 m` | `0,061 µs` | `0,011 µs` |
//! | `100 m` | `0,122 µs` | `0,011 µs` |
//! | `1 000 m` | `0,084 µs` | `0,011 µs` |
//! | `10 000 m` | `0,083 µs` | `0,011 µs` |
//! | `100 000 m` | `0,083 µs` | `0,011 µs` |
//!
//! ⇒ **o custo é do que o raio ATRAVESSA, nunca do número que se lhe escreve** — o `max_toi` do
//! rapier poda a descida no BVH, e um raio que varre o vazio custa o mesmo a `1` e a `100 000`.
//!
//! ⚠️ **Logo o tecto que este ficheiro escreve é de PRODUTO, e diz-se assim:** `1 000 m` é a escala
//! de uma cena grande deste app, e escrever mais do que isso descreve uma cena que não existe. ⛔ Um
//! número «por segurança» aqui seria um palpite à espera de um smoke — e a medição diz que não há
//! de que o proteger.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do [`RaySensor::default()`]**, e não zeros: um raio que nasce
/// com a direcção a `(0, 0)` **não é um raio** (a porta do motor devolve `None`), e lê-se
/// exactamente como um campo partido. ⭐ O default olha para BAIXO — o sensor de chão é o mais usado
/// dos cinco consumidores que o motor já tinha.
///
/// ⚠️ **A DIRECÇÃO vai de `−1` a `1` porque é uma DIRECÇÃO**: a porta normaliza-a, logo a magnitude
/// não significa nada, e uma faixa maior ofereceria um número que o motor deita fora. ⛔ Ela **não**
/// é prendida a `≠ 0`: nulo é um estado que o artista atravessa ao escrever, e a secção diz-lho na
/// queixa em vez de lhe impedir a escrita.
///
/// [`RaySensor::default()`]: https://github.com/dibrioli/PH2D/blob/main/crates/ph2d-physics-ecs/src/components/ray_sensor.rs
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 6] = [
    (ids::INSP_RAY_ORIGIN_X, 0.0, -50.0, 50.0, 0.1), // LITERAL-PX-OK: metros, locais
    (ids::INSP_RAY_ORIGIN_Y, 0.0, -50.0, 50.0, 0.1), // LITERAL-PX-OK: metros, locais
    (ids::INSP_RAY_DIR_X, 0.0, -1.0, 1.0, 0.1),      // LITERAL-PX-OK: direcção
    (ids::INSP_RAY_DIR_Y, -1.0, -1.0, 1.0, 0.1),     // LITERAL-PX-OK: direcção
    (ids::INSP_RAY_REACH, 1.0, 0.0, 1_000.0, 0.1),   // LITERAL-PX-OK: metros — ver o cabeçalho
    (ids::INSP_RAY_LAYER, 0.0, 0.0, 31.0, 1.0),      // LITERAL-PX-OK: índice de camada
];

pub(crate) fn populate_ray(store: &mut WidgetStore) {
    // ⚠️ **Os dois NOMES são TEXTO**, e vazio = calado — a regra do `SignalOnHit`, palavra por
    // palavra. Registá-los é o que os torna clicáveis; sem isto eles pintam e morrem sob o dedo.
    for id in [ids::INSP_RAY_ON_ENTER, ids::INSP_RAY_ON_EXIT] {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
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
