//! Gates da régua.

use super::*;
use crate::grid::GridView;

/// Uma vista de 800×600 com o canvas ocupando tudo, câmera na origem, 10 unidades de altura.
fn view() -> GridView {
    GridView {
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
    }
}

/// **A afirmação central do módulo:** a régua não tem projeção própria. Um traço marcado no
/// valor `v` cai na tela exatamente onde a grade desenharia a linha de `v`.
///
/// ⚠️ Este é o gate que torna a frase do cabeçalho verificável em vez de uma promessa. A
/// mutação que ele pega é qualquer re-derivação (um `(v - left) / span * w` escrito à mão
/// aqui): ela concorda no caso simples e diverge em pan/zoom/Y-flip.
#[test]
fn a_ruler_tick_lands_where_the_grid_would_draw_that_line() {
    let v = view();
    let (bounds, _) = crate::grid::world_bounds(&v);
    for t in ticks(&v, [0.0, 0.0], RulerAxis::Top) {
        let grid_x = crate::grid::world_to_screen_x(t.world as f32, &bounds, &v);
        assert!(
            (t.screen - grid_x).abs() < 1e-3,
            "traço em {} caiu em {} e a grade o poria em {grid_x}",
            t.world,
            t.screen
        );
    }
    for t in ticks(&v, [0.0, 0.0], RulerAxis::Left) {
        let grid_y = crate::grid::world_to_screen_y(t.world as f32, &bounds, &v);
        assert!((t.screen - grid_y).abs() < 1e-3, "o Y-flip também");
    }
}

/// A porta do GESTO é a inversa exata da do desenho: soltar o dedo sobre um traço pousa a
/// guia no valor daquele traço.
#[test]
fn the_drop_lands_on_the_value_the_tick_shows() {
    let v = view();
    for axis in [RulerAxis::Top, RulerAxis::Left] {
        for t in ticks(&v, [0.0, 0.0], axis) {
            let back = world_at(&v, t.screen, axis);
            assert!(
                (back - t.world).abs() < 1e-3,
                "{axis:?}: tela {} -> mundo {back}, esperado {}",
                t.screen,
                t.world
            );
        }
    }
}

/// A escada `1/2/5 × 10^k` inteira, construída **fora** da função sob teste — um oráculo que
/// chamasse `label_step` para saber o que esperar seria verde por construção.
fn ladder() -> Vec<f64> {
    let mut out = Vec::new();
    let mut decade = 1e-9_f64;
    for _ in 0..24 {
        for m in [1.0, 2.0, 5.0] {
            out.push(decade * m);
        }
        decade *= 10.0;
    }
    out
}

/// **O passo é legível em QUALQUER zoom** — a propriedade que a cadência própria existe para
/// garantir, varrida por dez décadas em vez de afirmada num ponto.
#[test]
fn the_label_step_keeps_labels_apart_at_every_zoom() {
    let mut px_per_world = 1e-4_f64;
    for _ in 0..40 {
        let step = label_step(px_per_world);
        let on_screen = step * px_per_world;
        assert!(
            on_screen >= f64::from(MIN_LABEL_PX) - 1e-9,
            "a {px_per_world} px/unidade o passo {step} ocupa {on_screen} px"
        );
        // E é o MENOR degrau que cabe — não um maior por folga.
        //
        // ⚠️ O oráculo é a escada CONSTRUÍDA, não uma lista de "candidatos anteriores": a
        // primeira versão deste gate enumerava `step/2`, `step/2.5` e `step/5`, e o `step/2`
        // de 5000 é 2500 — que **não é** um degrau (a escada vai 1000, 2000, 5000). Ele
        // reprovava código correto. Uma propriedade se afirma, não se enumera.
        let smaller_that_fits = ladder()
            .into_iter()
            // ⚠️ `< step` cru reprova por RUÍDO: a escada é construída por multiplicação
            // repetida, então o degrau que É o `step` pode sair `0.00019999999999999998` e
            // contar como "um menor que cabe". A comparação tem de ser relativa.
            .filter(|c| *c < step * (1.0 - 1e-9) && *c * px_per_world >= f64::from(MIN_LABEL_PX))
            .count();
        assert_eq!(
            smaller_that_fits, 0,
            "a {px_per_world} px/unidade o passo {step} é maior do que o necessário"
        );
        px_per_world *= 2.0;
    }
}

/// A escada só produz `1/2/5 × 10^k`, e nunca um número arbitrário.
#[test]
fn the_step_is_always_a_one_two_or_five() {
    let mut px = 0.01_f64;
    for _ in 0..60 {
        let step = label_step(px);
        // Normaliza para a década: o resultado tem de ser 1, 2 ou 5 (a menos de `f64`).
        let mut m = step;
        while m >= 10.0 {
            m /= 10.0;
        }
        while m < 1.0 {
            m *= 10.0;
        }
        assert!(
            [1.0, 2.0, 5.0].iter().any(|c| (m - c).abs() < 1e-6),
            "passo {step} normaliza para {m}, que não é 1/2/5"
        );
        px *= 1.7;
    }
}

/// Zoom degenerado não trava nem pede um vetor do tamanho do mundo.
#[test]
fn a_degenerate_camera_draws_nothing_instead_of_everything() {
    let v = GridView {
        camera_height_world: 1e-30,
        ..view()
    };
    assert!(
        ticks(&v, [0.0, 0.0], RulerAxis::Top).len() <= MAX_TICKS as usize + 1,
        "o teto de RECURSO segura a janela absurda"
    );
    assert_eq!(label_step(f64::INFINITY), 1.0);
    assert_eq!(label_step(0.0), 1.0);
    assert_eq!(label_step(f64::NAN), 1.0);
}

/// **O zero da régua é a ORIGEM**, não o zero do mundo — é o que torna "medir a partir daqui"
/// possível, e é a metade visível do fato de a origem ter um dono só.
#[test]
fn the_ruler_zero_is_the_origin_not_the_world_zero() {
    let v = view();
    let origin = [2.0, 0.0];
    let zero = ticks(&v, origin, RulerAxis::Top)
        .into_iter()
        .filter(|t| t.major)
        .min_by(|a, b| {
            (a.world - f64::from(origin[0]))
                .abs()
                .total_cmp(&(b.world - f64::from(origin[0])).abs())
        })
        .expect("há traços maiores na tela");
    assert!(
        (zero.world - 2.0).abs() < 1e-9,
        "o traço rotulado 0 marca o mundo 2 quando a origem é 2, e não o mundo 0"
    );
    assert_eq!(label_text(zero.world - f64::from(origin[0]), 1.0), "0");
}

/// O canto pertence à régua de CIMA — a regra existe para que a resposta não dependa da ordem
/// em que os dois `if` foram escritos.
#[test]
fn the_corner_belongs_to_the_top_ruler() {
    let canvas = Rect::new(100.0, 50.0, 800.0, 600.0);
    assert_eq!(hit(canvas, (101.0, 51.0)), Some(RulerAxis::Top));
    assert_eq!(hit(canvas, (400.0, 51.0)), Some(RulerAxis::Top));
    assert_eq!(hit(canvas, (101.0, 300.0)), Some(RulerAxis::Left));
    assert_eq!(hit(canvas, (400.0, 300.0)), None, "o miolo não é régua");
    assert_eq!(hit(canvas, (10.0, 10.0)), None, "fora do canvas, nada");
}

/// A ponte entre os dois vocabulários — a régua de CIMA cria uma linha HORIZONTAL, e a
/// inversão é o par que se troca sem o compilador reclamar.
#[test]
fn the_top_ruler_spawns_a_horizontal_guide() {
    assert_eq!(RulerAxis::Top.spawns(), GuideAxis::Horizontal);
    assert_eq!(RulerAxis::Left.spawns(), GuideAxis::Vertical);
}

/// O rótulo mostra as casas que o passo exige — senão um passo de 0,2 imprime `0 0 1`.
#[test]
fn a_fractional_step_gets_the_decimals_it_needs() {
    assert_eq!(label_text(0.2, 0.2), "0.2");
    assert_eq!(label_text(1.0, 1.0), "1");
    assert_eq!(label_text(0.05, 0.05), "0.05");
    assert_eq!(label_text(-0.0, 1.0), "0", "o zero negativo lê como erro");
}
