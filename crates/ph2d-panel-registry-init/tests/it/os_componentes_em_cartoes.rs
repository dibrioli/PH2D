//! ⭐ **CartoesUI C4: os componentes do objecto em cartões** — um objecto com física, vida… perdia-os
//! no modo cartões. Agora cada secção de componente é um cartão com o título dela; aberto, o corpo
//! de hoje (os mesmos ids) pinta-se dentro dele. No hero montado, com o Inspector armado com TODAS
//! as secções (a fixtura de `o_inspector_armado`) menos a sprite (a sprite fica no Inspector de hoje).
#![cfg(feature = "panel-inspector")]

use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn pinta(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..30 {
        scene.reset();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || {
                ph2d_editor_core::screens::hero::paint_hero_screen(
                    h, VIEWPORT, &mut scene, &mut text,
                )
            },
        );
    }
}

fn clique(h: &mut HeroScreen, x: f32, y: f32) {
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
                .to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

#[test]
fn every_component_section_becomes_a_card_with_its_body_inside() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    super::o_inspector_armado::arma_tudo();
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h);
    let seccao = ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION;
    let titulo = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(seccao))
        .expect("a física não virou cartão");
    // Controlo: dezenas de componentes armados ⇒ dezenas de cartões (não só a física); os de baixo
    // ficam fora da janela e chegam pela rolagem.
    let cartoes = ph2d_editor_core::ids::LIVE_SECTIONS
        .iter()
        .filter(|(s, _)| *s != seccao && h.store.is_card_section(*s))
        .count();
    assert!(cartoes >= 10, "só {cartoes} secções viraram cartão");
    let insp = ph2d_editor_core::ids::INSP_PANEL;
    assert!(
        h.store.panel_content_h(insp) > h.store.panel_visible_h(insp),
        "a lista longa não transborda para a rolagem"
    );
    let corpo = ph2d_panel_inspector::ids::INSP_PHYS_KIND[0];
    if h.store.is_collapsed(seccao) {
        assert!(
            h.hit_index.rect_for(corpo).is_none(),
            "cartão dobrado com o corpo à vista"
        );
        clique(&mut h, titulo.x + 10.0, titulo.y + titulo.h * 0.5);
        // A dobra do corpo de hoje anima pelo tique do app, que o teste não corre: dá-a por acabada.
        h.store.set_section_open_live(seccao, 1.0);
        pinta(&mut h);
    }
    assert!(
        !h.store.is_collapsed(seccao),
        "o clique no título não abriu o cartão"
    );
    let r = h
        .hit_index
        .rect_for(corpo)
        .expect("aberto, o cartão da física não pintou o corpo de hoje");
    let titulo = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(seccao))
        .expect("o título da física");
    assert!(
        r.y > titulo.y,
        "o corpo {r:?} não está abaixo do título {titulo:?}"
    );
    super::o_inspector_armado::desarma_tudo();
}
