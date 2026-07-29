//! **O PEDÁGIO DE ARQUITETURA** — o kill-criterion do motor novo (doc 12 §3.3, passo 0).
//!
//! Os dois candidatos (C1 buffer de dabs, C4 integral aditiva) precisam de um **alvo
//! intermediário por TRAÇO**, porque a lei de acúmulo é por-traço e entre traços a lei é `over`.
//! O Flip re-rasteriza **tudo a cada frame, em qualquer zoom** — então o pedágio é por frame.
//!
//! ⚠️ **A metade que decide é GEOMÉTRICA, e mede-se sem GPU:** o custo é *quanta área* cada
//! traço obriga o motor a tocar. Três granularidades, e a diferença entre elas é enorme:
//!
//! | granularidade | o que é |
//! |---|---|
//! | **fita** | a área que o traço de fato cobre — o que o motor de HOJE paga |
//! | **bbox** | o retângulo do traço — o que um `set_scissor_rect` ingênuo paga |
//! | **tiles** | só os ladrilhos que a fita toca — o que um binning paga |
//!
//! ⚠️ **A armadilha que este teste existe para expor:** um traço em DIAGONAL tem bbox igual à
//! tela inteira. Se o alvo por-traço for scissorado pela bbox, N traços diagonais = **N telas
//! cheias por frame**. É o modo de falha que mata o desenho, e ele não aparece num traço curto.
//!
//! Roda na CPU. `cargo test -p ph2d-flip-render --release --test architecture_toll -- --ignored --nocapture`

/// Um traço: polilinha em px de tela + raio em px.
struct Stroke {
    pts: Vec<(f32, f32)>,
    r: f32,
}

/// Área da FITA (a união dos discos ao longo do caminho), por amostragem de área exata:
/// conta os pixels a menos de `r` do caminho. É o que o motor de hoje sombreia.
fn ribbon_area(s: &Stroke, w: u32, h: u32) -> u64 {
    let mut n = 0u64;
    for y in 0..h {
        for x in 0..w {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            if dist_to_path(&s.pts, p) <= s.r {
                n += 1;
            }
        }
    }
    n
}

fn dist_to_path(pts: &[(f32, f32)], p: (f32, f32)) -> f32 {
    let mut best = f32::MAX;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (vx, vy) = (b.0 - a.0, b.1 - a.1);
        let (wx, wy) = (p.0 - a.0, p.1 - a.1);
        let len2 = vx * vx + vy * vy;
        let t = if len2 <= 1e-9 {
            0.0
        } else {
            ((wx * vx + wy * vy) / len2).clamp(0.0, 1.0)
        };
        let (cx, cy) = (a.0 + vx * t, a.1 + vy * t);
        let d = ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt();
        best = best.min(d);
    }
    best
}

/// Área da BBOX do traço, clipada à tela.
fn bbox_area(s: &Stroke, w: u32, h: u32) -> u64 {
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in &s.pts {
        x0 = x0.min(x - s.r);
        y0 = y0.min(y - s.r);
        x1 = x1.max(x + s.r);
        y1 = y1.max(y + s.r);
    }
    let x0 = x0.floor().max(0.0) as i64;
    let y0 = y0.floor().max(0.0) as i64;
    let x1 = (x1.ceil() as i64).min(i64::from(w));
    let y1 = (y1.ceil() as i64).min(i64::from(h));
    ((x1 - x0).max(0) as u64) * ((y1 - y0).max(0) as u64)
}

/// Área dos TILES que a fita de fato toca (granularidade de binning).
fn tile_area(s: &Stroke, w: u32, h: u32, tile: u32) -> u64 {
    let tx = w.div_ceil(tile);
    let ty = h.div_ceil(tile);
    let mut touched = 0u64;
    for ty_i in 0..ty {
        for tx_i in 0..tx {
            // o tile é tocado se algum ponto dele está a <= r do caminho;
            // conservador e barato: distância do CENTRO do tile ao caminho <= r + meia-diagonal
            let cx = (tx_i * tile) as f32 + tile as f32 * 0.5;
            let cy = (ty_i * tile) as f32 + tile as f32 * 0.5;
            let half_diag = (tile as f32) * std::f32::consts::SQRT_2 * 0.5;
            if dist_to_path(&s.pts, (cx, cy)) <= s.r + half_diag {
                touched += u64::from(tile) * u64::from(tile);
            }
        }
    }
    touched
}

// ————————————————————————————————— as cenas —————————————————————————————————

/// Traços curtos espalhados — hachura, detalhe, textura. O caso amigável.
fn short_strokes(n: usize, w: u32, h: u32, r: f32) -> Vec<Stroke> {
    let mut out = Vec::new();
    let mut seed = 0x9E3779B9u32;
    let mut rnd = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed >> 8) as f32 / (1 << 24) as f32
    };
    for _ in 0..n {
        let x = rnd() * w as f32;
        let y = rnd() * h as f32;
        let ang = rnd() * std::f32::consts::TAU;
        let len = 60.0 + rnd() * 120.0;
        out.push(Stroke {
            pts: vec![(x, y), (x + ang.cos() * len, y + ang.sin() * len)],
            r,
        });
    }
    out
}

/// Traços LONGOS atravessando a tela — o gesto de animação (um arco de braço, um contorno).
/// É aqui que a bbox explode.
fn long_strokes(n: usize, w: u32, h: u32, r: f32) -> Vec<Stroke> {
    let mut out = Vec::new();
    let mut seed = 0x85EBCA6Bu32;
    let mut rnd = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed >> 8) as f32 / (1 << 24) as f32
    };
    for _ in 0..n {
        // uma curva ampla de canto a canto, com 40 pontos
        let (x0, y0) = (rnd() * w as f32 * 0.2, rnd() * h as f32);
        let (x1, y1) = (w as f32 * 0.8 + rnd() * w as f32 * 0.2, rnd() * h as f32);
        let bow = (rnd() - 0.5) * h as f32 * 0.6;
        let pts = (0..40)
            .map(|k| {
                let t = k as f32 / 39.0;
                let x = x0 + (x1 - x0) * t;
                let y = y0 + (y1 - y0) * t + bow * (std::f32::consts::PI * t).sin();
                (x, y)
            })
            .collect();
        out.push(Stroke { pts, r });
    }
    out
}

fn report(name: &str, strokes: &[Stroke], w: u32, h: u32) {
    let screen = u64::from(w) * u64::from(h);
    let (mut fita, mut bbox, mut t16, mut t64) = (0u64, 0u64, 0u64, 0u64);
    for s in strokes {
        fita += ribbon_area(s, w, h);
        bbox += bbox_area(s, w, h);
        t16 += tile_area(s, w, h, 16);
        t64 += tile_area(s, w, h, 64);
    }
    let f = |v: u64| v as f64 / screen as f64;
    println!(
        "  {name:<28} {:>3} traços | fita {:>7.2}  bbox {:>8.2}  tile64 {:>7.2}  tile16 {:>7.2}",
        strokes.len(),
        f(fita),
        f(bbox),
        f(t64),
        f(t16)
    );
}

/// **A MEDIÇÃO QUE DECIDE.** Em "telas cheias" por frame: quanto o alvo por-traço obriga a
/// tocar, em cada granularidade. Um número aqui é um multiplicador de fill rate por frame.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_per_stroke_target_toll_in_screenfuls() {
    // 1080p — a tela onde o custo é medido (a razão não depende da resolução; a granularidade
    // de tile sim, e é por isso que os dois tamanhos de tile estão aqui).
    let (w, h) = (1920u32, 1080);
    println!("\n=== O PEDÁGIO EM TELAS CHEIAS POR FRAME (alvo por-traço) ===");
    println!("    tela {w}x{h} · raio 6 px · 'fita' = o que o motor de HOJE sombreia\n");
    for n in [10usize, 50, 200] {
        report(
            &format!("curtos (hachura) n={n}"),
            &short_strokes(n, w, h, 6.0),
            w,
            h,
        );
    }
    println!();
    for n in [10usize, 50, 200] {
        report(
            &format!("LONGOS (gesto) n={n}"),
            &long_strokes(n, w, h, 6.0),
            w,
            h,
        );
    }
    println!("\n  ⚠️ leia a coluna bbox nos LONGOS: é o modo de falha do scissor ingênuo.");
}

/// O mesmo, por TRAÇO, para ver a razão bbox/fita de um gesto isolado — o número que explica
/// o agregado acima.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_bbox_waste_of_a_single_gesture() {
    let (w, h) = (1920u32, 1080);
    println!("\n=== O DESPERDÍCIO DA BBOX NUM GESTO SÓ ===\n");
    println!("  caso                          fita px     bbox px    bbox/fita   tile64/fita");
    let cases: Vec<(&str, Stroke)> = vec![
        (
            "horizontal curto",
            Stroke {
                pts: vec![(100.0, 500.0), (300.0, 500.0)],
                r: 6.0,
            },
        ),
        (
            "DIAGONAL de canto a canto",
            Stroke {
                pts: vec![(20.0, 20.0), (1900.0, 1060.0)],
                r: 6.0,
            },
        ),
        (
            "arco amplo (gesto)",
            Stroke {
                pts: (0..40)
                    .map(|k| {
                        let t = k as f32 / 39.0;
                        (
                            100.0 + 1700.0 * t,
                            540.0 + 400.0 * (std::f32::consts::PI * t).sin(),
                        )
                    })
                    .collect(),
                r: 6.0,
            },
        ),
    ];
    for (name, s) in &cases {
        let fita = ribbon_area(s, w, h);
        let bbox = bbox_area(s, w, h);
        let t64 = tile_area(s, w, h, 64);
        println!(
            "  {name:<28} {fita:>8}  {bbox:>10}   {:>8.1}x   {:>8.1}x",
            bbox as f64 / fita as f64,
            t64 as f64 / fita as f64
        );
    }
}
