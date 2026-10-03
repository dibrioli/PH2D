//! **O registo dos widgets das secções NAV REGION e NAV AGENT** (plano 30, W4).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ Os tectos são de PRODUTO, e dizem-no (§0.0)
//!
//! A malha andável custa pelos OBSTÁCULOS que ela recorta, não pelo tamanho do rectângulo (o
//! rectângulo é só a fronteira exterior, quatro vértices) — ⇒ `1 000 m` de meia-largura é a escala
//! de uma cena grande deste app, a mesma régua que o alcance do raio já escreve, e não um limite de
//! máquina. Os raios e as distâncias param em `50 m` (um corpo de cem metros já não passa porta
//! nenhuma de uma cena), e o *Stuck After* em `60 s` (preso há um minuto é preso).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

/// `(id, valor de partida, mínimo, máximo, passo)` — os de partida são os do `NavRegion::default()`
/// e do `NavAgent::default()`, e não zeros (um raio de chegada a `0` lê-se como campo partido).
pub(crate) const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 10] = [
    (ids::INSP_NAV_HALF_W, 10.0, 0.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros — ver o cabeçalho
    (ids::INSP_NAV_HALF_H, 10.0, 0.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros — ver o cabeçalho
    (ids::INSP_NAV_TARGET_X, 0.0, -1_000.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros, mundo
    (ids::INSP_NAV_TARGET_Y, 0.0, -1_000.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros, mundo
    (ids::INSP_NAV_RADIUS, 0.0, 0.0, 50.0, 0.05),    // LITERAL-PX-OK: metros; 0 = derivado
    (ids::INSP_NAV_ARRIVE, 0.1, 0.0, 50.0, 0.05),    // LITERAL-PX-OK: metros
    (ids::INSP_NAV_REPATH, 0.5, 0.0, 50.0, 0.05),    // LITERAL-PX-OK: metros
    (ids::INSP_NAV_STUCK, 1.0, 0.0, 60.0, 0.1),      // LITERAL-PX-OK: segundos; 0 desliga
    // (W7) O custo da ÁREA: piso = a lei (`> 0`); o tecto é só o do stepper — o arrasto tem TAXA
    // (abaixo), logo não pára nele. `3` = o `NavCostArea::default()`.
    (ids::INSP_NAV_AREA_COST, 3.0, AREA_COST_MIN, 1_000.0, 0.05), // LITERAL-PX-OK: × chão
    // (W7) O custo do ATALHO, em metros de chão — a mesma régua de cena das meias-extensões.
    (ids::INSP_NAV_LINK_COST, 0.0, 0.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros
];

/// O piso do custo da área — o MESMO número que o dreno aplica.
const AREA_COST_MIN: f64 = ph2d_editor_core::nav_edits::NAV_AREA_COST_MIN as f64;

/// (W7) As caixas de custo arrastam-se por TAXA (unidades por pixel), não pela proporção do
/// intervalo: um intervalo que não acaba de facto não tem proporção (a receita do
/// `set_number_drag_rate`).
const TAXAS: [(ph2d_a11y::NodeId, f64); 2] = [
    (ids::INSP_NAV_AREA_COST, 0.01), // LITERAL-PX-OK: × chão por pixel
    (ids::INSP_NAV_LINK_COST, 0.05), // LITERAL-PX-OK: metros por pixel
];

pub(crate) fn populate_nav(store: &mut WidgetStore) {
    for id in [
        ids::INSP_NAV_ACTIVE,
        ids::INSP_NAV_AVOIDANCE,
        ids::INSP_NAV_AVOID_HARM,
        ids::INSP_NAV_AREA_FORBIDDEN,
        ids::INSP_NAV_LINK_TWO_WAY,
        ids::INSP_NAV_LINK_TELEPORT,
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Checked,
            },
        );
    }
    // ⚠️ **As camadas e os modos são BOTÕES** — sem registo eles pintam e morrem sob o dedo.
    for id in ids::INSP_NAV_LAYERS
        .into_iter()
        .chain(ids::INSP_NAV_TARGET_MODE)
        .chain(ids::INSP_NAV_TAG_OPT)
    {
        store.register(
            id,
            InteractiveState::Button {
                state: Default::default(),
            },
        );
    }
    for id in [
        ids::INSP_NAV_TARGET_NAME,
        ids::INSP_NAV_ON_ARRIVED,
        ids::INSP_NAV_ON_NO_PATH,
        ids::INSP_NAV_ON_STUCK,
        ids::INSP_NAV_LINK_TO,
        ids::INSP_NAV_LINK_ON_CROSSED,
    ] {
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
    // (W6) O chip da tag é um `Dropdown`; as opções (acima) são as linhas do popover.
    store.register(
        ids::INSP_NAV_TAG_PICK,
        InteractiveState::Dropdown {
            state: ph2d_editor_core::widget::DropdownState::Normal,
            open: false,
            selected_index: None,
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
    for (id, taxa) in TAXAS {
        store.set_number_drag_rate(id, taxa);
    }
}
