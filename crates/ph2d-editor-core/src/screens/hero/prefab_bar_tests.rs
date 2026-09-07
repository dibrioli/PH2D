//! Os gates da barra do modo de receita.
//!
//! ⚠️ **A régua principal é a SAÍDA**, e ela é medida pela porta que o app usa
//! ([`HeroScreen::apply_event`]) — não pelo handler. Um gate que chamasse o handler direto ficaria
//! verde com ele **fora** da cadeia gerada do `chrome::dispatch_all`, que é exactamente o estado em
//! que o botão nasce pintado, registado e **morto sob o ponteiro**.

use super::{PrefabEditView, bar_rect, done_rect, title};
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;

fn view(copies: usize) -> PrefabEditView {
    PrefabEditView {
        name: "Car".to_string(),
        copies,
    }
}

/// A área de desenho de uma janela com uma coluna docada à esquerda.
const AREA: Rect = Rect {
    x: 400.0,
    y: 40.0,
    w: 1200.0,
    h: 860.0,
};

/// ⭐⭐ **A barra diz O QUÊ e QUANTOS — e o plural é escolhido, nunca concatenado.**
///
/// *«1 copies follow»* é o erro que toda contagem de UI comete uma vez, e ele lê-se como um defeito
/// do app inteiro.
#[test]
fn the_bar_names_the_prefab_and_counts_who_follows() {
    assert!(title(&view(3)).contains("Car"), "o nome tem de aparecer");
    assert!(title(&view(3)).contains("3 copies follow"));
    assert!(
        title(&view(1)).contains("1 copy follows"),
        "plural concatenado: {}",
        title(&view(1))
    );
    assert!(
        title(&view(0)).contains("no copies yet"),
        "zero copias e' informacao, nao um erro: {}",
        title(&view(0))
    );
}

/// ⭐⭐⭐ **A barra pendura-se na ÁREA DE DESENHO, não na janela.**
///
/// O canvas é *full-bleed* e os painéis flutuam por cima: ancorar na janela poria a barra debaixo da
/// barra de menus e descentrada em relação ao que o artista vê.
///
/// **Mutação que deve sangrar:** trocar `draw_area` pelo viewport no chamador do pintor.
#[test]
fn the_bar_hangs_from_the_drawing_area_not_the_window() {
    let mut text = TextSystem::without_system_fonts();
    let bar = bar_rect(AREA, &mut text, &view(3));
    let centre = bar.x + bar.w * 0.5;
    let want = AREA.x + AREA.w * 0.5;
    assert!(
        (centre - want).abs() < 1.0,
        "a barra esta' centrada em {centre:.1} e a area em {want:.1}"
    );
    assert!(
        bar.y >= AREA.y && bar.y + bar.h <= AREA.y + AREA.h,
        "a barra saiu da area de desenho: {bar:?}"
    );
}

/// ⚠️ **O botão vive DENTRO da barra** — um `Done` a transbordar seria clicável fora do fundo que o
/// desenha, e o artista veria um clique morto ao lado do botão.
#[test]
fn the_done_button_lives_inside_the_bar() {
    let mut text = TextSystem::without_system_fonts();
    let bar = bar_rect(AREA, &mut text, &view(12));
    let done = done_rect(bar);
    assert!(
        done.x >= bar.x
            && done.y >= bar.y
            && done.x + done.w <= bar.x + bar.w + 0.01
            && done.y + done.h <= bar.y + bar.h + 0.01,
        "o botao saiu da barra: {done:?} em {bar:?}"
    );
}

/// ⛔ **O botão tem estado no store** — sem ele o `is_focusable` responde `false`, ele nunca vira
/// `active` no Down e **nunca emite `Click`**: pintado, hit-registado e morto sob o ponteiro.
#[test]
fn the_exit_button_is_alive_in_the_store() {
    let hero = HeroScreen::new(NodeId(1));
    assert!(
        hero.store.get(ids::PREFAB_EDIT_DONE).is_some(),
        "o `Done` da barra do modo nao esta' no store"
    );
}

/// ⭐⭐⭐ **Carregar em `Done` LARGA A SELECÇÃO** — e é isso que fecha o modo inteiro (a marca
/// `MasterEditing` é derivada dela, e com ela caem o vidro, o palco e a receita na cena).
///
/// ⚠️ **Pela porta do app** (`apply_event`), que é o que prende o handler à cadeia gerada do
/// `chrome::dispatch_all`.
///
/// **Mutação que deve sangrar:** apagar a linha do `prefab_bar` do `dispatch_all` (o gerador
/// re-escreve-a, mas um `z` removido não), ou o handler deixar de largar a selecção.
#[test]
fn clicking_done_drops_the_selection_and_closes_the_mode() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.replace_selection(Some(0x0BEE));
    assert!(
        hero.apply_event(WidgetEvent::Click(ids::PREFAB_EDIT_DONE)),
        "o clique no `Done` nao foi consumido por ninguem — o handler nao esta' na cadeia"
    );
    assert_eq!(
        hero.gizmo.selection, None,
        "o modo nao fechou: a receita continua seleccionada, logo continua aberta"
    );
}
