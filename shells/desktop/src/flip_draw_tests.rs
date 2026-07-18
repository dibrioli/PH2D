//! Gates da autoria do traço: a **porta única** entre o preview ao vivo e o traço assado,
//! e a simplificação que ela tornou segura.

use super::{STROKE_SIMPLIFY_FRACTION, stroke_from_samples};
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

/// **A simplificação corta vértices redundantes de verdade** — e a tolerância escala com
/// a espessura, então ela não depende do zoom em que se desenhou.
///
/// O sintoma (Enio, 2026-07-18): *"o traço gera muitos pontos muito próximos e até
/// sobrepostos"*. A causa era uma tolerância em px de TELA calibrada para ser inerte.
///
/// Mutação que ele mata: voltar a tolerância para uma constante minúscula — a contagem
/// não cai e o gate sangra.
#[test]
fn the_stroke_drops_redundant_vertices() {
    let (pts, prs) = hand_arc(400);
    let s = stroke_from_samples(&style(), &pts, &prs, &Xform::IDENTITY);
    let kept = s.positions().len();
    assert!(
        kept < 400 / 3,
        "400 amostras de mão deveriam virar bem menos de 133 pontos; vieram {kept}"
    );
    assert!(kept >= 8, "e nao pode colapsar a forma: {kept} pontos");

    // **O desvio fica DENTRO da linha** — é isso que torna o corte invisível, e é por
    // isso que a tolerância é uma fração da espessura e não uma distância.
    let width = ph2d_tool_flip::size_to_world(style().width_px);
    let mut worst = 0.0f32;
    for p in &pts {
        let mut best = f32::MAX;
        for w in s.positions().windows(2) {
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
        worst = worst.max(best);
    }
    assert!(
        worst <= STROKE_SIMPLIFY_FRACTION * width * 1.2,
        "o traço simplificado se afastou {worst} do desenhado — mais que a tolerância"
    );
    assert!(
        worst < width * 0.5,
        "o desvio ({worst}) saiu da SILHUETA da linha (meia espessura = {}), \
         entao ele e visivel",
        width * 0.5
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
