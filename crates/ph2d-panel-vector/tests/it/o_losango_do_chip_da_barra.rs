//! ⭐ **O chip numérico da barra de controle (Opacity) tem o losango à direita** (plano 35 §5) —
//! pintado, registado POR CIMA do chip, e com os campos dele como alvo. O clique é o mesmo braço do
//! `dispatch_down` que o `seam_key_mark` prova no painel.

use std::collections::BTreeMap;

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, key_mark_target, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{self, Appearance};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_text::TextSystem;
use ph2d_vec_scene::BlendMode;
use ph2d_vector::VectorScene;
use std::sync::Once;

fn hero() -> HeroScreen {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, true);
    hero
}

fn quadro(hero: &mut HeroScreen) {
    state::set_current_appearance(Some(Appearance {
        opacity: 0.5,
        blend: BlendMode::Normal,
        layers: Vec::new(),
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
    // A barra de controle é PUBLICADA pelo módulo (a shell chama `bars::tool_bars` por quadro).
    ph2d_panel_vector::set_current_vector_style(Some(
        ph2d_tool_vector::VectorStyleSnapshot::default(),
    ));
    ph2d_panel_vector::set_current_selection_count(1);
    let barras = ph2d_panel_vector::bars::tool_bars(&hero.store, None, Vec::new());
    hero.store.set_tool_bars(Some(barras));
    paint_hero_screen(
        hero,
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        &mut VectorScene::new(),
        &mut TextSystem::without_system_fonts(),
    );
}

/// Com canal, o losango existe, cobre a ponta direita do chip e o rato acha-o lá; sem canal, o
/// chip é o slot inteiro e não há alvo. (Mutação: não registar em `tool_bar` ⇒ o `expect` dispara;
/// não encolher o chip em `number_with_key_mark` ⇒ o chip pinta por baixo do losango.)
#[test]
fn o_chip_de_opacity_da_barra_tem_losango() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let alvo = key_mark_id(ids::VECTOR_OBJ_OPACITY_NUM);
    let mut hero = hero();
    set_key_marks(BTreeMap::new());
    quadro(&mut hero);
    assert!(
        hero.hit_index.rect_for(alvo).is_none(),
        "sem canal, sem alvo"
    );

    set_key_marks(BTreeMap::from([(
        ids::VECTOR_OBJ_OPACITY_NUM,
        KeyMark::Chave,
    )]));
    quadro(&mut hero);
    let r = hero
        .hit_index
        .rect_for(alvo)
        .expect("o losango do chip da barra foi pintado mas não registado");
    let chip = hero
        .hit_index
        .rect_for(ids::VECTOR_OBJ_OPACITY_NUM)
        .expect("o chip da barra");
    assert!(
        (r.x + r.w - (chip.x + chip.w)).abs() < 0.5,
        "o losango fica na ponta direita do slot"
    );
    assert_eq!(
        hero.hit_index.hit(r.x + r.w * 0.5, r.y + r.h * 0.5),
        Some(alvo),
        "o rato acha o losango, não o chip"
    );
    assert_eq!(
        key_mark_target(alvo).map(|c| c.fields),
        Some(vec![ids::VECTOR_OBJ_OPACITY_NUM])
    );
    set_key_marks(BTreeMap::new());
    state::set_current_appearance(None);
}

/// ⭐ F24d — o chip de OPÇÃO da barra (a tinta do traço) também tem o losango quando tem canal, e o
/// rato acha-o na ponta direita. (Mutação: o braço `Compound` fora do `matches!` do `tool_bar` ⇒
/// o `expect` dispara.)
#[test]
fn o_chip_de_opcao_da_barra_tem_losango() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let alvo = key_mark_id(ids::VECTOR_BAR_KIND);
    let mut hero = hero();
    state::set_stroke_paint_kind(Some(state::StrokePaintKind::Solid));
    set_key_marks(BTreeMap::from([(ids::VECTOR_BAR_KIND, KeyMark::Animado)]));
    quadro(&mut hero);
    let r = hero
        .hit_index
        .rect_for(alvo)
        .expect("o losango do chip de opção foi pintado mas não registado");
    assert_eq!(
        hero.hit_index.hit(r.x + r.w * 0.5, r.y + r.h * 0.5),
        Some(alvo),
        "o rato acha o losango, não o chip"
    );
    assert_eq!(
        key_mark_target(alvo).map(|c| c.fields),
        Some(vec![ids::VECTOR_BAR_KIND])
    );
    set_key_marks(BTreeMap::new());
    state::set_stroke_paint_kind(None);
    state::set_current_appearance(None);
}
