//! ⭐⭐⭐ **A DOBRA nunca superestima a distância** — e ela tem uma parede que a torção não tem.
//!
//! O mapa inverso curva o eixo `Z` no plano `XZ`. As duas linhas do bloco 2×2 do jacobiano são
//! **ortogonais**, com normas `1` e `ρ/Rr`, logo os valores singulares são exactamente `{1, ρ/Rr, 1}`
//! e o tecto é `σ = max(1, ρ/Rr) = max(1, 1/(1 − κ·W))`.
//!
//! ⛔ **A parede é do DOCUMENTO:** em `κ·W = 1` o lado de dentro colapsa no centro do arco, o mapa
//! deixa de ser injectivo e o campo devolve lixo nos dois sentidos. O operador **satura** ali — e
//! como só quem vê a peça sabe onde ela está, a saturação mora no operador e não num `MAX_*`.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform, mods::MAX_BEND_TURNS};
use ph2d_field_eval::Field;

fn vara(turns: f32, lower: f32, upper: f32) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.16, 0.16, 0.7],
            round: 0.02,
        }),
    );
    if turns != 0.0 {
        n.mods.push(Unary::Bend {
            turns,
            lower,
            upper,
            falloff: 0.0,
        });
    }
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// ⚠️ **A grelha é a CAIXA DE RECORTE, e isto é uma correcção.**
///
/// A 1.ª redacção varria um cubo fixo de `±1,2` e acusou `‖∇f‖ = 1,1425` — **fora da peça**. A lei do
/// minorante só tem de valer onde o avaliador de facto olha, e o avaliador é preso à AABB da
/// `bounding_ball` (`Scene::clip`); além dela ninguém pergunta nada. *Uma régua que mede fora do
/// domínio da lei acusa código correcto* — e a mesma armadilha esteve debaixo do gate da torção, que
/// passou por a caixa dele calhar de conter a grelha.
fn worst_gradient(doc: &FieldDoc, steps: i32) -> f64 {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, &reg).expect("a peça tem bordo");
    let (lo, hi_box) = bola.aabb();
    let f = Field::new(doc);
    let mut hi = 0.0f64;
    for i in 0..=steps {
        for j in 0..=steps {
            for k in 0..=steps {
                let p = |n: i32, e: usize| {
                    let t = f64::from(n) / f64::from(steps);
                    f64::from(lo[e]) + t * f64::from(hi_box[e] - lo[e])
                };
                let g = f.gradient_norm(p(i, 0), p(j, 1), p(k, 2), 1e-3);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

/// O centro da secção em `x`, à altura `z` — é ele que se desloca quando a vara dobra.
///
/// ⛔ **A 1.ª redacção bissectava a partir do EIXO, e isso é uma petição de princípio:** ela assume
/// que o eixo continua dentro da peça, que é exactamente o que a dobra desfaz. Numa vara dobrada a
/// `0,25` voltas o ponto `(0, 0, 0,45)` está a `−0,004` do bordo — e além disso, **fora**. *Uma
/// sonda que precisa que o efeito não tenha acontecido não pode medir o efeito.*
///
/// ⇒ ela varre o intervalo e devolve o meio do que estiver **dentro**.
fn centro_x(f: &Field, z: f64) -> Option<f64> {
    const N: i32 = 2000;
    let x = |i: i32| f64::from(i) / f64::from(N) * 4.0 - 2.0;
    let dentro: Vec<f64> = (0..=N).map(x).filter(|&v| f.at(v, 0.0, z) < 0.0).collect();
    match dentro.as_slice() {
        [] => None,
        [.., ultimo] => Some(f64::midpoint(dentro[0], *ultimo)),
    }
}

#[test]
fn the_bend_never_overestimates_the_distance() {
    const SLACK: f64 = 1.02;
    for turns in [0.05f32, 0.15, 0.3, MAX_BEND_TURNS] {
        for sinal in [1.0f32, -1.0] {
            let doc = vara(turns * sinal, -9.0, 9.0);
            let g = worst_gradient(&doc, 26);
            assert!(
                g <= SLACK,
                "{turns} voltas ({sinal:+}): ‖∇f‖ = {g:.4} — a marcha atravessa a superfície"
            );
        }
    }
}

/// ⛔ **O CONTROLE**: a vara tem de DOBRAR, e para os dois lados.
#[test]
fn the_bend_actually_curves_the_axis_and_knows_which_way() {
    let reta = vara(0.0, 0.0, 0.0);
    let f = Field::new(&reta);
    let base = centro_x(&f, 0.45).expect("RETA: nada dentro em z=0,45");
    assert!(
        base.abs() < 1e-3,
        "a fixtura já está torta sem dobra nenhuma: centro em {base:.4}"
    );
    for sinal in [1.0f32, -1.0] {
        let doc = vara(0.25 * sinal, -9.0, 9.0);
        let f = Field::new(&doc);
        let c = centro_x(&f, 0.45)
            .unwrap_or_else(|| panic!("DOBRADA {sinal:+}: nada dentro em z=0,45"));
        assert!(
            c.abs() > 0.05,
            "{sinal:+}: o centro da secção a z = 0,45 está em {c:.4} — a vara não dobrou"
        );
        assert!(
            (c > 0.0) == (sinal > 0.0),
            "{sinal:+}: a vara dobrou para o lado errado (centro {c:.4})"
        );
    }
}

/// ⭐ **Zero voltas é a peça INTACTA, ao bit** — e aqui é obrigatório por mais uma razão: `κ = 0` dá
/// `ρ = ∞`, e a conta do mapa seria `0/0`.
#[test]
fn a_bend_of_zero_is_the_piece_untouched() {
    let limpa = vara(0.0, 0.0, 0.0);
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.16, 0.16, 0.7],
            round: 0.02,
        }),
    );
    n.mods.push(Unary::Bend {
        turns: 0.0,
        lower: -1.0,
        upper: 1.0,
        falloff: 0.5,
    });
    let neutra = FieldDoc::new(vec![n], NodeId(0)).expect("peça");
    let (a, b) = (Field::new(&limpa), Field::new(&neutra));
    for i in 0..13 {
        for j in 0..13 {
            for k in 0..13 {
                let p = |n: i32| f64::from(n) / 6.0 - 1.0;
                let (x, y, z) = (p(i), p(j), p(k));
                assert!(
                    (a.at(x, y, z) - b.at(x, y, z)).abs() < f64::EPSILON,
                    "dobra de zero mudou o campo em ({x}, {y}, {z})"
                );
            }
        }
    }
}

/// ⭐⭐ **A PAREDE do vinco satura, e a peça sobrevive a pedir o impossível.**
///
/// ⚠️ Sem a saturação, `κ·W ≥ 1` põe o divisor em infinito e o campo em `NaN` — e um `NaN` na marcha
/// não é lento, é a peça a desaparecer.
#[test]
fn asking_for_more_bend_than_the_piece_allows_saturates_instead_of_breaking() {
    // Uma peça GORDA: a parede dela é baixa, e o teto do slider passa-a de longe.
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.6, 0.6, 0.6],
            round: 0.0,
        }),
    );
    n.mods.push(Unary::Bend {
        turns: MAX_BEND_TURNS,
        lower: -9.0,
        upper: 9.0,
        falloff: 0.0,
    });
    let doc = FieldDoc::new(vec![n], NodeId(0)).expect("peça");
    let f = Field::new(&doc);
    let mut vistos = 0usize;
    for i in 0..=20 {
        for j in 0..=20 {
            for k in 0..=20 {
                let p = |t: i32| f64::from(t) / 10.0 * 1.4 - 1.4;
                let v = f.at(p(i), p(j), p(k));
                assert!(v.is_finite(), "o campo devolveu {v} — a parede não saturou");
                if v < 0.0 {
                    vistos += 1;
                }
            }
        }
    }
    assert!(vistos > 50, "a peça desapareceu ({vistos} amostras dentro)");
    let g = worst_gradient(&doc, 22);
    assert!(
        g <= 1.02,
        "‖∇f‖ = {g:.4} na parede — o divisor não acompanhou a saturação"
    );
}
