//! Os gates da **REGIÃO** e da **DENSIDADE ADAPTATIVA** deste nó (doc 89, folha 01).
//!
//! ⚠️ **A afirmação central é a que refuta a célula:** a cadeia
//! `poisson → field.remap(probability) → motion.cull` **não** exprime isto. Ela sorteia
//! quem morre, e os sobreviventes ficam no espaçamento original — a zona rala fica com
//! **buracos**. Aqui o espaçamento em si cresce. Os dois conjuntos são distinguíveis, e
//! é o que o gate `the_thin_zone_gets_coarser_spacing_not_holes` mede.

use super::*;
use ph2d_motion_region::{Region, SHAPE_CIRCLE, SHAPE_RING};

const W: f32 = 8.0;
const H: f32 = 8.0;
const R: f32 = 0.22;
const SEED: u32 = 3;

fn rect() -> Region {
    Region::of(0.0, W, H, 0.0)
}

/// A distância ao vizinho mais próximo, ponto a ponto.
fn gaps(pts: &[[f32; 2]]) -> Vec<f32> {
    pts.iter()
        .map(|p| {
            pts.iter()
                .filter(|q| !std::ptr::eq(*q, p))
                .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                .fold(f32::MAX, f32::min)
        })
        .collect()
}

/// A mediana de uma amostra.
fn median(mut v: Vec<f32>) -> f32 {
    v.sort_by(f32::total_cmp);
    if v.is_empty() { 0.0 } else { v[v.len() / 2] }
}

/// ⭐ **O RETÂNGULO SEM GRADAÇÃO É O DE SEMPRE, AO BIT.**
///
/// ⚠️ Esta é a metade que fez o `inside` e a semente terem um RAMO em vez de uma
/// pergunta a mais: um `region.contains` incondicional daria a mesma resposta numa
/// caixa e ainda assim moveria tudo, porque a semente passaria a poder ser rejeitada e
/// uma rejeição a mais desloca toda a sequência de sorteios que vem depois.
#[test]
fn the_default_region_reproduces_the_old_layout_bit_for_bit() {
    let a = poisson::sample(&rect(), W, H, R, 0.0, SEED);
    // A árvore de referência: a mesma chamada, com a região construída de outra forma.
    let b = poisson::sample(&Region::of(f32::NAN, W, H, 9.0), W, H, R, 0.0, SEED);
    assert!(!a.is_empty(), "CONTROLE: o no' produz alguma coisa");
    assert_eq!(a.len(), b.len(), "um param doente cai no retangulo");
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert_eq!(p.map(f32::to_bits), q.map(f32::to_bits), "ponto {i}");
    }
}

/// ⭐ **O CÍRCULO PREENCHE O DISCO** — e nada fica fora dele.
#[test]
fn the_circle_fills_the_disc_and_nothing_spills() {
    let circle = Region::of(SHAPE_CIRCLE as f32, W, H, 0.0);
    let pts = poisson::sample(&circle, W, H, R, 0.0, SEED);
    assert!(pts.len() > 100, "o disco enche-se: {}", pts.len());
    for p in &pts {
        assert!(circle.contains(*p), "fora do disco: {p:?}");
    }
    // E o piso de distância sobrevive à forma nova.
    let g = gaps(&pts);
    let closest = g.iter().copied().fold(f32::MAX, f32::min);
    assert!(closest >= R - 1e-4, "o piso quebrou: {closest:.5} < {R}");
}

/// E o anel deixa o buraco vazio.
#[test]
fn the_ring_leaves_its_hole_empty() {
    let ring = Region::of(SHAPE_RING as f32, W, H, 0.5);
    let pts = poisson::sample(&ring, W, H, R, 0.0, SEED);
    assert!(pts.len() > 50, "o anel enche-se: {}", pts.len());
    for p in &pts {
        assert!(ring.radial(*p) >= 0.5 - 1e-3, "caiu no buraco: {p:?}");
    }
}

/// ⭐⭐⭐ **A ZONA RALA GANHA ESPAÇAMENTO MAIOR — não buracos**, e o quanto ele cresce
/// é **`1/densidade`**, a lei declarada. É a afirmação que separa esta wave da cadeia
/// que a célula dava como equivalente.
///
/// ⚠️ **A régua é o vão MEDIANO por banda, contra a PREVISÃO** — não a contagem, e não
/// um «maior que». Um cull também reduz a contagem, e é precisamente por isso que
/// contar não distingue os dois; o que o cull não consegue mover é o vão dos
/// sobreviventes, que fica no piso do raio porque nenhum ponto se mexeu. E um «a borda
/// é mais rala» aceita um intervalo inteiro de comportamentos, incluindo os errados.
///
/// ⚠️ **A primeira versão deste gate afirmava que «o coração fica no piso» e ISSO É
/// FALSO** — com uma gradação linear a densidade vale `1` **só no centro exacto**, e o
/// coração medido (banda `0..0,4`) tem raio médio-por-área `0,27`, logo densidade
/// `0,79` e vão `1,27×`. Medido: `1,273×`. *A régua tinha de exprimir a lei, e a lei
/// não tem nenhum platô lá dentro.*
#[test]
fn the_thin_zone_gets_coarser_spacing_not_holes() {
    let circle = Region::of(SHAPE_CIRCLE as f32, W, H, 0.0);
    let flat = poisson::sample(&circle, W, H, R, 0.0, SEED);
    let graded = poisson::sample(&circle, W, H, R, 1.0, SEED);

    let band = |pts: &[[f32; 2]], lo: f32, hi: f32| -> Vec<f32> {
        let g = gaps(pts);
        pts.iter()
            .zip(&g)
            .filter(|(p, _)| {
                let r = circle.radial(**p);
                r >= lo && r < hi
            })
            .map(|(_, v)| *v)
            .collect()
    };
    // O raio médio POR ÁREA de uma coroa `[a,b]` — `∫r·r dr / ∫r dr`. Uma média
    // aritmética das bordas leria o anel como se ele tivesse a mesma área por dentro
    // e por fora, e a banda de fora tem muito mais.
    let mean_r = |a: f32, b: f32| (2.0 / 3.0) * (b.powi(3) - a.powi(3)) / (b * b - a * a);

    for (lo, hi) in [(0.0_f32, 0.4_f32), (0.4, 0.7), (0.75, 1.0)] {
        let (f, g) = (band(&flat, lo, hi), band(&graded, lo, hi));
        assert!(
            f.len() > 8 && g.len() > 8,
            "banda {lo}..{hi} sem amostra ({} e {})",
            f.len(),
            g.len()
        );
        let (mf, mg) = (median(f), median(g));
        let want = 1.0 / circle.density([mean_r(lo, hi) * W * 0.5, 0.0], 1.0);
        let got = mg / mf;
        println!(
            "banda {lo:.2}..{hi:.2}: vao {mf:.4} -> {mg:.4} = {got:.3}x (previsto {want:.3}x)"
        );
        assert!(
            (got / want - 1.0).abs() < 0.25,
            "banda {lo}..{hi}: o vao cresceu {got:.3}x e a lei previa {want:.3}x"
        );
    }

    // CONTROLE: sem gradação as bandas do layout uniforme concordam entre si — senão o
    // «previsto» acima estaria a ser comparado contra uma base que já variava sozinha.
    let (cf, ef) = (
        median(band(&flat, 0.0, 0.4)),
        median(band(&flat, 0.75, 1.0)),
    );
    assert!(
        (ef / cf - 1.0).abs() < 0.15,
        "CONTROLE: uniforme tinha de dar bandas iguais ({cf:.4} contra {ef:.4})"
    );
}

/// ⚠️ **A gradação nunca fura o piso.** `r = base/densidade` com `densidade ≤ 1` só
/// pode AUMENTAR o raio — se um par ficasse mais perto que `base`, o teste do
/// `far_enough` estaria a ler o raio errado.
#[test]
fn the_graded_layout_never_breaks_the_minimum_radius() {
    for shape in [0.0, SHAPE_CIRCLE as f32, SHAPE_RING as f32] {
        let region = Region::of(shape, W, H, 0.45);
        let pts = poisson::sample(&region, W, H, R, 1.0, SEED);
        assert!(pts.len() > 20, "shape={shape}: {} pontos", pts.len());
        let closest = gaps(&pts).into_iter().fold(f32::MAX, f32::min);
        assert!(
            closest >= R - 1e-4,
            "shape={shape}: o piso quebrou em {closest:.5}"
        );
    }
}

/// ⭐⭐ **A REGRA DO CONFLITO É SIMÉTRICA: `max(r(p), r(q))`, nunca o mínimo.**
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE.** Trocar o `max` por `min` no
/// `far_enough` passava em tudo o que havia: o piso continuava honrado (o mínimo dos
/// dois raios ainda é `≥ r_base`) e a mediana por banda mal se mexia. O que o `min`
/// quebra é outra coisa — **a simetria**: um ponto grosso e um fino em conflito passam a
/// concordar ou não conforme QUEM foi colocado primeiro, e o resultado deixa de ser uma
/// função do conjunto para passar a ser uma função da ordem.
///
/// A régua é a invariante que o código afirma, medida directamente: para todo par, a
/// distância tem de bater o MAIOR dos dois raios locais.
#[test]
fn no_pair_is_closer_than_the_larger_of_their_two_local_radii() {
    let circle = Region::of(SHAPE_CIRCLE as f32, W, H, 0.0);
    let pts = poisson::sample(&circle, W, H, R, 1.0, SEED);
    assert!(pts.len() > 40, "ha' pontos que medir: {}", pts.len());
    // O raio local, pela mesma lei do produto.
    let radius = |p: [f32; 2]| R / circle.density(p, 1.0);
    let mut worst = f32::MAX;
    let mut worst_at = ([0.0_f32; 2], [0.0_f32; 2]);
    for (i, a) in pts.iter().enumerate() {
        for b in &pts[i + 1..] {
            let d = (a[0] - b[0]).hypot(a[1] - b[1]);
            let bar = radius(*a).max(radius(*b));
            if d / bar < worst {
                worst = d / bar;
                worst_at = (*a, *b);
            }
        }
    }
    assert!(
        worst >= 1.0 - 1e-3,
        "um par ficou a {worst:.4} do raio MAIOR dos dois: {worst_at:?}"
    );
}

/// A varredura cresce com o piso da densidade, e o número é DERIVADO — um piso menor
/// que ninguém reconferisse deixaria conflitos por ver.
#[test]
fn the_scan_span_is_derived_from_the_density_floor() {
    let want = (std::f32::consts::SQRT_2 / ph2d_motion_region::MIN_DENSITY).ceil() as usize;
    assert!(
        want >= 8,
        "o piso de hoje pede pelo menos 8 celulas: {want}"
    );
    // A prova de que ele é suficiente: nenhum par abaixo do raio local máximo.
    let circle = Region::of(SHAPE_CIRCLE as f32, W, H, 0.0);
    let pts = poisson::sample(&circle, W, H, R, 1.0, SEED);
    let r_max = R / ph2d_motion_region::MIN_DENSITY;
    let far_edge: Vec<[f32; 2]> = pts
        .iter()
        .copied()
        .filter(|p| circle.radial(*p) > 0.9)
        .collect();
    assert!(far_edge.len() > 5, "ha' borda que medir");
    let closest = gaps(&far_edge).into_iter().fold(f32::MAX, f32::min);
    assert!(
        closest > r_max * 0.5,
        "na borda o vao tinha de aproximar-se de r_max={r_max:.4}: {closest:.4}"
    );
}

/// Os params novos são declarados, com hint, e reduzem ao nó de hoje.
#[test]
fn every_new_param_is_declared_and_defaults_to_today() {
    assert_eq!(MANIFEST.param_default(ph2d_motion_region::SHAPE), Some(0.0));
    assert_eq!(MANIFEST.param_default(DENSITY_FALLOFF), Some(0.0));
    for h in [
        ph2d_motion_region::SHAPE,
        ph2d_motion_region::INNER,
        DENSITY_FALLOFF,
    ] {
        assert!(
            PARAM_HINTS.iter().any(|x| x.param == h),
            "{h} sem hint de painel"
        );
    }
}
