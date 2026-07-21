//! **Réguas de medição e sondas de diagnóstico** do Colorize — todas `#[ignore]`, rodadas à
//! mão (`--release --ignored --nocapture`). Irmãs do `lib_tests.rs`, separadas pelo teto de
//! LOC do workspace (700): elas MEDEM, não afirmam, e por isso formam uma unidade própria.

use super::tests::{GATE_TRAP_PX, boxed_with_divider};
use super::{Scribble, colorize};
use ph2d_core::Vec2;
use ph2d_flip_fill::{BOUNDARY, Grid};

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

/// 🔬 **Sonda do VAZAMENTO** (6º smoke: "trap 0 e trap máximo vazam parecidos. se há ajustes
/// possíveis coloque no painel"). Varre o Trap (raio da bola, px de buffer) e o `SQUEEZE`
/// (pedágio de aperto, via `PH2D_COLORIZE_SQUEEZE`) na cena do smoke, medindo:
///   · quantas regiões / a fronteira colou na linha? (o Trap SELOU o vão?)
///   · a LENTE — o quão fundo o azul entra pelo vão (min_x do azul).
#[test]
#[ignore = "sonda de diagnóstico — rode com --release --ignored --nocapture"]
fn probe_the_bleed_through_the_gap() {
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
    let precision = 40.0f32;
    // gap 1,2 doc → 48 px de buffer a esta precisão; a bola sela quando o raio > 24.
    let measure = |trap: f32, squeeze: u32| -> (usize, f32, f32) {
        let regions = super::colorize_with(&strokes, &scribbles, precision, trap, squeeze);
        // A LENTE: o menor x que o AZUL alcança (o bojo entra pela esquerda do divisor x=1).
        let lens = regions
            .iter()
            .filter(|r| r.label == 1)
            .flat_map(|r| r.fill.outer.iter())
            .fold(f32::MAX, |m, p| m.min(p.x));
        // A FRONTEIRA longe do vão: o maior x que o VERMELHO alcança (cola na linha = ~1,0).
        let front = regions
            .iter()
            .filter(|r| r.label == 0)
            .flat_map(|r| r.fill.outer.iter())
            .filter(|p| p.y.abs() > 1.2)
            .fold(f32::MIN, |m, p| m.max(p.x));
        (regions.len(), lens, front)
    };

    println!("\n=== VARRE O TRAP (px de buffer; sela em >24) ===");
    println!("  trap_px  regiões   lente(min_x azul)   fronteira(max_x verm, |y|>1.2)");
    for trap in [0.0f32, 6.0, 12.0, 18.0, 24.0, 30.0, 40.0] {
        let (n, lens, front) = measure(trap, super::DEFAULT_SQUEEZE);
        println!(
            "  {trap:>6.0}  {n:>6}    {lens:>+13.3}      {front:>+13.3}   (slider≈{:.0}px)",
            trap / 1.6
        );
    }
    println!("\n=== VARRE O SQUEEZE (Bleed; trap 0) ===");
    println!("  squeeze  lente(min_x azul)");
    for sq in [256u32, 1024, 4096, 16384, 32768, 65536, 131072] {
        let (_, lens, _) = measure(0.0, sq);
        println!("  {sq:>7}  {lens:>+13.3}");
    }
    println!("\nNota: o produto faz trap_px_buffer = slider × 1.6 (o doc_per_px cancela).");
}

/// 🔬 **Sonda do ZÍPER** (6º smoke, 2026-07-20: "ainda não perfeito" — dentes finos e
/// REGULARES alternando as duas cores em cima do divisor, mais fortes perto da lente).
/// Reproduz a cena do smoke na precisão do PRODUTO (~128 = 1,6/doc_per_px no zoom da foto)
/// e caça o zíper na GEOMETRIA final: caminha a borda de cada cor perto do divisor e
/// imprime os trechos onde o desvio ao eixo TROCA DE SINAL em sequência (a assinatura do
/// zíper — o tremor da mão não alterna a cada ~2 px).
#[test]
#[ignore = "sonda de diagnóstico — rode com --release --ignored --nocapture"]
fn probe_the_zipper_on_the_divider() {
    let hand = |pts: &[Vec2], seed: usize| -> Vec<Vec2> {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        pts.iter()
            .enumerate()
            .map(|(i, p)| Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05))
            .collect()
    };
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
    for (a, b, s, n) in [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize, 24usize),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7, 24),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13, 24),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29, 24),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6), 41, 41),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5), 53, 53),
    ] {
        let pts = hand(&seg_(a, b, n), s);
        let m = pts.len();
        strokes.push((pts, vec![0.13; m], false));
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
    // A arte de MÃO REAL: pontos na taxa do ponteiro (~2,5 px) com ruído de ±1,3 px — o
    // que um arrasto de mouse de fato produz (o `hand()` de 41 pontos é limpo demais).
    let noisy = |pts: &[Vec2], seed: usize, step: f32, jitter: f32| -> Vec<Vec2> {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        let mut out = Vec::new();
        let mut k = 0usize;
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let d = Vec2::new(b.x - a.x, b.y - a.y);
            let len = (d.x * d.x + d.y * d.y).sqrt();
            let n = (len / step).ceil().max(1.0) as usize;
            for s in 0..n {
                let t = s as f32 / n as f32;
                let p = Vec2::new(a.x + d.x * t, a.y + d.y * t);
                out.push(Vec2::new(
                    p.x + h(k + seed) * jitter,
                    p.y + h(k + seed + 91) * jitter,
                ));
                k += 1;
            }
        }
        out.push(*pts.last().expect("polyline"));
        out
    };
    let noisy_strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = strokes
        .iter()
        .enumerate()
        .map(|(i, (pts, _, closed))| {
            let np = noisy(pts, i * 977, 0.02, 0.02);
            let m = np.len();
            (np, vec![0.13; m], *closed)
        })
        .collect();

    for (name, art, precision) in [
        ("limpo@128", &strokes, 128.0f32),
        ("limpo@400", &strokes, 400.0),
        ("MÃO@128", &noisy_strokes, 128.0),
    ] {
        let regions = colorize(art, &scribbles, precision, 0.0);
        let dividers: Vec<&[Vec2]> = art[4..6].iter().map(|(p, ..)| p.as_slice()).collect();
        // Desvio COM SINAL ao eixo do divisor: >0 = à direita.
        let sdist = |p: Vec2| -> f32 {
            let mut best = f32::MAX;
            let mut sign = 1.0f32;
            for pts in &dividers {
                for w in pts.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    let ab = Vec2::new(b.x - a.x, b.y - a.y);
                    let l2 = ab.x * ab.x + ab.y * ab.y;
                    let t = if l2 <= 0.0 {
                        0.0
                    } else {
                        (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
                    };
                    let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < best {
                        best = d;
                        sign = (ab.x * dy - ab.y * dx).signum();
                    }
                }
            }
            best * sign
        };
        println!("\n== {name} (precision {precision}) ==");
        for r in &regions {
            for (ri, ring) in std::iter::once(&r.fill.outer)
                .chain(r.fill.holes.iter())
                .enumerate()
            {
                // Vértices do anel perto do divisor, com desvio assinado em px.
                let n = ring.len();
                let mut runs = 0usize;
                let mut prev_sign = 0.0f32;
                let mut zipper: Vec<(f32, f32, f32)> = Vec::new(); // (y, x, dev_px)
                for &p in ring.iter() {
                    if !(0.7..1.3).contains(&p.x) || p.y.abs() > 2.4 {
                        prev_sign = 0.0;
                        continue;
                    }
                    let dev = sdist(p) * precision;
                    if dev.abs() > 1.0 && prev_sign != 0.0 && dev.signum() != prev_sign {
                        runs += 1;
                        zipper.push((p.y, p.x, dev));
                    }
                    if dev.abs() > 1.0 {
                        prev_sign = dev.signum();
                    }
                }
                if runs > 0 {
                    println!(
                        "  label {} anel {} ({} verts): {} TROCAS de lado > 1px",
                        r.label,
                        if ri == 0 { "outer" } else { "hole" },
                        n,
                        runs
                    );
                    for (y, x, d) in zipper.iter().take(12) {
                        println!("    ({y:+.3}, {x:+.3}) dev {d:+.1}px");
                    }
                }
            }
        }
    }
}

/// 🔬 **Sonda do SERRILHADO** (handoff 2026-07-20 §3.1 — 5º smoke: onde a fronteira
/// azul/vermelho corre AO LONGO do divisor ondulado, o lado azul mostra dentes de serra
/// regulares; o vermelho é liso). A sonda responde UMA pergunta: **a serra nasce no plano
/// `assign` (métrica/pedágio do Voronoi) ou só na geometria (traçado/RDP)?**
///
/// Imprime, por linha da grade e por lado, o desvio da fronteira contra o EIXO da tinta —
/// no `assign` e na geometria final — com o divisor RETO como controle (uma serra que só
/// existe no ondulado é função da onda; uma que existe nos dois é da métrica).
#[test]
#[ignore = "sonda de diagnóstico — rode com --release --ignored --nocapture"]
fn probe_the_sawtooth_boundary_along_the_divider() {
    // O tremor do smoke (`flip_colorize_smoke::hand`), determinístico.
    let hand = |pts: &[Vec2], seed: usize| -> Vec<Vec2> {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        pts.iter()
            .enumerate()
            .map(|(i, p)| Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05))
            .collect()
    };
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let art = |wavy: bool| -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
        let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
        for (a, b, s) in [
            (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize),
            (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7),
            (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13),
            (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29),
        ] {
            let pts = if wavy {
                hand(&seg_(a, b, 24), s)
            } else {
                seg_(a, b, 24)
            };
            let n = pts.len();
            strokes.push((pts, vec![0.0; n], false));
        }
        for (a, b, s, n) in [
            (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6), 41usize, 41usize),
            (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5), 53, 53),
        ] {
            let pts = if wavy {
                hand(&seg_(a, b, n), s)
            } else {
                seg_(a, b, n)
            };
            let m = pts.len();
            strokes.push((pts, vec![0.0; m], false));
        }
        strokes
    };
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

    for (name, wavy, precision) in [
        ("RETO   @40", false, 40.0f32),
        ("ONDULADO@40", true, 40.0),
        ("ONDULADO@80", true, 80.0),
    ] {
        let strokes = art(wavy);
        // Replica os passos 1–3 do `colorize` para expor o plano `assign` (sonda, não produto).
        let mut lo = Vec2::splat(f32::INFINITY);
        let mut hi = Vec2::splat(f32::NEG_INFINITY);
        for (pts, _, _) in &strokes {
            for p in pts {
                lo = lo.min(*p);
                hi = hi.max(*p);
            }
        }
        for s in &scribbles {
            for &p in &s.points {
                lo = lo.min(p);
                hi = hi.max(p);
            }
        }
        let mut grid = Grid::new(lo, hi, precision, super::MARGIN_PX, super::MAX_SIDE);
        for (pts, _, closed) in &strokes {
            let n = pts.len();
            let last = if *closed { n } else { n - 1 };
            for i in 0..last {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                grid.stroke_capsule(a, b, 0.0);
                grid.ink_capsule(a, b, 0.0);
            }
        }
        let labels = super::group_scribbles(&grid, &scribbles);
        let (assign, _) = super::solve(&grid, &labels, 0.0, super::DEFAULT_SQUEEZE);

        // Por linha: eixo da tinta do divisor (média das colunas de BOUNDARY na banda
        // x∈[0.5,1.5]) e as fronteiras vermelha (máx x de 0) / azul (mín x de 1) na banda.
        let (w, h) = (grid.w, grid.h);
        let gscale = grid.scale;
        let to_world = move |x: usize| lo.x - super::MARGIN_PX as f32 / gscale + x as f32 / gscale;
        let col_of = |wx: f32| ((wx - lo.x) * gscale) as i64 + super::MARGIN_PX as i64;
        let (band_lo, band_hi) = (
            col_of(0.5).max(0) as usize,
            (col_of(1.5) as usize).min(w - 1),
        );
        let mut rows: Vec<(usize, f32, f32, f32)> = Vec::new(); // (y, ink, red, blue) em mundo
        for y in 0..h {
            let wy = lo.y - super::MARGIN_PX as f32 / grid.scale + y as f32 / grid.scale;
            // Longe do vão (|y|<0.75) e das bordas da caixa.
            if !((-2.3..-0.75).contains(&wy) || (0.75..2.3).contains(&wy)) {
                continue;
            }
            let (mut ink_sum, mut ink_n) = (0.0f32, 0usize);
            let (mut red, mut blue) = (f32::MIN, f32::MAX);
            for x in band_lo..=band_hi {
                let i = y * w + x;
                if grid.flags[i] & BOUNDARY != 0 {
                    ink_sum += to_world(x);
                    ink_n += 1;
                }
                match assign[i] {
                    Some(0) => red = red.max(to_world(x)),
                    Some(1) => blue = blue.min(to_world(x)),
                    _ => {}
                }
            }
            if ink_n > 0 && red > f32::MIN && blue < f32::MAX {
                rows.push((y, ink_sum / ink_n as f32, red, blue));
            }
        }

        // A geometria final, pela porta pública.
        let regions = colorize(&strokes, &scribbles, precision, 0.0);
        let edge_of = |label: u16, blue_side: bool| -> Vec<(f32, f32)> {
            let mut pts: Vec<(f32, f32)> = regions
                .iter()
                .filter(|r| r.label == label)
                .flat_map(|r| r.fill.outer.iter())
                .filter(|p| (0.5..1.5).contains(&p.x))
                .filter(|p| (-2.3..-0.75).contains(&p.y) || (0.75..2.3).contains(&p.y))
                .map(|p| (p.y, p.x))
                .collect();
            pts.sort_by(|a, b| a.0.total_cmp(&b.0));
            let _ = blue_side;
            pts
        };
        let red_geo = edge_of(0, false);
        let blue_geo = edge_of(1, true);

        // Resumo: desvio (fronteira − eixo) por linha → rugosidade = média |Δ desvio| entre
        // linhas consecutivas + pico-a-pico, em PX DA GRADE (o que se vê).
        let stats = move |dev: &[f32]| -> (f32, f32) {
            if dev.len() < 2 {
                return (0.0, 0.0);
            }
            let px = gscale;
            let rough =
                dev.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / (dev.len() - 1) as f32;
            let (mn, mx) = dev
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), &d| (l.min(d), h.max(d)));
            (rough * px, (mx - mn) * px)
        };
        let red_dev: Vec<f32> = rows.iter().map(|(_, ink, red, _)| red - ink).collect();
        let blue_dev: Vec<f32> = rows.iter().map(|(_, ink, _, blue)| blue - ink).collect();
        let (rr, rp) = stats(&red_dev);
        let (br, bp) = stats(&blue_dev);
        // Rugosidade das bordas da GEOMETRIA: desvio de x entre vértices consecutivos em y.
        let geo_stats = |pts: &[(f32, f32)]| -> (f32, f32, usize) {
            if pts.len() < 2 {
                return (0.0, 0.0, pts.len());
            }
            let px = gscale;
            let xs: Vec<f32> = pts.iter().map(|p| p.1).collect();
            let rough =
                xs.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / (xs.len() - 1) as f32;
            let (mn, mx) = xs
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), &d| (l.min(d), h.max(d)));
            (rough * px, (mx - mn) * px, pts.len())
        };
        let (grr, grp, grn) = geo_stats(&red_geo);
        let (gbr, gbp, gbn) = geo_stats(&blue_geo);
        println!(
            "\n== {name} · grade {}x{} ==\n\
             assign  VERMELHO: rugosidade {rr:.2} px · pico-a-pico {rp:.1} px\n\
             assign  AZUL    : rugosidade {br:.2} px · pico-a-pico {bp:.1} px\n\
             geom    VERMELHO: rugosidade {grr:.2} px · pico-a-pico {grp:.1} px · {grn} vertices na banda\n\
             geom    AZUL    : rugosidade {gbr:.2} px · pico-a-pico {gbp:.1} px · {gbn} vertices na banda",
            grid.w, grid.h
        );
        // As primeiras 30 linhas, cruas, para VER a forma da serra.
        for (y, ink, red, blue) in rows.iter().take(30) {
            println!(
                "  y={y:>4}  eixo {ink:+.3}  verm {:+.3} ({:+.1}px)  azul {:+.3} ({:+.1}px)",
                red,
                (red - ink) * grid.scale,
                blue,
                (blue - ink) * grid.scale
            );
        }
        // Os vértices CRUS da borda azul na metade de cima, LONGE do vão (y ∈ [1.0, 2.3]) —
        // se a serra é do traçado, ela aparece aqui como x oscilando.
        let far: Vec<(f32, f32)> = blue_geo
            .iter()
            .copied()
            .filter(|p| (1.0..2.3).contains(&p.0))
            .collect();
        println!("  vertices AZUIS longe do vao (y, x):");
        for (y, x) in &far {
            println!("    ({y:+.3}, {x:+.3})");
        }
        let far_red: Vec<(f32, f32)> = red_geo
            .iter()
            .copied()
            .filter(|p| (1.0..2.3).contains(&p.0))
            .collect();
        println!("  vertices VERMELHOS longe do vao (y, x): {far_red:?}");

        // O plano de PIXELS que o traçador VÊ: replica o trace_region (FILLED + expand)
        // e mede a fronteira por linha — a serra nasce aqui ou no smooth+RDP?
        use ph2d_flip_fill::FILLED;
        for (label, side_name) in [(0usize, "VERMELHO"), (1usize, "AZUL")] {
            for f in &mut grid.flags {
                *f &= !FILLED;
            }
            for (i, a) in assign.iter().enumerate() {
                if *a == Some(label) && grid.flags[i] & BOUNDARY == 0 {
                    grid.flags[i] |= FILLED;
                }
            }
            grid.expand_under_ink(super::AXIS_COVER_PASSES);
            let mut devs: Vec<f32> = Vec::new();
            for &(y, ink, ..) in &rows {
                let mut edge = if label == 0 { f32::MIN } else { f32::MAX };
                for x in band_lo..=band_hi {
                    if grid.flags[y * w + x] & FILLED != 0 {
                        let wx = to_world(x);
                        edge = if label == 0 {
                            edge.max(wx)
                        } else {
                            edge.min(wx)
                        };
                    }
                }
                if edge.abs() != f32::MAX {
                    devs.push(edge - ink);
                }
            }
            let (rough, p2p) = stats(&devs);
            println!(
                "  PIXEL pos-expand {side_name}: rugosidade {rough:.2} px · pico-a-pico {p2p:.1} px · {} linhas",
                devs.len()
            );
        }
    }
}
