//! Gates do [`super::twist_contour`] — o remoinho.
//!
//! O esqueleto de reamostragem e a mistura pelo Falloff são partilhados com o Warp e já têm gates
//! próprios; aqui prova-se o MODELO do Twist: gira em torno do centro (não escala), com o ângulo
//! e o sentido certos no raio de referência, neutro byte-idêntico, e modulável por um Falloff na
//! pilha REAL. A prova de que NÃO rasga sobre uma forma com quinas é a folha de contacto visual
//! (`tests/fx_look.rs`), como a cerca do `fx_warp` exige.

use super::{TwistSpec, twist_contour};
use crate::effect::{FxCtx, FxEntry, PathEffect, run_stack};
use crate::fx_falloff::{FalloffShape, FalloffSpec};
use crate::{VecPath, VecVertex};

/// Um quadrado de lado 80 centrado na origem — a forma COM QUINAS, o caso de falha da cerca.
fn square() -> VecPath {
    VecPath {
        verts: [[-40.0, -40.0], [40.0, -40.0], [40.0, 40.0], [-40.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn radius(v: &VecVertex) -> f64 {
    v.anchor[0].hypot(v.anchor[1])
}

/// A distância da âncora mais próxima de `p`.
fn nearest(out: &[VecVertex], p: [f64; 2]) -> f64 {
    out.iter()
        .map(|v| (v.anchor[0] - p[0]).hypot(v.anchor[1] - p[1]))
        .fold(f64::MAX, f64::min)
}

/// **Ângulo zero é um no-op byte-idêntico** — os `verts` de entrada saem intocados, e é isso que
/// mantém o `Cow::Borrowed` do `cooked()` vivo num documento que só armou o efeito.
#[test]
fn a_neutral_twist_is_a_byte_identical_no_op() {
    let sq = square();
    let ctx = FxCtx::of(&sq);
    let (out, closed) = twist_contour(&sq.verts, true, &TwistSpec { angle: 0.0 }, &ctx, None);
    assert!(closed);
    assert_eq!(
        out, sq.verts,
        "o Twist neutro tem de devolver a entrada AO BIT"
    );
    // E a pilha SALTA o efeito neutro (não há geometria a produzir).
    let mut p = sq.clone();
    p.effects = vec![FxEntry::new(PathEffect::Twist(TwistSpec { angle: 0.0 }))];
    assert!(
        run_stack(&p, &p.effects).is_none(),
        "uma pilha só com um Twist neutro tem de devolver None (Cow::Borrowed sobrevive)"
    );
}

/// **O Twist é um cisalhamento ANGULAR: preserva o raio de cada ponto (gira, não escala).**
///
/// Toda amostra é `rotate(orig, θ(|orig|))` em torno do centro, e uma rotação preserva a distância
/// ao centro. Logo o perfil radial da saída é o da entrada — o quadrado tem raios em `[40, 56.57]`
/// (meio-de-aresta a canto), e o Twist não pode movê-los. A mutação que troca a rotação por uma
/// escala move os raios e sangra aqui.
#[test]
fn a_twist_shears_angularly_it_does_not_scale() {
    let sq = square();
    let ctx = FxCtx::of(&sq);
    let (out, _) = twist_contour(&sq.verts, true, &TwistSpec { angle: 180.0 }, &ctx, None);
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for v in &out {
        let r = radius(v);
        lo = lo.min(r);
        hi = hi.max(r);
    }
    // Canto a `40√2 ≈ 56.5685`, meio-de-aresta a `40`. O Twist preserva os dois.
    assert!((hi - 56.5685).abs() < 1.0, "o raio máximo mudou: {hi}");
    assert!((lo - 40.0).abs() < 1.0, "o raio mínimo mudou: {lo}");
    // A forma continua centrada na origem (girar em torno do centro fixa o centro).
    let (mut cx, mut cy) = (0.0, 0.0);
    for v in &out {
        cx += v.anchor[0];
        cy += v.anchor[1];
    }
    let n = out.len() as f64;
    assert!(
        (cx / n).abs() < 1.0 && (cy / n).abs() < 1.0,
        "o centroide saiu da origem: ({}, {})",
        cx / n,
        cy / n
    );
}

/// **No raio de referência, o ponto gira EXATAMENTE o ângulo autorado, no sentido certo.**
///
/// O meio-de-aresta `(40, 0)` está no raio de referência (`r = ref_size/2 = 40`), então `t = 1` e
/// ele gira o ângulo inteiro. A 45° vai para `(40cos45, 40sin45) ≈ (28.28, 28.28)`, e DEIXA `(40, 0)`.
/// A mutação que anula a rotação deixa-o em `(40, 0)` e sangra nas DUAS asserções.
#[test]
fn a_twist_rotates_the_reference_point_by_the_authored_angle() {
    let sq = square();
    let ctx = FxCtx::of(&sq);
    let (out, _) = twist_contour(&sq.verts, true, &TwistSpec { angle: 45.0 }, &ctx, None);
    let rotated = [
        40.0 * core::f64::consts::FRAC_1_SQRT_2,
        40.0 * core::f64::consts::FRAC_1_SQRT_2,
    ];
    assert!(
        nearest(&out, rotated) < 1.5,
        "nenhuma âncora chegou ao ponto girado {rotated:?}"
    );
    assert!(
        nearest(&out, [40.0, 0.0]) > 3.0,
        "o ponto de referência não saiu de (40, 0) — o giro não aconteceu"
    );
}

/// **Um Falloff antes de um Twist MODULA o giro** — a pilha real, o contrato `takes_falloff`.
///
/// Um Falloff Radial estreito (raio ~40% da forma) antes do Twist deixa o miolo girar e poupa a
/// borda; sem o campo, a borda gira também. Os dois resultados TÊM de diferir, senão o Twist não
/// consome o campo (e o painel estaria a mentir com *"modulates the effect below"*).
#[test]
fn a_falloff_before_a_twist_localizes_the_spin() {
    let sq = square();
    let twist = FxEntry::new(PathEffect::Twist(TwistSpec { angle: 180.0 }));
    let radial = FxEntry::new(PathEffect::Falloff(FalloffSpec {
        shape: FalloffShape::Radial,
        amount: 1.0,
        size: 0.4,
        ..FalloffSpec::new(FalloffShape::Radial)
    }));

    let plain =
        run_stack(&sq, std::slice::from_ref(&twist)).expect("o Twist sozinho produz geometria");
    let modulated = run_stack(&sq, &[radial, twist]).expect("Falloff+Twist produz geometria");

    // Mesma grade de reamostragem ⇒ mesma contagem; comparam âncora a âncora.
    assert_eq!(
        plain.verts.len(),
        modulated.verts.len(),
        "os dois passam pela mesma reamostragem"
    );
    let moved = plain
        .verts
        .iter()
        .zip(&modulated.verts)
        .map(|(a, b)| (a.anchor[0] - b.anchor[0]).hypot(a.anchor[1] - b.anchor[1]))
        .fold(0.0_f64, f64::max);
    assert!(
        moved > 1.0,
        "o Falloff não mudou o Twist em lugar nenhum (deslocamento máximo {moved}) — o campo é inerte"
    );
}
