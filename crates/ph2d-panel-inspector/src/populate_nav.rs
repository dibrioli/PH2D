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
pub(crate) const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 8] = [
    (ids::INSP_NAV_HALF_W, 10.0, 0.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros — ver o cabeçalho
    (ids::INSP_NAV_HALF_H, 10.0, 0.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros — ver o cabeçalho
    (ids::INSP_NAV_TARGET_X, 0.0, -1_000.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros, mundo
    (ids::INSP_NAV_TARGET_Y, 0.0, -1_000.0, 1_000.0, 0.1), // LITERAL-PX-OK: metros, mundo
    (ids::INSP_NAV_RADIUS, 0.0, 0.0, 50.0, 0.05),    // LITERAL-PX-OK: metros; 0 = derivado
    (ids::INSP_NAV_ARRIVE, 0.1, 0.0, 50.0, 0.05),    // LITERAL-PX-OK: metros
    (ids::INSP_NAV_REPATH, 0.5, 0.0, 50.0, 0.05),    // LITERAL-PX-OK: metros
    (ids::INSP_NAV_STUCK, 1.0, 0.0, 60.0, 0.1),      // LITERAL-PX-OK: segundos; 0 desliga
];

pub(crate) fn populate_nav(store: &mut WidgetStore) {
    for id in [ids::INSP_NAV_ACTIVE, ids::INSP_NAV_AVOIDANCE] {
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
