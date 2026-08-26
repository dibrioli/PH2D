//! Registro dos pills de MODO do painel Vector + o Convert to Curves — módulo irmão de
//! `populate` (teto de 600 LOC). Registrar aqui é o que torna um pill CLICÁVEL: pintar e dar
//! hit-rect não basta, e um pill fora desta lista pinta e fica MORTO (o clique não vira
//! evento) — foi o bug do smoke do Line/Arc (Enio 2026-07-09).

use super::button;
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Registra os pills de modo (Select … Fillet / Chamfer) + o Convert to Curves. Chamado uma
/// vez pelo `populate`. O **Shape** é o 5º pill: sem ele, escolher uma forma punha a tool em
/// `DrawMode::Shape` e a fileira de modos ficava TODA apagada.
pub(super) fn mode_buttons(store: &mut WidgetStore) {
    button(store, ids::VECTOR_CONVERT_TO_CURVES);
    button(store, ids::VECTOR_MODE_SELECT);
    button(store, ids::VECTOR_MODE_NODE);
    button(store, ids::VECTOR_MODE_PEN);
    // **Lápis** — o modo de mão livre. Sem esta linha o chip PINTA e fica morto sob o mouse
    // (o `register` é o que o torna hittable/focável), que é o defeito exato que a lista de
    // pills já pagou duas vezes.
    button(store, ids::VECTOR_MODE_PENCIL);
    button(store, ids::VECTOR_MODE_SHAPE);
    button(store, ids::VECTOR_MODE_TEXT);
    // Connect (a linha que gruda em duas formas) + Build (Shape Builder).
    button(store, ids::VECTOR_MODE_CONNECT);
    button(store, ids::VECTOR_MODE_BUILD);
    // Pick Shapes (Blend): o botão mora na seção BLEND, mas registra-se aqui.
    button(store, ids::VECTOR_MODE_PICKBLEND);
    // Fillet / Chamfer: arredondar / chanfrar quina por clicar-e-arrastar.
    button(store, ids::VECTOR_MODE_FILLET);
    button(store, ids::VECTOR_MODE_CHAMFER);
    button(store, ids::VECTOR_MODE_WIDTH);
    // **Corte** (W4) — a ferramenta de corte. Ela foi esquecida aqui na 1ª escrita da wave (o
    // `populate` das SEÇÕES recebeu os botões novos, este não), e o preço foi exactamente o que
    // este doc-comment prometia: o pill pintava, acendia sob o mouse e o clique **nunca virava
    // evento**. Reportado no smoke: *"botão não funciona"*.
    button(store, ids::VECTOR_MODE_CUT);
    // **A FORMA do marquee** — o par colado na fileira TOOL. Sem estas duas linhas os chips
    // pintam, acendem sob o rato e o Click morre: a falha que o `wiring_parity` existe para pegar.
    button(store, ids::VECTOR_MARQUEE_BOX);
    button(store, ids::VECTOR_MARQUEE_LASSO);
    // **Moldura** (plano UI/UX W0) — o 14º pill. Sem esta linha ele pinta, acende sob o mouse e o
    // clique NUNCA vira evento; é o defeito que este arquivo já pagou três vezes.
    button(store, ids::VECTOR_MODE_FRAME);
}

/// **REGISTA os widgets das SETAS do Morph** (plano 32 W4) — o pool inteiro, de antemão.
///
/// ⚠️ **De antemão e não por linha viva**, ao contrário da janela do Input Map: aqui o teto é o
/// próprio `MAX_MORPH_STATES`, e um pool fixo é o que o resto deste painel faz. ⛔ Sem isto, cada
/// controlo nasce **morto sob o ponteiro** — o defeito que esta linha já pagou três vezes.
pub(crate) fn populate_morph_arrows(store: &mut WidgetStore) {
    // O botão que CRIA o conjunto: ele é o único controlo da seção quando não há máquina nenhuma.
    button(store, ids::VECTOR_MORPH_STATES_MAKE);
    // ⭐⭐ O interruptor da PRÉ-VISUALIZAÇÃO — o modo em que o teclado é da máquina.
    button(store, ids::VECTOR_MORPH_PREVIEW);
    // ⭐⭐ DESFAZER TUDO — o inverso exacto do botão que faz o conjunto.
    button(store, ids::VECTOR_MORPH_DISSOLVE);
    for row in 0..ids::MAX_MORPH_STATES {
        // ⭐ **O BOTÃO que abre a lista dos eventos** (W11b) — **Dropdown** no store, **botão** na
        // tela. Registá-lo como `Button` faria o clique acender e **nunca abrir lista nenhuma**: é
        // a cicatriz da swatch do painel de tokens e a dos dois números do Input Map.
        store.register(
            ids::morph_shape_key_button_id(row),
            ph2d_editor_core::interaction::InteractiveState::Dropdown {
                state: ph2d_editor_core::widget::DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        // Os DOIS verbos da linha: **Play** (viaja até esta forma) e **Desconectar** (tira-a).
        button(store, ids::morph_shape_play_id(row));
        button(store, ids::morph_shape_disconnect_id(row));
        for a in 0..ids::MAX_MORPH_ACTIONS {
            button(store, ids::morph_shape_key_option_id(row, a));
        }
    }
}
