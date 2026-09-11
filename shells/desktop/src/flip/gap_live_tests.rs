//! Os gates dos helpers ao vivo do Gap Closure — módulo-irmão do `flip_gap_live.rs`.
//!
//! A máquina (`GapHelpers`) é dirigível SEM janela: `drive(reach, drawing)` é a costura
//! inteira menos a resolução de alvo (que é o `flip_strip_resolve::target` de sempre) e
//! a projeção (que é o `screen_affine` do tween). O worker é uma thread REAL — os testes
//! que o esperam fazem poll com timeout, como o produto faz por frame.

use super::*;
use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};

fn stroke(pts: &[(f32, f32)]) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 0.2,
            opacity: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
        });
    }
    s
}

/// O fixture canônico do BUGS #23: a caixa cuja parede direita tem o vão COLINEAR de
/// 1,0 (entre (2,-0.5) e (2,0.5)) — o vão que só o par ponta-a-ponta fecha.
fn canonical_gap_drawing() -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes.push(stroke(&[(2.0, -2.0), (2.0, -0.5)]));
    d.strokes.push(stroke(&[(2.0, 0.5), (2.0, 2.0)]));
    d.strokes.push(stroke(&[
        (2.0, 2.0),
        (-2.0, 2.0),
        (-2.0, -2.0),
        (2.0, -2.0),
    ]));
    d
}

/// O helper do vão canônico está na lista — e é uma PONTE (as duas pontas reais)?
fn has_tip_pair(segments: &[ph2d_flip_fill::GapHelper]) -> bool {
    segments.iter().any(|h| {
        let (lo, hi) = if h.seg.a.y < h.seg.b.y {
            (h.seg.a, h.seg.b)
        } else {
            (h.seg.b, h.seg.a)
        };
        (lo.y - -0.5).abs() < 1e-3 && (hi.y - 0.5).abs() < 1e-3 && h.a_is_tip && h.b_is_tip
    })
}

/// Dirige a máquina até o worker instalar (ou o timeout estourar) — o mesmo poll por
/// frame do produto.
fn drive_until_installed(g: &mut GapHelpers, reach: f32, d: &FlipDrawing) {
    for _ in 0..400 {
        if g.drive(reach, d) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("o worker dos helpers nao instalou em 2s");
}

/// **A porta do modo** — helpers só em Fill, e NUNCA em Unpaint (que não roda o solver:
/// um helper ali prometeria um fechamento que o clique não faz).
#[test]
fn helpers_exist_only_in_fill_mode_and_never_in_unpaint() {
    use ph2d_tool_flip::{FillMode, FlipMode, FlipStyleSnapshot};
    let style = |mode, fill_mode| {
        Some(FlipStyleSnapshot {
            mode,
            fill_mode,
            ..Default::default()
        })
    };
    assert!(wants_gap_helpers(
        true,
        style(FlipMode::Fill, FillMode::Paint)
    ));
    assert!(wants_gap_helpers(
        true,
        style(FlipMode::Fill, FillMode::PaintBehind)
    ));
    assert!(
        !wants_gap_helpers(true, style(FlipMode::Fill, FillMode::Unpaint)),
        "Unpaint nao roda o solver — helper ali e' a tela mentindo"
    );
    assert!(!wants_gap_helpers(
        true,
        style(FlipMode::Draw, FillMode::Paint)
    ));
    assert!(!wants_gap_helpers(
        false,
        style(FlipMode::Fill, FillMode::Paint)
    ));
    assert!(!wants_gap_helpers(true, None));
}

/// 🔴 **A roda anda um passo de mundo (0,05 doc) por tique e clampa nas pontas** — e só
/// fala em modo Fill (fora dele a roda é do zoom, e devolver `Some` aqui roubaria o zoom).
///
/// Mutação que sangra: tirar o clamp (o track passa de 1.0 e o tool clamparia sozinho,
/// mas o KNOB do painel desenharia fora do trilho) — ou inverter o passo.
#[test]
fn the_wheel_moves_one_world_step_per_notch_and_clamps() {
    use ph2d_tool_flip::{FillMode, FlipMode, FlipStyleSnapshot, GAP_MAX_WORLD};
    let fill = |gap| {
        Some(FlipStyleSnapshot {
            mode: FlipMode::Fill,
            fill_mode: FillMode::Paint,
            gap,
            ..Default::default()
        })
    };
    // +1 tique de 0,25 doc → 0,30 doc (o passo é 0,05; em track: 0,30/1,0).
    let t = gap_wheel_track(true, fill(0.25), 1.0).expect("modo Fill responde");
    assert!(
        (t - 0.30 / GAP_MAX_WORLD).abs() < 1e-9,
        "0,25 + 1 tique = 0,30 doc"
    );
    // −1 tique de 0 → segue 0 (clamp de baixo).
    let t = gap_wheel_track(true, fill(0.0), -1.0).expect("clamp de baixo");
    assert!(t.abs() < 1e-9);
    // +100 tiques de 0,98 → 1,0, não 5,98 (clamp de cima).
    let t = gap_wheel_track(true, fill(0.98), 100.0).expect("clamp de cima");
    assert!((t - 1.0).abs() < 1e-9);
    // Fora do modo Fill (ou parado), a roda não é do Gap.
    assert!(
        gap_wheel_track(
            true,
            Some(FlipStyleSnapshot {
                mode: FlipMode::Draw,
                gap: 0.25,
                ..Default::default()
            }),
            1.0
        )
        .is_none(),
        "em Draw a roda e' do zoom"
    );
    assert!(gap_wheel_track(true, fill(0.25), 0.0).is_none());
}

/// **Alcance zero instala vazio SEM worker** — é o default do slider, e pagar uma
/// thread para computar uma lista vazia por definição seria ruído.
#[test]
fn reach_zero_installs_empty_without_a_worker() {
    let d = canonical_gap_drawing();
    let mut g = GapHelpers::default();
    let changed = g.drive(0.0, &d);
    assert!(!changed, "nada estava na tela, nada mudou");
    assert!(g.segments.is_empty());
    assert!(g.job.is_none(), "alcance 0 nao paga worker");
}

/// 🔴 **O vão canônico ganha o helper no alcance que o nomeia** — e o resultado fica
/// CACHEADO: dirigir de novo com o mesmo alvo não relança worker nenhum (é o cache que
/// torna o custo de 5-339 ms pagável: uma vez por mudança, não por frame).
///
/// Mutações que sangram: nunca lançar o worker (o poll estoura) · ignorar a chave e
/// relançar sempre (o `job.is_none()` do fim cai).
#[test]
fn the_canonical_gap_installs_its_helper_once_and_caches_it() {
    let d = canonical_gap_drawing();
    let mut g = GapHelpers::default();
    drive_until_installed(&mut g, 1.0, &d);
    assert!(
        has_tip_pair(&g.segments),
        "o helper do vao colinear tinha de instalar: {:?}",
        g.segments
    );
    // O mesmo alvo de novo: cache hit — nada muda, nada relança.
    let changed = g.drive(1.0, &d);
    assert!(!changed, "cache hit nao reinstala");
    assert!(g.job.is_none(), "cache hit nao relanca worker");
}

/// 🔴 **Mudar o alcance relança — e o helper segue a resposta nova** (abaixo do vão o
/// par não fecha, então o helper dele TEM de sumir; um helper que fica é o slider
/// mentindo para baixo).
#[test]
fn a_new_reach_relaunches_and_the_helper_follows() {
    let d = canonical_gap_drawing();
    let mut g = GapHelpers::default();
    drive_until_installed(&mut g, 1.0, &d);
    assert!(has_tip_pair(&g.segments));
    // 0,9 < o vão de 1,0: o par não fecha mais.
    for _ in 0..400 {
        g.drive(0.9, &d);
        if !has_tip_pair(&g.segments) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("o helper do vao tinha de sumir abaixo do alcance que o fecha");
}

/// 🔴 **Editar o desenho invalida o cache** (o fingerprint é de CONTEÚDO): o mesmo
/// alcance sobre um desenho mudado relança — sem isto o artista apaga uma parede e os
/// helpers continuam descrevendo a que não existe.
#[test]
fn an_edited_drawing_invalidates_the_cache() {
    let mut d = canonical_gap_drawing();
    let mut g = GapHelpers::default();
    drive_until_installed(&mut g, 1.0, &d);
    // Apaga a metade de baixo da parede direita: o vão colinear deixa de ter par.
    d.strokes.remove(0);
    for _ in 0..400 {
        g.drive(1.0, &d);
        if !has_tip_pair(&g.segments) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("desenho editado tinha de re-derivar os helpers");
}

/// 🔴 **Resultado STALE é descartado, nunca instalado**: pedir 1,0 e mudar para 0,9
/// antes de o worker voltar não pode fazer o helper do vão PISCAR na tela — um helper
/// que aparece e some descrevendo um alcance que já não é o do slider é a killer
/// feature mentindo por um frame de latência.
///
/// Determinístico apesar da thread real: a colheita só acontece DENTRO de `drive`, e
/// depois da primeira chamada todas as chamadas já querem 0,9 — então o resultado de
/// 1,0, chegue quando chegar, é sempre colhido sob o alvo novo.
///
/// Mutação que sangra: instalar incondicionalmente no `try_take` (ignorar `jk == want`).
#[test]
fn a_stale_result_is_discarded_not_installed() {
    let d = canonical_gap_drawing();
    let mut g = GapHelpers::default();
    let _ = g.drive(1.0, &d); // o worker de 1,0 sai…
    for _ in 0..400 {
        let _ = g.drive(0.9, &d); // …e o alvo já é 0,9 quando ele voltar
        assert!(
            !has_tip_pair(&g.segments),
            "o resultado stale de 1,0 foi instalado sob o alvo 0,9"
        );
        // Instalou o resultado CERTO (o de 0,9)? Então o stale já morreu — fim.
        if g.key == Some((super::fingerprint(&d), 0.9f32.to_bits())) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("o resultado de 0,9 nao instalou em 2s");
}
