//! **Varredura de SEAM da ESCADA do *Aplicar*** (ADR-0164 / F5, critério 4).
//!
//! Todo clique passa pelo `click_at` REAL, e não por um `WidgetEvent` sintético: um evento
//! fabricado pula a checagem de focabilidade do store, então um botão deixado de fora do
//! `populate` fica pintado, hit-registrado e **morto sob o ponteiro**, com um teste verde ao lado.
//! É o irmão do [`seam_properties`], e existe pela mesma razão — as duas primeiras corridas dele
//! acusaram exactamente esse defeito noutra superfície deste mesmo cartão.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{ApplyChoice, InspectorInstanceInfo, InspectorNameInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_instance, set_current_inspector_name,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_00A1;
const ROOT: u64 = 0x5EED_00A2;
/// A receita de fora (o Carro) e a de dentro (a Roda).
const OUTER: u64 = 41;
const INNER: u64 = 42;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn rung(master: u64, name: &str, innermost: bool) -> ApplyChoice {
    ApplyChoice {
        master,
        name: name.into(),
        innermost,
    }
}

/// Uma cópia aninhada com UMA excepção nesta peça — que é quando a escada existe.
fn info() -> InspectorInstanceInfo {
    InspectorInstanceInfo {
        entity_bits: ENTITY,
        master_name: "Car".into(),
        overridden: vec!["Sprite".into()],
        orphan_rows: Vec::new(),
        root_bits: ROOT,
        is_variant: false,
        apply_levels: vec![rung(OUTER, "Car", false), rung(INNER, "Wheel", true)],
        apply_levels_beyond: 0,
        removed_rows: Vec::new(),
        added_rows: Vec::new(),
    }
}

fn host_with(i: InspectorInstanceInfo) -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Rim".into(),
    }));
    set_current_inspector_instance(Some(i));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state)
}

fn clear() {
    set_current_inspector_instance(None);
    set_current_inspector_name(None);
}

/// Carrega no botão do degrau `level` e devolve o que foi publicado.
fn click_rung(
    host: &mut MockPanelHost,
    state: &mut InspectorState,
    level: usize,
) -> Vec<EditorAction> {
    let id = ids::INSP_INSTANCE_APPLY_LEVEL[level];
    let rects = host.paint::<InspectorPanel>(state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("o degrau nunca foi pintado nem registado");
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "carregar no degrau nao produziu evento — ele esta' morto sob o ponteiro"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
    host.drained_actions()
}

/// ⭐⭐⭐ **O degrau de DENTRO chega ao verbo com a receita certa, pelo ponteiro.**
///
/// **Mutação que deve sangrar:** apagar o `hit_index.register` do pintor, o registo do `populate`,
/// ou o braço do `apply_level_click` no despachante.
#[test]
fn the_inner_rung_reaches_the_verb_with_its_own_master() {
    let (mut host, mut state) = host_with(info());
    let sent = click_rung(&mut host, &mut state, 1);
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::InspectorApplyToLevel { entity_bits, master }
                if *entity_bits == ENTITY && *master == INNER
        )),
        "o clique nao chegou ao verbo com a receita de DENTRO: {sent:?}"
    );
    clear();
}

/// ⭐⭐ **E o de FORA chega com a dele** — sem isto os dois botões seriam o mesmo gesto com dois
/// rótulos, que é precisamente o defeito que a escada existe para curar.
#[test]
fn the_outer_rung_reaches_the_verb_with_a_different_master() {
    let (mut host, mut state) = host_with(info());
    let sent = click_rung(&mut host, &mut state, 0);
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::InspectorApplyToLevel { master, .. } if *master == OUTER
        )),
        "o degrau de fora publicou a receita errada: {sent:?}"
    );
    clear();
}

/// ⛔⛔ **UM DEGRAU SÓ NÃO É UM BOTÃO** — uma cópia não aninhada não entra no hit-index, logo não há
/// clique a morrer ali. *Um controlo que se pode carregar e que não escolhe nada é a 1.ª espécie
/// de knob morto da caça de 2026-08-30*, e *«aplicar ao mestre»* já é o item do menu da linha.
///
/// **Mutação que deve sangrar:** o `apply_rows` devolver a escada sem o teste do comprimento.
#[test]
fn a_single_rung_is_never_painted_as_a_button() {
    let i = InspectorInstanceInfo {
        apply_levels: vec![rung(OUTER, "Car", true)],
        ..info()
    };
    let (mut host, mut state) = host_with(i);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_INSTANCE_APPLY_LEVEL[0]),
        "uma escada de um degrau foi registada como botao — um clique que nao escolhe nada"
    );
    clear();
}

/// ⛔ **Sem excepção nesta peça a escada NÃO é pintada** — não há o que aplicar, e um botão
/// permanentemente inerte é ruído que o artista aprende a ignorar (a lei do gesto dos órfãos).
///
/// **Mutação que deve sangrar:** tirar o `self.overridden.is_empty()` do `apply_rows`.
#[test]
fn with_nothing_overridden_the_ladder_is_not_painted() {
    let i = InspectorInstanceInfo {
        overridden: Vec::new(),
        ..info()
    };
    let (mut host, mut state) = host_with(i);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_INSTANCE_APPLY_LEVEL[0]),
        "a escada foi pintada sobre uma peca que nao tem excepcao nenhuma"
    );
    clear();
}

/// ⭐⭐ **O RÓTULO distingue as duas leis** — *Apply to* muda a receita da peça; *Apply as override
/// in* deixa o valor como excepção da cópia que vive dentro daquela receita. ⚠️ São consequências
/// diferentes, e o cartão é o único sítio onde o artista as pode ler antes de carregar.
#[test]
fn the_two_rungs_say_different_things() {
    let i = info();
    let rows = i.apply_rows();
    assert_eq!(rows[0].label(), "Apply as override in \u{201c}Car\u{201d}");
    assert_eq!(rows[1].label(), "Apply to \u{201c}Wheel\u{201d}");
}
