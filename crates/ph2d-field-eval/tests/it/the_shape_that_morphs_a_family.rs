//! ⭐⭐⭐ **A SUPERQUADRÁTICA (W127) — a forma com a maior razão família/linha do catálogo.**
//!
//! > **Enio, 06/09:** *«vamos seguir implementando até termos tudo isso. Escolha e siga»* — e a
//! > escolha foi o lote das duas de maior alcance ([doc 08 §7.4](../../../docs/3DModeling/08_formas_por_formula.md)).
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `the_sphere_is_exact_at_exponent_two` | a fórmula ter deixado de degenerar na esfera |
//! | `the_knob_really_travels_the_family` | o expoente ter virado enfeite |
//! | `every_exponent_in_the_range_keeps_the_march_honest` | o **divisor** estar errado — a peça rasga |
//! | `the_two_exponents_are_not_the_same_axis` | a **permutação** dos eixos estar trocada |
//!
//! ⚠️ **O terceiro é o load-bearing**, e é o único que precisa de varrer: a fórmula fechada do
//! divisor é uma DEMONSTRAÇÃO, e um gate que a recalculasse ao lado seria um oráculo feito da
//! função sob teste. Ele mede o campo **já dividido** e exige `‖∇f‖ ≤ 1`.

use ph2d_field::{
    FieldDoc, MAX_SUPERQUADRIC_EXPONENT, MIN_SUPERQUADRIC_EXPONENT, Node, NodeId, NodeKind,
    Primitive, Xform,
};
use ph2d_field_eval::Field;

fn campo(half: [f32; 3], top: f32, side: f32) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Superquadric {
                    half,
                    exponent_top: top,
                    exponent_side: side,
                }),
            )],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

/// O raio da superfície na direcção `dir`, por bissecção.
fn raio(f: &Field, dir: [f64; 3]) -> f64 {
    let n = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let u = [dir[0] / n, dir[1] / n, dir[2] / n];
    let (mut lo, mut hi) = (0.0_f64, 4.0_f64);
    for _ in 0..70 {
        let m = 0.5 * (lo + hi);
        if f.at(u[0] * m, u[1] * m, u[2] * m) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐ **A `2` nos dois expoentes ela É a esfera, e o campo é a DISTÂNCIA EXACTA.**
///
/// ⚠️ Não é decoração: é o ponto em que a fórmula geral tem de colapsar na forma que a paleta já
/// tem, e o divisor fechado tem de dar exactamente `1/raio` ali. *Um divisor errado por `√3`
/// passaria em toda a varredura de gradiente e falharia aqui.*
#[test]
fn the_sphere_is_exact_at_exponent_two() {
    // ⛔⛔ **O RAIO TEM DE VIR DO `f32` DO DOCUMENTO, e não do literal.** A 1.ª redacção deste gate
    // comparava com `0.35_f64` e lia `5,962e-9` de erro — que é **exactamente**
    // `0,35 − f64::from(0.35_f32)`. *A régua media a conversão de tipo do documento e eu quase
    // arquivei isso como imprecisão da fórmula.*
    let r = f64::from(0.35_f32);
    let f = campo([0.35; 3], 2.0, 2.0);
    let mut pior = 0.0_f64;
    for i in 0..25 {
        for j in 0..25 {
            for k in 0..25 {
                let at = |t: usize| -0.9 + 1.8 * (t as f64 + 0.5) / 25.0;
                let (x, y, z) = (at(i), at(j), at(k));
                let verdade = (x * x + y * y + z * z).sqrt() - r;
                pior = pior.max((f.at(x, y, z) - verdade).abs());
            }
        }
    }
    // ⚠️ **A barra é a REGULARIZAÇÃO, e está medida.** O par `exp`/`ln` deste avaliador vale
    // `3,5e-16` de erro relativo (`probe_exp_ln_accuracy`), logo não é ele; o que sobra é o
    // `EPS = 1e-12` que tira o `NaN` do gradiente na origem, e o erro **segue-o**:
    //
    // | `EPS` | `1e-12` | `1e-14` |
    // |---|---:|---:|
    // | pior erro | `1,833e-12` | `1,843e-14` |
    //
    // ⇒ a barra é uma década acima do medido. *Baixar o `EPS` para o gate ficar mais bonito seria
    // afinar o teste em vez do produto* — `5e-12` de erro relativo é invisível a toda régua deste
    // módulo.
    assert!(
        pior < 1.0e-11,
        "a `2` ela tinha de ser a esfera EXACTA — pior erro {pior:.3e}"
    );
}

/// ⭐⭐⭐ **O KNOB ATRAVESSA MESMO A FAMÍLIA** — losango → esfera → caixa, medido pelo canto.
///
/// A régua é o **raio na diagonal a dividir pelo raio no eixo**: `1/√2` num losango (o canto está
/// recuado), `1` numa esfera, `√2` num quadrado. ⚠️ *Uma forma cujo controlo não mexe na silhueta é
/// um knob morto*, e num campo dividido por um `K` que depende do expoente isso passaria despercebido
/// a qualquer sonda que medisse só o VALOR do campo.
#[test]
fn the_knob_really_travels_the_family() {
    let h = 0.35_f64;
    let mut anterior = 0.0_f64;
    for (n, esperado) in [(1.0_f32, 0.707_f64), (2.0, 1.0), (8.0, 1.29), (64.0, 1.40)] {
        let f = campo([0.35; 3], n, n);
        let razao = raio(&f, [1.0, 0.0, 1.0]) / raio(&f, [1.0, 0.0, 0.0]);
        assert!(
            (razao - esperado).abs() < 0.03,
            "com expoente {n}: a diagonal mede {razao:.3} do eixo, e devia medir ~{esperado:.3}"
        );
        assert!(
            razao > anterior,
            "o controlo tem de ser MONÓTONO: {n} deu {razao:.3}, abaixo do anterior {anterior:.3}"
        );
        anterior = razao;
        // E o raio no eixo é sempre a meia-medida, em todo expoente.
        assert!(
            (raio(&f, [1.0, 0.0, 0.0]) - h).abs() < 1.0e-3,
            "o eixo tem de ficar na meia-medida com expoente {n}"
        );
    }
}

/// ⭐⭐⭐ **O DIVISOR É HONESTO EM TODA A FAIXA** — o gate que impede a peça de rasgar.
///
/// ⚠️ **Ele mede o campo JÁ DIVIDIDO**, e não recalcula a fórmula fechada: um oráculo feito da
/// função sob teste não prova nada. A barra é `1,02`, a mesma folga de instrumento do censo (a
/// diferença central lê um pouco acima de `1` numa quina, por amostragem).
#[test]
fn every_exponent_in_the_range_keeps_the_march_honest() {
    let mut pior_global = 0.0_f64;
    let mut onde = String::new();
    // ⚠️ Uma peça **torta**: com meias-medidas iguais o divisor tem os três pesos iguais e o ramo
    // côncavo da conta nunca é exercitado.
    for half in [[0.35_f32; 3], [0.42, 0.24, 0.30]] {
        for top in [
            MIN_SUPERQUADRIC_EXPONENT,
            1.4,
            2.0,
            3.0,
            8.0,
            MAX_SUPERQUADRIC_EXPONENT,
        ] {
            for side in [
                MIN_SUPERQUADRIC_EXPONENT,
                1.6,
                2.0,
                5.0,
                MAX_SUPERQUADRIC_EXPONENT,
            ] {
                let f = campo(half, top, side);
                let e = f64::from(half[0].max(half[1]).max(half[2])) * 2.4;
                let mut pior = 0.0_f64;
                let n = 34;
                let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-5);
                            if g.is_finite() && g > pior {
                                pior = g;
                            }
                        }
                    }
                }
                if pior > pior_global {
                    pior_global = pior;
                    onde = format!("half {half:?}, de cima {top}, de lado {side}");
                }
            }
        }
    }
    assert!(
        pior_global <= 1.02,
        "o divisor não segura a faixa: ‖∇f‖ = {pior_global:.4} em {onde} — a marcha atravessa a \
         superfície e a peça sai rasgada"
    );
}

/// ⭐⭐ **OS DOIS EXPOENTES NÃO SÃO O MESMO EIXO** — o gate da permutação.
///
/// ⚠️ **Numa peça cúbica com expoentes iguais, trocar os eixos é a IDENTIDADE.** O divisor faz uma
/// permutação (`X–Z` é o de cima, `Y` é o de lado, porque o eixo de cima desta casa é o `Y`), e uma
/// troca ali sairia byte-idêntica no caso de omissão. *Só uma peça com os dois expoentes diferentes
/// a separa.*
#[test]
fn the_two_exponents_are_not_the_same_axis() {
    // De cima quase-quadrado, de lado quase-losango.
    let f = campo([0.35; 3], 32.0, 1.0);
    let plano = raio(&f, [1.0, 0.0, 1.0]) / raio(&f, [1.0, 0.0, 0.0]);
    let perfil = raio(&f, [1.0, 1.0, 0.0]) / raio(&f, [1.0, 0.0, 0.0]);
    assert!(
        plano > 1.30,
        "visto de CIMA ela tinha de ser quase quadrada (diagonal/eixo ~1,41) e mediu {plano:.3}"
    );
    assert!(
        perfil < 0.80,
        "visto de LADO ela tinha de ser quase um losango (diagonal/eixo ~0,71) e mediu {perfil:.3}"
    );
    // E o par trocado dá o contrário — senão os dois nomes descrevem a mesma coisa.
    let g = campo([0.35; 3], 1.0, 32.0);
    let plano2 = raio(&g, [1.0, 0.0, 1.0]) / raio(&g, [1.0, 0.0, 0.0]);
    assert!(
        plano2 < 0.80,
        "trocar os dois expoentes tinha de trocar as duas vistas — de cima mediu {plano2:.3}"
    );
}
