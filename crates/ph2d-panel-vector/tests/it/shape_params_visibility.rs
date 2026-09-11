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

/// Com a seleção VAZIA **e a ferramenta a desenhar uma forma**, os campos são os da forma armada
/// — o default do próximo traço.
#[test]
fn with_an_empty_selection_the_armed_fields_still_show() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    publish(None, 0, DrawMode::Shape, ShapeKind::Star);
    paint_frame(&mut hero);
    assert!(
        shape_params_painted(&hero),
        "armado para desenhar, os campos sao o default do proximo traco — a secao tem de existir"
    );
    assert!(first_shape_field_painted(&hero));
}

/// ⭐⭐⭐ **O REPORT (Enio, 2026-08-31, com foto):** *"Mesmo com outras ferramentas selecionadas, as
/// Shapes ficam expostas e as propriedades das shapes também."*
///
/// Na ferramenta **Select**, sem nada selecionado, o painel oferecia *"ROUND / Radius"* — os
/// parâmetros do próximo traço numa ferramenta que não desenha traço nenhum. ⚠️ A regra antiga era
/// *"a seleção está VAZIA ⇒ mostre o catálogo"*, e **a pergunta nunca foi quantos objetos estão
/// selecionados**: é se a ferramenta na mão vai desenhar uma forma.
#[test]
fn an_empty_selection_in_a_tool_that_draws_nothing_shows_no_shape_fields() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    for mode in [
        DrawMode::Select,
        DrawMode::Node,
        DrawMode::Pen,
        DrawMode::Pencil,
        DrawMode::Cut,
        DrawMode::Width,
    ] {
        publish(None, 0, mode, ShapeKind::Star);
        paint_frame(&mut hero);
        assert!(
            !shape_params_painted(&hero),
            "{mode:?} nao desenha forma nenhuma — os parametros da Estrela nao tem o que descrever"
        );
        assert!(!first_shape_field_painted(&hero));
    }
    // Controle: a MESMA seleção vazia nas duas ferramentas que DESENHAM uma forma mostra os
    // campos — senão este gate passaria por a seção estar morta, e não pela lei.
    for mode in [DrawMode::Shape, DrawMode::Frame] {
        publish(None, 0, mode, ShapeKind::Star);
        paint_frame(&mut hero);
        assert!(
            shape_params_painted(&hero),
            "{mode:?} desenha uma forma — os campos dela tem de estar la'"
        );
    }
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

/// Quantos slots de campo esta seção pintou? (Um `rect_for` por índice — é o que separa
/// *"a seção existe"* de *"a seção é DAQUELA forma"*.)
fn shape_fields_painted(hero: &HeroScreen) -> usize {
    (0..ph2d_panel_vector::ids::MAX_SHAPE_FIELD_SLOTS)
        .filter(|&i| {
            hero.hit_index
                .rect_for(ids::vector_shape_field_id(i))
                .is_some()
        })
        .count()
}

/// ⭐⭐ **A MOLDURA desenha um `RoundRect`, e os campos têm de ser DELE** — não os do botão
/// aceso do catálogo.
///
/// ⚠️ **A metade da SEMENTE já lia o kind efectivo** (`vector_bridge::shape_catalog` chama
/// `DrawMode::shape_kind`) e esta metade lia o cru: sem alvo vivo, a shell escrevia nas caixas os
/// VALORES do `RoundRect` enquanto a seção pintava os CAMPOS da forma do catálogo. Com o catálogo
/// na Estrela, o painel oferecia *"Star: Points"* com o raio de quina dentro. *Cada metade estava
/// certa sozinha, e nenhum gate as comparava.*
#[test]
fn the_frame_mode_shows_the_round_rect_fields_not_the_lit_catalog_buttons() {
    use ph2d_tool_vector::shapes;
    // A fixture CONTÉM o fenômeno: as duas formas têm contagens de campo diferentes, senão
    // este gate passaria com o produto errado.
    let cru = shapes::desc(ShapeKind::Rectangle).fields.len();
    let efectivo = shapes::desc(ShapeKind::RoundRect).fields.len();
    assert_ne!(
        cru, efectivo,
        "a fixture precisa de duas formas distinguiveis"
    );

    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    // Moldura armada, nada selecionado, e o catálogo por acaso no Rect (que NÃO tem campo nenhum).
    publish(None, 0, DrawMode::Frame, ShapeKind::Rectangle);
    paint_frame(&mut hero);

    assert!(
        shape_params_painted(&hero),
        "a Moldura desenha um RoundRect: a secao tem de existir mesmo com o catalogo no Rect"
    );
    assert_eq!(
        shape_fields_painted(&hero),
        efectivo,
        "a Moldura pintou os campos da forma CRUA do catalogo ({cru} campos) em vez dos do \
         RoundRect que o gesto desenha ({efectivo}) — e a semente ja escrevia os valores do \
         RoundRect nesses mesmos slots"
    );
}
