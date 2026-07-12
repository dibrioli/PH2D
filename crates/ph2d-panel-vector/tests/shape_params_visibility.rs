//! **O bug do Enio, pelo PAINT** — "ao selecionar a seta, os parâmetros da última forma
//! criada continuam aparecendo no painel".
//!
//! A regra era `published.unwrap_or(active)`: sem forma VIVA em foco, os campos caíam na
//! forma ATIVA do catálogo. Certo para UM caso (nada selecionado ⇒ o default do próximo
//! traço), errado para todos os outros — selecionar um **conector**, ou uma curva comum já
//! convertida, deixava "Star: Points 5" pendurado sobre um objeto que não tem parâmetro
//! nenhum. **É uma classe, não um caso**, e é assim que este gate a varre.
//!
//! O teste morde onde um teste de unidade não chega: no `paint` de verdade
//! (`paint_hero_screen`, headless). O sinal é o **hit-rect**: o cabeçalho da seção e os
//! campos só o registram quando são desenhados. Reverter a regra para `unwrap_or(active)`
//! sai VERMELHO aqui.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::{
    ConnectorSnapshot, VectorPanel, set_current_connector, set_current_selection_count,
    set_current_shape_focus, set_current_vector_style,
};
use ph2d_text::TextSystem;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::DrawMode;
use ph2d_vec_scene::ShapeKind;
use ph2d_vector::VectorScene;
use std::sync::Once;

/// O painel Vector, docado e visível (é o que a shell faz enquanto a tool está ativa).
fn hero_with_vector_panel() -> HeroScreen {
    static INIT: Once = Once::new();
    ph2d_editor_core::test_support::ensure_panel_registry();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, true);
    hero
}

fn paint_frame(hero: &mut HeroScreen) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(
        hero,
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        &mut scene,
        &mut text,
    );
}

/// O estado que a shell publica num frame: a forma VIVA em foco (`None` = não é forma), a
/// contagem da seleção, o modo e a forma ATIVA do catálogo.
fn publish(live: Option<ShapeKind>, selected: usize, mode: DrawMode, active: ShapeKind) {
    set_current_shape_focus(live);
    set_current_selection_count(selected);
    set_current_vector_style(Some(VectorStyleSnapshot {
        shape: active,
        mode,
        ..VectorStyleSnapshot::default()
    }));
}

/// A seção de parâmetros de forma foi PINTADA neste frame? O cabeçalho é o sinal: ele
/// registra hit-rect sempre que a seção existe (e some inteiro quando ela não existe —
/// inclusive o separador, pelo `step` do orquestrador).
fn shape_params_painted(hero: &HeroScreen) -> bool {
    hero.hit_index
        .rect_for(ids::VECTOR_SECTION_SHAPE_PARAMS)
        .is_some()
}

/// O primeiro CAMPO da seção foi pintado? (A estrela tem parâmetros — Points / Inner —, então
/// o slot 0 existe quando ela está em foco.) Prova que não é só o cabeçalho que some.
fn first_shape_field_painted(hero: &HeroScreen) -> bool {
    hero.hit_index
        .rect_for(ids::vector_shape_field_id(0))
        .is_some()
}

/// **O BUG.** Um conector selecionado (a "seta" do Enio) não é forma viva — a seção de
/// parâmetros tem de SUMIR, e não cair no catálogo mostrando os campos da última forma criada.
#[test]
fn selecting_a_connector_hides_the_shape_params_instead_of_showing_the_last_shapes() {
    let mut hero = hero_with_vector_panel();

    // A shell publica: um conector na seleção (a seção Connector aparece), nenhuma forma viva
    // em foco, e o catálogo por acaso está na Estrela (a última forma criada).
    set_current_connector(Some(ConnectorSnapshot {
        route: 1,
        jetty: 0.35,
        spread: 0.0,
        corner: 0.0,
        curve: 1.0 / 3.0,
    }));
    publish(None, 1, DrawMode::Select, ShapeKind::Star);
    paint_frame(&mut hero);

    assert!(
        !shape_params_painted(&hero),
        "o conector selecionado NAO e uma forma viva: a secao de parametros de forma tem de \
         sumir. Ela esta pintando os campos da forma ATIVA do catalogo (o bug do Enio: \
         'os parametros da ultima forma criada continuam aparecendo')"
    );
    assert!(
        !first_shape_field_painted(&hero),
        "o campo 'Points' da Estrela continua clicavel sobre um conector — ele editaria o \
         default de desenho enquanto finge editar o objeto selecionado"
    );

    set_current_connector(None);
}

/// **E não é um caso, é uma CLASSE.** Uma curva comum (convertida — sem `VecShape`) também
/// não tem parâmetro nenhum: mesma regra, mesmo resultado. Sem conector na seleção desta vez,
/// então nem a seção Connector está em jogo — o que sobra é só a regra do foco.
#[test]
fn selecting_a_plain_curve_hides_the_shape_params_too() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    publish(None, 1, DrawMode::Select, ShapeKind::Star);
    paint_frame(&mut hero);
    assert!(
        !shape_params_painted(&hero),
        "uma curva comum selecionada nao tem parametros — a secao tem de sumir (mesma classe \
         de bug do conector)"
    );
}

/// O caso que a regra antiga acertava, e que a nova **não pode quebrar**: com a seleção
/// VAZIA, os campos são os da forma ATIVA do catálogo — o default do próximo traço.
#[test]
fn with_an_empty_selection_the_catalog_fields_still_show() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    publish(None, 0, DrawMode::Select, ShapeKind::Star);
    paint_frame(&mut hero);
    assert!(
        shape_params_painted(&hero),
        "sem NADA selecionado os campos sao o default do proximo traco — a secao tem de existir"
    );
    assert!(first_shape_field_painted(&hero));
}

/// A exceção: **armado para desenhar** (modo Shape), os campos são o default do próximo traço
/// mesmo com algo selecionado — escolher a forma no catálogo já põe a tool em Shape, sem
/// limpar a seleção, e sem isto o usuário não teria como configurar a forma que acabou de
/// escolher.
#[test]
fn arming_a_shape_keeps_the_catalog_fields_even_with_a_connector_selected() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(Some(ConnectorSnapshot {
        route: 0,
        jetty: 0.35,
        spread: 0.0,
        corner: 0.0,
        curve: 1.0 / 3.0,
    }));
    publish(None, 1, DrawMode::Shape, ShapeKind::Star);
    paint_frame(&mut hero);
    assert!(
        shape_params_painted(&hero),
        "no modo Shape os campos sao o default do PROXIMO traco — a secao tem de existir"
    );
    set_current_connector(None);
}

/// E o caso que define a feature Live Shape: uma forma VIVA selecionada traz os campos DELA,
/// mesmo na ferramenta Select (a seleção manda sobre o catálogo).
#[test]
fn a_selected_live_shape_still_shows_its_own_fields() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    publish(
        Some(ShapeKind::Star),
        1,
        DrawMode::Select,
        ShapeKind::Polygon,
    );
    paint_frame(&mut hero);
    assert!(shape_params_painted(&hero));
    assert!(
        first_shape_field_painted(&hero),
        "a forma viva selecionada tem de trazer os campos dela (o ciclo Live Shape)"
    );
}
