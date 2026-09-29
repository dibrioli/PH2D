//! Os gates da porta da rolagem. Cada um nomeia a mutação que o deve fazer sangrar.

use super::*;
use ph2d_a11y::NodeId;

const PANEL: NodeId = NodeId(9_000_001);
const BAR: NodeId = NodeId(9_000_002);
const ROW: NodeId = NodeId(9_000_003);

fn body() -> Rect {
    Rect::new(0.0, 100.0, 300.0, 400.0)
}

/// Abre, regista uma fileira a `y`, fecha com `content_h`.
fn frame(store: &mut WidgetStore, hit: &mut HitIndex, row_y: f32, content_h: f32) {
    let mut scene = VectorScene::new();
    let area = open_with(&mut scene, hit, store, PANEL, BAR, body());
    hit.register(ROW, Rect::new(10.0, row_y, 200.0, 20.0));
    close_with(area, &mut scene, hit, store, content_h, Theme::Forge);
}

/// ⭐⭐⭐ **O que se regista é a TRILHA, e o dono dela fica publicado.**
///
/// Mutação: registar o polegar (a forma dos 14 painéis de antes) · não publicar o dono.
#[test]
fn the_door_registers_the_whole_track_and_publishes_its_owner() {
    let (mut store, mut hit) = (WidgetStore::default(), HitIndex::new());
    frame(&mut store, &mut hit, 120.0, 2000.0);
    let track = hit.rect_for(BAR).expect("a barra tem de estar registada");
    assert_eq!(track.h, body().h, "registou {track:?}, não a trilha inteira");
    assert_eq!(store.scroll_bar_panel(BAR), Some(PANEL));
    assert_eq!(store.scroll_bar_track(BAR), Some(track));
}

/// **Sem transbordo não há barra** — nem desenho, nem registo, nem dono.
///
/// Mutação: registar a trilha fora do `is_needed`.
#[test]
fn no_overflow_no_bar() {
    let (mut store, mut hit) = (WidgetStore::default(), HitIndex::new());
    frame(&mut store, &mut hit, 120.0, 300.0);
    assert_eq!(hit.rect_for(BAR), None);
    assert_eq!(store.scroll_bar_track(BAR), None);
}

/// **As duas alturas são publicadas** — é delas que a roda, o corpo e a inércia tiram o fim.
///
/// Mutação: apagar um dos dois `set_panel_*_h` (a forma da Hierarquia de antes).
#[test]
fn both_heights_are_published() {
    let (mut store, mut hit) = (WidgetStore::default(), HitIndex::new());
    frame(&mut store, &mut hit, 120.0, 2000.0);
    assert_eq!(store.panel_content_h(PANEL), Some(2000.0));
    assert_eq!(store.panel_visible_h(PANEL), Some(400.0));
}

/// **Uma fileira rolada para fora do corpo não se clica.**
///
/// Mutação: apagar o `hit_index.push_clip` do `open_with`.
#[test]
fn a_row_scrolled_out_of_the_body_is_not_clickable() {
    let (mut store, mut hit) = (WidgetStore::default(), HitIndex::new());
    // A fileira está na faixa do TÍTULO (acima do corpo), onde uma lista rolada a deixa.
    frame(&mut store, &mut hit, 60.0, 2000.0);
    assert_ne!(hit.hit(50.0, 70.0), Some(ROW));
    // Controlo: dentro do corpo ela clica.
    let (mut store, mut hit) = (WidgetStore::default(), HitIndex::new());
    frame(&mut store, &mut hit, 150.0, 2000.0);
    assert_eq!(hit.hit(50.0, 160.0), Some(ROW));
}

/// **O alvo volta para dentro quando o conteúdo encolhe.**
///
/// Mutação: apagar o clamp do `publish`.
#[test]
fn the_target_is_clamped_when_the_content_shrinks() {
    let (mut store, mut hit) = (WidgetStore::default(), HitIndex::new());
    store.set_panel_scroll(PANEL, 1500.0);
    frame(&mut store, &mut hit, 120.0, 900.0);
    assert_eq!(store.panel_scroll_target(PANEL), 500.0);
}

/// **Carregar na trilha põe o CENTRO do polegar debaixo do dedo** — nas pontas, no topo e no fim.
///
/// Mutação: esquecer o `thumb_h * 0.5` · usar `track.h` como alcance.
#[test]
fn a_press_on_the_track_centres_the_thumb_under_the_finger() {
    let track = Rect::new(0.0, 0.0, 10.0, 400.0);
    // 2000 de conteúdo em 400 de vista ⇒ polegar de 80, alcance de 320, 1600 de rolagem.
    assert_eq!(scroll_for_track_press(track, 0.0, 2000.0, 400.0), 0.0);
    assert_eq!(scroll_for_track_press(track, 400.0, 2000.0, 400.0), 1600.0);
    let mid = scroll_for_track_press(track, 200.0, 2000.0, 400.0);
    assert!((mid - 800.0).abs() < 0.01, "o meio da trilha é o meio da lista; deu {mid}");
}
