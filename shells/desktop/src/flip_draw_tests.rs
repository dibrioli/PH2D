//! Gates da autoria do traço: a **porta única** entre o preview ao vivo e o traço assado,
//! e a simplificação que ela tornou segura.

use super::stroke_from_samples;
use ph2d_core::Vec2;
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;

/// Um arco de mão: 400 amostras (o que o `MIN_SAMPLE_PX` de 2 px produz), com tremor.
fn hand_arc(n: usize) -> (Vec<Vec2>, Vec<f32>) {
    let pts = (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * 2.2;
            let h = ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
            Vec2::new(a.cos() + h * 0.004, a.sin() + h * 0.004)
        })
        .collect();
    (pts, vec![1.0; n])
}

fn style() -> FlipStyleSnapshot {
    FlipStyleSnapshot {
        smoothing: 0.0,
        ..Default::default()
    }
}

/// Distância do ponto `p` à polilinha `poly` (mín. sobre os segmentos).
fn dist_to_polyline(p: Vec2, poly: &[Vec2]) -> f32 {
    let mut best = f32::MAX;
    for w in poly.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let l2 = ab.x * ab.x + ab.y * ab.y;
        let t = if l2 <= 0.0 {
            0.0
        } else {
            (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
        };
        let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
        best = best.min((dx * dx + dy * dy).sqrt());
    }
    best
}

/// **O traço ACOMPANHA o desenho, sem pontos redundantes** (T2.8). A reamostragem suave
/// densifica a curva (o antídoto do facetado), mas os pontos ficam EVENLY SPACED sobre a linha
/// desenhada: nem facetas (poucos pontos), nem o *"muitos pontos sobrepostos"* que o RDP curou
/// (Enio 2026-07-18). O RDP interno ainda limpa o tremor (control points); a Catmull-Rom os liga
/// densamente. **O desvio fica DENTRO da silhueta** — o traço reamostrado não sai do que a mão
/// desenhou. Mutação que sangra: a Catmull-Rom estourar (overshoot) além da silhueta.
#[test]
fn the_resampled_stroke_tracks_the_drawing_without_redundant_points() {
    let (pts, prs) = hand_arc(400);
    let s = stroke_from_samples(&style(), &pts, &prs, &Xform::IDENTITY);
    let out = s.positions();
    let width = ph2d_tool_flip::size_to_world(style().width_px);

    // (1) ACOMPANHA: cada ponto DESENHADO está dentro da silhueta do traço reamostrado.
    let mut worst = 0.0f32;
    for p in &pts {
        worst = worst.max(dist_to_polyline(*p, out));
    }
    assert!(
        worst < width * 0.5,
        "o traço saiu da SILHUETA do desenho (desvio {worst} > meia espessura {})",
        width * 0.5
    );

    // (2) SEM pontos redundantes/sobrepostos (o report antigo do Enio): nenhum par consecutivo
    // fica grudado. O passo da reamostragem é ~0.4×espessura; nada colapsa a uma fração disso.
    let min_gap = out
        .windows(2)
        .map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt())
        .filter(|&d| d > 1e-7)
        .fold(f32::MAX, f32::min);
    assert!(
        min_gap > width * 0.05,
        "pontos quase-sobrepostos (gap mínimo {min_gap}, espessura {width})"
    );
}

/// O maior GIRO (graus) entre segmentos consecutivos de `poly` — o oráculo de "facetado vs liso".
fn max_turn_deg(poly: &[Vec2]) -> f32 {
    let mut worst = 0.0f32;
    for w in poly.windows(3) {
        let a = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
        let b = Vec2::new(w[2].x - w[1].x, w[2].y - w[1].y);
        let (la, lb) = (
            (a.x * a.x + a.y * a.y).sqrt(),
            (b.x * b.x + b.y * b.y).sqrt(),
        );
        if la < 1e-7 || lb < 1e-7 {
            continue;
        }
        let cos = ((a.x * b.x + a.y * b.y) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.max(cos.acos().to_degrees());
    }
    worst
}

/// 🔴 **Um traço ESPARSO vira uma curva LISA — o report do Enio (2026-07-25):** *"o traço tem
/// baixo número de vértices e fica tracejado e não arredondado nas curvas."* Um "C" de POUCOS
/// pontos (o render liga por retas ⇒ facetado) passa pela porta REAL (`stroke_from_samples`) e sai
/// **densificado** e com giros PEQUENOS entre segmentos (arredondado). É o gate end-to-end de que a
/// reamostragem está PLUGADA no caminho de verdade. Mutação que sangra: tirar o `resample_smooth`
/// do `stroke_from_samples` — o traço fica com ~7 pontos e giros de ~45° (facetado), e o gate falha.
#[test]
fn a_sparse_stroke_becomes_a_smooth_curve() {
    // Um "C" esparso (6 pontos, passos de 45° num arco de 225°, raio 0.7) — como o traço do Enio:
    // giros de ~45° entre segmentos = facetado, mas abaixo do limiar de quina (60°) ⇒ se suaviza.
    // Smoothing 0 (não é o smoothing que densifica; é a reamostragem).
    let arc: Vec<Vec2> = (0..6)
        .map(|k| {
            let a = (235.0 - 45.0 * k as f32).to_radians();
            Vec2::new(0.7 * a.cos(), 0.7 * a.sin())
        })
        .collect();
    let coarse_turn = max_turn_deg(&arc);
    assert!(coarse_turn > 30.0, "o C cru É facetado: {coarse_turn}°");

    let s = stroke_from_samples(&style(), &arc, &vec![1.0; arc.len()], &Xform::IDENTITY);
    let out = s.positions();
    assert!(
        out.len() > 3 * arc.len(),
        "densificou o traço esparso: {} pontos (de {})",
        out.len(),
        arc.len()
    );
    let smooth_turn = max_turn_deg(out);
    assert!(
        smooth_turn < 15.0,
        "a curva reamostrada tem de ser LISA (giro máx {smooth_turn}° < 15; cru era {coarse_turn}°)"
    );
}

/// 🔴 **A dinâmica de pressão afina/engrossa o traço** (T2.6): uma reta desenhada com a pressão
/// subindo de 0 a 1 sai FINA na ponta e GROSSA no fim, com o piso `Min Width` na pressão zero.
/// É o gate end-to-end de que a `pressure_width_factor` está PLUGADA no `build_stroke`. Mutação que
/// sangra: `build_stroke` ignorar a curva (largura constante) — a ponta e o fim ficam iguais.
#[test]
fn pressure_tapers_the_stroke_width() {
    let style = FlipStyleSnapshot {
        smoothing: 0.0,
        pressure_min_width: 0.2, // piso 20% da espessura
        pressure_response: 0.5,  // linear
        ..Default::default()
    };
    // Uma reta com a pressão subindo 0 → 1 ao longo dela (5 amostras).
    let pts: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 0.3, 0.0)).collect();
    let prs: Vec<f32> = (0..5).map(|i| i as f32 / 4.0).collect();
    let s = stroke_from_samples(&style, &pts, &prs, &Xform::IDENTITY);
    let w = s.widths();
    let base = ph2d_tool_flip::size_to_world(style.width_px);

    let first = *w.first().unwrap();
    let last = *w.last().unwrap();
    // Ponta (pressão 0) = o piso; fim (pressão 1) = espessura cheia; fim MUITO maior que ponta.
    assert!(
        (first - base * 0.2).abs() < base * 0.02,
        "a ponta (pressao 0) e o piso de 20%: {first} vs {}",
        base * 0.2
    );
    assert!(
        (last - base).abs() < base * 0.02,
        "o fim (pressao 1) e a espessura cheia: {last} vs {base}"
    );
    assert!(
        last > first * 3.0,
        "o traco AFINA da ponta ao fim: {first} -> {last}"
    );
}

/// **Uma QUINA deliberada sobrevive.** O RDP mantém sempre o ponto de desvio máximo, e
/// uma quina é o desvio máximo do trecho — mas isso é uma propriedade que se afirma, não
/// que se assume, porque é o medo legítimo de quem aumenta a tolerância.
#[test]
fn a_deliberate_corner_survives_the_simplification() {
    let mut pts: Vec<Vec2> = (0..60).map(|i| Vec2::new(i as f32 * 0.01, 0.0)).collect();
    pts.extend((1..60).map(|i| Vec2::new(0.59, i as f32 * 0.01)));
    let prs = vec![1.0; pts.len()];
    let s = stroke_from_samples(&style(), &pts, &prs, &Xform::IDENTITY);
    let corner = Vec2::new(0.59, 0.0);
    let near = s
        .positions()
        .iter()
        .map(|p| ((p.x - corner.x).powi(2) + (p.y - corner.y).powi(2)).sqrt())
        .fold(f32::MAX, f32::min);
    assert!(
        near < 0.02,
        "a quina em (0,59, 0) sumiu — o ponto mais proximo esta a {near}"
    );
}

/// **SONDA — o PASSO que o pipeline REAL produz, em raios.** O rasterizador tem um penhasco
/// medido (`ph2d-flip-render/tests/painter_look.rs::measure_where_the_neighbour_budget_breaks`):
/// abaixo de `0,1875 × raio` o orçamento de vizinhos não cobre o alcance e a tinta some
/// (−184 de 255 em `0,10·r`; **−255** em `0,05·r`). Esta sonda responde a única pergunta que
/// decide se o penhasco é do PRODUTO: *que passo `stroke_from_samples` de fato entrega?*
///
/// Rode: `cargo test -p ph2d-host-desktop --release the_real_pipeline_step -- --nocapture`
#[test]
fn the_real_pipeline_step_in_radii() {
    let width = ph2d_tool_flip::size_to_world(style().width_px);
    let r = width * 0.5;
    println!("\n=== O PASSO REAL DO PIPELINE (penhasco do rasterizador: 0.1875 x raio) ===");
    println!("  figura                        pts   passo_min/r  passo_med/r  n<0.1875r");
    // Cada figura é um gesto de mão plausível, com tremor, na densidade que o
    // `MIN_SAMPLE_PX` do produto produz.
    let casos: Vec<(&str, Vec<Vec2>)> = vec![
        ("arco largo (400 amostras)", hand_arc(400).0),
        ("arco largo (1200, mao LENTA)", hand_arc(1200).0),
        (
            "rabisco APERTADO (raio ~ pincel)",
            (0..900)
                .map(|i| {
                    let a = i as f32 / 900.0 * 18.0;
                    let h = ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
                    Vec2::new(
                        a.cos() * (0.20 + h * 0.004),
                        a.sin() * (0.20 + h * 0.004) + a * 0.01,
                    )
                })
                .collect(),
        ),
        ("estrela de UM traco (5 quinas)", {
            let mut c: Vec<Vec2> = (0..5)
                .map(|k| {
                    let a = -std::f32::consts::FRAC_PI_2
                        + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
                    Vec2::new(0.80 * a.cos(), 0.80 * a.sin())
                })
                .collect();
            c.push(c[0]);
            // Densificado como a mão amostra (2 px de tela ~ passo curto).
            let mut out = vec![c[0]];
            for w in c.windows(2) {
                let (a, b) = (w[0], w[1]);
                let len = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
                let n = (len / 0.01).ceil() as usize;
                for k in 1..=n {
                    let t = k as f32 / n as f32;
                    out.push(Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
                }
            }
            out
        }),
    ];
    for (nome, pts) in casos {
        let prs = vec![1.0_f32; pts.len()];
        let s = stroke_from_samples(&style(), &pts, &prs, &Xform::IDENTITY);
        let out = s.positions();
        let mut passos: Vec<f32> = out
            .windows(2)
            .map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt() / r)
            .collect();
        passos.sort_by(f32::total_cmp);
        let med = passos.iter().sum::<f32>() / passos.len().max(1) as f32;
        let sob = passos.iter().filter(|&&p| p < 0.1875).count();
        println!(
            "  {nome:28}  {:4}   {:9.3}    {:9.3}   {sob:5}",
            out.len(),
            passos.first().copied().unwrap_or(0.0),
            med
        );
    }
}
