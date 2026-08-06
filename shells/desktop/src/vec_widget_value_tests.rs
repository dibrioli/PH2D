//! Gates da **POSIÇÃO AUTORADA de um controle** — irmão de `vec_widget_value.rs` (W8b.4).

use super::*;
use ph2d_editor::widget::{CheckboxState, SliderState, ToggleState};

fn scene(kind: WidgetKind, name: &str) -> (SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let id: VecPathId = 1;
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::VecPathRef(id),
            ph2d_ecs::Name::new(name),
            VecWidget { kind: kind.code() },
        ))
        .id();
    map.insert(id, e.to_bits());
    (sim, map, id)
}

fn row_id(name: &str) -> ph2d_editor::NodeId {
    ph2d_editor::ids::authored_row_id(&crate::ui_panel_spec::key_of(name))
}

fn store_with(name: &str, st: InteractiveState) -> WidgetStore {
    let mut s = WidgetStore::with_capacity(8);
    s.register(row_id(name), st);
    s
}

fn slider(v: f32) -> InteractiveState {
    InteractiveState::Slider {
        state: SliderState::default(),
        value: v,
        orientation: SliderOrientation::Horizontal,
    }
}

fn authored(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<f32> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecWidgetValue>(Entity::from_bits(bits))
        .map(|v| v.value)
}

/// **A tradução faz ROUND-TRIP em todo tipo que guarda posição.**
///
/// ⚠️ O gate central: uma tradução que não volta é um controle que muda de posição sozinho ao
/// reabrir o arquivo — e muda a ARTE junto, porque a row a dirige. As duas metades são portas
/// separadas de propósito (uma lê o estado vivo, a outra o constrói), e é isto que as prende.
#[test]
fn the_translation_round_trips_for_every_kind_that_holds_a_position() {
    for (kind, values) in [
        (WidgetKind::Slider, &[0.0, 0.25, 0.5, 1.0][..]),
        (WidgetKind::Toggle, &[0.0, 1.0][..]),
        (WidgetKind::Checkbox, &[0.0, 1.0][..]),
    ] {
        for &v in values {
            let st = seed_state(kind, v).unwrap_or_else(|| panic!("{kind:?} guarda posicao"));
            assert_eq!(
                value_of(&st),
                Some(v),
                "{kind:?} nao volta de {v}: o controle mudaria de posicao ao reabrir"
            );
        }
    }
}

/// **Um tipo que não guarda posição não inventa uma.**
///
/// O par do gate acima, do lado da ausência: um `Button` não tem posição, e dar-lhe um número
/// escreveria uma edição no documento a cada frame — cada uma delas um passo de undo.
#[test]
fn a_kind_with_no_position_holds_none() {
    for k in [
        WidgetKind::Button,
        WidgetKind::SectionHeader,
        WidgetKind::Divider,
    ] {
        assert!(seed_state(k, 0.5).is_none(), "{k:?} nao guarda posicao");
    }
    assert_eq!(
        value_of(&InteractiveState::Button {
            state: Default::default()
        }),
        None
    );
}

/// **O ARTISTA escreve o mundo** — arrastar o slider vira posição autorada.
///
/// ⚠️ A primeira passagem é INERTE de propósito (ver o gate abaixo): ela só anota onde o controle
/// está. Quem escreve é o MOVIMENTO.
#[test]
fn dragging_the_control_writes_the_world() {
    let (mut sim, map, id) = scene(WidgetKind::Slider, "Opacity");
    let mut store = store_with("Opacity", slider(0.5));
    let mut applied = Applied::new();
    reconcile(&mut sim, &map, &mut store, &mut applied);
    assert_eq!(
        authored(&sim, &map, id),
        None,
        "abrir a cena nao autora nada"
    );

    store.register(row_id("Opacity"), slider(0.25));
    assert!(reconcile(&mut sim, &map, &mut store, &mut applied));
    assert_eq!(authored(&sim, &map, id), Some(0.25));
}

/// **O MUNDO escreve o store** — um load, ou um Ctrl+Z, e o painel acompanha.
///
/// ⚠️ A metade sem a qual o Ctrl+Z devolveria o componente antigo e o painel continuaria a mostrar
/// a posição nova: o controle e a arte discordariam sobre o mesmo número, na tela, sem nada dizer
/// por quê.
#[test]
fn the_world_writes_the_store() {
    let (mut sim, map, id) = scene(WidgetKind::Slider, "Opacity");
    let mut store = store_with("Opacity", slider(0.5));
    let mut applied = Applied::new();
    reconcile(&mut sim, &map, &mut store, &mut applied);

    // Alguém escreveu o mundo por fora — o que um load ou um undo faz.
    let &bits = map.get(&id).expect("o widget esta' no mapa");
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(VecWidgetValue { value: 0.25 });

    assert!(
        !reconcile(&mut sim, &map, &mut store, &mut applied),
        "propagar do mundo para o store NAO e' uma edicao"
    );
    assert_eq!(
        store.get(row_id("Opacity")).and_then(value_of),
        Some(0.25),
        "o painel tem de acompanhar o que o mundo diz"
    );
}

/// **Nem a primeira passagem nem um frame parado escrevem o mundo.**
///
/// ⚠️ As duas metades, e cada uma mata um defeito diferente: sem a segunda ele escreveria todo
/// frame e a fila de Ctrl+Z encheria sozinha enquanto o artista olha para a tela; **sem a
/// primeira, abrir uma cena registaria um passo de undo que ninguém pediu** — o defeito exacto
/// que o `restore_painted_docs` custou ao load de projeto. A ausência do componente significa
/// *onde quer que o controle esteja*, e materializá-la é uma edição inventada.
#[test]
fn neither_the_first_pass_nor_a_quiet_frame_writes_the_world() {
    let (mut sim, map, id) = scene(WidgetKind::Slider, "Opacity");
    let mut store = store_with("Opacity", slider(0.5));
    let mut applied = Applied::new();
    assert!(
        !reconcile(&mut sim, &map, &mut store, &mut applied),
        "a primeira vez que vemos um controle nao pode AUTORAR nada"
    );
    assert_eq!(authored(&sim, &map, id), None);
    for _ in 0..5 {
        assert!(
            !reconcile(&mut sim, &map, &mut store, &mut applied),
            "um frame parado nao pode escrever o mundo"
        );
    }
}

/// **O ARTISTA ganha do mundo no mesmo frame.**
///
/// ⚠️ A ordem é lei: se os dois lados mudaram desde a última propagação, quem manda é o ponteiro —
/// senão o valor que o artista acabou de arrastar seria revertido por um componente que ele já
/// substituiu, e o slider saltaria para trás sob o dedo.
#[test]
fn the_artist_wins_over_the_world() {
    let (mut sim, map, id) = scene(WidgetKind::Slider, "Opacity");
    let mut store = store_with("Opacity", slider(0.5));
    let mut applied = Applied::new();
    reconcile(&mut sim, &map, &mut store, &mut applied);

    // Os DOIS mudam antes do próximo reconcile.
    let &bits = map.get(&id).expect("o widget esta' no mapa");
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(VecWidgetValue { value: 0.1 });
    store.register(row_id("Opacity"), slider(0.9));

    reconcile(&mut sim, &map, &mut store, &mut applied);
    assert_eq!(authored(&sim, &map, id), Some(0.9), "o ponteiro manda");
    assert_eq!(store.get(row_id("Opacity")).and_then(value_of), Some(0.9));
}

/// **Um controle que o painel não carrega não escreve nada.**
///
/// A row só existe no painel COMMITADO; um widget acabado de vestir ainda não tem row. Semear o
/// mundo a partir do nada escreveria uma edição que ninguém fez.
#[test]
fn a_row_the_panel_does_not_carry_writes_nothing() {
    let (mut sim, map, id) = scene(WidgetKind::Slider, "Opacity");
    let mut store = WidgetStore::with_capacity(8);
    let mut applied = Applied::new();
    assert!(!reconcile(&mut sim, &map, &mut store, &mut applied));
    assert_eq!(authored(&sim, &map, id), None);
}

/// **O toggle e o checkbox atravessam pelos dois lados.**
#[test]
fn the_boolean_kinds_survive_the_round_trip_through_the_world() {
    for (kind, on, off) in [
        (
            WidgetKind::Toggle,
            InteractiveState::Toggle {
                state: ToggleState::default(),
                on: true,
            },
            InteractiveState::Toggle {
                state: ToggleState::default(),
                on: false,
            },
        ),
        (
            WidgetKind::Checkbox,
            InteractiveState::Checkbox {
                state: CheckboxState::default(),
                value: CheckboxValue::Checked,
            },
            InteractiveState::Checkbox {
                state: CheckboxState::default(),
                value: CheckboxValue::Unchecked,
            },
        ),
    ] {
        let (mut sim, map, id) = scene(kind, "Visible");
        let mut store = store_with("Visible", off.clone());
        let mut applied = Applied::new();
        reconcile(&mut sim, &map, &mut store, &mut applied);

        store.register(row_id("Visible"), on);
        reconcile(&mut sim, &map, &mut store, &mut applied);
        assert_eq!(authored(&sim, &map, id), Some(1.0), "{kind:?} ligado");

        store.register(row_id("Visible"), off);
        reconcile(&mut sim, &map, &mut store, &mut applied);
        assert_eq!(authored(&sim, &map, id), Some(0.0), "{kind:?} desligado");
    }
}
