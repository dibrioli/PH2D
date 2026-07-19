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
}
