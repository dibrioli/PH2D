//! Os gates da leitura `C¹` do campo. ⚠️ Ela é **pura** — uma malha e uma tabela —, logo mede-se
//! sem mundo nenhum.
//!
//! ⛔⛔ **O porte de um interpolante valida-se por PROPRIEDADE e não por valor.** A SciPy estima
//! os gradientes por um método global de minimização de curvatura e esta casa usa mínimos
//! quadrados sobre o anel; comparar número a número mediria a escolha do estimador. O que TEM de
//! bater são as leis: exactidão nos vértices · partição da unidade · **derivada contínua através
//! da aresta** · precisão linear. É o que estes gates afirmam.

use super::CampoSuave;
use crate::pesos::CampoDoDominio;
use ph2d_poly2d::Mesh2d;

/// Uma grelha `k × k` triangulada sobre `[0, 1]²`, com `b` campos de peso quaisquer que somam `1`.
fn grelha(k: usize) -> CampoDoDominio {
    let n = k + 1;
    let mut rest = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            #[expect(clippy::cast_precision_loss, reason = "grelha pequena")]
            rest.push([i as f64 / k as f64, j as f64 / k as f64]);
        }
    }
    let mut tris = Vec::new();
    for j in 0..k {
        for i in 0..k {
            let (a, b, c, d) = (
                j * n + i,
                j * n + i + 1,
                (j + 1) * n + i,
                (j + 1) * n + i + 1,
            );
            tris.push([a as u32, b as u32, d as u32]);
            tris.push([a as u32, d as u32, c as u32]);
        }
    }
    // Três pesos que somam 1 e não são lineares — senão a precisão linear esconderia tudo.
    //
    // ⛔⛔ **Cada um tem de depender dos DOIS eixos**, e a 1.ª redacção falhou nisso: com
    // `w0 = f(x)` o campo é exactamente linear dentro de cada célula, os dois triângulos dela têm
    // o MESMO gradiente, e a leitura linear **não salta** — o controlo do gate da derivada leu
    // `0,000e0` e apanhou a fixtura. *Uma fixtura separável não contém o fenómeno da aresta.*
    let mut pesos = Vec::with_capacity(rest.len() * 3);
    for p in &rest {
        let w0 = p[0].mul_add(2.3, p[1] * 1.7).sin().mul_add(0.25, 0.5);
        let w1 = p[0].mul_add(-1.1, p[1] * 2.9).cos().mul_add(0.2, 0.3) * (1.0 - w0);
        pesos.extend_from_slice(&[w0, w1, 1.0 - w0 - w1]);
    }
    CampoDoDominio {
        malha: Mesh2d {
            rest,
            tris,
            size: [1, 1],
        },
        pesos,
        regua: [0.0, 0.0, 1.0],
    }
}

/// ⭐⭐⭐ **GATE — a leitura `C¹` é EXACTA nos vértices.**
///
/// Um interpolante que não passa pelos próprios dados não é um interpolante.
#[test]
fn a_leitura_c1_e_exacta_nos_vertices() {
    let c = grelha(6);
    let suave = CampoSuave::novo(&c).expect("campo válido");
    for v in 0..c.malha.rest.len() {
        let p = c.local_do_vertice(v).expect("régua");
        let lida = suave.linha(p).expect("dentro da malha");
        for (j, &l) in lida.iter().enumerate() {
            let esperado = c.pesos[v * 3 + j];
            assert!(
                (l - esperado).abs() < 1e-9,
                "no vértice {v} o osso {j} leu {l} e o dado é {esperado}"
            );
        }
    }
}

/// ⭐⭐⭐ **GATE — A PARTIÇÃO DA UNIDADE é exacta, e é ela que impede a arte de encolher.**
///
/// ⚠️ **Com `Σw < 1` a mistura ENCOLHE o ponto para a origem** — o doc do `maiores_k` já o
/// escreve. A lei é linear nos dados, e com `f ≡ 1` e `g ≡ 0` cada Hermite devolve `1`; este gate
/// mede-o no produto em vez de o supor.
#[test]
fn a_leitura_c1_soma_um_em_todo_o_lado() {
    let c = grelha(5);
    let suave = CampoSuave::novo(&c).expect("campo válido");
    let mut pior = 0.0_f64;
    for i in 0..=97 {
        for j in 0..=97 {
            let p = [f64::from(i) / 97.0, f64::from(j) / 97.0];
            let Some(l) = suave.linha(p) else { continue };
            pior = pior.max((l.iter().sum::<f64>() - 1.0).abs());
        }
    }
    assert!(pior < 1e-9, "a soma dos pesos afastou-se de 1 em {pior:e}");
}

/// ⭐⭐⭐ **GATE — A DERIVADA É CONTÍNUA ATRAVÉS DA ARESTA. É a wave inteira numa asserção.**
///
/// ⛔⛔⛔ **O veredito é a COLUNA e não a célula, e foi isso que apanhou a minha implementação.**
/// Uma sonda de largura `δ` sobre um campo **liso** lê uma diferença que **desaparece com `δ`**
/// (é a 2.ª derivada vezes `δ`); sobre um campo com um DEGRAU ela lê o degrau, **constante**. Um
/// limiar absoluto não separa os dois: a 1.ª redacção deste ficheiro lia `2,4e−2` — cinco vezes
/// menos que a lei linear, e a passar por *«bem melhor»* — e era um campo `C⁰` na mesma.
///
/// Medido:
///
/// | `δ` | `C¹` | LINEAR |
/// |---:|---:|---:|
/// | `1e−2` | `8,87e−3` | `1,1468e−1` |
/// | `1e−3` | `7,09e−4` | `1,1468e−1` |
/// | `1e−4` | `6,92e−5` | `1,1468e−1` |
///
/// ⚠️ **O CONTROLO é obrigatório:** a leitura linear tem de **saltar** na mesma aresta e **não
/// encolher**. Sem ele este gate ficava verde sobre um campo constante, sobre uma malha de um
/// triângulo, ou sobre uma travessia que por acaso não cruza aresta nenhuma — *e a 1.ª fixtura
/// deste ficheiro era exactamente essa: com `w = f(x)` o campo é linear dentro da célula, os dois
/// triângulos dela têm o MESMO gradiente, e o controlo leu `0,000e0`.*
#[test]
fn a_derivada_nao_salta_na_aresta_e_a_linear_salta() {
    let c = grelha(4);
    let suave = CampoSuave::novo(&c).expect("campo válido");
    let arestax = 0.1;
    let salto = |e: f64, c1: bool| -> f64 {
        let deriva = |x: f64| -> f64 {
            let ler = |q: f64| -> f64 {
                let p = [q, 0.1];
                if c1 {
                    suave.linha(p).expect("dentro")[0]
                } else {
                    c.linha(p).expect("dentro")[0]
                }
            };
            (ler(x + e * 0.1) - ler(x - e * 0.1)) / (2.0 * e * 0.1)
        };
        (deriva(arestax + e) - deriva(arestax - e)).abs()
    };
    let (largo, estreito) = (1e-2, 1e-4);
    let (c1_l, c1_e) = (salto(largo, true), salto(estreito, true));
    let (lin_l, lin_e) = (salto(largo, false), salto(estreito, false));
    println!("  C¹    δ=1e-2 {c1_l:.3e} → δ=1e-4 {c1_e:.3e}");
    println!("  LINEAR δ=1e-2 {lin_l:.3e} → δ=1e-4 {lin_e:.3e}");

    // (1) A LEI LINEAR SALTA, e o salto NÃO encolhe — é um degrau. Sem esta metade a fixtura
    //     podia não conter aresta nenhuma e o resto ficava verde a afirmar nada.
    assert!(
        lin_l > 1e-3 && lin_e > lin_l * 0.5,
        "a leitura LINEAR devia ler um DEGRAU que não encolhe, e leu {lin_l:e} → {lin_e:e}"
    );
    // (2) ⭐ A LEI C¹ DESAPARECE COM A SONDA — pelo menos `20×` ao estreitar `100×`.
    assert!(
        c1_e < c1_l * 0.05,
        "o salto da leitura C¹ devia DESAPARECER com a sonda e foi de {c1_l:e} para {c1_e:e} —          um campo que só fica «mais liso» estabiliza, e um C¹ tende para zero"
    );
    // (3) E no fim ela é ordens de grandeza abaixo da linear.
    assert!(
        c1_e < lin_e * 1e-2,
        "à sonda mais estreita o C¹ devia estar 100× abaixo da linear ({c1_e:e} contra {lin_e:e})"
    );
}

/// ⭐⭐ **GATE — a lei reproduz um campo LINEAR ao bit.**
///
/// ⚠️ Sem isto um interpolante pode ser liso e **errado**: um que devolvesse sempre a média dos
/// vértices é `C∞` e não interpola nada. *A precisão linear é o que o prende aos dados.*
#[test]
fn a_leitura_c1_reproduz_um_campo_linear() {
    let mut c = grelha(4);
    // `w0 = x`, `w1 = y`, `w2 = 1 − x − y` — três campos lineares que somam 1.
    for (v, p) in c.malha.rest.clone().iter().enumerate() {
        c.pesos[v * 3] = p[0];
        c.pesos[v * 3 + 1] = p[1];
        c.pesos[v * 3 + 2] = 1.0 - p[0] - p[1];
    }
    let suave = CampoSuave::novo(&c).expect("campo válido");
    let mut pior = 0.0_f64;
    for i in 1..40 {
        for j in 1..40 {
            let p = [f64::from(i) / 40.0, f64::from(j) / 40.0];
            let Some(l) = suave.linha(p) else { continue };
            pior = pior.max((l[0] - p[0]).abs()).max((l[1] - p[1]).abs());
        }
    }
    assert!(
        pior < 1e-9,
        "um campo linear não foi reproduzido: erro {pior:e}"
    );
}

/// ⚠️ **SONDA — o salto da derivada em função da LARGURA da sonda.**
///
/// ⛔⛔ **É ela que separa `C¹` de `quase C¹`, e o valor sozinho não separa nada:** uma sonda de
/// largura `δ` sobre um campo **liso** lê uma diferença que **desaparece com `δ`** (é a 2.ª
/// derivada vezes `δ`); sobre um campo com um degrau ela lê **o degrau, constante**. ⇒ o veredito
/// é a COLUNA e não a célula.
#[test]
fn diag_o_salto_em_funcao_da_largura_da_sonda() {
    let c = grelha(4);
    let suave = CampoSuave::novo(&c).expect("campo válido");
    println!("\n{:=<64}", "");
    println!("SONDA · o salto da derivada quando a sonda ESTREITA");
    println!("  C¹ ⇒ desaparece com δ · C⁰ ⇒ fica no degrau");
    println!("{:=<64}", "");
    println!("{:>10} | {:>14} | {:>14}", "δ", "C¹", "LINEAR");
    let arestax = 0.1;
    for e in [1e-2_f64, 3e-3, 1e-3, 3e-4, 1e-4] {
        let deriva = |x: f64, c1: bool| -> f64 {
            let ler = |q: f64| -> f64 {
                let p = [q, 0.1];
                if c1 {
                    suave.linha(p).expect("dentro")[0]
                } else {
                    c.linha(p).expect("dentro")[0]
                }
            };
            (ler(x + e * 0.1) - ler(x - e * 0.1)) / (2.0 * e * 0.1)
        };
        let salto = |c1: bool| (deriva(arestax + e, c1) - deriva(arestax - e, c1)).abs();
        println!(
            "{e:>10.0e} | {:>14.4e} | {:>14.4e}",
            salto(true),
            salto(false)
        );
    }
    println!("{:=<64}", "");
}
