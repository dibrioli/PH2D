//! O registro dos widgets — percorre a MESMA lista que o `paint`.
//!
//! ⚠️ Um widget que o `paint` põe no índice de hit e que ninguém regista tem
//! `is_focusable() == false`, e o clique dele é descartado **em silêncio**: sem erro de
//! compilação, sem warning, só um controlo que não faz nada. É a classe que o
//! `architecture_panel_wiring_parity` existe para pegar, e derivar esta lista da tabela que o
//! `paint` percorre é o que faz dela algo que ninguém pode esquecer.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};
use ph2d_tokens::{ColorToken, NumToken};

/// O passo de arrasto do chip de px.
///
/// ⚠️ **Não é decoração.** Sem uma faixa registada o chip deriva o passo do texto do buffer e
/// varre ~50 unidades por PIXEL — um pixel de arrasto bate no teto e o chip vira um interruptor
/// min↔max, com a digitação a continuar a funcionar (que é porque esta classe de bug sobrevive a
/// revisão). O `0.5` é o degrau mais fino que a escala de fábrica usa: `stroke.hairline` vale
/// exactamente 0.5 px, e um passo maior tornaria o menor token da família inalcançável no arrasto.
const PX_STEP: f64 = 0.5;

/// O teto da FAIXA DE ARRASTO — e ele **não é** o teto do valor.
///
/// A porta (`set_num_override`) recusa o que não é um comprimento e não impõe máximo nenhum
/// (o porquê está no `ph2d_tokens::num`); isto aqui é a régua de um GESTO: quanto um arrasto de
/// ponta a ponta atravessa. Sai do maior valor de fábrica da família — `radius.full` = 999 —,
/// porque uma régua que não alcançasse a fábrica tornaria um token inautorável pelo arrasto.
/// Digitar continua a alcançar qualquer número que a porta aceite.
const PX_DRAG_MAX: f64 = 1000.0;

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub fn populate(store: &mut WidgetStore) {
    button(store, ids::TOKENS_CLOSE);
    button(store, ids::TOKENS_RESET_ALL);
    for row in 0..ColorToken::ALL.len() {
        // ⚠️ A swatch é alvo de PICKER, não botão: registá-la como botão faria o clique acender o
        // widget e **nunca abrir o picker** — a cor ficaria ineditável com todos os gates verdes.
        store.register_picker_swatch(ids::tokens_swatch_id(row));
        button(store, ids::tokens_reset_id(row));
        // O elo: um botao por linha, vivo em TODAS elas (qualquer token pode seguir qualquer
        // outro). Sem o registro ele seria pintado, hit-registrado e MORTO sob o rato.
        button(store, ids::tokens_link_id(row));
    }

    // A família NUMÉRICA (plano UI/UX W4c.1) — a SEGUNDA lista, registada pelo mesmo laço e pelo
    // mesmo motivo. ⚠️ Ela percorre `NumToken::ALL`, o mesmo intervalo do `paint` e do `event`.
    for row in 0..NumToken::ALL.len() {
        // ⚠️ Um `NumberInput`, nunca um botão: registá-lo como botão faria o clique acender o
        // widget e **nunca abrir o campo** — o número ficaria inedidável com todos os gates verdes,
        // que é exactamente a cicatriz que a swatch de cor já pagou.
        store.register(
            ids::tokens_num_chip_id(row),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.set_number_range(ids::tokens_num_chip_id(row), 0.0, PX_DRAG_MAX, PX_STEP);
        button(store, ids::tokens_num_reset_id(row));
        button(store, ids::tokens_num_link_id(row));
    }
}
