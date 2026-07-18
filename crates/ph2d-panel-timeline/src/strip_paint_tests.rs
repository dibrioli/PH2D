//! Gates da geometria das **change bars** — o traço que cada quina deixa depois de
//! esticar ou cortar (Enio, 2026-07-16).
//!
//! O que se afirma aqui é o **LADO** em que a barra cai, não o pixel: o sinal da marca
//! é a única coisa que decide se ela sai pra fora ou fica pra dentro, e é exatamente
//! o que erra em silêncio (uma barra invertida ainda desenha, só descreve a edição
//! oposta).

use super::*;
use crate::graph::TimeView;
use ph2d_timeline::{StripId, StripLoop, StripView};

/// 100 px por segundo, tempo 0 na coluna 100 — números do produto, arredondados só o
/// bastante pra a conta ser lida à mão.
fn view() -> TimeView {
    TimeView {
        time_x: 100.0,
        right: 900.0,
        view_start: 0.0,
        px_per_s: 100.0,
    }
}

/// Um strip em `[2, 4)` — corpo em `[300, 500)`, faixa vertical `[60, 80)`.
fn strip(marks: [f64; 4]) -> StripView {
    StripView {
        id: StripId(7),
        clip_name: "X".into(),
        container: None,
        t_start: 2.0,
        t_end: 4.0,
        blend_in: 0.0,
        blend_out: 0.0,
        lead_in: 0.0,
        ease_locked_in: false,
        ease_locked_out: false,
        loop_mode: StripLoop::Once,
        speed: 1.0,
        marks,
    }
}

fn body() -> Rect {
    Rect::new(300.0, 60.0, 200.0, 20.0)
}

/// **Uma quina não editada não desenha nada.** Zero é "não mexi aqui" — e uma barra de
/// largura zero que ainda assim é oferecida vira um traço de 1 px em toda quina de todo
/// strip do documento.
#[test]
fn an_unedited_corner_draws_no_bar() {
    let s = strip([0.0; 4]);
    for (stretch, edge) in [(false, 0u8), (false, 1), (true, 0), (true, 1)] {
        assert!(
            mark_bar(&s, view(), body(), stretch, edge).is_none(),
            "corner (stretch={stretch}, edge={edge})"
        );
    }
}

/// **O sinal escolhe o lado, e é a mesma expressão nas duas pontas.** Cresceu na
/// esquerda ⇒ o vão ganho está DENTRO; encolheu na direita ⇒ o vão perdido está FORA.
/// É o desenho que o Enio mandou (foto de 2026-07-16), afirmado no número que o painel
/// lê.
#[test]
fn a_bar_lies_inside_where_the_strip_grew_and_outside_where_it_shrank() {
    // Start edge pulled OUT by 0.5 s (2.0 <- 1.5 ... a marca é +0.5).
    let grew = mark_bar(&strip([0.5, 0.0, 0.0, 0.0]), view(), body(), false, 0)
        .expect("a corner that moved draws a bar");
    assert!(
        grew.x >= body().x && grew.x + grew.w <= body().x + body().w,
        "o vão ganho está DENTRO do corpo: {grew:?}"
    );
    assert!((grew.w - 50.0).abs() < 0.01, "meio segundo = 50 px");

    // End edge pushed IN by 1 s (4.0 -> 3.0 ... a marca é +1.0).
    let shrank = mark_bar(&strip([0.0, 1.0, 0.0, 0.0]), view(), body(), false, 1)
        .expect("a corner that moved draws a bar");
    assert!(
        shrank.x >= body().x + body().w,
        "o vão perdido está FORA, à direita do corpo: {shrank:?}"
    );
    assert!((shrank.w - 100.0).abs() < 0.01, "um segundo = 100 px");
}

/// …e o espelho: encolher na esquerda sai pra fora, crescer na direita sai pra fora do
/// outro lado. Sem este par o gate acima ficaria verde com a regra escrita ao contrário
/// numa das pontas.
#[test]
fn the_same_rule_mirrors_at_the_other_end_of_each_edge() {
    let shrank_left =
        mark_bar(&strip([-0.5, 0.0, 0.0, 0.0]), view(), body(), false, 0).expect("bar");
    assert!(
        shrank_left.x + shrank_left.w <= body().x,
        "start pushed IN: o vão perdido está FORA, à esquerda: {shrank_left:?}"
    );
    let grew_right =
        mark_bar(&strip([0.0, -1.0, 0.0, 0.0]), view(), body(), false, 1).expect("bar");
    assert!(
        grew_right.x + grew_right.w <= body().x + body().w,
        "end pulled OUT: o vão ganho está DENTRO: {grew_right:?}"
    );
}

/// **Cada operação na sua banda.** O verde mora na borda de CIMA (esticar) e o vermelho
/// na de BAIXO (cortar) — a mesma banda em que o braço da quina corre, senão a barra
/// deixa de ler como "esta quina, esticada".
#[test]
fn a_stretch_bar_rides_the_top_edge_and_a_trim_bar_the_bottom() {
    let b = body();
    let top = mark_bar(&strip([0.0, 0.0, 0.5, 0.0]), view(), b, true, 0).expect("bar");
    assert!((top.y - b.y).abs() < 0.01, "stretch: borda de cima");
    let bottom = mark_bar(&strip([0.5, 0.0, 0.0, 0.0]), view(), b, false, 0).expect("bar");
    assert!(
        (bottom.y + bottom.h - (b.y + b.h)).abs() < 0.01,
        "trim: borda de baixo"
    );
    assert!(
        top.y + top.h <= bottom.y,
        "e as duas bandas não se encostam — um strip com as quatro marcas mostra as quatro"
    );
}
