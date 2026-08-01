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
        seam: None,
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
                slice: None,
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
                slice: None,
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
                slice: None,
            },
        )
        .is_empty()
    );
}

/// **O que se LISTRA é o que se CLICA** — a porta única das faixas.
///
/// O pintor desenha `fade_bands` e o índice de hit registra `fade_bands`. Duas respostas
/// para *"onde está o fade?"* seriam um menu que abre sobre tela vazia, ou uma listra que
/// não aceita clique — e nenhuma das duas falha um teste que só olhe um dos lados.
#[test]
fn every_striped_band_is_a_band_the_pointer_can_reach() {
    let (view, body) = (view(), Rect::new(300.0, 60.0, 200.0, 20.0));
    let mut s = strip([0.0; 4]); // [2, 4) -> corpo [300, 500)
    s.blend_in = 0.5;
    s.blend_out = 0.5;
    s.lead_in = 0.0; // exclusivo com o blend_in
    s.lead_out = 0.5;
    let bands = crate::strip_paint::fade_bands(&s, view, body);
    // Três faixas: as duas de dentro e a de fora na cauda.
    assert_eq!(bands.len(), 3, "faixas desenhadas: {bands:?}");
    // Cada uma leva o código de ZONA da sua borda — é ele que o menu lê.
    let codes: Vec<u8> = bands.iter().map(|&(e, _, _)| e).collect();
    assert!(codes.contains(&crate::stack_ease_grip::BAND_IN));
    assert!(codes.contains(&crate::stack_ease_grip::BAND_OUT));
    // E a de FORA vive além do fim do strip — no vão, onde não há corpo por baixo.
    let (_, out_gap, _) = bands
        .iter()
        .find(|(e, r, _)| *e == crate::stack_ease_grip::BAND_OUT && r.x >= view.x(4.0) - 0.001)
        .expect("a faixa de fora tem de existir e viver no vão");
    assert!(out_gap.w > 0.0);
}

/// **Faixa de largura zero não entra na lista.** Uma faixa que não se vê não se clica — e
/// registrá-la seria um hit invisível roubando o clique do corpo.
///
/// Duas maneiras de ter largura zero, e são casos DIFERENTES: o strip sem fade nenhum (os
/// campos em `0`), e — o que o guard de pixels defende — um fade que EXISTE em segundos e
/// mede zero pixels neste zoom. O segundo é o que acontece de verdade: afaste o suficiente e
/// meio segundo de fade colapsa numa coluna.
#[test]
fn a_band_with_no_width_is_not_offered() {
    let body = Rect::new(300.0, 60.0, 200.0, 20.0);
    // (a) sem fade nenhum.
    let s = strip([0.0; 4]);
    assert!(crate::strip_paint::fade_bands(&s, view(), body).is_empty());

    // (b) fade REAL, zoom em que ele não mede um pixel.
    let far = TimeView {
        time_x: 100.0,
        right: 900.0,
        view_start: 0.0,
        px_per_s: 1e-9,
    };
    let mut s = strip([0.0; 4]);
    s.blend_in = 0.5;
    s.blend_out = 0.5;
    s.lead_out = 0.5;
    assert!(
        crate::strip_paint::fade_bands(&s, far, body).is_empty(),
        "meio segundo de fade que não mede um pixel não é uma faixa clicável"
    );
}

// ── A costura desenhada como UMA curva (Enio, 2026-08-01) ───────────────────

/// **As duas cunhas da costura desenham UMA curva, e as pontas se ENCONTRAM na volta.**
///
/// *"A curva começa a ser desenhada na FADE final e acaba de ser desenhada na FADE
/// inicial"* — a cauda mostra `[0, f]`, a cabeça `[f, 1]`, e o valor no corte é o mesmo nas
/// duas. Sem o encontro seriam duas curvas vizinhas, que é o que o Enio viu; sem a
/// monotonicidade seriam duas fatias certas desenhadas ao contrário.
#[test]
fn the_seam_is_drawn_as_one_curve_across_the_two_wedges() {
    let r = band();
    let f = 0.375; // uma divisão assimétrica: em 0,5 um erro de fatia se esconde
    let curve = Some(ph2d_timeline::Easing {
        family: ph2d_timeline::EasingFamily::Quint,
        mode: ph2d_timeline::EasingMode::InOut,
    });
    let slice = |u0: f64, u1: f64| {
        crate::strip_paint::fade_curve_points(
            r,
            crate::strip_paint::FadeCurve {
                curve,
                rising: true,
                slice: Some((u0, u1)),
            },
        )
    };
    let tail = slice(0.0, f);
    let head = slice(f, 1.0);
    assert!(!tail.is_empty() && !head.is_empty());
    assert!(
        (tail.last().unwrap().1 - head.first().unwrap().1).abs() < 1e-9,
        "as duas fatias têm de se encontrar na volta: {:?} vs {:?}",
        tail.last().unwrap().1,
        head.first().unwrap().1
    );
    // …e juntas sobem sem voltar atrás (`y` cresce para BAIXO, então ele só DECRESCE).
    let ys: Vec<f64> = tail.iter().chain(head.iter()).map(|p| p.1).collect();
    assert!(
        ys.windows(2).all(|w| w[1] <= w[0] + 1e-9),
        "a curva da costura é uma só, monotônica: {ys:?}"
    );
    // A fatia da CAUDA é a primeira: ela termina abaixo da metade da altura da banda…
    let mid = f64::from(r.y) + f64::from(r.h) / 2.0;
    assert!(
        tail.first().unwrap().1 > mid,
        "a cauda COMEÇA a curva (peso baixo, y alto): {:?}",
        tail.first().unwrap().1
    );
    assert!(
        head.last().unwrap().1 < mid,
        "e a cabeça a TERMINA (peso cheio, y baixo): {:?}",
        head.last().unwrap().1
    );
}

/// **CONTROLE: sem costura, cada cunha segue desenhando o peso da própria strip** — a
/// entrada sobe, a saída desce. Sem esta metade, um desenho que sempre subisse passaria no
/// gate acima e inverteria em silêncio toda cunha de saída do documento.
#[test]
fn without_a_seam_a_fade_out_still_falls() {
    let mut s = strip([0.0; 4]);
    s.lead_out = 0.5;
    s.seam = None;
    let bands = crate::strip_paint::fade_bands(&s, view(), body());
    let (_, _, fade) = bands
        .iter()
        .find(|(e, _, _)| *e == crate::stack_ease_grip::BAND_OUT)
        .expect("a cunha de saída");
    assert!(fade.slice.is_none() && !fade.rising, "{fade:?}");
}
