//! ⭐ **«Edit curves on canvas»** (dono 10/11): a caixa de marcar PINTADA na barra liga e desliga, e o
//! painel publica-a à shell (`state::curves_on_canvas`) só na aba Keys e com o painel à vista; no
//! clássico (`PH2D_CARDS=0`) a caixa não existe. Gesto real sobre rects pintados.

use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::panel::{PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::slot::SlotSet;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{
    TimelinePanelState, curves_on_canvas, drain_intents, set_current_timeline,
};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 700.0);

fn mesa(cartoes: bool) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 24.0,
        view_length_seconds: 2.0,
        view_length_explicit: true,
        ..TimelineViewSnapshot::default()
    }));
    let mut m = Mesa::nova(TimelinePanelState::default());
    m.assenta_em(JANELA);
    let _ = drain_intents();
    let _ = m.host.drained_actions();
    m
}

fn tem(m: &Mesa<TimelinePanel>) -> bool {
    m.rect(ids::TIMELINE_CURVES_ON_CANVAS).is_some()
}

/// Pinta com o painel ESCONDIDO: o `paint` do arnês força-o visível, este não.
fn pinta_escondido(m: &mut Mesa<TimelinePanel>) {
    m.host.set_panel_visible(TimelinePanel::ID, false);
    let layout = HeroLayout::for_viewport(JANELA);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut texto = ph2d_text::TextSystem::without_system_fonts();
    let mut ctx = PaintCtx {
        host: &mut m.host,
        layout: &layout,
        slot: layout
            .slot_rects(SlotSet::of(TimelinePanel::DEFAULT_SLOT))
            .get(TimelinePanel::DEFAULT_SLOT),
        viewport: JANELA,
        scene: &mut scene,
        text_system: &mut texto,
    };
    TimelinePanel::paint(&mut m.st, &mut ctx);
}

/// Clicar a caixa pintada liga e publica; clicar outra vez desliga.
/// *Mutação: o braço `Toggled` que não inverte ⇒ vermelho.*
#[test]
fn clicar_a_caixa_liga_desliga_e_publica() {
    let mut m = mesa(true);
    assert!(tem(&m), "a caixa está na barra, aba Keys");
    assert!(!curves_on_canvas(), "começa desligada");
    m.clica(ids::TIMELINE_CURVES_ON_CANVAS);
    m.assenta_em(JANELA);
    assert!(m.st.curves_on_canvas);
    assert!(curves_on_canvas(), "ligada na aba Keys = publicada");
    m.clica(ids::TIMELINE_CURVES_ON_CANVAS);
    m.assenta_em(JANELA);
    assert!(!m.st.curves_on_canvas);
    assert!(!curves_on_canvas());
    ferramenta::force_cards_on_this_thread(None);
}

/// Fora da aba Keys a caixa some e a publicação é `false`, mesmo ligada; voltar a Keys religa.
/// *Mutação: `&& state.tab.shows_keys()` → `true` ⇒ vermelho.*
#[test]
fn fora_da_aba_keys_publica_false() {
    let mut m = mesa(true);
    m.clica(ids::TIMELINE_CURVES_ON_CANVAS);
    m.assenta_em(JANELA);
    assert!(curves_on_canvas());
    m.clica(ids::TIMELINE_TAB_ARRANGE);
    m.assenta_em(JANELA);
    assert!(!tem(&m), "fora de Keys a caixa não existe");
    assert!(m.st.curves_on_canvas, "a escolha fica na sessão");
    assert!(!curves_on_canvas(), "fora de Keys não se publica");
    m.clica(ids::TIMELINE_TAB_KEYS);
    m.assenta_em(JANELA);
    assert!(curves_on_canvas(), "de volta a Keys, ligada outra vez");
    ferramenta::force_cards_on_this_thread(None);
}

/// Com o painel escondido a publicação é `false`.
/// *Mutação: tirar o `publish_curves_on_canvas(false)` da lista do painel escondido ⇒ vermelho.*
#[test]
fn com_o_painel_escondido_publica_false() {
    let mut m = mesa(true);
    m.clica(ids::TIMELINE_CURVES_ON_CANVAS);
    m.assenta_em(JANELA);
    assert!(curves_on_canvas());
    pinta_escondido(&mut m);
    assert!(
        !curves_on_canvas(),
        "painel escondido não entorta caminho nenhum"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// No clássico (`PH2D_CARDS=0`) a caixa não existe.
#[test]
fn no_classico_nao_ha_caixa() {
    let m = mesa(false);
    assert!(!tem(&m));
    assert!(!curves_on_canvas());
    ferramenta::force_cards_on_this_thread(None);
}
