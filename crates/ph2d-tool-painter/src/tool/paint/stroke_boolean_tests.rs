//! **O composite booleano roda na janela das FORMAS, não na do canvas** — a identidade primeiro, o
//! relógio depois.
//!
//! ⚠️ **O oráculo é o PRODUTO ANTERIOR:** a `stroke_boolean_contours_whole_canvas` é a rota que este
//! módulo rodava até 2026-08-06, congelada sob `cfg(test)`, com o buffer do canvas supersampleado
//! inteiro. As duas rotas comparadas aqui são, literalmente, *a de antes* e *a de agora*.

use super::stroke_multi::StrokeOp;
use crate::tool::PainterTool;
use crate::undo::{CurveState, EllipseState, PolygonState, ShapeEditState};
use ph2d_editor_core::tool::RasterEditTool;

fn tool(side: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t
}

fn ellipse(center: [f32; 2], rx: f32, ry: f32, u: [f32; 2]) -> ShapeEditState {
    ShapeEditState::Ellipse(EllipseState {
        center,
        u,
        rx,
        ry,
        editing: true,
        seed: 7,
    })
}

fn polygon(center: [f32; 2], rx: f32, ry: f32, sides: u32) -> ShapeEditState {
    ShapeEditState::Polygon(PolygonState {
        center,
        u: [1.0, 0.0],
        rx,
        ry,
        sides,
        editing: true,
        seed: 11,
    })
}

/// Uma CURVA fechada cujas **alças saem da caixa dos pontos** — o único caso que separa um casco
/// medido sobre pontos+alças de um medido só sobre os pontos.
///
/// ⚠️ Sem ela a fixture não contém o fenômeno, e a mutação *"esqueça as alças"* **passa**: o Polygon e
/// a Line fechada produzem `Freehand` com as alças **EM CIMA** dos pontos (`[q, q]`, quinas vivas),
/// então nesses dois dropar as alças não muda uma coordenada. Medido — a mutação sobreviveu à primeira
/// versão deste arquivo.
fn curve_with_handles_outside_the_points() -> ShapeEditState {
    // Um losango cujas alças do vértice direito apontam MUITO para fora: a Bézier estufa além de x=160.
    let points = vec![[100.0, 60.0], [160.0, 120.0], [100.0, 180.0], [40.0, 120.0]];
    let handles = vec![
        [[70.0, 60.0], [130.0, 60.0]],
        [[235.0, 70.0], [235.0, 170.0]],
        [[130.0, 180.0], [70.0, 180.0]],
        [[40.0, 150.0], [40.0, 90.0]],
    ];
    ShapeEditState::Curve(CurveState {
        kinds: vec![0; points.len()],
        anchor: points[0],
        stabilized: points[0],
        points,
        handles,
        selected: None,
        added_point: false,
        closed: true,
        editing: true,
        freehand: true,
        seed: 3,
    })
}

/// Os contornos das duas rotas são os MESMOS pontos, na mesma ordem.
fn same_contours(a: &[Vec<[f32; 2]>], b: &[Vec<[f32; 2]>], case: &str) {
    assert_eq!(
        a.len(),
        b.len(),
        "contagem de contornos difere ({case}): {} contra {}",
        a.len(),
        b.len()
    );
    for (i, (ca, cb)) in a.iter().zip(b).enumerate() {
        assert_eq!(
            ca.len(),
            cb.len(),
            "o contorno {i} tem {} pontos contra {} ({case})",
            ca.len(),
            cb.len()
        );
        for (j, (pa, pb)) in ca.iter().zip(cb).enumerate() {
            assert!(
                (pa[0] - pb[0]).abs() < 1e-4 && (pa[1] - pb[1]).abs() < 1e-4,
                "o ponto {j} do contorno {i} andou ({case}): {pa:?} contra {pb:?}"
            );
        }
    }
}

/// **A wave inteira se apoia nisto:** encolher a janela do composite não pode mover o desenho.
///
/// ⚠️ **A fixture tem de conter as bordas.** Uma figura confortavelmente dentro do canvas não
/// distingue as duas rotas de uma que esqueceu o `clamp`: o caso que separa é a forma que **sai** do
/// canvas, onde a janela antiga recortava na aresta da tela e a nova tem de recortar no mesmo lugar.
/// A elipse **girada** separa uma caixa exata de uma que lê `rx`/`ry` como se fossem os eixos, e a
/// **curva com as alças fora da caixa dos pontos** separa um casco que as inclui de um que não.
#[test]
fn the_boolean_window_draws_what_the_whole_canvas_window_drew() {
    let t = tool(256);
    let cases: Vec<(&str, Vec<(ShapeEditState, StrokeOp)>)> = vec![
        (
            "uma Add sozinha, no meio",
            vec![(
                ellipse([128.0, 128.0], 40.0, 30.0, [1.0, 0.0]),
                StrokeOp::Add,
            )],
        ),
        (
            "duas Add que se cruzam",
            vec![
                (
                    ellipse([100.0, 128.0], 40.0, 30.0, [1.0, 0.0]),
                    StrokeOp::Add,
                ),
                (
                    ellipse([150.0, 128.0], 40.0, 30.0, [1.0, 0.0]),
                    StrokeOp::Add,
                ),
            ],
        ),
        (
            "duas Add SEPARADAS (duas componentes)",
            vec![
                (ellipse([60.0, 60.0], 25.0, 25.0, [1.0, 0.0]), StrokeOp::Add),
                (
                    ellipse([200.0, 200.0], 25.0, 25.0, [1.0, 0.0]),
                    StrokeOp::Add,
                ),
            ],
        ),
        (
            "Add menos Remove (um buraco)",
            vec![
                (
                    ellipse([128.0, 128.0], 60.0, 60.0, [1.0, 0.0]),
                    StrokeOp::Add,
                ),
                (
                    ellipse([128.0, 128.0], 25.0, 25.0, [1.0, 0.0]),
                    StrokeOp::Remove,
                ),
            ],
        ),
        (
            "Remove LONGE da Add — nao pode mudar nada",
            vec![
                (
                    ellipse([128.0, 128.0], 40.0, 40.0, [1.0, 0.0]),
                    StrokeOp::Add,
                ),
                (
                    ellipse([240.0, 20.0], 15.0, 15.0, [1.0, 0.0]),
                    StrokeOp::Remove,
                ),
            ],
        ),
        (
            "a figura SAI do canvas",
            vec![(
                ellipse([20.0, 128.0], 70.0, 60.0, [1.0, 0.0]),
                StrokeOp::Add,
            )],
        ),
        (
            "a figura encosta na aresta",
            vec![(ellipse([64.0, 64.0], 64.0, 64.0, [1.0, 0.0]), StrokeOp::Add)],
        ),
        (
            "elipse GIRADA (a caixa nao e a dos eixos)",
            vec![(
                ellipse([128.0, 128.0], 70.0, 25.0, [0.6, 0.8]),
                StrokeOp::Add,
            )],
        ),
        (
            "poligono",
            vec![(polygon([128.0, 128.0], 50.0, 40.0, 5), StrokeOp::Add)],
        ),
        (
            "curva fechada com as alcas FORA da caixa dos pontos",
            vec![(curve_with_handles_outside_the_points(), StrokeOp::Add)],
        ),
        (
            "curva fechada menos elipse",
            vec![
                (curve_with_handles_outside_the_points(), StrokeOp::Add),
                (
                    ellipse([120.0, 120.0], 18.0, 18.0, [1.0, 0.0]),
                    StrokeOp::Remove,
                ),
            ],
        ),
        (
            "poligono menos elipse",
            vec![
                (polygon([128.0, 128.0], 60.0, 60.0, 6), StrokeOp::Add),
                (
                    ellipse([140.0, 120.0], 20.0, 20.0, [1.0, 0.0]),
                    StrokeOp::Remove,
                ),
            ],
        ),
        // ⚠️ **A forma DEGENERADA é onde um off-by-one na sub-janela é mais alto:** a caixa de um
        // semi-eixo mínimo tem uns poucos texels de largura, então errá-la por um é errar uma fração
        // grande do desenho — numa figura de 40 px o mesmo erro se esconde.
        //
        // ⚠️ E o que estes dois casos **não** contêm: o `rx = 0.1` NÃO alcança o `max(0.5)` de texel do
        // `rasterize_ellipse`, porque o `stroke_state_to_fill_shape` já clampa em meio **px de imagem**
        // (`SS` vezes maior) antes de chegar lá. Escrevi o contrário na primeira versão; a mutação
        // `PAD = 0` sobreviveu e foi ela que corrigiu a afirmação.
        (
            "elipse de semi-eixo MINIMO — a caixa tem poucos texels",
            vec![(
                ellipse([128.0, 128.0], 0.1, 40.0, [1.0, 0.0]),
                StrokeOp::Add,
            )],
        ),
        (
            "a mesma, GIRADA — a caixa cruza os dois eixos",
            vec![(
                ellipse([128.0, 128.0], 0.1, 40.0, [0.6, 0.8]),
                StrokeOp::Add,
            )],
        ),
    ];
    for (case, shapes) in cases {
        for off in [0.0f32, 6.0, -6.0] {
            let a = t.stroke_boolean_contours(&shapes, off);
            let b = t.stroke_boolean_contours_whole_canvas(&shapes, off);
            assert!(!b.is_empty(), "a fixture nao produziu contorno ({case})");
            same_contours(&a, &b, &format!("{case}, offset {off}"));
        }
    }
}

/// **Sem nenhuma Add não há região** — a saída é vazia, e não um contorno inventado.
///
/// ⚠️ É a metade que o early-out novo introduziu: a rota antiga rasterizava o Remove sobre um `crisp`
/// zerado e o traçado não achava nada; a nova nem chega a alocar. As duas têm de dizer a mesma coisa.
#[test]
fn a_composite_with_no_add_shape_has_no_contour() {
    let t = tool(256);
    let only_remove = vec![(
        ellipse([128.0, 128.0], 40.0, 40.0, [1.0, 0.0]),
        StrokeOp::Remove,
    )];
    assert!(t.stroke_boolean_contours(&only_remove, 0.0).is_empty());
    assert!(
        t.stroke_boolean_contours_whole_canvas(&only_remove, 0.0)
            .is_empty(),
        "a rota antiga tem de concordar — senão o early-out mudou o produto"
    );
}

/// **A CONSEQUÊNCIA, que só um relógio vê:** o composite deixa de ser função do CANVAS.
///
/// A MESMA figura em duas telas. Antes, dobrar o lado quadruplicava o custo (o buffer é do canvas);
/// agora a janela é a da figura e as duas telas custam o mesmo.
///
/// ⚠️ **É uma RAZÃO entre duas telas, não um wall-clock** — as duas metades pagam a mesma deriva de
/// máquina e o mesmo perfil de compilação.
#[test]
#[ignore = "clock — run explicitly with --test-threads=1"]
fn the_boolean_is_a_fact_of_the_figure_not_of_the_canvas() {
    let shapes = vec![
        (
            ellipse([300.0, 300.0], 90.0, 70.0, [1.0, 0.0]),
            StrokeOp::Add,
        ),
        (
            ellipse([360.0, 300.0], 90.0, 70.0, [1.0, 0.0]),
            StrokeOp::Add,
        ),
    ];
    let ms = |side: u32| -> f64 {
        let t = tool(side);
        let mut best = f64::MAX;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let c = t.stroke_boolean_contours(&shapes, 0.0);
            assert!(!c.is_empty(), "a fixture nao produziu contorno em {side}");
            best = best.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        best
    };
    let (a, b) = (ms(1024), ms(2048));
    assert!(
        b <= a * 1.6,
        "quadruplicar a ÁREA da tela custou {:.2}x ({b:.3} ms contra {a:.3}) com a MESMA figura — a \
         janela do composite voltou a ser a do canvas",
        b / a.max(1e-9)
    );
}
