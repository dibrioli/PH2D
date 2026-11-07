//! ⭐ Onda 9b (GEF02): a aba central «Collision Layers» funciona SOZINHA, com o painel Physics fechado e
//! fora da ordem z dos painéis (um layout sem ele). Medido: o passeio já o alcança pela
//! `PANEL_Z_ORDER_FALLBACK` e pelo `pinta_com_a_aba_atras`, e o `paint` do painel pinta a aba antes de olhar
//! se ele está visível; este gate prende essas três peças juntas.
//! *Mutação: só pintar a aba com o painel visível (`paint.rs`) ⇒ vermelho.*

use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_panel_physics::{aba, ids};

#[test]
fn a_aba_das_camadas_pinta_sem_o_painel_physics_no_layout() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut h = HeroScreen::new(NodeId(1));
    let nomes: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in nomes {
        h.panel_visibility.insert(n, n == "inspector");
    }
    ph2d_panel_physics::set_current_physics(Some(Default::default()));
    ph2d_panel_physics::set_visible_layers(0b111);
    aba::abre(Some(1));
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, 900.0);
    for _ in 0..3 {
        h.store
            .retain_panel_z(|id| id != ph2d_editor_core::ids::PHYSICS_PANEL);
        aba::poe_area(Rect::new(300.0, 60.0, 900.0, 700.0));
        let mut cena = ph2d_vector::VectorScene::new();
        paint_hero_screen(&mut h, vp, &mut cena, &mut ts);
    }
    let cel = ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(2, 1)];
    assert!(
        h.hit_index.rect_for(cel).is_some(),
        "a célula (2, 1) tem de estar pintada e responder"
    );
    assert!(h.hit_index.rect_for(ids::PHYSICS_LAYER_BUMPS[2]).is_some());
    assert!(
        h.store
            .panel_rect(ph2d_editor_core::ids::PHYSICS_PANEL)
            .is_none(),
        "o painel não abre"
    );
    aba::fecha();
}
