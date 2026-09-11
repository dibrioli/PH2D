//! **«Show sheet on canvas»** — a caixa que abre a grelha de uma sprite no canvas.
//!
//! Enio, 2026-08-23: *«você digita 8 quadros e não vê onde eles começam ou terminam»*.
//!
//! # ⚠️ Ela é a única caixa do Inspector que NINGUÉM despacha
//!
//! O valor dela vive no `WidgetStore` e a shell lê-o direto no quadro — sem `EditorAction`, sem
//! commit, sem undo, sem save. É uma **vista**, e uma edição levá-la-ia ao ficheiro: o artista
//! reabriria o projeto com a folha aberta sem se lembrar de a ter aberto.
//!
//! ⇒ Por isso este gate **não** procura uma ação no barramento. Ele afirma as duas coisas que
//! importam: que a caixa só existe onde há grelha para abrir, e que carregar nela **muda o valor
//! que a shell lê**. *Um interruptor de vista prova-se pelo estado que ele deixa, não pelo evento
//! que ele levanta.*

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource,
};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 4000.0,
};

/// Pinta o Inspector sobre uma sprite cuja grelha no MUNDO é `hf × vf`.
///
/// ⚠️ A grelha vai no **snapshot**, e não nos campos do store: é dali que a caixa decide, pela
/// mesma razão que o canvas usa o `Sprite` do mundo.
fn painted_with_grid(hf: u32, vf: u32) -> (MockPanelHost, Vec<(ph2d_a11y::NodeId, Rect)>) {
    let mut host = MockPanelHost::with_panel_and_shared_chrome::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    set_current_inspector_sprite(Some(InspectorSpriteInfo {
        entity_bits: 0x5EED_0777,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        emissive: 0.0,
        sheet_label: None,
        source_pixels: Some((256, 256)),
        can_reimport: true,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: hf,
        vframes: vf,
        frame: 0,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: false,
        region_rect: [0.0; 4],
        region_filter_clip: false,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_sprite(None);
    (host, rects)
}

/// **Sem grelha, sem interruptor** — e com grelha ele está lá.
///
/// ⚠️ A metade da AUSÊNCIA é a que carrega o desenho: numa sprite `1×1` a caixa não teria o que
/// desdobrar, e um interruptor que não faz nada ensina a desconfiar dos outros. É a mesma lei do
/// `+ Add Animator` da §11 — a face sem estado mostra o que se **pode** fazer.
#[test]
fn the_switch_exists_only_where_there_is_a_grid() {
    let has = |hf: u32, vf: u32| {
        painted_with_grid(hf, vf)
            .1
            .iter()
            .any(|(n, _)| *n == ids::INSP_SHEET_PREVIEW)
    };
    assert!(!has(1, 1), "uma sprite 1x1 nao tem folha para abrir");
    assert!(has(8, 1), "uma tira de 8 tem");
    assert!(has(1, 4), "uma coluna de 4 tambem");
    assert!(has(4, 2), "e uma grelha de verdade");
    // ⚠️ Zero nos campos é o piso de 1, não uma grelha — o modelo trata `0` como `1`, e a caixa
    // tem de concordar com ele em vez de oferecer uma folha de zero células.
    assert!(!has(0, 0), "zero e' o piso de UM, e um nao e' folha");
}

/// **Carregar na caixa muda o valor que a SHELL lê** — pelo ponteiro real.
///
/// ⚠️ `click_at`, e não um `Toggled` fabricado: é o `is_focusable` do store que decide se o clique
/// chega, e uma caixa pintada sem `register` fica viva na tela e morta sob o rato.
#[test]
fn clicking_the_switch_flips_the_value_the_shell_reads() {
    let (mut host, rects) = painted_with_grid(8, 1);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_SHEET_PREVIEW)
        .map(|(_, r)| *r)
        .expect("a caixa e' pintada com grelha");
    assert_eq!(
        host.store()
            .checkbox(ids::INSP_SHEET_PREVIEW)
            .map(|(_, v)| v),
        Some(CheckboxValue::Unchecked),
        "ela nasce DESLIGADA -- uma vista que se liga sozinha e' uma cena que o artista nao montou"
    );
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "clicar no meio da caixa nao produziu evento nenhum: ela esta' MORTA sob o rato"
    );
    assert_eq!(
        host.store()
            .checkbox(ids::INSP_SHEET_PREVIEW)
            .map(|(_, v)| v),
        Some(CheckboxValue::Checked),
        "o clique tem de deixar a caixa marcada -- e' este valor que a shell le' por quadro"
    );
}
