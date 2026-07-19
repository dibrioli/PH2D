//! Gates for the Colorize engine (`docs/Flip/09 §3`, §7.3).
//!
//! The load-bearing gate is `the_colour_cut_hugs_the_ink_not_the_midpoint`: it is what only
//! LazyBrush does — the boundary between two colours is *attracted to the line*, not drawn
//! halfway between the scribbles. It is red-first-able by construction: strip the ink from
//! the energy (`V_pq` uniform) and the split falls at the scribble midpoint, failing the
//! assertion. The flow solver itself is pinned by `flow_tests.rs` (BK ≡ Edmonds–Karp); these
//! gates pin the WIRING — energy assembly, the guloso multiway, the geometry — on a fixture
//! that contains the phenomenon.

use super::{Scribble, colorize};
use ph2d_core::Vec2;
use ph2d_flip_fill::{FillResult, signed_area};

/// A closed box with an internal vertical divider at `divider_x`, broken by a gap
/// `(gap.0, gap.1)` in the middle — the line-art the colour cut must respect.
fn boxed_with_divider(divider_x: f32, gap: (f32, f32)) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    vec![
        (
            vec![
                Vec2::new(0.1, 0.1),
                Vec2::new(0.9, 0.1),
                Vec2::new(0.9, 0.9),
                Vec2::new(0.1, 0.9),
            ],
            vec![0.0; 4],
            true,
        ),
        (
            vec![Vec2::new(divider_x, 0.1), Vec2::new(divider_x, gap.0)],
            vec![0.0; 2],
            false,
        ),
        (
            vec![Vec2::new(divider_x, gap.1), Vec2::new(divider_x, 0.9)],
            vec![0.0; 2],
            false,
        ),
    ]
}

fn max_x(f: &FillResult) -> f32 {
    f.outer.iter().fold(f32::MIN, |m, p| m.max(p.x))
}
fn centroid_x(f: &FillResult) -> f32 {
    f.outer.iter().map(|p| p.x).sum::<f32>() / f.outer.len() as f32
}
fn area(f: &FillResult) -> f32 {
    signed_area(&f.outer).abs()
}

/// The defining LazyBrush property: with the divider OFF-CENTER at x=0.7, the boundary
/// between the two colours sits AT the ink (x≈0.7) — so the left colour owns most of the box
/// — and NOT halfway between the scribbles (x≈0.55). An energy that ignored the ink would
/// split at the midpoint and fail here.
#[test]
fn the_colour_cut_hugs_the_ink_not_the_midpoint() {
    let strokes = boxed_with_divider(0.7, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.25, 0.3), Vec2::new(0.25, 0.7)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.85, 0.3), Vec2::new(0.85, 0.7)],
            width: 0.0,
        },
    ];
    let regions = colorize(&strokes, &scribbles, 80.0);
    assert_eq!(regions.len(), 2, "two colours → two regions");

    let r0 = &regions.iter().find(|r| r.label == 0).expect("label 0").fill;
    let r1 = &regions.iter().find(|r| r.label == 1).expect("label 1").fill;

    let b = max_x(r0);
    assert!(
        (0.63..=0.77).contains(&b),
        "the left colour must reach the ink at 0.7, not the scribble midpoint 0.55 (got {b})"
    );
    assert!(
        area(r0) > area(r1),
        "the off-center divider gives the left colour more area ({} vs {})",
        area(r0),
        area(r1)
    );
}

/// A gap in the divider does NOT leak the colour across it — the cut jumps the gap in white
/// (paying its width) rather than routing a whole colour through it. Symmetric divider ⇒ two
/// comparable regions, one per side.
#[test]
fn a_gap_in_the_divider_does_not_leak_the_colour() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.28, 0.3), Vec2::new(0.28, 0.7)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.72, 0.3), Vec2::new(0.72, 0.7)],
            width: 0.0,
        },
    ];
    let regions = colorize(&strokes, &scribbles, 80.0);
    assert_eq!(regions.len(), 2, "two comparable regions, one per side");

    let r0 = &regions.iter().find(|r| r.label == 0).expect("label 0").fill;
    let r1 = &regions.iter().find(|r| r.label == 1).expect("label 1").fill;
    assert!(centroid_x(r0) < 0.5, "label 0 on the left");
    assert!(centroid_x(r1) > 0.5, "label 1 on the right");

    let (a0, a1) = (area(r0), area(r1));
    let ratio = a0.min(a1) / a0.max(a1);
    assert!(
        ratio > 0.55,
        "the gap must not leak a colour: areas {a0} vs {a1}"
    );
}

/// HR-5: two identical clicks give the same drawing — same geometry, to the vertex.
#[test]
fn the_colorize_is_deterministic() {
    let strokes = boxed_with_divider(0.6, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.25, 0.3), Vec2::new(0.25, 0.7)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.8, 0.3), Vec2::new(0.8, 0.7)],
            width: 0.0,
        },
    ];
    let a = colorize(&strokes, &scribbles, 80.0);
    let b = colorize(&strokes, &scribbles, 80.0);
    assert_eq!(
        a.len(),
        2,
        "the fixture must actually colour (else 0==0 is vacuous)"
    );
    assert_eq!(a.len(), b.len());
    for (ra, rb) in a.iter().zip(&b) {
        assert_eq!(ra.label, rb.label);
        assert_eq!(ra.fill.outer, rb.fill.outer, "geometry must be identical");
    }
}

/// **O que o artista PINTA é o que SEMEIA.** Um TOQUE curto com pincel grosso tem eixo de
/// poucos pixels, e uma semente desse tamanho DEGENERA o corte: o mínimo passa a ser "cercar
/// o pixel" (perímetro pequeno em papel caro) em vez de correr pela tinta, e a região não
/// colore. Com a espessura semeando, o mesmo toque funciona. É o red-first do campo `width`:
/// zerá-lo derruba a metade `fat`.
#[test]
fn a_short_fat_dab_seeds_its_region_where_a_thin_one_degenerates() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    // Um TOQUE: dois pontos quase coincidentes — o que um clique curto de fato produz.
    let dab = |width: f32| {
        vec![
            Scribble {
                label: 0,
                points: vec![Vec2::new(0.28, 0.50), Vec2::new(0.28, 0.51)],
                width,
            },
            Scribble {
                label: 1,
                points: vec![Vec2::new(0.72, 0.50), Vec2::new(0.72, 0.51)],
                width,
            },
        ]
    };
    let thin = colorize(&strokes, &dab(0.0), 80.0);
    let fat = colorize(&strokes, &dab(0.12), 80.0);
    assert_eq!(
        fat.len(),
        2,
        "um toque com pincel grosso tem de colorir os dois lados (got {})",
        fat.len()
    );
    assert!(
        thin.len() < 2,
        "se o eixo puro já colorisse, a largura não seria load-bearing (fino {})",
        thin.len()
    );
}

/// No lines or no scribbles → nothing (a colour with no line to bound it is not a region).
#[test]
fn no_lines_or_no_scribbles_yields_nothing() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    assert!(colorize(&strokes, &[], 80.0).is_empty());
    let scr = vec![Scribble {
        label: 0,
        points: vec![Vec2::new(0.5, 0.5)],
        width: 0.0,
    }];
    assert!(colorize(&[], &scr, 80.0).is_empty());
}

/// **A régua do CAMINHO REAL** (`--release --ignored --nocapture`).
///
/// A régua do `flow_tests.rs` mede um corte binário com `v_ink = 1` — um pior caso FORÇADO,
/// onde atravessar a tinta custa e o fluxo percorre a grade inteira. O produto roda
/// `v_ink = 0` (atravessar a tinta é de graça, e é isso que confina o corte à linha), então
/// aquele número **não descreve o que o artista paga**. Esta régua entra pela porta pública,
/// com a arte e os rabiscos que o produto tem, e varre até o `MAX_SIDE` que o `Grid` impõe.
#[test]
#[ignore = "régua de medição — rode com --release --ignored --nocapture"]
fn measure_the_product_colorize_cost() {
    // O `Grid` reserva `MARGIN_PX` dos dois lados e depois CLAMPA a escala em `MAX_SIDE`,
    // então o lado pedido sai de `scale = (side - 2*margem) / vão_da_arte`.
    let span = 0.8_f32;
    let sides = [512_usize, 1024, 2048, 4096];
    let strokes = boxed_with_divider(0.7, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.3, 0.3), Vec2::new(0.3, 0.7)],
            width: 0.02,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.8, 0.3), Vec2::new(0.8, 0.7)],
            width: 0.02,
        },
    ];
    println!("\n  lado     precision      ms   regiões");
    for side in sides {
        let precision = (side as f32 - 40.0) / span;
        let t = std::time::Instant::now();
        let out = colorize(&strokes, &scribbles, precision);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        println!("  {side:>4}²  {precision:>9.0}  {ms:>8.1}  {}", out.len());
        assert_eq!(
            out.len(),
            2,
            "a régua tem de estar medindo um corte que COLORE"
        );
    }
}

/// **A contagem de rótulos NÃO é o multiplicador — a régua REFUTOU a hipótese.**
///
/// Eu a escrevi esperando que o guloso um-contra-todos (`§3`) fizesse o Apply custar a grade
/// VEZES o número de cores. Medido a 2048², o custo **CAI** com mais cores: 2 → 172,6 s ·
/// 4 → 39,2 s · 8 → 9,0 s. Cada corte binário adicional dá ao fluxo mais fonte e mais
/// sumidouro, e ele termina mais cedo; o que domina não é quantas cores há, é se as sementes
/// **se contradizem sobre a mesma linha** (vide `measure_a_scribble_that_crosses_the_ink`).
/// Fica como régua porque a hipótese é intuitiva e alguém vai reconstruí-la.
#[test]
#[ignore = "régua de medição — rode com --release --ignored --nocapture"]
fn measure_the_cost_is_not_driven_by_label_count() {
    let side = 2048.0_f32;
    let precision = (side - 40.0) / 0.8;
    let strokes = boxed_with_divider(0.7, (0.45, 0.55));
    println!("\n  rótulos      ms");
    for n in [2_usize, 4, 8] {
        // Rabiscos empilhados na vertical: cada um pede a sua fatia da mesma arte.
        let scribbles: Vec<Scribble> = (0..n)
            .map(|k| {
                let y = 0.15 + 0.7 * (k as f32 + 0.5) / n as f32;
                Scribble {
                    label: k as u16,
                    points: vec![Vec2::new(0.3, y), Vec2::new(0.85, y)],
                    width: 0.02,
                }
            })
            .collect();
        let t = std::time::Instant::now();
        let out = colorize(&strokes, &scribbles, precision);
        println!(
            "  {n:>7}  {:>8.1}   ({} regiões)",
            t.elapsed().as_secs_f64() * 1e3,
            out.len()
        );
    }
}

/// **A régua que isola a CAUSA**: o mesmo tamanho, o mesmo número de rótulos, mudando só se
/// o rabisco ATRAVESSA a linha. Um rabisco que cruza o divisor pede uma coisa que a arte
/// contradiz (a semente diz "um só rótulo dos dois lados", a tinta diz "corte aqui").
#[test]
#[ignore = "régua de medição — rode com --release --ignored --nocapture"]
fn measure_a_scribble_that_crosses_the_ink() {
    let side = 2048.0_f32;
    let precision = (side - 40.0) / 0.8;
    let strokes = boxed_with_divider(0.7, (0.45, 0.55));
    let scr = |crosses: bool| {
        let x_end = if crosses { 0.85 } else { 0.6 };
        vec![
            Scribble {
                label: 0,
                points: vec![Vec2::new(0.3, 0.35), Vec2::new(x_end, 0.35)],
                width: 0.02,
            },
            Scribble {
                label: 1,
                points: vec![Vec2::new(0.75, 0.65), Vec2::new(0.85, 0.65)],
                width: 0.02,
            },
        ]
    };
    // O 3º caso é o que explodiu na régua de rótulos: os DOIS rabiscos reivindicam os dois
    // lados, então as sementes se contradizem uma à outra através da mesma linha.
    let both = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.3, 0.35), Vec2::new(0.85, 0.35)],
            width: 0.02,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.3, 0.65), Vec2::new(0.85, 0.65)],
            width: 0.02,
        },
    ];
    let t = std::time::Instant::now();
    let out = colorize(&strokes, &both, precision);
    println!(
        "  AMBOS atravessam           →  {:>9.1} ms  ({} regiões)",
        t.elapsed().as_secs_f64() * 1e3,
        out.len()
    );
    for crosses in [false, true] {
        let t = std::time::Instant::now();
        let out = colorize(&strokes, &scr(crosses), precision);
        println!(
            "  atravessa a tinta: {crosses:>5}  →  {:>9.1} ms  ({} regiões)",
            t.elapsed().as_secs_f64() * 1e3,
            out.len()
        );
    }
}
