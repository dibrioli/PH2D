//! P13 U-Paleta: as rampas sob a grelha pelo rato de verdade — New Ramp… abre o diálogo e o Create
//! pede a rampa com os campos do diálogo; o Cycle ligado pede o ciclo da rampa da amostra clicada
//! (o documento: `shells/pixel-scenes/tests/it/rampas_p13.rs`).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{
    NewSpriteMode, PixelCycle, PixelPanelInfo, PixelRampAction, PixelRampConfig, PixelRampInfo,
    PixelRequest,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tools::seam_rampas as r;
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, set_current_pixel};

/// O painel nos cartões (o do dono, `cartoes = true`) ou o de antes (`PH2D_CARDS=0`).
fn host(cartoes: bool) -> ph2d_ui_testkit::MockPanelHost {
    if !cartoes {
        return crate::host_de_antes();
    }
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    ph2d_ui_testkit::MockPanelHost::with_panel::<PixelToolsPanel>()
}

fn viewport() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,  // LITERAL-PX-OK: a largura de uma doca
        h: 6000.0, // LITERAL-PX-OK: alto o bastante para o painel inteiro caber sem rolar
    }
}

fn info(ramps: Vec<PixelRampInfo>, anchor: Option<usize>) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        mode: NewSpriteMode::Indexed,
        palette: (0..8u8).map(|i| [i * 30, 40, 200 - i * 20, 255]).collect(),
        fg: [200, 120, 60, 255],
        ramps,
        anchor,
        ..Default::default()
    }
}

/// Pinta e carrega no meio de `id`; devolve os pedidos.
fn carrega(
    host: &mut ph2d_ui_testkit::MockPanelHost,
    st: &mut PixelToolsPanelState,
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let rects = host.paint::<PixelToolsPanel>(st, viewport());
    let q = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
    let ev = host.click_at(q.x + q.w * 0.5, q.y + q.h * 0.5);
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) | WidgetEvent::Toggled(c) if *c == id)),
        "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
    );
    for e in ev {
        let _ = host.apply_panel_event::<PixelToolsPanel>(st, e);
    }
    host.drained_actions()
}

#[test]
fn new_ramp_e_create_pedem_a_rampa_do_dialogo() {
    new_ramp_e_create(false);
}

fn new_ramp_e_create(cartoes: bool) {
    let mut host = host(cartoes);
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(info(Vec::new(), None));
    assert!(
        carrega(&mut host, &mut st, r::NEW).is_empty(),
        "abrir não pede"
    );
    let a = carrega(&mut host, &mut st, r::CREATE);
    let quer = PixelRequest::Ramp(PixelRampAction::New(PixelRampConfig::default()));
    assert_eq!(a, [EditorAction::Pixel(quer)], "cartões = {cartoes}");
}

#[test]
fn ligar_cycle_pede_o_ciclo_da_rampa_da_amostra() {
    ligar_cycle(false);
}

fn ligar_cycle(cartoes: bool) {
    let mut host = host(cartoes);
    let mut st = PixelToolsPanelState::default();
    let rampa = PixelRampInfo {
        id: 7,
        indices: vec![2, 3, 4],
        cycle: None,
    };
    set_current_pixel(info(vec![rampa], Some(3)));
    let a = carrega(&mut host, &mut st, r::CYCLE);
    let quer = PixelRequest::Ramp(PixelRampAction::Cycle {
        ramp: 7,
        cycle: Some(PixelCycle::default_for(3)),
    });
    assert_eq!(a, [EditorAction::Pixel(quer)], "cartões = {cartoes}");
}

/// O painel do dono (os cartões): só o cartão Palette aberto, e o clique pela `Mesa`.
fn mesa(i: PixelPanelInfo) -> ph2d_ui_testkit::mesa::Mesa<PixelToolsPanel> {
    use ph2d_panel_pixel_tools::ids;
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    set_current_pixel(i);
    let mut m =
        ph2d_ui_testkit::mesa::Mesa::<PixelToolsPanel>::nova(PixelToolsPanelState::default());
    for s in [
        ids::PIXEL_SEC_OPTIONS,
        ids::PIXEL_SEC_BRUSHES,
        ids::PIXEL_SEC_TRANSFORM,
        ids::PIXEL_SEC_COLORS,
        ids::PIXEL_SEC_LAYERS,
        ids::PIXEL_SEC_PALETTE,
        ids::PIXEL_SEC_PICKER,
        ids::PIXEL_SEC_SYMMETRY,
        ids::PIXEL_SEC_SELECT,
        ids::PIXEL_SEC_EDIT,
        ids::PIXEL_SEC_SPRITE,
    ] {
        m.host.set_collapsed(s, s != ids::PIXEL_SEC_PALETTE);
    }
    m.assenta();
    m.host.drained_actions();
    m
}

#[test]
fn nos_cartoes_new_ramp_e_create_pedem_a_rampa() {
    let mut me = mesa(info(Vec::new(), None));
    assert!(me.rect(r::NEW).is_some(), "New Ramp… não pintado no cartão");
    assert!(me.clica(r::NEW).is_empty(), "abrir não pede");
    me.assenta();
    assert!(
        me.rect(r::CREATE).is_some(),
        "o diálogo não abriu no cartão"
    );
    let a = me.clica(r::CREATE);
    let quer = PixelRequest::Ramp(PixelRampAction::New(PixelRampConfig::default()));
    assert_eq!(a, [EditorAction::Pixel(quer)]);
}

#[test]
fn nos_cartoes_ligar_cycle_pede_o_ciclo() {
    let rampa = PixelRampInfo {
        id: 7,
        indices: vec![2, 3, 4],
        cycle: None,
    };
    let mut me = mesa(info(vec![rampa], Some(3)));
    assert!(me.rect(r::CYCLE).is_some(), "Cycle não pintado no cartão");
    let a = me.clica(r::CYCLE);
    let quer = PixelRequest::Ramp(PixelRampAction::Cycle {
        ramp: 7,
        cycle: Some(PixelCycle::default_for(3)),
    });
    assert_eq!(a, [EditorAction::Pixel(quer)]);
}
