//! **A POSIÇÃO de um controle autorado sobrevive ao arquivo** (plano UI/UX W8b.4).
//!
//! Irmão do [`crate::vec_widget_drive`], e o corte entre os dois é por PERGUNTA: ali mora *o que
//! este controle DIZ à forma*, aqui *que posição este controle GUARDA*. Não são a mesma coisa e
//! não se derivam uma da outra — um slider que o artista ainda não prendeu a forma nenhuma tem
//! posição, e a opacidade é quantizada em 255 degraus que não descrevem um slider contínuo.
//!
//! # O reconcile, e por que ele tem DUAS direções
//!
//! O valor tem dois escritores legítimos, e ignorar qualquer um deles quebra alguma coisa:
//!
//! - o **PONTEIRO** escreve o `WidgetStore` (arrastar o slider);
//! - o **MUNDO** escreve o componente (abrir um projeto · um Ctrl+Z · o smoke a semear a cena).
//!
//! ⚠️ Um sentido só não basta, e as duas metades falham de formas diferentes: sem *store →
//! componente* nada é salvo; sem *componente → store* o Ctrl+Z devolve o componente antigo e o
//! painel continua a mostrar a posição nova — o controle e a arte discordariam sobre o mesmo
//! número, na tela, sem nada dizer por quê.
//!
//! A desempate é um memo do que **já foi propagado** (`applied`), e a ordem é lei: **o artista
//! ganha**. Se o store difere do memo, foi o ponteiro; senão, se o componente difere do memo, foi
//! o mundo. Sem o memo os dois lados se sobrescreveriam em frames alternados e o slider tremeria.
//!
//! ⚠️ **Escrever o componente é uma EDIÇÃO** — o undo global é por DIFF do mundo, então arrastar o
//! slider vira um passo de undo. É o correto: mover um controle é um ato do artista, e o
//! `post_frame_undo` já suprime enquanto o botão está preso ⇒ **um passo por gesto**, não por
//! frame. (O que continua fora do undo é a ARTE: ela não é escrita por ninguém aqui.)

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecWidget, VecWidgetValue};
use ph2d_editor::interaction::{InteractiveState, WidgetStore};
use ph2d_editor::widget::{CheckboxValue, SliderOrientation, WidgetKind};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// O que já foi propagado entre o store e o mundo, por forma.
pub(crate) type Applied = BTreeMap<VecPathId, f32>;

/// **Que posição este controle carrega?** `None` = o estado não guarda posição nenhuma.
///
/// ⚠️ Normalizado: um slider dá `0..=1`, um toggle e um checkbox dão `0` ou `1`. O
/// `Indeterminate` de um checkbox vira **1** pela mesma leitura do `drive_of` (*parte dos filhos*
/// conta como ligado), e o preço está NOMEADO: reabrir devolve `Checked`, porque um controle
/// autorado não tem filhos para estar em parte.
#[must_use]
pub(crate) fn value_of(st: &InteractiveState) -> Option<f32> {
    match st {
        InteractiveState::Slider { value, .. } => Some(value.clamp(0.0, 1.0)),
        InteractiveState::Toggle { on, .. } => Some(f32::from(u8::from(*on))),
        InteractiveState::Checkbox { value, .. } => {
            Some(f32::from(u8::from(*value != CheckboxValue::Unchecked)))
        }
        _ => None,
    }
}

/// **O estado que essa posição descreve** — a inversa de [`value_of`].
///
/// ⚠️ As duas TÊM de fazer round-trip, e há gate: uma tradução que não volta é um controle que
/// muda de posição sozinho ao reabrir o arquivo — e mudaria a ARTE junto, porque a row a dirige.
#[must_use]
pub(crate) fn seed_state(kind: WidgetKind, value: f32) -> Option<InteractiveState> {
    let on = value >= 0.5;
    match kind {
        WidgetKind::Slider => Some(InteractiveState::Slider {
            state: ph2d_editor::widget::SliderState::default(),
            value: value.clamp(0.0, 1.0),
            orientation: SliderOrientation::Horizontal,
        }),
        WidgetKind::Toggle => Some(InteractiveState::Toggle {
            state: ph2d_editor::widget::ToggleState::default(),
            on,
        }),
        WidgetKind::Checkbox => Some(InteractiveState::Checkbox {
            state: ph2d_editor::widget::CheckboxState::default(),
            value: if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            },
        }),
        _ => None,
    }
}

/// **Mantém o store e o mundo de acordo, nas duas direções.**
///
/// Devolve `true` quando o MUNDO foi escrito — o chamador usa isso para saber que houve edição.
pub(crate) fn reconcile(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    store: &mut WidgetStore,
    applied: &mut Applied,
) -> bool {
    let mut wrote_world = false;
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let Some(kind) = sim
            .world()
            .get::<VecWidget>(e)
            .and_then(|w| WidgetKind::from_code(w.kind))
        else {
            continue;
        };
        let Some(name) = sim
            .world()
            .get::<ph2d_ecs::Name>(e)
            .map(|n| n.0.to_string())
        else {
            continue;
        };
        let row = ph2d_editor::ids::authored_row_id(&crate::ui_panel_spec::key_of(&name));
        // Um controle que o painel COMMITADO ainda não carrega não tem posição viva — semear o
        // mundo a partir do nada escreveria uma edição que ninguém fez.
        let Some(live) = store.get(row).and_then(value_of) else {
            continue;
        };
        let authored = sim.world().get::<VecWidgetValue>(e).map(|v| v.value);

        // ⚠️ **A primeira vez que vemos um controle NÃO escreve o mundo.** A ausência do
        // componente significa *onde quer que o controle esteja*, e materializá-la faria abrir uma
        // cena registar um passo de undo que ninguém pediu — o defeito exacto que o
        // `restore_painted_docs` custou ao load de projeto. Se há valor autorado, ele manda (é um
        // load); senão, só anotamos onde o controle está.
        let Some(memo) = applied.get(&id).copied() else {
            match authored.and_then(|a| seed_state(kind, a).map(|st| (a, st))) {
                Some((a, st)) => {
                    store.register(row, st);
                    applied.insert(id, a);
                }
                None => {
                    applied.insert(id, live);
                }
            }
            continue;
        };

        // O ARTISTA ganha: se o store saiu de onde o deixámos, foi o ponteiro.
        if memo != live {
            if authored != Some(live) {
                sim.world_mut()
                    .entity_mut(e)
                    .insert(VecWidgetValue { value: live });
                wrote_world = true;
            }
            applied.insert(id, live);
            continue;
        }
        // Senão, quem se moveu foi o MUNDO (um load, um Ctrl+Z, a cena de um smoke).
        if let Some(a) = authored
            && a != memo
            && let Some(st) = seed_state(kind, a)
        {
            store.register(row, st);
            applied.insert(id, a);
        }
    }
    wrote_world
}

#[cfg(test)]
#[path = "vec_widget_value_tests.rs"]
mod tests;
