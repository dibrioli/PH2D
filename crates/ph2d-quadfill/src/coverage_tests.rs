//! ⭐⭐⭐ **OS GATES DA COBERTURA** — e o que cada um tem de provar é que a régua ANTIGA fica
//! **verde** sobre a mesma fixtura. *Uma régua nova que só concorda com as que já existiam não
//! compra nada.*

use super::{COVERAGE_DEFECT, COVERAGE_SHELL, coverage};
use ph2d_mesh::{Face, Mesh};

/// Uma pirâmide de base quadrada `2 × 2` com o ápice em `y = altura`.
///
/// ⚠️ **Os vértices do TOPO CORTADO caem exactamente sobre as arestas da inteira** (a `y = 2` a
/// meia-largura é `0,5`), e é isso que faz a direcção inversa medir **zero** — que é o defeito
/// desta família.
fn piramide(altura: f32) -> Mesh {
    Mesh::from_parts(
        vec![
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [0.0, altura, 0.0],
        ],
        vec![
            Face::quad(0, 1, 2, 3),
            Face::tri(0, 1, 4),
            Face::tri(1, 2, 4),
            Face::tri(2, 3, 4),
            Face::tri(3, 0, 4),
        ],
    )
    .expect("a fixtura e' construida aqui")
}

/// A mesma pirâmide com a ponta **cortada** a `y = 2` e tapada — a amputação.
fn piramide_amputada() -> Mesh {
    Mesh::from_parts(
        vec![
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [0.5, 2.0, 0.5],
            [-0.5, 2.0, 0.5],
            [-0.5, 2.0, -0.5],
            [0.5, 2.0, -0.5],
        ],
        vec![
            Face::quad(0, 1, 2, 3),
            Face::quad(7, 6, 5, 4),
            Face::quad(0, 1, 5, 4),
            Face::quad(1, 2, 6, 5),
            Face::quad(2, 3, 7, 6),
            Face::quad(3, 0, 4, 7),
        ],
    )
    .expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **O GATE DA LINHA — a ponta amputada acusa numa direcção e é INVISÍVEL na outra.**
///
/// ⛔⛔ É a razão de este ficheiro existir. A direcção que os dois lados já medem —
/// *«a malha nova está pousada na escultura?»* — dá **zero** sobre uma peça com a ponta comida,
/// porque **todos** os vértices da saída estão de facto sobre a entrada.
#[test]
fn uma_ponta_amputada_acusa_numa_direccao_e_e_invisivel_na_outra() {
    let inteira = piramide(4.0);
    let cortada = piramide_amputada();

    // ⛔ O CONTROLE: a direcção que toda a gente mede diz que está perfeito.
    let inversa = coverage(&cortada, &inteira);
    assert!(inversa.measured(), "⛔ o controle tem de ter medido");
    assert!(
        inversa.worst < 1e-3,
        "⛔ a direcção saida->entrada TEM de dar zero aqui, senao esta fixtura nao demonstra a \
         cegueira que motiva a regua: {:?}",
        inversa
    );

    // ⭐⭐ E a direcção certa acusa, e acusa NA CASCA.
    let c = coverage(&inteira, &cortada);
    assert!(c.measured());
    assert!(
        c.shell_samples > 0,
        "⛔ o apice tem de cair na casca exterior"
    );
    assert!(
        c.shell_worst > 0.2,
        "⛔ o apice esta' 2 unidades acima do corte numa peca de diagonal 4,9 -- esperado ~41 %, \
         medido {:.3}",
        c.shell_worst
    );
    assert!(
        c.shell_is_defective(),
        "⛔ e tem de passar a barra: {} vs {COVERAGE_DEFECT}",
        c.shell_p50
    );
}

/// ⭐⭐⭐ **GATE — a distância é ao TRIÂNGULO, não ao vértice mais próximo.**
///
/// ⚠️ Amostrar a saída por vértices é o atalho fácil e **sobre-estima**. A fixtura separa os dois:
/// a entrada paira a `1` sobre o INTERIOR de um quadrado grande, e o vértice mais próximo dele
/// está a `√3 ≈ 1,73`.
#[test]
fn a_distancia_e_a_superficie_e_nao_ao_vertice_mais_proximo() {
    let chao = Mesh::from_parts(
        vec![
            [5.0, 0.0, 5.0],
            [-5.0, 0.0, 5.0],
            [-5.0, 0.0, -5.0],
            [5.0, 0.0, -5.0],
        ],
        vec![Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");
    // ⭐ Um triângulo a `y = 1`, com os vértices a `4` do centro: o vértice mais próximo do chão
    // (a `5`) fica a `√(1 + 1 + 1)`; a superfície fica a `1`.
    let voando = Mesh::from_parts(
        vec![[4.0, 1.0, 4.0], [-4.0, 1.0, 4.0], [0.0, 1.0, -4.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("a fixtura e' construida aqui");

    let c = coverage(&voando, &chao);
    let diag = 8.0f32.mul_add(8.0, 8.0 * 8.0).sqrt(); // ⚠️ a caixa da ENTRADA, que é o denominador
    let esperado = 1.0 / diag;
    assert!(
        (c.worst - esperado).abs() < 1e-3,
        "⛔ esperado {esperado:.4} (a superficie), medido {:.4}; o vertice mais proximo daria \
         {:.4}",
        c.worst,
        3.0f32.sqrt() / diag
    );
}

/// ⭐⭐⭐ **GATE — uma saída IDÊNTICA cobre tudo, e o zero é REAL.**
///
/// ⚠️ Sem este par, um `coverage` que devolvesse `0` por não medir nada passaria em todos os
/// outros gates que só olham o valor.
#[test]
fn uma_saida_identica_cobre_tudo_e_a_contagem_prova_que_mediu() {
    let m = piramide(4.0);
    let c = coverage(&m, &m);
    assert_eq!(c.samples, 5, "⛔ tem de medir os CINCO vertices da entrada");
    assert!(c.worst < 1e-6, "⛔ a peca cobre-se a si propria: {:?}", c);
    assert!(!c.shell_is_defective());
}

/// ⭐⭐⭐ **GATE — entrada degenerada devolve NÃO MEDIDO, e não «perfeito».**
///
/// ⛔ *Um zero de «não medido» e um de «perfeito» são o mesmo byte.* Esta linha pagou-o três
/// vezes, e a terceira foi nesta mesma crate.
#[test]
fn a_regua_recusa_em_vez_de_inventar_um_zero() {
    let vazia = Mesh::from_parts(vec![], vec![]).expect("a fixtura e' construida aqui");
    let m = piramide(4.0);

    for (a, b, porque) in [
        (&vazia, &m, "entrada vazia"),
        (&m, &vazia, "saida vazia"),
        (&vazia, &vazia, "as duas vazias"),
    ] {
        let c = coverage(a, b);
        assert_eq!(c.samples, 0, "⛔ {porque}: tem de dizer NAO MEDIDO");
        assert!(
            !c.measured() && !c.shell_is_defective(),
            "⛔ {porque}: e nao pode ler-se como aprovado"
        );
    }
    // ⭐ Uma peça sem extensão nenhuma também não tem denominador.
    let ponto = Mesh::from_parts(
        vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("a fixtura e' construida aqui");
    assert_eq!(
        coverage(&ponto, &m).samples,
        0,
        "⛔ diagonal zero: nao ha' denominador"
    );
}

/// ⭐⭐⭐ **GATE — a CASCA separa o defeito da ponta do defeito do corpo.**
///
/// ⚠️ Sem esta separação o número global não se move: numa peça com muitos vértices, duas pontas
/// comidas não mexem uma mediana de milhares — *é a mesma cegueira que o `edge_max` global e o `χ`
/// já cobraram a esta linha, uma régua de cada vez.*
#[test]
fn a_casca_separa_a_ponta_do_corpo() {
    let inteira = piramide(4.0);
    let cortada = piramide_amputada();
    let c = coverage(&inteira, &cortada);

    assert_eq!(
        c.shell_samples, 1,
        "⛔ so' o apice esta' acima de {COVERAGE_SHELL} do raio maximo"
    );
    assert!(
        c.p50 < COVERAGE_DEFECT,
        "⛔ a MEDIANA da peca inteira tem de continuar limpa -- e' isso que torna a casca \
         necessaria: {:.4}",
        c.p50
    );
    assert!(
        c.shell_p50 > 10.0 * c.p50,
        "⛔ e a casca tem de ser uma ordem de grandeza pior: {:.4} contra {:.4}",
        c.shell_p50,
        c.p50
    );
}
