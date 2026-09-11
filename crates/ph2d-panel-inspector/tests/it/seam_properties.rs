//! **Varredura de SEAM do cartão de PROPRIEDADES.**
//!
//! Todo clique passa pelo `click_at` REAL, e não por um `WidgetEvent` sintético: um evento
//! fabricado pula a checagem de focabilidade do store, então um widget deixado de fora do
//! `populate` fica pintado, hit-registrado e **morto sob o ponteiro**, com um teste verde ao lado.
//!
//! # ⛔ O que este ficheiro já mediu, e por que encolheu (2026-09-01)
//!
//! Ele chegou a ter quinze gates — o campo que renomeia o valor de uma propriedade, o formulário
//! de *Salvar Variação…*, o chip da combinação em falta, o botão de actualizar. O mecanismo de
//! propriedades foi **adiado** pelo dono, e o código saiu inteiro; os gates dele saíram com ele.
//! *Um gate sobre uma superfície que não existe é uma afirmação sobre nada.*
//!
//! ⚠️ **O que ele apanhou fica registado**: as duas primeiras corridas dele acusaram um botão
//! **morto sob o ponteiro** (fora do `populate`) e um chip com **id posicional** que mudava de
//! identidade com o tamanho da família. Nenhum gate unitário via qualquer um dos dois.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorPropertiesInfo, VariantChoice, variant_axes::VariantAxis,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_name, set_current_inspector_properties,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0077;
const ROOT: u64 = 0x5EED_0078;
const MINE: u64 = 11;
const OTHER: u64 = 22;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn choice(master: u64, label: &str, current: bool) -> VariantChoice {
    VariantChoice {
        master,
        label: label.into(),
        current,
    }
}

/// A fileira das VERSÕES, com duas — que é quando ela existe.
fn info() -> InspectorPropertiesInfo {
    InspectorPropertiesInfo {
        entity_bits: ENTITY,
        root_bits: ROOT,
        rows: vec![VariantAxis {
            name: String::new(),
            options: vec![choice(MINE, "Casa", true), choice(OTHER, "Casa Big", false)],
        }],
        beyond: 0,
        source_name: Some("Casa".into()),
    }
}

fn host_with(i: InspectorPropertiesInfo) -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Casa".into(),
    }));
    set_current_inspector_properties(Some(i));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state)
}

fn clear() {
    set_current_inspector_properties(None);
    set_current_inspector_name(None);
}

/// ⭐⭐⭐ **O chip de uma versão CHEGA ao `swap`, pelo ponteiro.**
///
/// **Mutação que deve sangrar:** apagar o `hit_index.register(id, host)` do pintor, o registo do
/// `populate`, ou o braço do `chip_click` no despachante.
#[test]
fn a_chip_of_another_version_reaches_the_swap() {
    let (mut host, mut state) = host_with(info());
    let id = ids::INSP_INSTANCE_AXIS_OPTION[0][1];
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("o chip «Casa Big» nunca foi pintado nem registado");
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "clicar no chip nao produziu evento — ele esta' morto sob o ponteiro"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let sent = host.drained_actions();
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::InspectorSwapVariant { root_bits, master }
                if *root_bits == ROOT && *master == OTHER
        )),
        "o clique nao chegou ao swap com o mestre certo: {sent:?}"
    );
    clear();
}

/// ⛔⛔ **UMA VERSÃO SÓ NÃO É UM BOTÃO** — ela não entra no hit-index, logo não há clique a morrer.
///
/// *Um controlo que se pode carregar e que não faz nada é a 1.ª espécie de knob morto da caça de
/// 2026-08-30.*
///
/// **Mutação que deve sangrar:** o pintor desenhar o valor único como `Button` + `register`.
#[test]
fn a_single_version_is_text_and_never_a_dead_button() {
    let i = InspectorPropertiesInfo {
        rows: vec![VariantAxis {
            name: String::new(),
            options: vec![choice(0, "Casa", true)],
        }],
        ..info()
    };
    let (mut host, mut state) = host_with(i);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][0]),
        "uma versao unica foi registada como botao — um clique que nao faz nada"
    );
    clear();
}

/// ⛔ **Carregar na versão VIGENTE não publica nada** — o artista carregou no botão que diz onde
/// ele já está, e a ausência de resposta é a resposta certa.
#[test]
fn clicking_the_current_version_publishes_nothing() {
    let (mut host, mut state) = host_with(info());
    let id = ids::INSP_INSTANCE_AXIS_OPTION[0][0];
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("o chip vigente nunca foi pintado");
    for ev in host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    assert!(
        host.drained_actions().is_empty(),
        "carregar na versao vigente publicou uma accao"
    );
    clear();
}
