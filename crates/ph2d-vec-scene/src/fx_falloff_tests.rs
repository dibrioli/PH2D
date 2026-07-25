//! Gates do Falloff — o campo escalar e a modulação da força do deformador seguinte.

use super::{Falloff, FalloffShape, FalloffSpec};
use crate::effect::{FxCtx, FxEntry, PathEffect, run_stack};
use crate::fx_warp::BloatSpec;
use crate::fx_warp_presets::{WarpSpec, WarpStyle};
use crate::fx_zigzag::ZigZagSpec;
use crate::{VecPath, VecVertex};

/// Uma caixa de controle de referência: centro na origem, `half = [1, 1]`, `ref_size = 2`.
fn ctx() -> FxCtx {
    FxCtx {
        ref_size: 2.0,
        center: [0.0, 0.0],
        half: [1.0, 1.0],
    }
}

fn radial(f: impl FnOnce(&mut FalloffSpec)) -> Falloff {
    let mut s = FalloffSpec::new(FalloffShape::Radial);
    s.amount = 1.0;
    f(&mut s);
    let mut field = Falloff::default();
    field.push(&s, &ctx());
    field
}

// ── O campo, forma a forma ──────────────────────────────────────────────────

/// O Radial vale `1` no centro (força cheia) e `0` além do raio.
#[test]
fn the_radial_field_is_one_at_the_centre_and_zero_past_the_radius() {
    // `size = 1` ⇒ raio = `ref_size` = 2.
    let f = radial(|_| {});
    assert!(
        (f.eval([0.0, 0.0]) - 1.0).abs() < 1e-9,
        "centro devia ser força cheia"
    );
    assert!(
        f.eval([5.0, 0.0]) < 1e-9,
        "muito além do raio devia ser zero"
    );
}

/// `amount == 0` é NEUTRO (o Add não move nada) e, se ainda assim construído, o campo é `1` em
/// todo lugar — a garantia de que ligar a seção não muda um pixel (ADR-0132, invariante 2).
#[test]
fn amount_zero_is_neutral_and_a_flat_field_of_one() {
    assert!(FalloffSpec::new(FalloffShape::Radial).is_neutral());
    let f = radial(|s| s.amount = 0.0);
    for p in [[0.0, 0.0], [5.0, 0.0], [-3.0, 2.0]] {
        assert!(
            (f.eval(p) - 1.0).abs() < 1e-12,
            "amount 0 tem de ser campo plano de 1 em {p:?}"
        );
    }
}

/// Inverter troca o forte pelo fraco.
#[test]
fn invert_swaps_strong_and_weak() {
    let f = radial(|s| s.invert = true);
    assert!(f.eval([0.0, 0.0]) < 1e-9, "invertido: o centro fica fraco");
    assert!(
        (f.eval([5.0, 0.0]) - 1.0).abs() < 1e-9,
        "invertido: fora do raio fica forte"
    );
}

/// A curva de resposta aperta o meio-termo para baixo (gama > 1).
#[test]
fn a_sharper_curve_lowers_the_midpoint() {
    // `[1, 0]`: distância 1, raio 2 ⇒ `s01 = 0.5`.
    let at = [1.0, 0.0];
    let lin = radial(|s| s.curve = 1.0).eval(at);
    let sharp = radial(|s| s.curve = 2.0).eval(at);
    assert!((lin - 0.5).abs() < 1e-9, "curve 1 no meio = 0.5, deu {lin}");
    assert!(
        sharp < lin - 0.1,
        "curve 2 devia apertar o meio para baixo ({sharp} < {lin})"
    );
}

/// Dois Falloffs antes do mesmo deformador compõem por PRODUTO (interseção das influências).
#[test]
fn two_falloffs_compose_by_product() {
    let mut field = Falloff::default();
    let mut s = FalloffSpec::new(FalloffShape::Radial);
    s.amount = 1.0;
    field.push(&s, &ctx());
    field.push(&s, &ctx());
    // No meio cada layer dá 0.5 ⇒ produto 0.25.
    assert!(
        (field.eval([1.0, 0.0]) - 0.25).abs() < 1e-9,
        "dois radiais deviam compor por produto"
    );
}

/// O Linear é uma rampa ao longo do eixo: forte atrás da linha média, fraco à frente.
#[test]
fn the_linear_field_ramps_along_its_axis() {
    let mut s = FalloffSpec::new(FalloffShape::Linear);
    s.amount = 1.0; // angle 0 (+x), offset 0, softness 1 ⇒ soft = ref_size = 2.
    let mut f = Falloff::default();
    f.push(&s, &ctx());
    assert!(
        (f.eval([-1.0, 0.0]) - 1.0).abs() < 1e-9,
        "meia-suavidade atrás do eixo = forte"
    );
    assert!(f.eval([1.0, 0.0]) < 1e-9, "meia-suavidade à frente = fraco");
    assert!(
        (f.eval([0.0, 0.0]) - 0.5).abs() < 1e-9,
        "na linha média = 0.5"
    );
}

/// O Rect decai pela distância de Chebyshev à caixa.
#[test]
fn the_rect_field_falls_off_to_the_box_edge() {
    let mut s = FalloffSpec::new(FalloffShape::Rect);
    s.amount = 1.0; // size 1 ⇒ meia-caixa = half = [1, 1].
    let mut f = Falloff::default();
    f.push(&s, &ctx());
    assert!((f.eval([0.0, 0.0]) - 1.0).abs() < 1e-9, "centro forte");
    assert!((f.eval([0.5, 0.0]) - 0.5).abs() < 1e-9, "meia caixa = 0.5");
    assert!(f.eval([2.0, 0.0]) < 1e-9, "fora da caixa = 0");
}

/// O Sweep é angular: forte no ângulo inicial, desvanecendo pela varredura.
#[test]
fn the_sweep_field_fades_around_the_circle() {
    let mut s = FalloffSpec::new(FalloffShape::Sweep);
    s.amount = 1.0; // angle 0, spread 0.5 ⇒ varre meia-volta (span = pi).
    let mut f = Falloff::default();
    f.push(&s, &ctx());
    assert!(
        (f.eval([1.0, 0.0]) - 1.0).abs() < 1e-9,
        "no ângulo inicial = 1"
    );
    assert!(
        (f.eval([0.0, 1.0]) - 0.5).abs() < 1e-9,
        "a um quarto de volta = 0.5"
    );
    assert!(f.eval([-1.0, 0.0]) < 1e-9, "a meia-volta (fim do span) = 0");
}

// ── A modulação, pela pilha REAL ────────────────────────────────────────────

fn square() -> VecPath {
    VecPath {
        verts: [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (lo, hi)
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn run(base: &VecPath, stack: &[PathEffect]) -> VecPath {
    let entries: Vec<FxEntry> = stack.iter().cloned().map(FxEntry::new).collect();
    run_stack(base, &entries).expect("a pilha devia produzir geometria")
}

/// **O CORAÇÃO** — um Falloff modula o deformador seguinte SÓ onde é forte.
///
/// Um Radial sobre UM canto + um Pucker & Bloat: o canto sob o campo bloata quase por inteiro
/// (bate o Bloat sozinho), o canto oposto fica quase intacto (bate a forma de entrada). O Bloat
/// preserva a contagem de vértices, então os índices alinham 1-a-1 com a entrada.
///
/// ⚠️ Mutação que este gate mata: tirar o fator `w` do `bloat_contour` (aplicar a deformação
/// cheia em todo vértice) ⇒ o canto longe do campo passa a mover-se como o de perto, e a
/// asserção `far_from_field` reprova.
#[test]
fn a_falloff_modulates_the_next_deformer_where_it_is_strong_and_spares_where_it_is_weak() {
    let base = square();
    let field = FalloffSpec {
        shape: FalloffShape::Radial,
        amount: 1.0,
        size: 0.5, // raio = 1, centrado no canto [+1, +1]
        off_x: 1.0,
        off_y: 1.0,
        ..FalloffSpec::new(FalloffShape::Radial)
    };
    let bloat = PathEffect::Bloat(BloatSpec { amount: 60.0 });

    let modulated = run(&base, &[PathEffect::Falloff(field), bloat.clone()]);
    let full = run(&base, &[bloat]);

    // Índice 2 = canto [1, 1] (sob o campo); índice 0 = canto [-1, -1] (longe).
    let near_matches_full = dist(modulated.verts[2].anchor, full.verts[2].anchor);
    let near_moved = dist(modulated.verts[2].anchor, base.verts[2].anchor);
    let far_moved = dist(modulated.verts[0].anchor, base.verts[0].anchor);
    let far_would_move = dist(full.verts[0].anchor, base.verts[0].anchor);

    assert!(
        near_matches_full < near_moved * 0.05,
        "o canto sob o campo devia bloatar por inteiro (delta ao Bloat cheio {near_matches_full}, \
         delta à base {near_moved})"
    );
    assert!(
        far_moved < far_would_move * 0.05,
        "o canto longe do campo devia ficar intacto (moveu {far_moved} de {far_would_move} possíveis)"
    );
}

/// **A modulação vale para os deformadores que REAMOSTRAM (Warp, Zig Zag), não só o Bloat.**
///
/// Robusto à reamostragem (a saída do Warp não tem correspondência de vértice): compara CAIXAS.
/// Um campo cheio (`w ≈ 1` em toda a forma: Radial de raio enorme) reproduz o deformador sozinho;
/// um campo vazio (`w ≈ 0`: o mesmo, invertido) reproduz a ENTRADA. Cobre os três de uma vez.
#[test]
fn a_full_field_is_the_effect_and_an_empty_field_is_the_input_for_every_deformer() {
    let base = square();
    let (blo, bhi) = bbox(&base);
    let deformers = [
        PathEffect::Bloat(BloatSpec { amount: 60.0 }),
        PathEffect::ZigZag(ZigZagSpec {
            amplitude: 40.0,
            ridges: 6.0,
            smooth: false,
            rough_seed: None,
        }),
        PathEffect::Warp(WarpSpec {
            style: WarpStyle::Bulge,
            bend: 80.0,
            h_distort: 0.0,
            v_distort: 0.0,
        }),
    ];
    // `size = 100` ⇒ o raio cobre a forma inteira: `w ≈ 1` em todo ponto.
    let full_field = FalloffSpec {
        shape: FalloffShape::Radial,
        amount: 1.0,
        size: 100.0,
        ..FalloffSpec::new(FalloffShape::Radial)
    };
    let empty_field = FalloffSpec {
        invert: true,
        ..full_field
    };
    for d in deformers {
        let alone = bbox(&run(&base, std::slice::from_ref(&d)));
        let with_full = bbox(&run(&base, &[PathEffect::Falloff(full_field), d.clone()]));
        let with_empty = bbox(&run(&base, &[PathEffect::Falloff(empty_field), d.clone()]));
        let label = d.label();
        for k in 0..2 {
            assert!(
                (with_full.0[k] - alone.0[k]).abs() < 0.02
                    && (with_full.1[k] - alone.1[k]).abs() < 0.02,
                "{label}: campo cheio devia reproduzir o efeito sozinho ({with_full:?} vs {alone:?})"
            );
            assert!(
                (with_empty.0[k] - blo[k]).abs() < 0.02 && (with_empty.1[k] - bhi[k]).abs() < 0.02,
                "{label}: campo vazio devia reproduzir a entrada ({with_empty:?} vs \
                 {:?})",
                (blo, bhi)
            );
        }
    }
}

/// Um Falloff sobre um efeito que NÃO consome força (Trim, Repeater) é inerte na geometria — e o
/// painel sabe disso por [`PathEffect::takes_falloff`], em vez de deixar o artista adivinhar.
#[test]
fn a_falloff_before_a_non_deformer_is_inert_and_takes_falloff_says_so() {
    let base = square();
    let field = FalloffSpec {
        amount: 1.0,
        ..FalloffSpec::new(FalloffShape::Radial)
    };
    let trim = PathEffect::Trim(crate::fx_trim::TrimSpec {
        start: 0.0,
        end: 0.5,
        offset: 0.0,
    });
    let with = run(&base, &[PathEffect::Falloff(field), trim.clone()]);
    let without = run(&base, &[trim]);
    assert_eq!(
        with.verts, without.verts,
        "um Falloff antes do Trim não pode mover a geometria"
    );

    assert!(!PathEffect::Trim(crate::fx_trim::TrimSpec::default()).takes_falloff());
    assert!(!PathEffect::Repeat(crate::fx_repeat::RepeatSpec::default()).takes_falloff());
    assert!(!PathEffect::Falloff(FalloffSpec::default()).takes_falloff());
    assert!(PathEffect::Bloat(BloatSpec::default()).takes_falloff());
    assert!(PathEffect::Warp(WarpSpec::default()).takes_falloff());
    assert!(PathEffect::ZigZag(ZigZagSpec::default()).takes_falloff());
}

/// Um Falloff sozinho (sem deformador que o consuma) NÃO produz geometria — o `cooked()` devolve
/// a fonte emprestada, e a pilha o salta.
#[test]
fn a_lone_falloff_produces_no_geometry() {
    let base = square();
    let field = FalloffSpec {
        amount: 1.0,
        ..FalloffSpec::new(FalloffShape::Radial)
    };
    assert!(run_stack(&base, &[FxEntry::new(PathEffect::Falloff(field))]).is_none());
}
