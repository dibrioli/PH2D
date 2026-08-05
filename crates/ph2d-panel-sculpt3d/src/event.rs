//! Painel → shell. Todo braço de knob é derivado de [`crate::rows`], então uma
//! row que existe é uma row que despacha.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal, seam_reset_button};
use ph2d_sculpt3d::{Falloff, Verb};

use crate::rows;
use crate::state::{self, Sculpt3dIntent};

/// A posição de um id num array de opções — o inverso exato da ordem que o
/// pintor usa, e por isso o chip que acende e o valor que pousa não podem
/// discordar.
fn index_of(group: &[ph2d_a11y::NodeId], id: ph2d_a11y::NodeId) -> Option<usize> {
    group.iter().position(|&g| g == id)
}

/// Todo comando de um só toque: o id e o que ele significa. Uma tabela, e não uma
/// cascata de `if id == …`, porque o `event` e o `populate` têm de concordar sobre
/// a LISTA e uma cascata é o formato que apodrece calado.
const COMMANDS: &[(ph2d_a11y::NodeId, Sculpt3dIntent)] = &[
    (ids::SCULPT3D_DYNTOPO, Sculpt3dIntent::ToggleDyntopo),
    (ids::SCULPT3D_LEVEL_DOWN, Sculpt3dIntent::ChangeLevel(false)),
    (ids::SCULPT3D_LEVEL_UP, Sculpt3dIntent::ChangeLevel(true)),
    (ids::SCULPT3D_SUBDIVIDE, Sculpt3dIntent::Subdivide),
    (ids::SCULPT3D_REVERSE, Sculpt3dIntent::ReverseLevel),
    (ids::SCULPT3D_REMESH, Sculpt3dIntent::Remesh),
    (ids::SCULPT3D_CLOSE_HOLES, Sculpt3dIntent::CloseHoles),
    (ids::SCULPT3D_DUPLICATE, Sculpt3dIntent::Duplicate),
    (ids::SCULPT3D_DELETE, Sculpt3dIntent::Delete),
    (ids::SCULPT3D_ISOLATE, Sculpt3dIntent::ToggleIsolate),
    (ids::SCULPT3D_MERGE, Sculpt3dIntent::Merge),
];

/// As quatro primitivas e as quatro operações de máscara, na ordem em que o
/// painel as lista.
const ADD_INTENTS: [Sculpt3dIntent; 4] = [
    Sculpt3dIntent::AddSphere,
    Sculpt3dIntent::AddCube,
    Sculpt3dIntent::AddCylinder,
    Sculpt3dIntent::AddTorus,
];
const MASK_INTENTS: [Sculpt3dIntent; 4] = [
    Sculpt3dIntent::MaskClear,
    Sculpt3dIntent::MaskInvert,
    Sculpt3dIntent::MaskBlur,
    Sculpt3dIntent::MaskSharpen,
];

pub(crate) fn apply_event(
    _state: &mut crate::state::Sculpt3dPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Sem cena não há o que editar — e um intent emitido agora seria aplicado
    // ao primeiro barro que aparecesse, o que é pior que não fazer nada.
    let Some(snapshot) = state::current() else {
        return EventOutcome::from_bool(false);
    };
    let consumed = match ev {
        // Uma pista moveu: lê a pista que o arrasto deixou, transforma no valor
        // desta row, e emite o estado autorado INTEIRO com aquele campo trocado.
        WidgetEvent::ValueChanged(id) => {
            if let Some(row) = rows::row_for(id) {
                if id == row.chip {
                    // A edição do chip já foi espelhada na pista ligada a ele,
                    // que disparou o próprio `ValueChanged` e foi tratada lá.
                    // Engolir, ou uma edição notifica duas vezes.
                    true
                } else {
                    let track = host.store().slider(id).map_or(0.5, |(_, v)| v);
                    let mut ui = snapshot.ui;
                    (row.set)(&mut ui, row.value_of(track));
                    state::push_intent(Sculpt3dIntent::SetUi(ui));
                    true
                }
            } else {
                false
            }
        }
        // A ferramenta. A lista É `Verb::ALL`, então a ordem dos chips e a do
        // modelo não podem derivar.
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_VERB, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_VERB, id).expect("guard casou");
            let mut ui = snapshot.ui;
            let verb = Verb::ALL[i];
            // ⚠️ **Arma o default do verbo, e só se o artista ainda não mexeu.**
            // A MESMA lei do teclado (`sculpt3d_keys.rs`) e o mesmo precedente do
            // `arm_inflate_defaults` do Painter: a máscara nasce em força cheia,
            // senão ela protege pela metade e o barro se move por baixo — e
            // nenhum verbo pode APAGAR uma escolha deliberada.
            if (ui.brush.strength - ui.brush.verb.default_strength()).abs() < 1e-6 {
                ui.brush.strength = verb.default_strength();
            }
            ui.brush.verb = verb;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_FALLOFF, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_FALLOFF, id).expect("guard casou");
            let mut ui = snapshot.ui;
            ui.brush.falloff = Falloff::ALL[i];
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_DETAIL, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_DETAIL, id).expect("guard casou");
            let mut ui = snapshot.ui;
            ui.detail = u8::try_from(i).unwrap_or(0);
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // Os três eixos do espelho. Botões independentes e não um rádio: um
        // segmented é *um de N* por construção, e o ZBrush espelha em dois eixos
        // ao mesmo tempo.
        WidgetEvent::Click(id)
            if id == ids::SCULPT3D_SYM_X
                || id == ids::SCULPT3D_SYM_Y
                || id == ids::SCULPT3D_SYM_Z =>
        {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            let axis = if id == ids::SCULPT3D_SYM_X {
                &mut ui.symmetry.x
            } else if id == ids::SCULPT3D_SYM_Y {
                &mut ui.symmetry.y
            } else {
                &mut ui.symmetry.z
            };
            *axis = !*axis;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_ADD, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_ADD, id).expect("guard casou");
            state::push_intent(ADD_INTENTS[i]);
            true
        }
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_MASK_OP, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_MASK_OP, id).expect("guard casou");
            state::push_intent(MASK_INTENTS[i]);
            true
        }
        WidgetEvent::Click(id) if COMMANDS.iter().any(|(k, _)| *k == id) => {
            seam_reset_button(host, id);
            let intent = COMMANDS
                .iter()
                .find(|(k, _)| *k == id)
                .map(|(_, i)| *i)
                .expect("guard casou");
            state::push_intent(intent);
            true
        }
        // Os cabeçalhos dobram. Estado de VISTA do painel, então nunca vira
        // intent — o shell não tem opinião sobre que seções estão abertas.
        WidgetEvent::Click(id) if is_section_header(id) => {
            seam_reset_button(host, id);
            let collapsed = host.store().is_collapsed(id);
            host.store_mut().set_collapsed(id, !collapsed);
            true
        }
        WidgetEvent::Click(id) if id == ids::SCULPT3D_CLOSE => {
            seam_reset_button(host, id);
            host.set_panel_visible(crate::Sculpt3dPanel::ID, false);
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}

/// As seis seções — as duas com tabela de rows mais as quatro de botões.
fn is_section_header(id: ph2d_a11y::NodeId) -> bool {
    rows::SECTIONS.iter().any(|s| s.id == id)
        || id == ids::SCULPT3D_SEC_TOOL
        || id == ids::SCULPT3D_SEC_SYMMETRY
        || id == ids::SCULPT3D_SEC_TOPOLOGY
        || id == ids::SCULPT3D_SEC_SCENE
}
