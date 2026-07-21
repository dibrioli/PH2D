//! Gates da **BORDA sobre a tinta** — o serrilhado (5º smoke) e a fresta na parede que duas
//! faces da mesma cor deixavam. Irmãos do `lib_tests.rs` pelo teto de LOC (700).

use super::{Scribble, colorize};
use ph2d_core::Vec2;

/// 🔴 **A borda da cor SEGUE a linha ondulada — sem dentes de serra** (5º smoke, 2026-07-20,
/// handoff §3.1: onde a fronteira azul/vermelho corre AO LONGO do divisor ondulado, o lado
/// azul mostrava dentes regulares de ~5-10 px e o vermelho saía liso).
///
/// A sonda (`probe_the_sawtooth_boundary_along_the_divider`) inocentou o Voronoi e o
/// `expand_under_ink` — o plano de pixels é liso e simétrico (rugosidade 0,2 px, os dois
/// lados idênticos). A serra nasce na VETORIZAÇÃO: a amplitude da onda da linha (±2 px a
/// precisão 80) senta exatamente no ε=1,25 px do RDP, então cada anel cai por sorte num de
/// dois regimes — corda reta que IGNORA a onda (o vermelho: 0 vértices na banda) ou
/// zigue-zague com vértices só nos extremos (o azul: x oscilando 0,989↔1,025) — e o
/// zigue-zague fica visível sob a saia translúcida do pincel macio.
///
/// O oráculo é a APARÊNCIA, independente do mecanismo do fix: a borda de CADA cor,
/// amostrada a ~1 px na janela do divisor (longe do vão), tem de ficar colada no EIXO da
/// linha — a distância é medida por ponto-a-segmento local contra as polilinhas do divisor,
/// nunca pela função que o fix usa (oráculo, não espelho).
///
/// ⚠️ As larguras são as REAIS (meia-largura 0,13, como `boundaries()` entrega) — com
/// largura 0 o `nearest_on_axis` não tem linha para vestir e o fenômeno do produto some.
///
/// Mutação que sangra: neutralizar o snap (devolver o anel intocado) — o vermelho volta a
/// cortar a onda em corda e o azul a serrar.
#[test]
fn the_colour_edge_follows_a_wavy_line_without_sawtooth() {
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
    let precision = 80.0f32;
    let regions = colorize(&strokes, &scribbles, precision, 0.0);

    // O eixo do divisor: as duas polilinhas trêmulas (índices 4 e 5 acima).
    let dividers: Vec<&[Vec2]> = strokes[4..6].iter().map(|(p, ..)| p.as_slice()).collect();
    let dist_to_axis = |p: Vec2| -> f32 {
        let mut best = f32::MAX;
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
                best = best.min((dx * dx + dy * dy).sqrt());
            }
        }
        best
    };
    // A janela do divisor, LONGE do vão e das paredes: x∈[0.8,1.2], 1.1≤|y|≤2.3.
    // ⚠️ O corte em 1.1 (não 0.75) exclui a REATACAGEM da lente: na boca do vão a
    // fronteira volta do bojo para a linha por caminho geodésico honesto (desvios de
    // ~14 px que são a LENTE, §3.2 — outro fenômeno, outra decisão), e este gate mede o
    // SERRILHADO de quem já está colado na linha.
    let in_window = |p: Vec2| (0.8..1.2).contains(&p.x) && (1.1..2.3).contains(&p.y.abs());
    let step = 1.0 / precision; // ~1 px
    for label in [0u16, 1u16] {
        let mut devs: Vec<f32> = Vec::new();
        let mut worst = 0.0f32;
        let mut worst_at = Vec2::new(0.0, 0.0);
        let mut samples = 0usize;
        // A costura pode viver no OUTER ou num HOLE (nesta arte o vermelho engole a margem
        // externa — §3.3 — e o divisor vira parede de um furo dele).
        for ring in regions
            .iter()
            .filter(|r| r.label == label)
            .flat_map(|r| std::iter::once(&r.fill.outer).chain(r.fill.holes.iter()))
        {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                let d = Vec2::new(b.x - a.x, b.y - a.y);
                let len = (d.x * d.x + d.y * d.y).sqrt();
                let steps = (len / step).ceil().max(1.0) as usize;
                for s in 0..steps {
                    let t = s as f32 / steps as f32;
                    let p = Vec2::new(a.x + d.x * t, a.y + d.y * t);
                    if in_window(p) {
                        let d = dist_to_axis(p);
                        if d > worst {
                            worst = d;
                            worst_at = p;
                        }
                        devs.push(d);
                        samples += 1;
                    }
                }
            }
        }
        assert!(
            samples > 50,
            "controle positivo: a borda do rotulo {label} tem de atravessar a janela \
             (só {samples} amostras — a fixture não contém o fenômeno)"
        );
        let _ = worst_at;
        let (mn, mx) = devs
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &d| (l.min(d), h.max(d)));
        let (mn_px, mx_px) = (mn * precision, mx * precision);
        if std::env::var("PH2D_GATE_DUMP").is_ok() {
            eprintln!("[dump] label {label}: dev [{mn_px:.2}, {mx_px:.2}] px");
        }
        // (a) SEGUE a linha: o desvio ao eixo é ~constante (spread pequeno). Uma corda que
        //     ignora a onda ou um zigue-zague de extremos (5º smoke) espalham 2-4 px.
        assert!(
            mx_px - mn_px <= 1.5,
            "a borda do rotulo {label} tem de SEGUIR a linha ondulada: desvio ao eixo \
             espalhado em [{mn_px:.2}, {mx_px:.2}] px (o serrilhado do 5º smoke)"
        );
        // (b) SOBREPÕE: a borda cruza ATÉ A FACE OPOSTA (~2 px além do eixo) — bordas que
        //     apenas ENCOSTAM no eixo rasterizam com costura aberta (o zíper do 6º smoke:
        //     dois polígonos adjacentes quantizam a aresta cada um por si).
        assert!(
            mn_px >= 0.8 && mx_px <= 3.5,
            "a borda do rotulo {label} tem de SOBREPOR ~2 px além do eixo, nunca só \
             encostar (dev [{mn_px:.2}, {mx_px:.2}] px — o zíper do 6º smoke)"
        );
    }
}

/// 🔴 **Uma cor que inunda os DOIS lados de uma linha cobre a linha por baixo — sem
/// fenda** (6º smoke, foto 2: o divisor virava uma corrente escura em zigue-zague quando o
/// vermelho vazava pelo vão e abraçava o traço).
///
/// O `expand_under_ink` só entra em pixels `INK` (o filamento do eixo); o aro de AA da
/// PAREDE ficava sem preencher, e a região que abraça a linha ganhava uma FENDA suja de
/// ~1 px que o traçador percorria em dupla-visita — qualquer destino dela é artefato de
/// winding na tela. Com `close_wall_slits`, parede com a MESMA cor dos dois lados é
/// INTERIOR: o anel da região nem passa perto do divisor.
///
/// Mutação que sangra: apagar o `close_wall_slits` do `trace_region` (o anel volta a
/// percorrer a fenda — dezenas de vértices na banda).
#[test]
fn one_colour_flooding_both_sides_covers_the_wall_without_a_slit() {
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
    // UM rabisco só: o vermelho vaza pelo vão e abraça o divisor pelos dois lados.
    let scribbles = vec![Scribble {
        label: 0,
        points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
        width: 0.15,
    }];
    let regions = colorize(&strokes, &scribbles, 80.0, 0.0);
    assert!(!regions.is_empty(), "a cor tem de existir");
    // Nenhum vértice de anel algum na banda do divisor, LONGE do vão (o traço é interior).
    let near_divider = |p: Vec2| (0.9..1.1).contains(&p.x) && (0.8..2.3).contains(&p.y.abs());
    let offenders: usize = regions
        .iter()
        .flat_map(|r| std::iter::once(&r.fill.outer).chain(r.fill.holes.iter()))
        .flat_map(|ring| ring.iter())
        .filter(|p| near_divider(**p))
        .count();
    assert_eq!(
        offenders, 0,
        "a linha abraçada pela mesma cor é INTERIOR — o anel não pode percorrer uma fenda \
         sobre o divisor ({offenders} vertices na banda)"
    );
}
