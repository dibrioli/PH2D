//! ⭐⭐⭐ **A TORÇÃO nunca superestima a distância** — a condição que impede a marcha de furar.
//!
//! ⚠️ **Tudo aqui entra pela API do DOCUMENTO** (`Unary::Twist` numa pilha de nó), e nunca pela
//! árvore crua: a W101 pagou **cinco mutações sobreviventes** por gatear o campo em vez da porta.
//!
//! # A lei, e por que ela não tem constante ajustada
//!
//! O jacobiano do mapa inverso tem as duas primeiras colunas ortonormais e a terceira igual a
//! `(k·q_y, −k·q_x, 1)`. Com `β = |k|·r`, o maior valor singular sai em forma fechada:
//!
//! ```text
//! σ_max(β) = β/2 + √(1 + β²/4)
//! ```
//!
//! ⛔ **Não é `√(1 + β²)`** — os dois termos podem ALINHAR-SE, e por isso somam-se linearmente e não
//! em quadratura. A diferença chega a `13,4 %` (em `β ≈ 0,7`), e `13 %` acima da distância verdadeira
//! não fica lento: **fura**.
//!
//! # ⛔⛔ E a MEDIÇÃO refutou a primeira forma do divisor, não apenas a constante
//!
//! Dividir por `σ(k·r)` **no ponto** parece mais apertado e é pior: `∇(f/d) = ∇f/d − f·∇d/d²`, e o
//! segundo termo cresce **com o próprio divisor**. Medido a uma volta por unidade, com a margem a
//! subir: `1,78 · 2,11 · 2,32 · 2,51 · 2,55` — *subir a margem PIORA*.
//!
//! ⭐ O divisor **constante** `σ(k·R)` não tem gradiente próprio, e a tabela fecha sem afinar nada:
//!
//! | voltas/un | `σ(k·R)` | `‖∇f‖` |
//! |---:|---:|---:|
//! | 0,05 | `1,1421` | `0,9617` |
//! | 0,30 | `2,0802` | `0,8167` |
//! | 1,00 | `5,5129` | `0,7068` |
//! | 2,00 | `10,7559` | `0,7039` |

use ph2d_field::{
    FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform, mods::MAX_TWIST_TURNS,
};
use ph2d_field_eval::Field;

/// Uma caixa larga o suficiente para haver raio grande onde a torção morde, **com o modificador na
/// pilha do nó** — a porta que o produto usa.
fn torcida(turns: f32, lower: f32, upper: f32) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.6, 0.6, 0.35],
            round: 0.0,
        }),
    );
    if turns != 0.0 {
        n.mods.push(Unary::Twist {
            turns,
            lower,
            upper,
        });
    }
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

fn worst_gradient(doc: &FieldDoc, steps: i32) -> f64 {
    let f = Field::new(doc);
    let mut hi = 0.0f64;
    for i in 0..=steps {
        for j in 0..=steps {
            for k in 0..=steps {
                let p = |n: i32| f64::from(n) / f64::from(steps) * 2.0 - 1.0;
                let g = f.gradient_norm(p(i), p(j), p(k), 1e-3);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

/// ⭐ **O gate**: em todo o alcance do slider, o campo é 1-Lipschitz — logo é minorante da distância.
#[test]
fn the_twist_never_overestimates_the_distance() {
    // ⚠️ A folga é da DIFERENÇA CENTRAL sobre um campo em `f32`, não uma tolerância de calibração.
    const SLACK: f64 = 1.02;
    for turns in [0.05f32, 0.25, 0.5, 1.0, MAX_TWIST_TURNS] {
        for sinal in [1.0f32, -1.0] {
            let doc = torcida(turns * sinal, -9.0, 9.0);
            let g = worst_gradient(&doc, 28);
            assert!(
                g <= SLACK,
                "{turns} voltas ({sinal:+}): ‖∇f‖ = {g:.4} — a marcha atravessa a superfície, e o \
                 sintoma é pixel de fundo no meio da peça"
            );
        }
    }
}

/// ⛔ **O CONTROLE**: sem ele o gate acima passaria numa lei que devolvesse zero em todo o lado.
///
/// ⚠️ E ele mede a torção **nos dois sentidos**: sem a metade negativa, um `abs()` a mais passa.
#[test]
fn the_twist_actually_turns_the_section_and_knows_which_way() {
    // A largura da peça ao longo de X, na altura `z`, medida por bissecção do zero.
    let largura = |doc: &FieldDoc, z: f64| {
        let f = Field::new(doc);
        let (mut lo, mut hi) = (0.0f64, 2.0f64);
        for _ in 0..40 {
            let mid = f64::midpoint(lo, hi);
            if f.at(mid, 0.0, z) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        f64::midpoint(lo, hi)
    };
    let reta = torcida(0.0, 0.0, 0.0);
    // Uma caixa não torcida tem a mesma largura em toda a altura.
    let base = largura(&reta, 0.0);
    assert!(
        (largura(&reta, 0.3) - base).abs() < 1e-3,
        "a fixtura não é uma caixa: a largura já muda com a altura sem torção nenhuma"
    );
    // ⭐ Com torção, a diagonal roda para o eixo X e a largura MUDA — e muda para os dois lados.
    for sinal in [1.0f32, -1.0] {
        let doc = torcida(0.25 * sinal, -9.0, 9.0);
        let alto = largura(&doc, 0.3);
        assert!(
            (alto - base).abs() > 0.02,
            "{sinal:+}: a secção a z = 0,3 mede {alto:.4} contra {base:.4} na origem — a torção não \
             está a rodar nada"
        );
    }
}

/// ⭐⭐ **A BANDA**: fora dela a peça não torce, e o que está além roda como CORPO RÍGIDO.
///
/// ⚠️ É o que separa um deformador de um brinquedo — sem limites não há «torcer só o topo».
#[test]
fn the_band_leaves_the_outside_rigid() {
    let largura = |doc: &FieldDoc, z: f64| {
        let f = Field::new(doc);
        let (mut lo, mut hi) = (0.0f64, 2.0f64);
        for _ in 0..40 {
            let mid = f64::midpoint(lo, hi);
            if f.at(mid, 0.0, z) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        f64::midpoint(lo, hi)
    };
    // A banda cobre só a metade de cima.
    let doc = torcida(0.5, 0.0, 9.0);
    let reta = torcida(0.0, 0.0, 0.0);
    let base = largura(&reta, 0.0);
    assert!(
        (largura(&doc, -0.3) - base).abs() < 1e-3,
        "abaixo da banda a peça devia estar INTACTA, e mudou"
    );
    assert!(
        (largura(&doc, 0.3) - base).abs() > 0.02,
        "dentro da banda a peça devia torcer, e não torceu"
    );
    // ⭐ E acima do topo da banda o ângulo CONGELA: duas alturas além dela medem o mesmo.
    let alta = torcida(0.5, -0.1, 0.1);
    assert!(
        (largura(&alta, 0.2) - largura(&alta, 0.34)).abs() < 1e-3,
        "além da banda o ângulo devia congelar (corpo rígido), e a secção continua a rodar"
    );
}

/// ⭐ **Zero voltas é a peça INTACTA, ao bit** — senão toda peça já gravada mudaria de forma no dia
/// em que o modificador nascesse.
#[test]
fn a_twist_of_zero_is_the_piece_untouched() {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.6, 0.6, 0.35],
            round: 0.0,
        }),
    );
    let limpa = FieldDoc::new(vec![n.clone()], NodeId(0)).expect("peça");
    n.mods.push(Unary::Twist {
        turns: 0.0,
        lower: -1.0,
        upper: 1.0,
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
                    "torção de zero mudou o campo em ({x}, {y}, {z})"
                );
            }
        }
    }
}
