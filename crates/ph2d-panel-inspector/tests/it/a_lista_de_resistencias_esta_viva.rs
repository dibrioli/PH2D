//! ⭐⭐⭐ **A tabela de RESISTÊNCIAS da secção HEALTH** (plano 28, W6) — a lista, os dois botões e o
//! editor da linha aberta, pelo gesto REAL do painel.
//!
//! O idioma é o da máquina de estados, e os gates são os dela, com uma pergunta a mais: **o editor
//! mostra a linha ABERTA** — abrir a 2.ª linha tem de re-semear o tipo e a taxa dela, senão o artista
//! escreve por cima da 1.ª a julgar que edita a 2.ª.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::vida_edits::{
    InspectorHealthInfo, InspectorResistanceRow, InspectorVidaInfo, VidaFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_vida};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

fn linha(kind: &str, rate: f32) -> InspectorResistanceRow {
    InspectorResistanceRow {
        kind: kind.into(),
        rate,
        absorbs: false,
        repetida: false,
    }
}

fn vida(resistances: Vec<InspectorResistanceRow>) -> InspectorHealthInfo {
    InspectorHealthInfo {
        max: 100.0,
        start: 100.0,
        invincible_s: 0.0,
        overheal: false,
        regen: 0.0,
        regen_delay_s: 0.0,
        shield_start: 0.0,
        shield_max: 0.0,
        shield_duration_s: 5.0,
        shield_regen: 0.0,
        shield_regen_delay_s: 0.0,
        shield_blocks_excess: false,
        armor_flat: 0.0,
        armor_percent: 0.0,
        dodge: 0.0,
        team: String::new(),
        on_damage: String::new(),
        on_heal: String::new(),
        on_death: String::new(),
        seed: 0,
        death_hitstop_s: 0.0,
        blink_s: 0.0,
        knockback_taken: 1.0,
        numbers: false,
        numbers_color: [1.0; 4],
        numbers_size: 0.45,
        resistances,
        agora: None,
    }
}

fn info(resistances: Vec<InspectorResistanceRow>) -> InspectorVidaInfo {
    InspectorVidaInfo {
        entity_bits: 0x00AB_1234,
        health: Some(vida(resistances)),
        damage: None,
        bar: None,
        has_body: true,
        clock_playing: true,
        selected_count: 1,
    }
}

fn salamandra() -> Vec<InspectorResistanceRow> {
    vec![linha("fogo", 0.0), linha("gelo", 2.0)]
}

fn edicoes(h: &mut MockPanelHost) -> Vec<E> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Vida(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

fn centro(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> (f32, f32) {
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} nao foi pintado"));
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐⭐ **Abrir a 2.ª linha re-semeia o editor com o tipo e a taxa DELA** — e o clique na linha
/// NÃO vai ao barramento (qual linha se edita é um facto da UI).
///
/// **Mutações que devem sangrar:** tirar a `aberta` da `assinatura` do `sync_vida` (o editor fica
/// com a 1.ª linha); e mandar ao barramento o clique na linha.
#[test]
fn abrir_uma_linha_mostra_a_linha_aberta_no_editor() {
    set_current_inspector_vida(Some(info(salamandra())));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    // O CONTROLO: de fábrica abre a 1.ª linha.
    assert_eq!(h.store().text(ids::INSP_VIDA_RESIST_KIND), Some("fogo"));
    let (x, y) = centro(&rects, ids::INSP_VIDA_RESIST_ROW[1]);
    for ev in h.click_at(x, y) {
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    assert_eq!(st.resist_selected, 1, "o clique na 2.ª linha não a abriu");
    assert!(
        edicoes(&mut h).is_empty(),
        "abrir uma linha foi ao barramento — um passo de undo por clique"
    );
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert_eq!(
        h.store().text(ids::INSP_VIDA_RESIST_KIND),
        Some("gelo"),
        "o editor continua a mostrar a 1.ª linha depois de abrir a 2.ª"
    );
    let taxa = h.store().number_value(ids::INSP_VIDA_RESIST_RATE);
    assert_eq!(taxa, Some(2.0));
    set_current_inspector_vida(None);
}

/// ⭐⭐ **`+ Add` junta uma linha E abre-a; `x Remove` tira a aberta e desce a selecção.**
///
/// **Mutações que devem sangrar:** o `+` não abrir a nova (`resist_selected = n`); o `x` tirar
/// outra linha que não a aberta.
#[test]
fn juntar_abre_a_nova_e_tirar_leva_a_aberta() {
    set_current_inspector_vida(Some(info(salamandra())));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::Click(ids::INSP_VIDA_RESIST_ADD),
    );
    assert_eq!(edicoes(&mut h), vec![E::AddResistance]);
    assert_eq!(st.resist_selected, 2, "o `+` não abriu a linha que nasceu");

    st.resist_selected = 1;
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::Click(ids::INSP_VIDA_RESIST_REMOVE),
    );
    assert_eq!(edicoes(&mut h), vec![E::RemoveResistance(1)]);
    assert_eq!(st.resist_selected, 0);
    set_current_inspector_vida(None);
}

/// ⭐⭐ **Os três campos do editor escrevem na linha ABERTA** — com a 2.ª aberta, o índice é `1`.
///
/// **Mutação que deve sangrar:** escrever sempre na linha `0`.
#[test]
fn o_editor_escreve_na_linha_aberta() {
    set_current_inspector_vida(Some(info(salamandra())));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState {
        resist_selected: 1,
        ..InspectorState::default()
    };
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_text(ids::INSP_VIDA_RESIST_KIND, "raio");
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::TextChanged(ids::INSP_VIDA_RESIST_KIND),
    );
    h.set_number_value(ids::INSP_VIDA_RESIST_RATE, 0.5);
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(ids::INSP_VIDA_RESIST_RATE),
    );
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::Toggled(ids::INSP_VIDA_RESIST_ABSORBS),
    );
    assert_eq!(
        edicoes(&mut h),
        vec![
            E::ResistanceKind(1, "raio".into()),
            E::ResistanceRate(1, 0.5),
            E::ResistanceAbsorbs(1, true),
        ]
    );
    set_current_inspector_vida(None);
}

/// ⭐⭐ **O `+` DESAPARECE no tecto** e não fica cinzento a mentir — e o tecto é o da tabela de ids.
///
/// **Mutação que deve sangrar:** pintar o `+` sempre.
#[test]
fn o_mais_desaparece_no_tecto() {
    let cheia: Vec<_> = (0..ids::INSP_VIDA_RESIST_ROW.len())
        .map(|i| linha(&format!("t{i}"), 1.0))
        .collect();
    set_current_inspector_vida(Some(info(cheia)));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(!rects.iter().any(|(n, _)| *n == ids::INSP_VIDA_RESIST_ADD));
    // Todas as linhas pintadas, e o `x` continua.
    for id in ids::INSP_VIDA_RESIST_ROW {
        assert!(rects.iter().any(|(n, _)| *n == id), "a linha {id:?} sumiu");
    }
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_VIDA_RESIST_REMOVE)
    );
    set_current_inspector_vida(None);

    // O CONTROLO: uma abaixo do tecto, o `+` está lá.
    set_current_inspector_vida(Some(info(salamandra())));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rects.iter().any(|(n, _)| *n == ids::INSP_VIDA_RESIST_ADD));
    set_current_inspector_vida(None);
}

/// ⭐⭐ **Os botões e as linhas estão VIVOS sob o dedo** — o gesto real dá-lhes um evento.
///
/// **Mutação que deve sangrar:** tirar os `register_button_ids` do `populate_vida`.
#[test]
fn a_lista_e_os_botoes_estao_vivos_sob_o_dedo() {
    for id in [
        ids::INSP_VIDA_RESIST_ROW[0],
        ids::INSP_VIDA_RESIST_ROW[1],
        ids::INSP_VIDA_RESIST_ADD,
        ids::INSP_VIDA_RESIST_REMOVE,
    ] {
        set_current_inspector_vida(Some(info(salamandra())));
        let mut h = MockPanelHost::with_panel::<InspectorPanel>();
        let mut st = InspectorState::default();
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let (x, y) = centro(&rects, id);
        let evs = h.click_at(x, y);
        assert!(
            evs.contains(&WidgetEvent::Click(id)),
            "clicar em {id:?} não deu o clique — ele está MORTO sob o dedo"
        );
        set_current_inspector_vida(None);
    }
}
