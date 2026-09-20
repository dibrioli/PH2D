//! Os gates da âncora — a lei é PURA (duas listas de pontos e um cursor), logo não precisa de mundo
//! nenhum. ⚠️ É por isso que ela vive aqui e não dentro do gesto: *uma lei que só se mede montando
//! um esqueleto acaba sem gate*.

use super::{Ancora, no_contorno};

/// As triplas de um rectângulo de `(x0,y0)` a `(x1,y1)`, com as alças nos TERÇOS de cada aresta —
/// que é o que o cozinhado desta casa produz para uma recta.
fn caixa(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    let cantos = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
    let mut out = Vec::new();
    for k in 0..4 {
        let a = cantos[k];
        let ant = cantos[(k + 3) % 4];
        let seg = cantos[(k + 1) % 4];
        let terco = |p: [f64; 2], q: [f64; 2]| {
            [
                (2.0_f64 / 3.0).mul_add(p[0], q[0] / 3.0),
                (2.0_f64 / 3.0).mul_add(p[1], q[1] / 3.0),
            ]
        };
        out.push(a);
        out.push(terco(a, ant));
        out.push(terco(a, seg));
    }
    out
}

/// ⭐⭐⭐ **A MANCHA POUSA ENTRE OS NÓS — a lei que a F30 precisava e não tinha.**
///
/// ⛔⛔ Até 2026-09-19 o gesto ancorava no NÓ mais perto: numa barra com os oito nós nas duas
/// pontas, o meio ficava a `3,04` de qualquer um deles e o pincel recusava. *A lei da curva movia
/// a arte e nenhum gesto conseguia pedir-lho.*
#[test]
fn a_ancora_pousa_no_meio_da_aresta_e_nao_no_no() {
    let r = caixa(0.0, 0.0, 40.0, 10.0);
    let a = no_contorno(&r, &r, [20.0, 0.0]).expect("a caixa tem contorno");
    assert!(
        (a.repouso[0] - 20.0).abs() < 1e-9 && a.repouso[1].abs() < 1e-9,
        "a ancora pousou em {:?} e nao no meio da aresta de baixo",
        a.repouso
    );
    // ⚠️ **O CONTROLO que torna a medição legível:** o nó mais perto está a `20`, logo um gate que
    // só exigisse «perto do dedo» passaria com a lei antiga se a fixtura fosse pequena.
    let no_mais_perto = r
        .iter()
        .step_by(3)
        .map(|p| (p[0] - 20.0).hypot(p[1]))
        .fold(f64::INFINITY, f64::min);
    assert!(
        no_mais_perto > 10.0,
        "a fixtura tem um no' a {no_mais_perto} do dedo — ela nao distingue as duas leis"
    );
}

/// ⭐⭐⭐ **O REPOUSO SAI DO MESMO PARÂMETRO DA CURVA, e não do cursor.**
///
/// Com a forma posada — aqui, o dobro de largura —, o dedo aponta para `x = 40` do que se VÊ, que é
/// o meio da aresta; no repouso isso é `x = 20`. ⚠️ *Guardar o cursor cru poria a correcção no
/// sítio errado, e ela andaria com a pose no quadro seguinte.*
#[test]
fn o_repouso_sai_do_mesmo_parametro_e_nao_do_cursor() {
    let repouso = caixa(0.0, 0.0, 40.0, 10.0);
    let posado: Vec<[f64; 2]> = repouso.iter().map(|p| [p[0] * 2.0, p[1]]).collect();
    let dedo = [40.0, 0.0];
    let a = no_contorno(&repouso, &posado, dedo).expect("contorno");
    assert!(
        (a.mundo[0] - 40.0).abs() < 1e-9,
        "o ponto posado devia estar debaixo do dedo, e leu {:?}",
        a.mundo
    );
    assert!(
        (a.repouso[0] - 20.0).abs() < 1e-9,
        "o repouso devia ser o MEIO da aresta ({:?} lido) — guardar o cursor daria 40",
        a.repouso
    );
}

/// ⭐ **Uma forma sem contorno não tem parâmetro a traduzir** — e quem chama cai no outro caminho.
#[test]
fn sem_contorno_nao_ha_ancora() {
    assert_eq!(no_contorno(&[], &[], [0.0, 0.0]), None);
    let um = vec![[0.0, 0.0]; 3];
    assert_eq!(no_contorno(&um, &um, [0.0, 0.0]), None);
}

/// ⚠️ **As duas listas têm de ser a MESMA forma** — comprimentos diferentes são um erro de quem
/// chama, e devolver uma âncora ali daria um repouso tirado de outro ponto da curva.
#[test]
fn duas_poses_de_tamanhos_diferentes_nao_produzem_ancora() {
    let r = caixa(0.0, 0.0, 40.0, 10.0);
    let p = caixa(0.0, 0.0, 40.0, 10.0)[..9].to_vec();
    assert_eq!(no_contorno(&r, &p, [20.0, 0.0]), None);
}

/// ⭐⭐ **A âncora é o ponto mais perto do CONTORNO INTEIRO**, e não da primeira aresta que serve.
#[test]
fn a_ancora_escolhe_a_aresta_mais_perto() {
    let r = caixa(0.0, 0.0, 40.0, 10.0);
    for (dedo, esperado, qual) in [
        ([20.0, -3.0], [20.0, 0.0], "a de baixo"),
        ([20.0, 13.0], [20.0, 10.0], "a de cima"),
        ([-3.0, 5.0], [0.0, 5.0], "a da esquerda"),
        ([43.0, 5.0], [40.0, 5.0], "a da direita"),
        // ⛔⛔ **O dedo LONGE, que é o que prende a fracção a `[0,1]`** — esta célula nasceu de uma
        // mutação SOBREVIVENTE. Sem a prisão, o cursor projecta-se **para lá do fim** da corda de
        // baixo, em `(100, 0)`, e essa distância (`5`) ganha da verdadeira (`60`): a mancha
        // pousaria a `60` unidades da arte, **fora da peça**. *Um `clamp` não é defensivo — é a
        // diferença entre projectar numa CORDA e projectar na RECTA que a contém.*
        ([100.0, 5.0], [40.0, 5.0], "a da direita, com o dedo longe"),
    ] {
        let a: Ancora = no_contorno(&r, &r, dedo).expect("contorno");
        let erro = (a.repouso[0] - esperado[0]).hypot(a.repouso[1] - esperado[1]);
        assert!(
            erro < 1e-9,
            "dedo {dedo:?}: esperava {qual} e leu {:?}",
            a.repouso
        );
    }
}
