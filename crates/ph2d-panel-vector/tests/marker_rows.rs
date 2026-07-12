//! **As linhas de PONTA da seção Stroke, pelo PAINT** — Head Size, Head Round e Both Ends.
//!
//! Três fatos que só a pintura prova:
//!
//! 1. **Os três controles existem e são clicáveis** (têm hit-rect). Um controle pintado sem
//!    hit-rect — ou sem registro no `populate` — é o "verde-e-morto" clássico do projeto.
//! 2. **As caixas nascem no valor EFETIVO** da tool, não em `0`. `Head Size = 0` é uma seta
//!    INVISÍVEL: o campo saltaria para lá no primeiro toque. (O mesmo cuidado da seção
//!    Connector.)
//! 3. **Both Ends é DERIVADO** das duas pontas — o rótulo do botão sai de `both_ends()`, não
//!    de um flag guardado. Duas fontes de verdade sobre "é bidirecional?" divergiriam no
//!    instante em que o usuário trocasse uma ponta pelo chip.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::{VectorPanel, set_current_vector_style};
use ph2d_text::TextSystem;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{MARKER_ROUND, MARKER_SCALE, both_ends_label};
use ph2d_vec_scene::Marker;
use ph2d_vector::VectorScene;
use std::sync::Once;

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

/// O Style que a shell publica (o que a tool tem armado).
fn publish(start: Marker, end: Marker, scale: f64, round: f64) {
    set_current_vector_style(Some(VectorStyleSnapshot {
        marker_start: start,
        marker_end: end,
        marker_scale: scale,
        marker_round: round,
        ..VectorStyleSnapshot::default()
    }));
}

/// **Os três controles nascem vivos** — pintados E clicáveis.
#[test]
fn head_size_head_round_and_both_ends_are_painted_and_clickable() {
    let mut hero = hero_with_vector_panel();
    publish(Marker::None, Marker::None, 1.0, 0.0);
    paint_frame(&mut hero);

    for (id, what) in [
        (ids::VECTOR_MARKER_SCALE, "Head Size"),
        (ids::VECTOR_MARKER_ROUND, "Head Round"),
        (ids::VECTOR_MARKER_BOTH, "Both Ends"),
    ] {
        assert!(
            hero.hit_index.rect_for(id).is_some(),
            "{what} nao registrou hit-rect: o controle esta MORTO (pintado e sem clique)"
        );
    }
}

/// **As caixas nascem no EFETIVO, não em `0`.** Uma cabeça de tamanho zero é uma seta
/// invisível — e o primeiro toque no campo a produziria.
#[test]
fn the_head_boxes_are_born_showing_the_effective_value_not_zero() {
    let mut hero = hero_with_vector_panel();
    publish(Marker::None, Marker::Triangle, 2.5, 0.4);
    paint_frame(&mut hero);

    let scale = hero
        .store
        .number_value(ids::VECTOR_MARKER_SCALE)
        .expect("Head Size tem de estar registrado no populate");
    let round = hero
        .store
        .number_value(ids::VECTOR_MARKER_ROUND)
        .expect("Head Round tem de estar registrado no populate");
    assert!(
        (scale - 2.5).abs() < 1e-9,
        "o Head Size nasceu em {scale}, e nao no efetivo (2.5) — o campo vai SALTAR no \
         primeiro toque"
    );
    assert!((round - 0.4).abs() < 1e-9, "o Head Round nasceu em {round}");

    // E acompanha o frame seguinte (o seed é por-frame, como o do conector — a tool muda
    // quando o usuário seleciona outro caminho).
    publish(Marker::None, Marker::Triangle, 0.75, 0.0);
    paint_frame(&mut hero);
    let scale = hero
        .store
        .number_value(ids::VECTOR_MARKER_SCALE)
        .expect("registrado");
    assert!(
        (scale - 0.75).abs() < 1e-9,
        "o campo nao acompanhou o novo valor publicado: {scale}"
    );
}

/// **A FAIXA está registrada** (`set_number_range`) — sem ela o arrasto escala errado, o
/// gotcha conhecido da caixa limitada. O sinal observável: um valor fora da faixa publicado
/// pela tool é CLAMPADO ao entrar no campo.
#[test]
fn the_head_boxes_declare_their_range() {
    let mut hero = hero_with_vector_panel();
    publish(Marker::None, Marker::Triangle, 99.0, 9.0);
    paint_frame(&mut hero);
    let scale = hero
        .store
        .number_value(ids::VECTOR_MARKER_SCALE)
        .expect("registrado");
    let round = hero
        .store
        .number_value(ids::VECTOR_MARKER_ROUND)
        .expect("registrado");
    assert!(
        (scale - MARKER_SCALE.max).abs() < 1e-9,
        "o Head Size nao saturou no teto da faixa: {scale}"
    );
    assert!(
        (round - MARKER_ROUND.max).abs() < 1e-9,
        "o Head Round nao saturou no teto da faixa: {round}"
    );
}

/// **Both Ends é DERIVADO.** O rótulo do botão é função das duas pontas — nunca de um flag.
#[test]
fn both_ends_is_derived_from_the_two_markers() {
    // A tabela inteira: só as DUAS pontas presentes contam como bidirecional.
    for (start, end, expected) in [
        (Marker::None, Marker::None, false),
        (Marker::None, Marker::Triangle, false),
        (Marker::Triangle, Marker::None, false),
        (Marker::Triangle, Marker::Triangle, true),
        (Marker::Diamond, Marker::Triangle, true),
    ] {
        let snap = VectorStyleSnapshot {
            marker_start: start,
            marker_end: end,
            ..VectorStyleSnapshot::default()
        };
        assert_eq!(
            snap.both_ends(),
            expected,
            "bidirecional({start:?}, {end:?}) deveria ser {expected}"
        );
    }
    // E os dois rótulos são distintos (um botão que diz a mesma coisa nos dois estados não
    // comunica nada).
    assert_ne!(both_ends_label(true), both_ends_label(false));
}
