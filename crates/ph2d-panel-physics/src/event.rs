//! Panel → shell. Every arm is derived from [`crate::rows`], so a row that
//! exists is a row that dispatches.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal, seam_reset_button};
use ph2d_physics_ecs::{InteractionSettings, PhysicsSettings};

use crate::state::{self, PhysicsIntent};
use crate::{interact, rows};

/// Inverse of `ids::physics_layer_cell_index` — the `(row, col)` a flat cell
/// index names. Derived by walking the triangle rather than stored, so the two
/// directions cannot drift apart.
fn cell_pair(index: usize) -> (usize, usize) {
    let mut i = 0usize;
    let mut remaining = index;
    while remaining > i {
        remaining -= i + 1;
        i += 1;
    }
    (i, remaining)
}

/// ⭐ **Toda escrita do painel sobre o mundo** (F41): a MESMA edição vai ao autorado (o que o save
/// grava) e ao vivo que a linha mostrava ([`state::Toque`]) — uma porta, para que nenhum controlo
/// esqueça a mão escondida pela curva.
/// ⭐ Duas edições no MESMO quadro (o arrasto que pinta a matriz) somam-se: cada uma parte da última
/// ainda na fila, e não do instantâneo — senão a segunda apagava a primeira.
fn editar(f: impl Fn(PhysicsSettings) -> PhysicsSettings) {
    let snap = state::current();
    let (autorado, pedido) =
        state::ultimo_set_settings().map_or((snap.settings, snap.vivo), |(s, t)| (s, t.pedido));
    state::push_intent(PhysicsIntent::SetSettings(
        Box::new(f(autorado)),
        Box::new(state::Toque {
            visto: snap.vivo,
            pedido: f(pedido),
        }),
    ));
}

/// ⭐ **Alterna um par de camadas**: a célula da matriz e o quadro do objecto são esta mesma edição.
/// A matriz MOSTRA o vivo (a curva da Cena, F41): vira o que se vê, e só esse par.
pub fn alterna_par(a: usize, b: usize) {
    let on = !ph2d_physics_ecs::LayerMatrix::from_rows(state::current().vivo.layer_matrix)
        .collides(a, b);
    editar(|s| {
        let mut m = ph2d_physics_ecs::LayerMatrix::from_rows(s.layer_matrix);
        m.set(a, b, on);
        PhysicsSettings {
            layer_matrix: m.rows(),
            ..s
        }
    });
}

/// «Edit matrix…» do Inspector: o modal das camadas, com a linha da camada do objeto acesa.
pub fn abre_a_matriz(_host: &mut dyn PanelHostInternal, layer: u8) {
    crate::modal::abre(Some(usize::from(layer)));
}

/// O nome em edição no modal: grava UM passo (Enter) ou desiste (Esc). Vazio volta a «Layer n».
fn evento_do_nome(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    let camada = |id| crate::ids::PHYSICS_LAYER_NAME.iter().position(|&c| c == id);
    let rotulo = |id| {
        crate::ids::PHYSICS_LAYER_ROW_LABEL
            .iter()
            .chain(&crate::ids::PHYSICS_LAYER_COL_LABEL)
            .position(|&c| c == id)
            .map(|p| p % ph2d_physics_ecs::MAX_LAYERS)
    };
    match ev {
        WidgetEvent::DoubleClick(id) if rotulo(id).is_some() => {
            let n = rotulo(id).expect("guard matched");
            let campo = crate::ids::PHYSICS_LAYER_NAME[n];
            let nome = state::layer_names().0[n].clone();
            let store = host.store_mut();
            if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput {
                text,
                caret,
                ..
            }) = store.get_mut(campo)
            {
                *caret = nome.len();
                nome.clone_into(text);
            }
            ph2d_editor_core::interaction::dispatch::focus_text_field(store, campo);
            crate::modal::comeca_a_renomear(n);
            Some(true)
        }
        // Um clique simples no rótulo é do cartão: não fecha, não pinta.
        WidgetEvent::Click(id) if rotulo(id).is_some() => Some(true),
        WidgetEvent::Submit(id) if camada(id).is_some() => {
            let layer = camada(id).expect("guard matched") as u8;
            let name = host.store().text(id).unwrap_or("").trim().to_string();
            crate::modal::acaba_de_renomear();
            if name != state::layer_names().0[usize::from(layer)] {
                host.bus_mut()
                    .push(ph2d_editor_core::action_bus::EditorAction::LayerRename { layer, name });
            }
            Some(true)
        }
        WidgetEvent::Cancel(id) if camada(id).is_some() => {
            crate::modal::acaba_de_renomear();
            Some(true)
        }
        // O foco saiu sem Enter nem Esc (um clique noutro sítio): desiste, como o Esc.
        WidgetEvent::Blur(id) if camada(id).is_some() => {
            crate::modal::acaba_de_renomear();
            Some(true)
        }
        // Escrever não grava: um nome = um passo de undo, no Enter.
        WidgetEvent::TextChanged(id) if camada(id).is_some() => Some(true),
        WidgetEvent::Click(id) if id == crate::ids::PHYSICS_MATRIX_OPEN => {
            seam_reset_button(host, id);
            crate::modal::abre(None);
            Some(true)
        }
        WidgetEvent::Click(id)
            if id == crate::ids::PHYSICS_MATRIX_CLOSE || id == crate::ids::PHYSICS_MATRIX_SCRIM =>
        {
            seam_reset_button(host, id);
            if crate::modal::editando().is_none() {
                crate::modal::fecha();
            } else {
                crate::modal::acaba_de_renomear();
                host.store_mut().set_focus(None);
            }
            Some(true)
        }
        WidgetEvent::Click(id) if id == crate::ids::PHYSICS_MATRIX_CARD => Some(true),
        _ => None,
    }
}

pub(crate) fn apply_event(
    _state: &mut crate::state::PhysicsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let Some(c) = evento_do_nome(host, ev) {
        return EventOutcome::from_bool(c);
    }
    let consumed = match ev {
        // A slider moved: read the track the drag left behind, turn it into
        // this row's value, and emit the WHOLE settings struct with that one
        // field replaced.
        WidgetEvent::ValueChanged(id) => {
            if let Some(row) = rows::row_for(id) {
                if id == row.chip {
                    // The chip edit was already mirrored onto its linked slider,
                    // which fired its own ValueChanged and was handled there.
                    // Swallow, or one edit notifies twice.
                    true
                } else {
                    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                    let v = row.value_of(track);
                    editar(|mut s| {
                        (row.set)(&mut s, v);
                        s
                    });
                    true
                }
            } else if let Some(row) = interact::irow_for(id) {
                // The SECOND table (W-Hand). Same shape, other destination: a
                // runtime tool, not the document.
                if id == row.chip {
                    true
                } else {
                    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                    let mut it = state::current().interaction;
                    (row.set)(&mut it, row.value_of(track));
                    state::push_intent(PhysicsIntent::SetInteraction(it));
                    true
                }
            } else {
                false
            }
        }
        // The two radios of the Interaction section. Each option id names exactly
        // one variant, resolved through `interact::tool_for`/`hold_for` — the
        // inverse of the ALL-order the painter uses, so the chip that lights up and
        // the value that lands cannot disagree.
        WidgetEvent::Click(id) if interact::tool_for(id).is_some() => {
            seam_reset_button(host, id);
            let tool = interact::tool_for(id).expect("guard matched");
            let it = state::current().interaction;
            state::push_intent(PhysicsIntent::SetInteraction(InteractionSettings {
                tool,
                ..it
            }));
            true
        }
        // O rádio de modo de joint (W-JointTools). Braço próprio pela mesma
        // razão dos outros: cada rádio nomeia um fato, e uma função resolvendo
        // por dois é como o terceiro nasce lendo o array errado.
        WidgetEvent::Click(id) if interact::joint_for(id).is_some() => {
            seam_reset_button(host, id);
            let joint = interact::joint_for(id).expect("guard matched");
            let it = state::current().interaction;
            state::push_intent(PhysicsIntent::SetInteraction(InteractionSettings {
                joint,
                ..it
            }));
            true
        }
        // O rádio de ângulo da ponta (W-IK). Braço próprio e não um `bool`
        // enfiado no `hold_for`: os dois rádios nomeiam fatos diferentes.
        WidgetEvent::Click(id) if interact::ik_angle_for(id).is_some() => {
            seam_reset_button(host, id);
            let ik_match_angle = interact::ik_angle_for(id).expect("guard matched");
            let it = state::current().interaction;
            state::push_intent(PhysicsIntent::SetInteraction(InteractionSettings {
                ik_match_angle,
                ..it
            }));
            true
        }
        WidgetEvent::Click(id) if interact::hold_for(id).is_some() => {
            seam_reset_button(host, id);
            let hold = interact::hold_for(id).expect("guard matched");
            let it = state::current().interaction;
            state::push_intent(PhysicsIntent::SetInteraction(InteractionSettings {
                hold,
                ..it
            }));
            true
        }
        // Section headers fold. Panel-local view state, so it never becomes an
        // intent — the shell has no opinion about which sections are open.
        // ⛔ Este braço listava TRÊS nomes à mão + a tabela, e o quarto cabeçalho pintado
        //    (`PHYSICS_SEC_LAYERS`) caía no `_ => false`: pintado, registado, e mudo ao clique.
        //    Hoje pergunta à MESMA lista que `populate` regista.
        // ⭐⭐ **A dobra já foi feita pelo DESPACHO** (2026-09-24): o cabeçalho é uma secção
        //    dobrável (`mark_collapsible_section`, no `populate`), e o `apply_click` da casa dobra-a
        //    ANTES de emitir este `Click`. ⛔ Dobrar outra vez aqui desfazia o clique — o braço fica
        //    só para CONSUMIR o evento, que senão cairia no `_ => false`.
        WidgetEvent::Click(id) if rows::is_section_header(id) => true,
        // A matrix cell toggles ONE pair. `LayerMatrix::set` writes both halves,
        // so the asymmetric state the panel could otherwise author does not exist.
        WidgetEvent::Click(id) if crate::ids::PHYSICS_LAYER_CELL.contains(&id) => {
            seam_reset_button(host, id);
            let idx = crate::ids::PHYSICS_LAYER_CELL
                .iter()
                .position(|&c| c == id)
                .expect("guard matched");
            let (a, b) = cell_pair(idx);
            alterna_par(a, b);
            true
        }
        // ⭐ **O interruptor de adormecer** (2026-08-30). Era um slider sobre
        // `sleep_angular_threshold`, e a `rapier2d` 0.35 lê desse campo só o SINAL — o slider
        // mentia em todo o seu curso. A escrita vai pela porta `with_sleep_enabled`, nunca por uma
        // comparação com zero aqui: a lei do sinal mora num sítio (`ph2d-physics-ecs`).
        // ⭐ Caixa de marcar desde 2026-09-24: o despacho vira-a e emite `Toggled`. O valor NOVO
        //    sai do MODELO (`!on`), nunca do store — a lei da `paint_check_row`.
        WidgetEvent::Toggled(id) if id == crate::ids::PHYSICS_SLEEP_SPIN => {
            let on = state::current().vivo.sleep_enabled();
            editar(|s| s.with_sleep_enabled(!on));
            true
        }
        WidgetEvent::Toggled(id) if id == crate::ids::PHYSICS_SHOW_COLLIDERS => {
            state::push_intent(PhysicsIntent::ToggleColliders);
            true
        }
        // Os dois verbos de FITA (W25). Não tocam `PhysicsSettings`: a corrida
        // gravada mora na shell e viaja no seu próprio campo do `ProjectFile`,
        // então enfiá-la nas settings de mundo seriam duas cópias dela.
        WidgetEvent::Click(id) if id == crate::ids::PHYSICS_CLEAR_RUN => {
            seam_reset_button(host, id);
            state::push_intent(PhysicsIntent::ClearRun);
            true
        }
        WidgetEvent::Click(id) if id == crate::ids::PHYSICS_RESTORE_RUN => {
            seam_reset_button(host, id);
            state::push_intent(PhysicsIntent::RestoreRun);
            true
        }
        WidgetEvent::Click(id) if id == crate::ids::PHYSICS_RESET_DEFAULTS => {
            seam_reset_button(host, id);
            editar(|_| PhysicsSettings::default());
            true
        }
        WidgetEvent::Click(id) if id == crate::ids::PHYSICS_CLOSE => {
            seam_reset_button(host, id);
            host.set_panel_visible(crate::PhysicsPanel::ID, false);
            state::painel_fechado();
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
