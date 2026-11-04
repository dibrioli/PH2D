//! ⭐⭐ **O painel Color Equalization em cartões: nenhum controlo morreu**, e a lei `show` continua:
//! um número cuja etapa não corre não se pinta.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::dropdown;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_panel_color_equalization::state::set_current_snapshot;
use ph2d_panel_color_equalization::{ColorEqualizationPanel, ColorEqualizationPanelState};
use ph2d_tool_color_equalization::ids as t;
use ph2d_tool_color_equalization::lut_presets::LutPreset;
use ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot as Snap;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_ui_testkit::mesa::Mesa;

const PANEL: NodeId = ph2d_editor_core::ids::CEQ_PANEL;
const NUMEROS: [(NodeId, NodeId); 8] = [
    (t::CEQ_EXPOSURE_NUM, t::CEQ_EXPOSURE),
    (t::CEQ_BRIGHTNESS_NUM, t::CEQ_BRIGHTNESS),
    (t::CEQ_CONTRAST_NUM, t::CEQ_CONTRAST),
    (t::CEQ_TEMPERATURE_NUM, t::CEQ_TEMPERATURE),
    (t::CEQ_TINT_NUM, t::CEQ_TINT),
    (t::CEQ_VIBRANCE_NUM, t::CEQ_VIBRANCE),
    (t::CEQ_SATURATION_NUM, t::CEQ_SATURATION),
    (t::CEQ_SHARPEN_AMOUNT_NUM, t::CEQ_SHARPEN_AMOUNT),
];
const SEM_LEI: [(NodeId, NodeId); 2] = [
    (t::CEQ_SHARPEN_RADIUS_NUM, t::CEQ_SHARPEN_RADIUS),
    (t::CEQ_CLIP_LIMIT_NUM, t::CEQ_CLIP_LIMIT),
];

fn mesa(snap: Snap) -> Mesa<ColorEqualizationPanel> {
    force_cards_on_this_thread(Some(true));
    set_current_snapshot(Some(snap));
    let mut m = Mesa::<ColorEqualizationPanel>::nova(ColorEqualizationPanelState);
    m.host.set_collapsed(t::CEQ_SEC_LUT, false);
    m.host.set_collapsed(t::CEQ_SEC_STYLIZE, false);
    m.assenta();
    m
}

/// Todas as etapas a correr: nenhum número fica escondido pela lei `show`.
fn tudo() -> Snap {
    Snap {
        clip_limit: 2.0,
        lut_preset_1: LutPreset::ALL[1],
        lut_preset_2: LutPreset::ALL[2],
        posterize_levels: 4,
        posterize_dithering: true,
        ..Snap::default()
    }
}

#[test]
fn every_id_of_the_old_panel_is_painted_and_no_bar_ever() {
    let pintados = mesa(tudo()).pintados();
    for id in [
        t::CEQ_AUTO_LEVELS,
        t::CEQ_AUTO_CONTRAST,
        t::CEQ_AUTO_COLORS,
        t::CEQ_AUTO_WB,
        t::CEQ_LUT_1_DROPDOWN,
        t::CEQ_LUT_2_DROPDOWN,
        t::CEQ_POSTERIZE_DROPDOWN,
        t::CEQ_QUANTIZE_DROPDOWN,
        t::CEQ_POSTERIZE_DITHERING,
        t::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
        t::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
        t::CEQ_TILE_GRID_NUM,
        t::CEQ_LUT_INTENSITY_NUM,
        t::CEQ_LUT_MIX_NUM,
        t::CEQ_RESET,
        t::CEQ_CANCEL,
        t::CEQ_APPLY,
    ]
    .into_iter()
    .chain(NUMEROS.map(|(chip, _)| chip))
    .chain(SEM_LEI.map(|(chip, _)| chip))
    {
        assert!(pintados.contains(&id), "{id:?} nunca pintado");
    }
    for (_, barra) in NUMEROS.into_iter().chain(SEM_LEI) {
        assert!(
            !pintados.contains(&barra),
            "a barra {barra:?} ainda se pinta"
        );
    }
}

/// ⭐ A lei das etapas: no estado de fábrica, os botões que não chegam a nada não se pintam.
/// *Mutação: tirar o `filter(show)` de `campos` ⇒ o Tile Grid aparece.*
#[test]
fn a_row_whose_stage_does_not_run_is_not_painted() {
    let pintados = mesa(Snap::default()).pintados();
    for id in [
        t::CEQ_TILE_GRID_NUM,
        t::CEQ_LUT_MIX_NUM,
        t::CEQ_POSTERIZE_DITHERING,
        t::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
        t::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
    ] {
        assert!(!pintados.contains(&id), "{id:?} sem etapa não se pinta");
    }
}

#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for (chip, barra) in NUMEROS.into_iter().chain(SEM_LEI) {
        mesa(tudo()).prova_o_scrub(chip, barra, 40.0);
    }
    for (chip, barra) in [
        (t::CEQ_TILE_GRID_NUM, t::CEQ_TILE_GRID),
        (t::CEQ_LUT_INTENSITY_NUM, t::CEQ_LUT_INTENSITY),
        (t::CEQ_LUT_MIX_NUM, t::CEQ_LUT_MIX),
        (
            t::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            t::CEQ_POSTERIZE_DITHER_STRENGTH,
        ),
        (
            t::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
            t::CEQ_POSTERIZE_DITHER_GRAIN,
        ),
    ] {
        // Estes dois nascem no máximo.
        let dx = if chip == t::CEQ_LUT_INTENSITY_NUM || chip == t::CEQ_POSTERIZE_DITHER_STRENGTH_NUM
        {
            -40.0
        } else {
            40.0
        };
        mesa(tudo()).prova_o_scrub(chip, barra, dx);
    }
}

#[test]
fn typing_a_number_reaches_the_tool() {
    let mut m = mesa(Snap::default());
    let v = m.prova_a_escrita(t::CEQ_EXPOSURE_NUM, t::CEQ_EXPOSURE, "1");
    assert!(v > 0.5, "EV +1 fica acima do meio do trilho: {v}");
}

#[test]
fn every_check_and_verb_answers_a_real_click() {
    let mut m = mesa(tudo());
    for id in [
        t::CEQ_AUTO_LEVELS,
        t::CEQ_AUTO_CONTRAST,
        t::CEQ_AUTO_COLORS,
        t::CEQ_AUTO_WB,
        t::CEQ_POSTERIZE_DITHERING,
        t::CEQ_RESET,
        t::CEQ_APPLY,
    ] {
        m.prova_o_clique(id);
    }
    m.prova_o_cancelar(t::CEQ_CANCEL);
}

/// ⭐ Cada lista abre pelo clique na caixa e a opção chega à ferramenta. *Mutação: tirar o
/// `dropdown::declare` ⇒ a caixa não abre.*
#[test]
fn every_dropdown_opens_and_its_option_reaches_the_tool() {
    for (caixa, opcao) in [
        (t::CEQ_LUT_1_DROPDOWN, t::CEQ_LUT_1_OPTS[2]),
        (t::CEQ_LUT_2_DROPDOWN, t::CEQ_LUT_2_OPTS[2]),
        (t::CEQ_POSTERIZE_DROPDOWN, t::CEQ_POSTERIZE_OPTS[1]),
        (t::CEQ_QUANTIZE_DROPDOWN, t::CEQ_QUANTIZE_OPTS[1]),
    ] {
        let mut m = mesa(tudo());
        let r = m.rect(caixa).expect("a caixa pinta-se");
        let ev = m.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            ev.iter().any(|e| {
                matches!(e, ph2d_editor_core::interaction::WidgetEvent::Click(id) if *id == caixa)
            }),
            "a caixa tem de emitir Click: {ev:?}"
        );
        assert!(dropdown::on_click(caixa), "a caixa não foi declarada");
        m.assenta();
        m.prova_o_clique(opcao);
        assert!(!dropdown::on_click(opcao));
    }
}

#[test]
fn the_rare_cards_are_born_folded_and_the_verbs_stay() {
    force_cards_on_this_thread(Some(true));
    set_current_snapshot(Some(Snap::default()));
    let m = Mesa::<ColorEqualizationPanel>::nova(ColorEqualizationPanelState);
    assert!(m.host.store().is_collapsed(t::CEQ_SEC_LUT));
    assert!(m.host.store().is_collapsed(t::CEQ_SEC_STYLIZE));
    assert!(m.rect(t::CEQ_LUT_INTENSITY_NUM).is_none());
    m.prova_a_faixa(
        PANEL,
        &[t::CEQ_SEC_HISTOGRAM, t::CEQ_SEC_STYLIZE],
        &[t::CEQ_RESET, t::CEQ_APPLY],
    );
}

#[test]
fn a_card_folds() {
    let mut m = mesa(tudo());
    m.prova_a_dobra(t::CEQ_SEC_TONE, t::CEQ_EXPOSURE_NUM);
    m.prova_a_dobra(t::CEQ_SEC_AUTO, t::CEQ_AUTO_WB);
}

#[test]
fn cards_off_paints_the_old_body_and_keeps_the_real_dropdown() {
    force_cards_on_this_thread(Some(false));
    set_current_snapshot(Some(Snap::default()));
    let mut host = MockPanelHost::with_panel::<ColorEqualizationPanel>();
    let mut st = ColorEqualizationPanelState;
    let rects = host.paint::<ColorEqualizationPanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 340.0, 1200.0),
    );
    assert!(rects.iter().any(|(id, _)| *id == t::CEQ_EXPOSURE));
    assert!(host.store().card_field(t::CEQ_EXPOSURE_NUM).is_none());
    assert_eq!(host.dropdown_is_open(t::CEQ_LUT_1_DROPDOWN), Some(false));
    let _ = host.store();
    force_cards_on_this_thread(None);
}
