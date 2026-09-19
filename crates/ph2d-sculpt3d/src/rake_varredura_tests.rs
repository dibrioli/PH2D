//! **A VARREDURA DO VIÉS — o INSTRUMENTO que escolheu as constantes do pente.**
//!
//! Filho (`#[path]`, `cfg(test)`) do [`super`], e o corte é o SUJEITO: o pai
//! julga a LEI (as três barras que o produto tem de passar) e aqui vive a sonda
//! que varre os números para os escolher.
//!
//! ⛔⛔ **Ela NÃO é o produto, e é por isso que ela mora à parte:** os dois `k` e
//! a escala são LIVRES aqui, e no produto eles saem do
//! [`ph2d_rake::VIES_DA_GRADE`] vezes o botão. *Uma sonda que escolhe os
//! próprios números não pode ser a régua de quem os escolheu* — quem julga é o
//! [`super::a_nossa_malha_penteia_se_acima_da_barra_do_oraculo`], que chama o
//! [`super::traco`] e este crava a lei.
//!
//! ⚠️⚠️ **E ela mentiu DUAS vezes antes de dizer a verdade**, as duas por não
//! reproduzir uma cerca do produto: o ramo livre ignorava a FORÇA (⇒ as células
//! «colapso NU» e «colapso ENVIESADO» liam o mesmo número, e a varredura dizia
//! que aquela metade não tinha efeito) e não honrava a regra do PRIMEIRO
//! CARIMBO (⇒ lia `Q 0,0598` onde o produto lê `0,0435`, e escolhia a constante
//! da lei com essa leitura). *Toda cerca do produto tem de estar na sonda.*

use super::{Colapso, Vies, ph2d_sculpt3d_ang, ph2d_sculpt3d_q, traco_com};

/// ⭐⭐⭐ **A VARREDURA DO VIÉS — o instrumento que escolhe o número.**
///
/// Varre o `k` do refino e o do colapso, **incluindo o sinal**, e imprime o `Q`
/// e a contagem de vértices. ⚠️ A contagem está ali de propósito: um `k` que
/// compra `Q` dobrando a malha não é uma grade, é um refino descontrolado — e
/// *uma coluna de `Q` sozinha não distingue as duas coisas*.
#[test]
#[ignore = "sonda: a varredura do vies"]
fn diag_a_varredura_do_vies() {
    for (rotulo, colapso) in [
        ("colapso CONSERVADOR", Colapso::Conservador),
        ("colapso ENVIESADO", Colapso::Enviesado),
        ("colapso NU", Colapso::Nu),
    ] {
        let (m0, c0) = traco_com(
            0.0,
            true,
            Vies {
                livre: Some((0.0, 1.0)),
                colapso,
            },
        );
        let (q0, n0) = ph2d_sculpt3d_q(&m0, &c0, 0.30);
        println!(
            "{rotulo}  ·  DESLIGADO: Q {q0:+.4} (arestas={n0}, V={})",
            m0.vert_count()
        );
        // ⛔⛔ **A coluna `pior°` é a SEGUNDA RÉGUA, e sem ela esta varredura
        // escolheria um número que destrói a malha:** o `Q` sozinho aprova uma
        // malha em lascas que por acaso ficou alinhada — medido nesta linha, a
        // lei refutada de 17/09 levava o pior triângulo a `0,31°` **enquanto o
        // `Q` subia**. O chão é `2°` (gate `o_pente_nao_compra_alinhamento_com_lascas`).
        println!(
            "{:>8} {:>9} {:>9} {:>8} {:>7}",
            "k", "Q", "arestas", "V", "pior°"
        );
        for k in [0.0f32, 0.45, 0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80] {
            let (m, c) = traco_com(
                1.0,
                true,
                Vies {
                    livre: Some((k, 1.0 / (1.0 - k))),
                    colapso,
                },
            );
            let (q, n) = ph2d_sculpt3d_q(&m, &c, 0.30);
            let (pior, _) = ph2d_sculpt3d_ang(&m, &c, 0.30);
            println!("{k:>8.2} {q:>9.4} {n:>9} {:>8} {pior:>7.2}", m.vert_count());
        }
        println!();
    }

    // ⛔⛔⛔ **O CONTROLO — e sem ele a tabela acima está CONFUNDIDA.** O viés
    // compra `Q` **e** adensa a malha; esta metade adensa **sem enviesar**, pela
    // escala isotrópica, e mede o `Q` nas mesmas contagens de vértices. *Se uma
    // malha só mais fina lesse o mesmo `Q`, o que a H2 compra seria densidade.*
    println!("CONTROLO isotropico (k = 0, so' o alvo a encolher):");
    println!("{:>8} {:>9} {:>9} {:>8}", "escala", "Q", "arestas", "V");
    for escala in [1.0f32, 0.85, 0.72, 0.60, 0.45, 0.30] {
        let (m, c) = traco_com(
            1.0,
            true,
            Vies {
                livre: Some((0.0, escala)),
                colapso: Colapso::Enviesado,
            },
        );
        let (q, n) = ph2d_sculpt3d_q(&m, &c, 0.30);
        println!("{escala:>8.2} {q:>9.4} {n:>9} {:>8}", m.vert_count());
    }

    // ⭐⭐⭐ **AS DUAS DIMENSÕES JUNTAS, que é a única leitura honesta:** o viés
    // compra `Q` e a normalização devolve densidade, e as duas mexem no mesmo
    // campo. A coluna `V` tem o baseline ao lado (`k = 0` lê `6 759`), e a
    // `pior°` é a segunda régua — *um `k` que compra `Q` em lascas não é grade*.
    //
    // ⚠️ O colapso é sempre o CONSERVADOR (a `Porta::Colapso` do produto): a
    // alternativa está medida acima e lê `1,96°` no gate das lascas.
    println!();
    println!("k x normalizacao do REFINO  (colapso conservador; V base = 6759)");
    println!(
        "{:>6} {:>10} {:>8} {:>9} {:>8} {:>7}",
        "k", "norm", "escala", "Q", "V", "pior°"
    );
    for k in [0.45f32, 0.55, 0.65, 0.75] {
        for (nome, escala) in [
            ("crua", 1.0f32),
            ("densidade", (1.0 - k * k).powf(-0.75)),
            ("minimo", 1.0 / (1.0 - k)),
        ] {
            let (m, c) = traco_com(
                1.0,
                true,
                Vies {
                    livre: Some((k, escala)),
                    colapso: Colapso::Conservador,
                },
            );
            let (q, n) = ph2d_sculpt3d_q(&m, &c, 0.30);
            let (pior, _) = ph2d_sculpt3d_ang(&m, &c, 0.30);
            let _ = n;
            println!(
                "{k:>6.2} {nome:>10} {escala:>8.3} {q:>9.4} {:>8} {pior:>7.2}",
                m.vert_count()
            );
        }
    }
}
