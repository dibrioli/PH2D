//! **O registo dos widgets das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ As faixas, e de que recurso cada uma é
//!
//! - **pontos** `0..100 000` — a vida de um chefe de jogo de acção passa dos milhares; o tecto é a
//!   escala de um número que o artista ainda lê num campo, não da lei (que é `f64`);
//! - **segundos** `0..600` — dez minutos de invencibilidade ou de escudo já é um modo de jogo;
//! - **fracções** `0..1` — armadura percentual e esquiva são probabilidades/fracções, e a lei prende-as
//!   na porta: acima de `1` a armadura CURARIA;
//! - **semente** `0..2^32` — um `f64` guarda-a exacta, e é a escala de uma semente escrita à mão.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do `Health::default()`/`Damage::default()`**, e não zeros: uma
/// vida que nasce a `0` lê-se como um campo partido.
pub(crate) const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 21] = [
    (ids::INSP_VIDA_MAX, 100.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: pontos
    (ids::INSP_VIDA_START, 100.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: pontos
    (ids::INSP_VIDA_INVINCIBLE, 0.0, 0.0, 600.0, 0.05), // LITERAL-PX-OK: segundos
    (ids::INSP_VIDA_REGEN, 0.0, 0.0, 100_000.0, 0.5), // LITERAL-PX-OK: pontos/s
    (ids::INSP_VIDA_REGEN_DELAY, 0.0, 0.0, 600.0, 0.05), // LITERAL-PX-OK: segundos
    (ids::INSP_VIDA_SHIELD_START, 0.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: pontos
    (ids::INSP_VIDA_SHIELD_MAX, 0.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: pontos
    (ids::INSP_VIDA_SHIELD_DURATION, 5.0, 0.0, 600.0, 0.1), // LITERAL-PX-OK: segundos
    (ids::INSP_VIDA_SHIELD_REGEN, 0.0, 0.0, 100_000.0, 0.5), // LITERAL-PX-OK: pontos/s
    (ids::INSP_VIDA_SHIELD_REGEN_DELAY, 0.0, 0.0, 600.0, 0.05), // LITERAL-PX-OK: segundos
    (ids::INSP_VIDA_ARMOR, 0.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: pontos
    (ids::INSP_VIDA_ARMOR_PCT, 0.0, 0.0, 1.0, 0.05),  // LITERAL-PX-OK: fracção
    (ids::INSP_VIDA_DODGE, 0.0, 0.0, 1.0, 0.05),      // LITERAL-PX-OK: fracção
    (ids::INSP_VIDA_SEED, 0.0, 0.0, 4_294_967_295.0, 1.0), // LITERAL-PX-OK: semente
    (ids::INSP_DANO_AMOUNT, 10.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: pontos
    // ⭐ A BARRA (plano 28, W4) — metros e segundos. ⚠️ Os deslocamentos descem abaixo de zero: uma
    // barra por baixo dos pés é uma escolha legítima. Os tectos são a escala de um campo que o
    // artista lê, como os da vida — a lei aceita qualquer número finito.
    (ids::INSP_BARRA_WIDTH, 1.0, 0.0, 1_000.0, 0.05), // LITERAL-PX-OK: metros
    (ids::INSP_BARRA_HEIGHT, 0.14, 0.0, 1_000.0, 0.01), // LITERAL-PX-OK: metros
    (ids::INSP_BARRA_OFFSET_X, 0.0, -1_000.0, 1_000.0, 0.05), // LITERAL-PX-OK: metros
    (ids::INSP_BARRA_OFFSET_Y, 0.75, -1_000.0, 1_000.0, 0.05), // LITERAL-PX-OK: metros
    (ids::INSP_BARRA_TRAIL_DELAY, 0.4, 0.0, 600.0, 0.05), // LITERAL-PX-OK: segundos
    (ids::INSP_BARRA_TRAIL_SPEED, 1.0, 0.0, 1_000.0, 0.1), // LITERAL-PX-OK: barras/s
];

/// As caixas, com o valor de partida de cada uma.
const CAIXAS: [(ph2d_a11y::NodeId, bool); 7] = [
    (ids::INSP_VIDA_OVERHEAL, false),
    (ids::INSP_VIDA_SHIELD_BLOCKS, false),
    (ids::INSP_DANO_PER_SECOND, false),
    (ids::INSP_DANO_IGNORES_SHIELD, false),
    (ids::INSP_DANO_IGNORES_ARMOR, false),
    (ids::INSP_DANO_VANISH, false),
    (ids::INSP_BARRA_HIDE_FULL, false),
];

/// Os campos de texto — a equipa e os três sinais da vida, e a equipa do dano.
pub(crate) const TEXTOS: [ph2d_a11y::NodeId; 6] = [
    ids::INSP_VIDA_TEAM,
    ids::INSP_VIDA_ON_DAMAGE,
    ids::INSP_VIDA_ON_HEAL,
    ids::INSP_VIDA_ON_DEATH,
    ids::INSP_DANO_TEAM,
    ids::INSP_BARRA_TARGET,
];

pub(crate) fn populate_vida(store: &mut WidgetStore) {
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
    for (id, on) in CAIXAS {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            },
        );
    }
    // ⭐⭐ **As três AMOSTRAS de cor da barra** — `Plain` mais `register_picker_swatch`: o despacho
    // abre o selector ele próprio e NÃO emite evento nenhum (a lei das amostras das propriedades de
    // script), logo não há braço de clique a escrever no `event_vida`. ⚠️ Sem o `Plain` o clique
    // morre no `is_focusable`; sem o `register_picker_swatch` ele chega e o selector não abre.
    for id in ids::INSP_BARRA_CORES {
        store.register(id, InteractiveState::Plain);
        store.register_picker_swatch(id);
    }
    for id in TEXTOS {
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
}
