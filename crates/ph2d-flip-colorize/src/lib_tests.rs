//! Gates for the Colorize engine (`docs/Flip/09 §3`, §7.3).
//!
//! The load-bearing gate is `the_colour_cut_hugs_the_ink_not_the_midpoint`: it is what only
//! LazyBrush does — the boundary between two colours is *attracted to the line*, not drawn
//! halfway between the scribbles. It is red-first-able by construction: strip the ink from
//! the energy (`V_pq` uniform) and the split falls at the scribble midpoint, failing the
//! assertion. The flow solver itself is pinned by `flow_tests.rs` (BK ≡ Edmonds–Karp); these
//! gates pin the WIRING — energy assembly, the guloso multiway, the geometry — on a fixture
//! that contains the phenomenon.

use super::{ColorRegion, Scribble, colorize};

/// O raio da bola (px de buffer) que os gates usam.
///
/// As fixtures têm um vão de 0,1 unidade de mundo e rodam a `precision` 80 ⇒ 8 px. A bola
/// tem de ser mais larga que ele (diâmetro > 8) para **costurar** o vão, senão os dois lados
/// caem na mesma região e a propriedade que estes gates medem — a fronteira colar na tinta
/// em vez de vazar pelo vão — não tem por onde acontecer (`§8`).
const GATE_TRAP_PX: f32 = 6.0;
use ph2d_core::Vec2;
use ph2d_flip_fill::{BOUNDARY, FillResult, Grid, signed_area};

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
    let regions = colorize(&strokes, &scribbles, 80.0, GATE_TRAP_PX);
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
    let regions = colorize(&strokes, &scribbles, 80.0, GATE_TRAP_PX);
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
    let a = colorize(&strokes, &scribbles, 80.0, GATE_TRAP_PX);
    let b = colorize(&strokes, &scribbles, 80.0, GATE_TRAP_PX);
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

/// **A redução por regiões CUROU a degenerescência da semente de um pixel.**
///
/// Antes do `§8` este gate afirmava o contrário — que um toque fino *não* colore — e estava
/// certo sobre o grafo de PIXELS: ali o corte mais barato para uma semente de 1 px é *cercar
/// aquele pixel* (perímetro minúsculo em papel caro) em vez de correr pela tinta. Sobre o
/// grafo de REGIÕES esse corte não existe como opção: um pixel identifica a região dele, e a
/// região inteira é o nó. O gate agora pina a cura, para ninguém "consertar" a largura de
/// volta para um requisito de correção que ela deixou de ser.
#[test]
fn the_region_reduction_cures_the_one_pixel_seed_degeneracy() {
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
    for width in [0.0, 0.12] {
        let out = colorize(&strokes, &dab(width), 80.0, GATE_TRAP_PX);
        assert_eq!(
            out.len(),
            2,
            "um toque de largura {width} tem de colorir os dois lados (got {})",
            out.len()
        );
    }
}

/// O que a largura AINDA significa: **cobertura**. Ela decide quantas regiões o rabisco
/// reivindica — um traço gordo o bastante para atravessar a linha reivindica os dois lados
/// para a MESMA cor, e um fino reivindica só o lado em que o eixo caiu. Não é mais correção;
/// é o artista dizendo, com o pincel, o quanto ele está pegando.
#[test]
fn the_width_decides_how_many_regions_one_scribble_claims() {
    // Divisor SÓLIDO (vão de largura zero). Com um vão, uma cor sozinha atravessa a costura
    // de papel de qualquer jeito — que é o comportamento certo do LazyBrush e faz a largura
    // parar de decidir coisa alguma. A fixture tem de conter o fenômeno que ela mede.
    let strokes = boxed_with_divider(0.5, (0.5, 0.5));
    // Eixo à ESQUERDA do divisor, perto dele. Gordo o bastante, a cápsula alcança a direita.
    let one = |width: f32| {
        vec![Scribble {
            label: 0,
            points: vec![Vec2::new(0.44, 0.30), Vec2::new(0.44, 0.70)],
            width,
        }]
    };
    let thin = colorize(&strokes, &one(0.0), 80.0, GATE_TRAP_PX);
    let fat = colorize(&strokes, &one(0.24), 80.0, GATE_TRAP_PX);
    assert_eq!(thin.len(), 1, "o eixo fino pega so o lado dele");
    assert_eq!(
        fat.len(),
        2,
        "a capsula gorda atravessa a linha e reivindica os dois lados"
    );
}

/// No lines or no scribbles → nothing (a colour with no line to bound it is not a region).
#[test]
fn no_lines_or_no_scribbles_yields_nothing() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    assert!(colorize(&strokes, &[], 80.0, GATE_TRAP_PX).is_empty());
    let scr = vec![Scribble {
        label: 0,
        points: vec![Vec2::new(0.5, 0.5)],
        width: 0.0,
    }];
    assert!(colorize(&[], &scr, 80.0, GATE_TRAP_PX).is_empty());
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
        let out = colorize(&strokes, &scribbles, precision, GATE_TRAP_PX);
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
        let out = colorize(&strokes, &scribbles, precision, GATE_TRAP_PX);
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
    let out = colorize(&strokes, &both, precision, GATE_TRAP_PX);
    println!(
        "  AMBOS atravessam           →  {:>9.1} ms  ({} regiões)",
        t.elapsed().as_secs_f64() * 1e3,
        out.len()
    );
    for crosses in [false, true] {
        let t = std::time::Instant::now();
        let out = colorize(&strokes, &scr(crosses), precision, GATE_TRAP_PX);
        println!(
            "  atravessa a tinta: {crosses:>5}  →  {:>9.1} ms  ({} regiões)",
            t.elapsed().as_secs_f64() * 1e3,
            out.len()
        );
    }
}

/// **Uma cor sozinha não toma a tela por um furo de agulha.**
///
/// Achado MEDINDO durante o `§8`: espalhar uma cor única por qualquer aresta de papel do
/// grafo colore **94% da grade** — a linha rasterizada tem furos de um pixel, cada furo vira
/// uma costura barata, e a cor escorre por ela para fora da caixa. Uma costura existe
/// justamente porque a bola NÃO passou ali; atravessá-la desfaz a promessa da trapped-ball.
/// O oráculo é a ÁREA (a aparência), não a contagem de regiões.
#[test]
fn one_colour_does_not_take_the_canvas_through_a_pinhole() {
    let strokes = boxed_with_divider(0.5, (0.5, 0.5));
    let scr = vec![Scribble {
        label: 0,
        points: vec![Vec2::new(0.30, 0.30), Vec2::new(0.30, 0.70)],
        width: 0.02,
    }];
    let out = colorize(&strokes, &scr, 80.0, GATE_TRAP_PX);
    let painted: f32 = out.iter().map(|r| area(&r.fill)).sum();
    // A metade esquerda da caixa mede 0,4 x 0,8 = 0,32. A caixa inteira, 0,64. Qualquer
    // coisa perto disso (ou acima) é a cor tendo escapado do lado em que foi pedida.
    assert!(
        painted > 0.15 && painted < 0.45,
        "a cor tem de ficar na metade que o rabisco tocou (pintou {painted:.3})"
    );
}

/// **A cor não escapa da caixa** — o gate de regressão do smoke reprovado em 2026-07-19.
///
/// A arte é a da cena `PH2D_FLIP_COLORIZE_SMOKE=1`: caixa `[-4,4]x[-2.5,2.5]`, divisor em
/// `x=1` com um vão LARGO (1,2 unidades) e um rabisco de cada lado. O produto pintou a GRADE
/// INTEIRA de uma cor só. Causa: a margem da grade é estreita demais para a bola, então não
/// tinha núcleo próprio, e a dilatação — que então atravessava a tinta — deixava a região do
/// lobo esquerdo ENGOLIR o lado de fora (54.927 px numa grade de 99.441).
///
/// O oráculo é a **ÁREA pintada**, não a contagem de regiões: era assim que o defeito se via
/// na tela, e contar regiões dava 4 (parecia plausível) enquanto a tinta cobria tudo.
#[test]
fn the_colour_does_not_escape_the_box() {
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
    for (a, b) in [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5)),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5)),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5)),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5)),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6)),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5)),
    ] {
        let pts = seg_(a, b, 24);
        let n = pts.len();
        strokes.push((pts, vec![0.26; n], false));
    }
    let scribbles = vec![
        Scribble {
            label: 0,
            points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg_(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
            width: 0.15,
        },
    ];
    let ratio_of = |out: &[ColorRegion]| -> (f32, f32, f32) {
        let a0: f32 = out
            .iter()
            .filter(|r| r.label == 0)
            .map(|r| area(&r.fill))
            .sum();
        let a1: f32 = out
            .iter()
            .filter(|r| r.label == 1)
            .map(|r| area(&r.fill))
            .sum();
        (a0, a1, a0 + a1)
    };

    // (1) ESCAPE — a cor NÃO vaza para fora da caixa (o bug do smoke reprovado). Vale com o
    //     Trap no default (0): mesmo sem fechar o vão, a cor fica confinada ao componente.
    let out = colorize(&strokes, &scribbles, 40.0, 0.0);
    let labels: std::collections::BTreeSet<u16> = out.iter().map(|r| r.label).collect();
    assert_eq!(labels.len(), 2, "as DUAS cores tem de sobreviver");
    let (.., painted) = ratio_of(&out);
    assert!(
        painted < 42.0,
        "a cor escapou da caixa (pintou {painted:.1} de area, a caixa tem 40)"
    );

    // (2) HUG — a fronteira cola no divisor fora-do-centro (x=+1 ⇒ esquerda:direita = 5:3 ≈
    //     1,67). Num único componente aberto o Voronoi parte no MEIO dos rabiscos; colar na
    //     LINHA é o que o **Trap** faz — fecha o vão de 1,2 unidades (48 px a esta precisão),
    //     separando em dois componentes de uma cor cada, que são PREENCHIDOS até a tinta.
    //     Isto testa o mecanismo REAL de hug (o knob), não uma promessa que o corte não cumpre.
    let seam_trap = 26.0; // raio > metade do vão (24) ⇒ a bola não passa ⇒ costura
    let hugged = colorize(&strokes, &scribbles, 40.0, seam_trap);
    let (a0, a1, _) = ratio_of(&hugged);
    let ratio = a0 / a1;
    assert!(
        (1.4..2.0).contains(&ratio),
        "com o vao fechado (Trap) a fronteira tem de colar no divisor (razao {ratio:.2}, ~1,67)"
    );
}

/// **4 cores num blob ABERTO viram FAIXAS PARELHAS** (3º smoke, 2026-07-19). Sem uma linha
/// entre as cores não há tinta a que colar, e o meio geométrico é a resposta honesta — quatro
/// rabiscos parelhos têm de render quatro faixas parelhas, nunca a 1ª cor levando tudo (o
/// bug original) nem uma cor do meio espremida (o guloso e o min-cut de Potts, ambos medidos
/// e reprovados — vide `solve`).
#[test]
fn four_colours_in_an_open_blob_split_into_even_bands() {
    // Um octógono ABERTO (sem divisores internos), bbox ~[-4,4]x[-3,3].
    let ring: Vec<Vec2> = (0..8)
        .map(|k| {
            let a = std::f32::consts::TAU * k as f32 / 8.0;
            Vec2::new(4.0 * a.cos(), 3.0 * a.sin())
        })
        .collect();
    let n = ring.len();
    let strokes = vec![(ring, vec![0.0; n], true)];
    // Quatro rabiscos verticais, um por cor, espalhados no interior.
    let scr = |x: f32, label: u16| Scribble {
        label,
        points: vec![Vec2::new(x, -1.5), Vec2::new(x, 1.5)],
        width: 0.15,
    };
    let scribbles = vec![scr(-2.4, 0), scr(-0.8, 1), scr(0.8, 2), scr(2.4, 3)];
    let out = colorize(&strokes, &scribbles, 40.0, 0.0);
    let mut by_label: std::collections::BTreeMap<u16, f32> = std::collections::BTreeMap::new();
    for r in &out {
        *by_label.entry(r.label).or_default() += area(&r.fill);
    }
    assert_eq!(by_label.len(), 4, "as quatro cores tem de sobreviver");
    let (lo, hi) = by_label
        .values()
        .fold((f32::MAX, f32::MIN), |(l, h), &a| (l.min(a), h.max(a)));
    assert!(
        hi / lo < 1.8,
        "faixas parelhas: a maior nao pode passar de ~1,8x a menor (areas {by_label:?})"
    );
}

/// 🔴 **A fronteira de um componente CONTESTADO cai NA TINTA, não no meio dos rabiscos**
/// (4º smoke, 2026-07-20: a caixa com divisor saía com a fronteira numa reta vertical no meio
/// do papel, à ESQUERDA da linha — o Voronoi de células pesava `V_pq`, que é custo de CORTE:
/// tinta de graça = a linha era invisível para a frente).
///
/// Mesma arte do gate irmão (`the_colour_cut_hugs_the_ink_not_the_midpoint`: divisor
/// fora-do-centro em x=0,7, vão no meio), mas com **Trap 0** — o vão não é costurado, a caixa
/// inteira é UM componente contestado, e é o caminho do VORONOI (não o do preenchimento) que
/// tem de colar na linha. A tinta é INTRANSPONÍVEL para a frente: quem quer o outro lado paga
/// a volta pelo vão, e longe do vão cada lado pertence a quem está nele.
///
/// O filtro em `y` exclui a vizinhança do vão de propósito: ali a disputa é honesta (não há
/// tinta), e o que o gate afirma é o comportamento LONGE dele.
#[test]
fn a_contested_boundary_lands_on_the_ink_not_between_the_scribbles() {
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
    // Precisão 400 ⇒ grade ~360 px. ⚠️ Load-bearing: numa grade ≤ 160 px a célula da
    // sobre-segmentação antiga tinha 1 px e o defeito não existia — o fixture tem de conter
    // o fenômeno do produto (célula > 1 cavalgando a linha).
    let regions = colorize(&strokes, &scribbles, 400.0, 0.0); // Trap 0: nada costurado.
    let r0 = &regions.iter().find(|r| r.label == 0).expect("label 0").fill;
    let b = r0
        .outer
        .iter()
        .filter(|p| p.y < 0.35 || p.y > 0.65)
        .fold(f32::MIN, |m, p| m.max(p.x));
    assert!(
        (0.63..=0.77).contains(&b),
        "longe do vao, a cor esquerda tem de alcancar a linha em 0,7 — nao parar no meio \
         dos rabiscos (max_x filtrado = {b})"
    );
}

/// 🔴 **Uma cor não sangra através da linha que divide dois cômodos** (4º smoke: nos lobos do
/// blob, o teal atravessava a linha para dentro do lobo vermelho). Planta: duas salas lado a
/// lado abertas para um corredor embaixo — UM componente contestado. O rabisco da direita fica
/// COLADO na parede; com a tinta transponível (o bug), ele reivindica uma tira da sala
/// esquerda através da parede. Com a tinta intransponível, o caminho dele até lá é a volta
/// pelo corredor, e a sala esquerda pertence a quem está nela.
#[test]
fn a_colour_does_not_bleed_across_the_wall_between_two_rooms() {
    let strokes = vec![
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
        // A parede interna: desce de y=0,1 até 0,62 — o corredor [0,62..0,9] fica aberto.
        (
            vec![Vec2::new(0.5, 0.1), Vec2::new(0.5, 0.62)],
            vec![0.0; 2],
            false,
        ),
    ];
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.18, 0.3), Vec2::new(0.18, 0.45)],
            width: 0.0,
        },
        // Colado na parede, do lado direito — o pior caso para o sangramento.
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.54, 0.3), Vec2::new(0.54, 0.45)],
            width: 0.0,
        },
    ];
    // Precisão 400: grade > 160 px, o regime onde a célula antiga cavalgava a parede.
    let regions = colorize(&strokes, &scribbles, 400.0, 0.0);
    let r1 = &regions.iter().find(|r| r.label == 1).expect("label 1").fill;
    // Dentro da SALA (y < 0,55, longe da boca do corredor), a cor da direita nunca aparece à
    // esquerda da parede (x=0,5).
    let min_x = r1
        .outer
        .iter()
        .filter(|p| p.y < 0.55)
        .fold(f32::MAX, |m, p| m.min(p.x));
    assert!(
        min_x >= 0.44,
        "a cor da direita sangrou atraves da parede para a sala esquerda (min_x = {min_x})"
    );
}

/// **O pedágio de aperto segura a cor rival na banda do VÃO.** Sem ele (`SQUEEZE = 0`,
/// medido na tabela de `voronoi.rs`), a cor cuja semente está perto do vão vence a corrida
/// geodésica e (a) pinta uma película de ~8 px na face ALHEIA do divisor, empurrando a cor
/// da esquerda para 0,679 em vez de 0,697, e (b) se espalha PELO LADO ALHEIO até y bem fora
/// do vão (min_x longe do vão = 0,576). Com o pedágio, fora da banda do vão a cor rival
/// simplesmente não existe à esquerda da linha (0,702 = a própria linha).
///
/// Mutação que sangra: `SQUEEZE = 0` (ou apagar o `toll` da relaxação).
#[test]
fn the_squeeze_toll_keeps_the_rival_inside_the_gap_band() {
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
    let regions = colorize(&strokes, &scribbles, 400.0, 0.0);
    let spread = regions
        .iter()
        .filter(|r| r.label == 1)
        .flat_map(|r| r.fill.outer.iter())
        .filter(|p| p.y < 0.35 || p.y > 0.65)
        .fold(f32::MAX, |m, p| m.min(p.x));
    assert!(
        spread >= 0.68,
        "longe do vao, a cor da direita nao pode existir a esquerda da linha (min_x = {spread})"
    );
}

/// **Uma linha DIAGONAL é parede, não peneira.** A frente do Voronoi anda em 8 vizinhanças
/// (o chanfro 5/7 que deixa as faixas retas), e um passo diagonal entre dois pixels de tinta
/// diagonais escorregaria POR DENTRO da linha — a mesma linha que o flood de componentes
/// (4-conexo) trata como parede. As duas leis têm de concordar: o passo diagonal com tinta
/// nos DOIS cantos é recusado.
///
/// ⚠️ Este é o pin de PRODUTO (a cápsula rasterizada a 45° sai 4-conexa, então a recusa do
/// canto raramente dispara aqui); quem carrega a mutação da recusa é o gate isolado
/// `a_diagonal_pinch_is_sealed_even_with_zero_toll`, sobre uma corrente diagonal PURA.
#[test]
fn a_diagonal_line_is_a_wall_not_a_sieve() {
    let strokes = vec![
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
        // O divisor diagonal (x=y), partido por um vão no meio.
        (
            vec![Vec2::new(0.1, 0.1), Vec2::new(0.44, 0.44)],
            vec![0.0; 2],
            false,
        ),
        (
            vec![Vec2::new(0.56, 0.56), Vec2::new(0.9, 0.9)],
            vec![0.0; 2],
            false,
        ),
    ];
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.62, 0.3), Vec2::new(0.7, 0.38)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.3, 0.62), Vec2::new(0.38, 0.7)],
            width: 0.0,
        },
    ];
    let regions = colorize(&strokes, &scribbles, 400.0, 0.0);
    let r0 = &regions.iter().find(|r| r.label == 0).expect("label 0").fill;
    // Longe do vão (|p − centro| > 0,15), a cor de baixo-direita nunca cruza para y > x.
    let worst = r0
        .outer
        .iter()
        .filter(|p| {
            let (dx, dy) = (p.x - 0.5, p.y - 0.5);
            dx * dx + dy * dy > 0.15 * 0.15
        })
        .fold(f32::MIN, |m, p| m.max(p.y - p.x));
    assert!(
        worst <= 0.08,
        "a cor atravessou a linha diagonal fora do vao (max(y-x) filtrado = {worst})"
    );
}

/// **A camada dura, sozinha: tinta é PAREDE mesmo com pedágio zero.** O pedágio de aperto
/// (`SQUEEZE/(1+d²)`) também encarece a tinta, então nas grades pequenas dos gates acima as
/// duas camadas se cobrem — mutar só a recusa `!is_ink` do `member` fica verde
/// ([[feedback_layered_defenses_need_per_layer_gates]]). Mas as camadas DIVERGEM num canvas
/// grande: um desvio mais longo que `SQUEEZE/STEP ≈ 820 px` por pixel de tinta tornaria
/// ATRAVESSAR a linha mais barato que contorná-la, e a cor cruzaria uma linha SÓLIDA — o
/// contrato que a linha é. Este gate isola a camada: EDT fabricada longe-de-tudo (pedágio
/// 0 em toda parte) e uma parede de tinta no meio — só a recusa dura segura a frente.
///
/// Mutação que sangra: `member` sem o `!is_ink` (a frente entra na tinta e sai do outro lado).
#[test]
fn ink_is_a_wall_even_with_zero_toll() {
    let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(30.0, 30.0), 1.0, 4, 4096);
    let (w, h) = (g.w, g.h);
    for y in 0..h {
        g.flags[y * w + 15] |= BOUNDARY; // a parede: coluna sólida, sem vão
    }
    let seg = super::segment::Segmentation {
        component: vec![0; w * h], // UM componente à força: só a lei da tinta separa
        count: 1,
        ink_dist2: vec![1_000_000; w * h], // "longe de tudo" ⇒ pedágio 0 em toda parte
    };
    let labels = vec![(0u16, vec![10 * w + 5])];
    let mut scratch = super::voronoi::Scratch::new(&seg.ink_dist2);
    let mut assign = vec![None; w * h];
    super::voronoi::claim(&g, &seg, 0, &labels, &mut scratch, &mut assign);
    assert_eq!(
        assign[12 * w + 5],
        Some(0),
        "controle positivo: a frente tem de correr no lado semeado"
    );
    assert_eq!(
        assign[10 * w + 25],
        None,
        "a frente atravessou uma parede solida de tinta"
    );
}

/// **A recusa do canto, sozinha: uma corrente DIAGONAL pura é selada.** Irmão do gate acima,
/// para a lei 2 de `voronoi.rs`: dois pixels de tinta em diagonal não são 8-adjacentes, e o
/// passo diagonal da frente passaria POR ENTRE eles — a peneira. A corrente vai de canto a
/// canto (não há volta por fora), o pedágio é zero (EDT fabricada): só a recusa do canto
/// duplo separa os dois triângulos.
///
/// Mutação que sangra: apagar o `continue` do canto duplo em `voronoi::claim`.
#[test]
fn a_diagonal_pinch_is_sealed_even_with_zero_toll() {
    let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(30.0, 30.0), 1.0, 4, 4096);
    let (w, h) = (g.w, g.h);
    let side = w.min(h);
    for j in 0..side {
        g.flags[j * w + j] |= BOUNDARY; // a corrente diagonal pura, canto a canto
    }
    let seg = super::segment::Segmentation {
        component: vec![0; w * h],
        count: 1,
        ink_dist2: vec![1_000_000; w * h],
    };
    let labels = vec![(0u16, vec![5 * w + 20])]; // semente no triangulo x > y
    let mut scratch = super::voronoi::Scratch::new(&seg.ink_dist2);
    let mut assign = vec![None; w * h];
    super::voronoi::claim(&g, &seg, 0, &labels, &mut scratch, &mut assign);
    assert_eq!(
        assign[3 * w + 25],
        Some(0),
        "controle positivo: a frente corre no proprio triangulo"
    );
    assert_eq!(
        assign[20 * w + 5],
        None,
        "a frente escorreu POR DENTRO da corrente diagonal (a peneira)"
    );
}
