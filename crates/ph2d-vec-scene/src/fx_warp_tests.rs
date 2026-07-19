//! Gates do Twist e do Pucker & Bloat. Cada um tem UM botão, então o que há a provar é que ele
//! faz o que diz e que o neutro não custa nada.

use super::*;
use crate::VecPath;

/// Um quadrado de lado 40 na origem — de propósito NÃO centrado no mundo, para os gates
/// distinguirem "o centro da forma" de "a origem".
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn ctx() -> FxCtx {
    FxCtx::of(&square())
}

/// A distância de cada âncora ao centro da forma.
fn radii(v: &[VecVertex], c: &FxCtx) -> Vec<f64> {
    v.iter()
        .map(|w| (w.anchor[0] - c.center[0]).hypot(w.anchor[1] - c.center[1]))
        .collect()
}

/// **Os dois neutros são no-op byte-idêntico** — o que a pilha exige de todo efeito.
#[test]
fn both_neutral_points_are_byte_identical_no_ops() {
    let p = square();
    let c = ctx();
    assert_eq!(
        twist_contour(&p.verts, true, &TwistSpec::default(), &c).0,
        p.verts
    );
    assert_eq!(
        bloat_contour(&p.verts, true, &BloatSpec::default(), &c).0,
        p.verts
    );
}

/// **O Twist GIRA e não muda o raio** — cada ponto anda sobre a sua circunferência.
///
/// É o gate que separa um giro de uma escala: se o campo tivesse componente radial, os raios
/// mudariam. E o centro é o da FORMA — num quadrado de `0..40` ele é `(20,20)`, longe da origem.
#[test]
fn the_twist_turns_without_changing_any_radius() {
    let p = square();
    let c = ctx();
    // ⚠️ Emparelhar por ÍNDICE morreu quando o Twist passou a SUBDIVIDIR (um campo não-afim
    // tem de o fazer): a saída tem mais âncoras do que a entrada. O oráculo mede o ALCANCE dos
    // raios, que é invariante sob subdivisão — e é o que a aparência realmente promete.
    let span = |v: &[VecVertex]| -> (f64, f64) {
        radii(v, &c)
            .into_iter()
            .fold((f64::MAX, f64::MIN), |(lo, hi), r| (lo.min(r), hi.max(r)))
    };
    let (_, hi0) = span(&p.verts);
    let out = twist_contour(&p.verts, true, &TwistSpec { angle: 90.0 }, &c).0;
    let (_, hi1) = span(&out);
    assert!(
        (hi1 - hi0).abs() < 1e-9,
        "o ponto mais distante mudou de raio: {hi0} -> {hi1}. Girar não mexe na distância."
    );
    assert!(
        out.iter().any(|o| {
            p.verts
                .iter()
                .all(|q| (o.anchor[0] - q.anchor[0]).hypot(o.anchor[1] - q.anchor[1]) > 1.0)
        }),
        "e alguma coisa tem de se ter MEXIDO"
    );
}

/// **A força do Twist cresce com a DISTÂNCIA** — é isto, e só isto, que o separa de uma rotação
/// rígida.
///
/// ⚠️ A 1ª versão deste gate media um ponto colocado NO centro e exigia que ele não andasse.
/// Ficava verde com o campo errado: o centro é ponto fixo de **qualquer** rotação em torno dele,
/// rígida ou não. A mutação que troca a força variável por uma constante sobreviveu, e o
/// culpado era o fixture — estava no único sítio onde as duas leis concordam.
///
/// O oráculo tem de comparar DOIS raios diferentes, ambos não-nulos: o de fora tem de girar
/// mais. O centro parado fica como asserção secundária, agora que não é a única.
#[test]
fn the_twists_force_grows_with_distance() {
    let c = ctx();
    // Raios 5 e 15 a partir do centro, no mesmo eixo: a razão dos ângulos girados tem de ser a
    // razão dos raios (o campo é linear em `t`), e numa rotação rígida seria 1.
    let pts = vec![
        VecVertex::corner(c.center),
        VecVertex::corner([c.center[0] + 5.0, c.center[1]]),
        VecVertex::corner([c.center[0] + 15.0, c.center[1]]),
    ];
    let out = twist_contour(&pts, false, &TwistSpec { angle: 60.0 }, &c).0;
    // O Twist subdivide, então não há correspondência 1-a-1 — mas ele PRESERVA o raio, e os
    // raios 5 e 15 são únicos nesta fixture. Procura-se por raio, não por índice.
    let turned = |want: f64| -> f64 {
        let dist = |q: &VecVertex| {
            ((q.anchor[0] - c.center[0]).hypot(q.anchor[1] - c.center[1]) - want).abs()
        };
        let v = out
            .iter()
            .min_by(|a, b| dist(a).total_cmp(&dist(b)))
            .expect("saída não vazia");
        (v.anchor[1] - c.center[1]).atan2(v.anchor[0] - c.center[0])
    };
    let d0 = out
        .iter()
        .map(|v| (v.anchor[0] - c.center[0]).hypot(v.anchor[1] - c.center[1]))
        .fold(f64::MAX, f64::min);
    assert!(d0 < 1e-9, "o ponto no centro andou {d0}");
    let (near, far) = (turned(5.0), turned(15.0));
    assert!(near > 1e-6, "o ponto de dentro não girou nada ({near} rad)");
    assert!(
        (far / near - 3.0).abs() < 1e-6,
        "o ponto três vezes mais longe devia girar três vezes mais: {near} e {far} rad          (razão {}). Razão 1 significa rotação RÍGIDA, que não é um twist.",
        far / near
    );
}

/// **O Bloat move âncoras e curva em sentidos OPOSTOS** — é isso, e só isso, que o separa de
/// uma escala.
///
/// ⚠️ A 1ª versão escalava âncoras e alças pelo MESMO fator, e o gate media exatamente isso:
/// verde sobre uma escala uniforme, que é o gizmo e não um efeito (Enio, 2026-07-18:
/// *"só aumenta e reduz a escala do objeto"*). O oráculo tem de comparar os DOIS.
#[test]
fn the_bloat_moves_anchors_and_handles_in_opposite_directions() {
    let c = ctx();
    // Um vértice com alça DESTACADA da âncora, senão não há como ver os dois fatores.
    let v = VecVertex {
        anchor: [c.center[0] + 10.0, c.center[1]],
        in_handle: [c.center[0] + 10.0, c.center[1] - 4.0],
        out_handle: [c.center[0] + 10.0, c.center[1] + 4.0],
        kind: crate::VertexKind::Smooth,
        corner_radius: 0.0,
    };
    let r = |p: [f64; 2]| (p[0] - c.center[0]).hypot(p[1] - c.center[1]);
    let (a0, h0) = (r(v.anchor), r(v.out_handle));

    let out = bloat_contour(
        std::slice::from_ref(&v),
        false,
        &BloatSpec { amount: 50.0 },
        &c,
    )
    .0;
    let (a1, h1) = (r(out[0].anchor), r(out[0].out_handle));
    assert!(
        a1 < a0 && h1 > h0,
        "bloat: a âncora tem de ir para DENTRO ({a0} -> {a1}) e a alça para FORA ({h0} -> {h1}).          Os dois no mesmo sentido são uma escala."
    );

    let out = bloat_contour(&[v], false, &BloatSpec { amount: -50.0 }, &c).0;
    let (a2, h2) = (r(out[0].anchor), r(out[0].out_handle));
    assert!(
        a2 > a0 && h2 < h0,
        "pucker: o inverso — âncora para fora ({a0} -> {a2}), alça para dentro ({h0} -> {h2})"
    );
}

/// **O raio de quina segue a ÂNCORA**, não a alça: ele é um comprimento ancorado nela, e os dois
/// fatores divergem.
#[test]
fn the_bloat_scales_the_corner_radius_with_the_anchor() {
    let c = ctx();
    let mut v = VecVertex::corner([c.center[0] + 20.0, c.center[1]]);
    v.corner_radius = 6.0;
    let out = bloat_contour(&[v], false, &BloatSpec { amount: -100.0 }, &c).0;
    assert!(
        (out[0].corner_radius - 12.0).abs() < 1e-9,
        "com as âncoras a duplicar, o raio devia duplicar; deu {}",
        out[0].corner_radius
    );
}

/// Uma forma degenerada (sem tamanho) não tem centro nem raio — o Twist é **inerte**, não um
/// `NaN` que se espalha por toda a geometria a jusante.
#[test]
fn a_degenerate_shape_leaves_the_twist_inert() {
    let dot: Vec<VecVertex> = (0..4).map(|_| VecVertex::corner([3.0, 7.0])).collect();
    let c = FxCtx::of(&VecPath {
        verts: dot.clone(),
        closed: true,
        ..VecPath::default()
    });
    assert_eq!(
        twist_contour(&dot, true, &TwistSpec { angle: 90.0 }, &c).0,
        dot
    );
}
