//! A costura do painel do ESQUELETO — o registo dos widgets e o que um clique faz.
//!
//! ⚠️ **Registar é o que os torna clicáveis**: pintar + hit-rect não basta — é a classe de bug que
//! já matou botões do vector duas vezes (o mais recente é o bug #29, em que TRÊS rotas morreram ao
//! mesmo tempo com o gate de registo **verde**).
//!
//! ⚠️ **Todos são registados incondicionalmente**, mesmo os que só são PINTADOS com uma forma presa
//! ou um osso em foco: o store é agnóstico de estado, e quem decide se o clique é possível é a
//! PINTURA (sem hit-rect não há `Click`). Registar só o pintado faria o outro nascer **morto sob o
//! dedo** — o defeito dos quatro chips da booleana, que só o gesto real apanhou.
//!
//! ⚠️ **Tudo o que este painel toca mora no MUNDO** (um componente de uma entidade), então **tudo**
//! atravessa para a shell. Um controlo fora do encaminhamento aceita o clique e **não fala com
//! ninguém** — a forma mais cara de nascer morto, porque parece vivo.

use crate::state::SkeletonPanelState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{ButtonState, DropdownState, TextInputState};

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// ⚠️ **Pela porta do MUNDO — sem `set_number_range`.** O comprimento de um osso vive nas unidades
/// do documento, e emprestar-lhe a faixa de outro recurso é o defeito que o `CLAUDE.md` §0.0 nomeia
/// (a v21 pagou-o com a largura de traço a limitar um deslocamento). `Mix` e `Softness` são
/// adimensionais e `Chain` conta ossos: nenhum é medida de desenho.
fn world_number_field(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: "0".to_string(),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    // ⚠️ Cada fileira é registada **pela TABELA que o `paint` percorre** — é a regra que o gate
    // `table_driven_chips_are_registered_too` lê no fonte, e a que impede a próxima fileira de
    // nascer morta sob o dedo.
    for id in crate::ids::VECTOR_BONE_ACTION_IDS {
        button(store, id);
    }
    for id in ids::VECTOR_BONE_VERBS {
        button(store, id);
    }
    for id in ids::VECTOR_BONE_BEND_IDS {
        button(store, id);
    }
    // ⭐ De onde vêm as alças de curvatura — pela mesma tabela, e pela mesma razão.
    for id in ids::VECTOR_BONE_HANDLES_IDS {
        button(store, id);
    }
    // ⭐ A alternativa do DESENHO da pele (report de 2026-09-10) — pela mesma tabela, e foi um gate
    // de costura que a apanhou pintada e **não registada**: morta sob o dedo, sem nada na tela que
    // o diga.
    for id in crate::ids::VECTOR_BONE_DEFORM_IDS {
        button(store, id);
    }
    // ⚠️ **`Dropdown` no store, botão na tela**: é o `InteractiveState::Dropdown` que faz o dispatch
    // genérico alternar o `open` (e fechar o dos outros). Registá-lo como `Button` faria o clique
    // acender e **nunca abrir lista nenhuma**.
    store.register(
        crate::ids::VECTOR_BONE_SMART_CLIP,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    for id in ids::VECTOR_BONE_SMART_CLIP_IDS {
        button(store, id);
    }
    for id in ids::VECTOR_BONE_FIELDS {
        world_number_field(store, id);
    }
}

/// **Este id é deste painel?** — a mesma lista que o `populate` regista e que o `paint` pinta.
fn meu(id: ph2d_a11y::NodeId) -> bool {
    ids::VECTOR_BONE_VERBS.contains(&id)
        || ids::VECTOR_BONE_FIELDS.contains(&id)
        || ids::VECTOR_BONE_BEND_IDS.contains(&id)
        || ids::VECTOR_BONE_HANDLES_IDS.contains(&id)
        || ids::VECTOR_BONE_SMART_CLIP_IDS.contains(&id)
        || crate::ids::VECTOR_BONE_ACTION_IDS.contains(&id)
        || crate::ids::VECTOR_BONE_DEFORM_IDS.contains(&id)
        || id == crate::ids::VECTOR_BONE_SMART_CLIP
}

pub(crate) fn apply_event(
    _state: &mut SkeletonPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) if meu(id) => {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            EventOutcome::Consumed
        }
        // ⚠️ **O COMMIT é que viaja, não cada tecla** — `ValueChanged` chega no fim da edição, e é
        // o valor do store que atravessa (o buffer ainda pode estar a meio de um número).
        WidgetEvent::ValueChanged(id) if ids::VECTOR_BONE_FIELDS.contains(&id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)));
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
