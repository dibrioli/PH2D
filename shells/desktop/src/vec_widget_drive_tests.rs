//! Gates da **ROW QUE DIRIGE** — arquivo irmão de `vec_widget_drive.rs` (plano UI/UX W8b.3).

use super::*;
use ph2d_editor::widget::{SliderOrientation, SliderState, ToggleState};

/// Uma cena com um WIDGET (nomeado, vestido, opcionalmente vinculado) e uma FORMA alvo.
fn scene(kind: WidgetKind, name: &str, bound: bool) -> (SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let (widget_id, target_id): (VecPathId, VecPathId) = (1, 2);
    let t = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::VecPathRef(target_id),
            ph2d_ecs::Name::new("Star"),
        ))
        .id();
    map.insert(target_id, t.to_bits());
    let mut w = sim.world_mut().spawn((
        ph2d_ecs::Transform::IDENTITY,
        ph2d_ecs::VecPathRef(widget_id),
        ph2d_ecs::Name::new(name),
        VecWidget { kind: kind.code() },
    ));
    if bound {
        w.insert(VecWidgetBind { target: target_id });
    }
    map.insert(widget_id, w.id().to_bits());
    (sim, map, target_id)
}

/// O store com a row `name` no estado dado.
fn store_with(name: &str, st: InteractiveState) -> WidgetStore {
    let mut s = WidgetStore::with_capacity(8);
    s.register(
        ph2d_editor::ids::authored_row_id(&crate::ui_panel_spec::key_of(name)),
        st,
    );
    s
}

fn slider(v: f32) -> InteractiveState {
    InteractiveState::Slider {
        state: SliderState::default(),
        value: v,
        orientation: SliderOrientation::Horizontal,
    }
}

fn toggle(on: bool) -> InteractiveState {
    InteractiveState::Toggle {
        state: ToggleState::default(),
        on,
    }
}

/// **Só os tipos que produzem VALOR dirigem.**
///
/// ⚠️ Afirmado por TIPO e não pela cena: a cena muda de desenho, a lei não. Um `Button` produz um
/// EVENTO — vinculá-lo daria um conta-gotas que resolve e não faz nada, que é o item-de-menu-morto
/// com outro nome.
#[test]
fn only_the_kinds_that_produce_a_value_can_drive() {
    for k in [WidgetKind::Slider, WidgetKind::Toggle, WidgetKind::Checkbox] {
        assert!(bindable(k), "{k:?} produz um valor e nao pode dirigir");
    }
    for k in [
        WidgetKind::Button,
        WidgetKind::SectionHeader,
        WidgetKind::Divider,
        WidgetKind::ProgressBar,
        WidgetKind::TextInput,
    ] {
        assert!(!bindable(k), "{k:?} nao produz um valor e foi oferecido");
    }
}

/// **O slider vira OPACIDADE e o toggle vira APARECER** — a porta única, pelo estado vivo.
#[test]
fn the_live_state_becomes_what_the_shape_shows() {
    assert_eq!(drive_of(&slider(1.0)), Some(Drive::Opacity(255)));
    assert_eq!(drive_of(&slider(0.0)), Some(Drive::Opacity(0)));
    assert_eq!(drive_of(&toggle(true)), Some(Drive::Visible(true)));
    assert_eq!(drive_of(&toggle(false)), Some(Drive::Visible(false)));
    // Um estado que não produz valor não produz drive — o par do gate de tipos, do lado do estado.
    assert_eq!(
        drive_of(&InteractiveState::Button {
            state: Default::default()
        }),
        None
    );
}

/// **A row é achada pela MESMA chave que o gerador escreve.**
///
/// ⚠️ Este é o gate central da wave, e o defeito que ele mata é MUDO: uma segunda derivação de
/// chave (`"Opacity"` → `opacity` aqui, `Opacity` ali) daria um painel cujas rows respondem ao
/// ponteiro e um resolvedor que procura por um id que ninguém registou. Nada falharia — o slider
/// simplesmente não faria nada.
#[test]
fn the_drive_is_found_by_the_key_the_generator_writes() {
    // ⚠️ O rótulo tem de conter o FENÓMENO: `key_of` faz duas coisas — minúsculas **e** trocar o
    // que não é alfanumérico por `_`. Num rótulo de uma palavra só (`"Opacity"`) uma derivação à
    // mão com `to_lowercase()` dá a MESMA resposta, e o gate ficaria verde sobre a segunda porta
    // que ele existe para proibir (medido: a mutação sobreviveu com aquela fixture).
    let (sim, map, target) = scene(WidgetKind::Slider, "Fill Opacity", true);
    let store = store_with("Fill Opacity", slider(0.5));
    assert_eq!(
        resolve(&sim, &map, &store),
        vec![(target, Drive::Opacity(128))],
        "o rotulo do artista tem de casar com a chave da row"
    );
}

/// **Sem vínculo, nada é dirigido** — o CONTROLE, e é ele que garante que todo documento que já
/// existe desenha byte-idêntico ao mundo pré-W8b.3.
#[test]
fn an_unbound_widget_drives_nothing() {
    let (sim, map, _) = scene(WidgetKind::Slider, "Opacity", false);
    let store = store_with("Opacity", slider(0.5));
    assert!(resolve(&sim, &map, &store).is_empty());
}

/// **Um vínculo cujo tipo não dirige é inerte.**
///
/// ⚠️ Ele pode existir: o artista prende um Slider, troca o tipo para Button, e o componente fica.
/// Honrá-lo faria um `Button` apagar uma forma; apagá-lo na troca destruiria o vínculo que volta
/// quando ele desfizer a troca. Inerte é a única resposta que não perde trabalho nem inventa.
#[test]
fn a_bound_widget_of_a_kind_that_cannot_drive_is_inert() {
    let (sim, map, _) = scene(WidgetKind::Button, "Reset", true);
    let store = store_with("Reset", slider(0.5));
    assert!(resolve(&sim, &map, &store).is_empty());
}

/// **Sem valor no store o vínculo fica DORMENTE.**
///
/// A row só existe no painel COMMITADO; um widget acabado de vestir ainda não tem row. Inventar um
/// default aqui mexeria na arte sem ninguém ter tocado em nada.
#[test]
fn a_row_the_panel_does_not_carry_yet_drives_nothing() {
    let (sim, map, _) = scene(WidgetKind::Slider, "Opacity", true);
    let store = WidgetStore::with_capacity(8);
    assert!(resolve(&sim, &map, &store).is_empty());
}

/// **Um toggle DESLIGADO esconde; ligado não toca em nada.**
#[test]
fn the_toggle_hides_and_only_when_off() {
    let mut view = VecViewState::default();
    apply(&[(7, Drive::Visible(false))], &mut view);
    assert_eq!(view.hidden, vec![7]);

    let mut view = VecViewState::default();
    apply(&[(7, Drive::Visible(true))], &mut view);
    assert!(
        view.hidden.is_empty(),
        "ligado nao pode ESCREVER nada — a forma ja' esta' visivel"
    );
}

/// **A opacidade FUNDE na entrada de tinta que a forma já tem, nunca acrescenta uma segunda.**
///
/// ⚠️ O consumidor lê UMA entrada por forma; uma segunda seria descartada em silêncio, e qual das
/// duas some dependeria da ordem de iteração de um mapa. O fixture põe um token na frente
/// exatamente para o gate poder ver isso.
#[test]
fn the_opacity_merges_into_the_paint_the_shape_already_has() {
    let mut view = VecViewState::default();
    view.bound.push(BoundStyle {
        path: 7,
        fill: Some(ph2d_vec_scene::Rgba8::new(1, 2, 3, 255)),
        ..BoundStyle::default()
    });
    apply(&[(7, Drive::Opacity(128))], &mut view);
    assert_eq!(view.bound.len(), 1, "uma forma, uma entrada");
    assert_eq!(view.bound[0].alpha, Some(128));
    assert_eq!(
        view.bound[0].fill,
        Some(ph2d_vec_scene::Rgba8::new(1, 2, 3, 255)),
        "a tinta do token tem de sobreviver a' fusao"
    );
}

/// **E sem tinta prévia ela nasce sozinha.**
#[test]
fn the_opacity_stands_alone_when_the_shape_has_no_bound_paint() {
    let mut view = VecViewState::default();
    apply(&[(7, Drive::Opacity(200))], &mut view);
    assert_eq!(view.bound.len(), 1);
    assert_eq!(view.bound[0].path, 7);
    assert_eq!(view.bound[0].alpha, Some(200));
    assert_eq!(view.bound[0].fill, None);
}
