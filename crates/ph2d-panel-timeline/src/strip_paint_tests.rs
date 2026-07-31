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
        lead_out: 0.0,
        ease_locked_in: false,
        ease_locked_out: false,
        curve_in: None,
        curve_out: None,
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

// ── A curva desenhada dentro da cunha do fade (Enio, 2026-07-31) ────────────

/// Uma banda de fade folgada, em pixels de tela.
fn band() -> Rect {
    Rect::new(100.0, 50.0, 80.0, 20.0)
}

/// **A altura de cada ponto é a que o AVALIADOR computa** — o desenho não tem uma segunda
/// resposta para *"que forma tem este fade?"*.
///
/// O oráculo é o `ph2d_timeline::fade_ramp`, a mesma função que o `weight_at_with` chama;
/// reimplementar o `smoothstep` aqui só provaria que dois erros combinam.
#[test]
fn the_drawn_curve_is_the_ramp_the_evaluator_uses() {
    let r = band();
    for curve in [
        None,
        Some(ph2d_timeline::Easing {
            family: ph2d_timeline::EasingFamily::Bounce,
            mode: ph2d_timeline::EasingMode::Out,
        }),
    ] {
        let pts = crate::strip_paint::fade_curve_points(
            r,
            crate::strip_paint::FadeCurve {
                curve,
                rising: true,
            },
        );
        assert!(pts.len() > 2, "uma banda folgada tem de render curva");
        let inset = f64::from(crate::strip_paint::FADE_CURVE_INSET);
        let (top, h) = (f64::from(r.y) + inset, f64::from(r.h) - inset * 2.0);
        for (x, y) in &pts {
            // A fração horizontal É a fração da janela, então o peso esperado sai dela.
            let f = (x - f64::from(r.x)) / f64::from(r.w);
            let want = top + (1.0 - ph2d_timeline::fade_ramp(f, curve)) * h;
            assert!(
                (y - want).abs() < 1e-9,
                "o ponto em f={f} desenha {y}, o avaliador diz {want}"
            );
        }
    }
}

/// **Um fade-in SOBE e um fade-out DESCE.**
///
/// A direção não está na geometria da banda (as duas outward moram FORA da caixa, uma de
/// cada lado), então ela é passada — e uma que chegasse errada desenharia uma entrada onde
/// há uma saída, com o teste de altura acima ainda verde: ele afirma a FORMA, não o sentido.
#[test]
fn a_fade_in_climbs_and_a_fade_out_falls() {
    let r = band();
    let ends = |rising: bool| {
        let p = crate::strip_paint::fade_curve_points(
            r,
            crate::strip_paint::FadeCurve {
                curve: None,
                rising,
            },
        );
        (p.first().unwrap().1, p.last().unwrap().1)
    };
    // `y` cresce para BAIXO: subir de peso é o `y` DIMINUIR.
    let (a, b) = ends(true);
    assert!(b < a, "um fade-in tem de subir: {a} -> {b}");
    let (a, b) = ends(false);
    assert!(b > a, "um fade-out tem de descer: {a} -> {b}");
}

/// **Uma banda pequena demais não desenha curva.** Ali a linha seria indistinguível da
/// diagonal do hatch, e uma cunha de 2 px com um traço dentro lê como listra mais grossa,
/// não como forma.
#[test]
fn a_sliver_of_a_fade_draws_no_curve() {
    assert!(
        crate::strip_paint::fade_curve_points(
            Rect::new(0.0, 0.0, 3.0, 20.0),
            crate::strip_paint::FadeCurve {
                curve: None,
                rising: true,
            },
        )
        .is_empty()
    );
}
