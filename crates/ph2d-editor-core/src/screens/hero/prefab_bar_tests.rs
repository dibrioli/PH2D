//! Os gates da barra do modo de receita.
//!
//! ⚠️ **A régua principal é a SAÍDA**, e ela é medida pela porta que o app usa
//! ([`HeroScreen::apply_event`]) — não pelo handler. Um gate que chamasse o handler direto ficaria
//! verde com ele **fora** da cadeia gerada do `chrome::dispatch_all`, que é exactamente o estado em
//! que o botão nasce pintado, registado e **morto sob o ponteiro**.

use super::{PrefabEditView, PrefabExit, bar_rect, cancel_rect, done_rect, title};
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

/// ⛔ **Os DOIS botões têm estado no store** — sem ele o `is_focusable` responde `false`, o botão
/// nunca vira `active` no Down e **nunca emite `Click`**: pintado, hit-registado e morto sob o
/// ponteiro.
#[test]
fn the_exit_buttons_are_alive_in_the_store() {
    let hero = HeroScreen::new(NodeId(1));
    for (id, nome) in [
        (ids::PREFAB_EDIT_DONE, "Done"),
        (ids::PREFAB_EDIT_CANCEL, "Cancel"),
    ] {
        assert!(
            hero.store.get(id).is_some(),
            "o `{nome}` da barra do modo nao esta' no store"
        );
    }
}

/// ⭐⭐ **O `Cancel` fica à ESQUERDA do `Done`**, e os dois dentro da barra.
///
/// ⚠️ A ordem é a do sistema operativo e a de todo diálogo desta casa: a acção destrutiva à
/// esquerda, a de confirmação encostada ao canto. Trocá-las faz a mão que decorou o canto
/// **desfazer** o trabalho ao tentar guardá-lo.
#[test]
fn the_cancel_sits_left_of_the_done_and_both_fit_inside() {
    let mut text = TextSystem::without_system_fonts();
    let bar = bar_rect(AREA, &mut text, &view(4));
    let done = done_rect(bar);
    let cancel = cancel_rect(bar);
    assert!(
        cancel.x + cancel.w <= done.x + 0.01,
        "o Cancel encavalitou o Done: {cancel:?} contra {done:?}"
    );
    assert!(
        cancel.x >= bar.x && done.x + done.w <= bar.x + bar.w + 0.01,
        "um dos botoes saiu da barra"
    );
}

/// ⭐⭐⭐ **Cada botão PEDE a sua saída** — e o pedido chega ao campo que a shell serve.
///
/// ⚠️ **Pela porta do app** (`apply_event`), que é o que prende o handler à cadeia gerada do
/// `chrome::dispatch_all`. Um gate que chamasse o handler direto ficaria verde com o botão fora
/// dela — pintado, registado e morto sob o ponteiro.
///
/// ⚠️ **E os dois pedidos são DISTINTOS**: um `Cancel` que chegasse como `Done` sairia da sessão
/// **guardando** o que o artista mandou deitar fora, e nada na tela diria porquê.
///
/// **Mutação que deve sangrar:** o `z` do handler sair da cadeia gerada, ou os dois braços
/// colapsarem num só.
#[test]
fn each_button_asks_for_its_own_exit() {
    for (id, want) in [
        (ids::PREFAB_EDIT_DONE, PrefabExit::Done),
        (ids::PREFAB_EDIT_CANCEL, PrefabExit::Cancel),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        assert!(
            hero.apply_event(WidgetEvent::Click(id)),
            "o clique nao foi consumido por ninguem — o handler nao esta' na cadeia"
        );
        assert_eq!(
            hero.prefab_exit,
            Some(want),
            "o botao pediu a saida errada (ou nenhuma)"
        );
    }
}

/// ⭐⭐⭐ **A barra pousa ABAIXO DA RÉGUA** (report do Enio, 2026-09-07: *«agora está em cima da
/// régua»*).
///
/// ⚠️ A lei é pura e recebe um rect — quem escolhe QUAL rect é o chamador, e é lá que o defeito
/// vive. ⛔ Por isso este gate lê o sítio da chamada: a `draw_area` inclui a faixa das réguas; o
/// `last_content` é o que sobra depois delas.
#[test]
fn the_bar_hangs_below_the_ruler_not_over_it() {
    let paint = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/screens/hero/paint.rs"),
    )
    .expect("paint.rs");
    let at = paint
        .find("prefab_bar::paint(")
        .expect("a barra deixou de ser pintada");
    let arm = &paint[at..(at + 200).min(paint.len())];
    assert!(
        arm.contains("hero.last_content"),
        "a barra voltou a ancorar na `draw_area`, que inclui a faixa das reguas:\n{arm}"
    );
}
