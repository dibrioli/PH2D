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
    button(store, ids::VECTOR_MODE_MORPH_LINK);
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
