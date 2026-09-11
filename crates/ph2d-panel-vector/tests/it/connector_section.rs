//! **A seção CONNECTOR, pelo PAINT** — o gate que um teste de unidade não dá.
//!
//! Os dois fatos que só a pintura prova:
//!
//! 1. **A seção aparece SSE há um conector na seleção.** O snapshot publicado pela shell é
//!    a única chave: com ele, os três campos registram hit-rect (existem, e são clicáveis);
//!    sem ele, NENHUM registra — a seção some inteira, cabeçalho incluso. Inverter a
//!    condição do `paint` sai vermelho nos dois lados.
//! 2. **O campo nasce mostrando o valor EFETIVO** (o automático que a shell resolveu), e não
//!    `0`. É a diferença entre um slider que já está onde o olho vê a linha e um que SALTA
//!    no primeiro toque — que é exatamente o que o Enio precisa para calibrar com o olho.
//!
//! Roda o `paint_hero_screen` de verdade (headless, sem GPU: a `VectorScene` é uma
//! codificação em CPU e o `TextSystem` dispensa as fontes do sistema).

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::{ConnectorSnapshot, VectorPanel, set_current_connector};
use ph2d_text::TextSystem;
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

/// Um frame de pintura completo do painel.
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

/// Os campos do conector estão pintados (têm hit-rect) neste frame?
/// `[Route, Jetty, Spread, Corner]`.
fn connector_fields_painted(hero: &HeroScreen) -> [bool; 4] {
    [
        hero.hit_index.rect_for(ids::VECTOR_CONNECTOR_ROUTE),
        hero.hit_index.rect_for(ids::VECTOR_CONNECTOR_JETTY),
        hero.hit_index.rect_for(ids::VECTOR_CONNECTOR_SPREAD),
        hero.hit_index.rect_for(ids::VECTOR_CONNECTOR_CORNER),
    ]
    .map(|r| r.is_some())
}

/// **A seção aparece SSE há conector na seleção** — o teste bilateral. Sem snapshot, nenhum
/// dos três campos existe (nem o cabeçalho); com snapshot, os três registram hit-rect e o
/// clique tem onde cair. Um controle pintado sem hit-rect é o bug "verde-e-morto" clássico
/// do projeto; um controle que aparece sempre é uma UI que mente sobre o objeto selecionado.
#[test]
fn the_connector_section_is_painted_only_when_a_connector_is_selected() {
    let mut hero = hero_with_vector_panel();

    // 1. Sem conector selecionado (o caso comum: desenhando formas).
    set_current_connector(None);
    paint_frame(&mut hero);
    assert_eq!(
        connector_fields_painted(&hero),
        [false; 4],
        "sem conector na selecao a secao Connector NAO pode existir — \
         ela esta pintando os campos de um objeto que nao e conector"
    );

    // 2. Um conector selecionado: a shell publica o snapshot (com os valores EFETIVOS).
    set_current_connector(Some(ConnectorSnapshot {
        route: 1,
        jetty: 0.35,
        spread: 0.0,
        corner: 0.0,
        curve: 1.0 / 3.0,
    }));
    paint_frame(&mut hero);
    assert_eq!(
        connector_fields_painted(&hero),
        [true; 4],
        "com um conector selecionado os quatro campos (Route/Jetty/Spread/Corner) tem de \
         estar pintados E clicaveis — um deles sem hit-rect e um controle MORTO"
    );

    // 3. E a seção some de novo quando a seleção muda (não fica presa pintada).
    set_current_connector(None);
    paint_frame(&mut hero);
    assert_eq!(
        connector_fields_painted(&hero),
        [false; 4],
        "a secao ficou presa na tela depois que o conector saiu da selecao"
    );
}

/// **O campo nasce no valor EFETIVO.** A shell publica o automático (o jetty derivado do
/// tamanho das caixas); o `paint` o semeia no store — que é de onde a caixa numérica lê e de
/// onde o arrasto parte. Um campo semeado em `0` faria o número SALTAR no primeiro toque,
/// para longe da linha que está na tela.
#[test]
fn the_fields_are_born_showing_the_effective_value_not_zero() {
    let mut hero = hero_with_vector_panel();
    // Valores EFETIVOS típicos: o jetty automático de duas caixas e o spread do 2º conector
    // paralelo (que já nasce deslocado — e é esse deslocamento que o campo tem de mostrar).
    let effective_jetty = 0.7;
    let effective_spread = -0.35;
    // A quina do PERCURSO: o valor autorado (nao ha automatico a resolver — `0` = afiado).
    let effective_corner = 0.3;
    set_current_connector(Some(ConnectorSnapshot {
        route: 0,
        jetty: effective_jetty,
        spread: effective_spread,
        corner: effective_corner,
        curve: 1.0 / 3.0,
    }));
    paint_frame(&mut hero);

    let jetty = hero
        .store
        .number_value(ids::VECTOR_CONNECTOR_JETTY)
        .expect("o campo Jetty tem de estar registrado no populate");
    let spread = hero
        .store
        .number_value(ids::VECTOR_CONNECTOR_SPREAD)
        .expect("o campo Spread tem de estar registrado no populate");
    assert!(
        (jetty - effective_jetty).abs() < 1e-9,
        "o Jetty nasceu em {jetty}, e nao no EFETIVO ({effective_jetty}): \
         o slider vai SALTAR no primeiro toque"
    );
    assert!(
        (spread - effective_spread).abs() < 1e-9,
        "o Spread nasceu em {spread}, e nao no EFETIVO ({effective_spread})"
    );
    let corner = hero
        .store
        .number_value(ids::VECTOR_CONNECTOR_CORNER)
        .expect("o campo Corner tem de estar registrado no populate");
    assert!(
        (corner - effective_corner).abs() < 1e-9,
        "o Corner nasceu em {corner}, e nao no valor do conector ({effective_corner})"
    );

    // E o valor publicado no frame SEGUINTE (a forma cresceu, o automático mudou) segue o
    // campo — o seed é por-frame, não uma vez só.
    set_current_connector(Some(ConnectorSnapshot {
        route: 0,
        jetty: 0.9,
        spread: effective_spread,
        corner: effective_corner,
        curve: 1.0 / 3.0,
    }));
    paint_frame(&mut hero);
    let jetty = hero
        .store
        .number_value(ids::VECTOR_CONNECTOR_JETTY)
        .expect("registrado");
    assert!(
        (jetty - 0.9).abs() < 1e-9,
        "o campo nao acompanhou o novo valor efetivo publicado pela shell: {jetty}"
    );

    set_current_connector(None);
}
