//! ⭐⭐⭐ **OS GATES DA MALHA DO DOMÍNIO** — a cerca de cobertura e a régua que a decidiu.
//!
//! # ⛔⛔ O defeito que estes gates fecham (medido 2026-09-20)
//!
//! A cerca da célula era **cinco amostras** (o centro e os quatro cantos, com folga). Isso é um
//! sorteio: uma ponta fina ou um entalhe estreito atravessa a célula **sem** que nenhuma das cinco
//! caia dentro, e a célula é descartada com o desenho a passar por ela.
//!
//! | forma | amostras da curva SEM malha, cerca de 5 pontos | com a fronteira |
//! |---|---:|---:|
//! | `Rectangle 40×10` | `0,00 %` | `0,00 %` |
//! | `Ellipse 40×10` | `0,00 %` | `0,00 %` |
//! | `Polygon 40×40` | `4,10 %` | **`0,00 %`** |
//! | `Star 40×40` | **`19,23 %`** | **`0,00 %`** |
//!
//! ⭐ E não custa triângulos: a estrela desce de `654` para `608`, o polígono de `1 018` para
//! `1 016`. *O que faltava não era resolução, era a pergunta.*
//!
//! ⚠️ **Isto é anterior a amostrar a curva:** o [`super::pesos_do_caminho`] já resolve os pontos de
//! controlo contra esta malha, e um que caia fora herda o vértice mais próximo — é a queixa que ele
//! imprime. *Uma estrela já era resolvida contra uma malha que não a cobria.*

use super::*;
use ph2d_vec_scene::{ShapeKind, cook};

/// Dois ossos deitados ao longo do eixo maior — o arranjo de um membro.
fn ossos_do_membro(w: f64, h: f64) -> Vec<Handle> {
    vec![
        Handle {
            a: [0.0, h * 0.5],
            b: [w * 0.5, h * 0.5],
        },
        Handle {
            a: [w * 0.5, h * 0.5],
            b: [w, h * 0.5],
        },
    ]
}

/// Quantas amostras da curva DESENHADA caem fora da malha do domínio, e quantas ao todo.
///
/// ⚠️ **A curva e não os nós.** Os nós de uma estrela são as pontas e o vale — eles caem dentro por
/// construção; quem fica sem malha é o desenho **entre** eles, que é exactamente o que a lei da
/// curva vai consultar.
fn cobertura(kind: ShapeKind, size: [f64; 2]) -> (usize, usize) {
    let path = cook(kind, [0.0, 0.0], size, &[]);
    let aneis = contornos_fechados(&path);
    assert!(!aneis.is_empty(), "a fixtura tem de ter contorno fechado");
    let ossos = ossos_do_membro(size[0], size[1]);
    let (malha, regua) = malha_do_dominio_com_regua(&aneis, &ossos).expect("domínio");
    let para_malha = |p: [f64; 2]| [(p[0] - regua[0]) * regua[2], (p[1] - regua[1]) * regua[2]];
    // O campo é irrelevante aqui — o que se mede é se o ponto ACERTA um triângulo.
    let campo: Vec<f64> = vec![0.5; malha.rest.len() * 2];
    let cozido = path.cooked();
    let (mut total, mut fora) = (0usize, 0usize);
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let nv = verts.len();
        let ultimo = if fechado { nv } else { nv - 1 };
        for i in 0..ultimo {
            let (a, b) = (&verts[i], &verts[(i + 1) % nv]);
            const M: usize = 64;
            for k in 0..=M {
                let t = k as f64 / f64::from(M as u32);
                let u = 1.0 - t;
                let (c0, c1, c2, c3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                let p = [
                    c0 * a.anchor[0]
                        + c1 * a.out_handle[0]
                        + c2 * b.in_handle[0]
                        + c3 * b.anchor[0],
                    c0 * a.anchor[1]
                        + c1 * a.out_handle[1]
                        + c2 * b.in_handle[1]
                        + c3 * b.anchor[1],
                ];
                total += 1;
                if amostra_achatada(&malha, &campo, para_malha(p), 2).is_none() {
                    fora += 1;
                }
            }
        }
    }
    (total, fora)
}

/// ⭐⭐⭐ **A MALHA COBRE A ARTE** — nenhuma amostra da curva desenhada fica sem domínio.
///
/// ⚠️ **O piso de população é metade do gate.** Sem ele, uma forma que deixasse de ter contorno
/// (ou uma régua partida que varresse zero segmentos) leria `0` fora de `0` e passaria a afirmar
/// nada — que é como um censo fica verde a medir o vazio.
#[test]
fn a_malha_cobre_toda_a_curva_desenhada() {
    for (nome, kind, size) in [
        ("Rectangle", ShapeKind::Rectangle, [40.0, 10.0]),
        ("Ellipse", ShapeKind::Ellipse, [40.0, 10.0]),
        ("Polygon", ShapeKind::Polygon, [40.0, 40.0]),
        ("Star", ShapeKind::Star, [40.0, 40.0]),
    ] {
        let (total, fora) = cobertura(kind, size);
        assert!(
            total >= 150,
            "{nome}: a régua varreu {total} amostras — poucas para afirmar seja o que for"
        );
        assert_eq!(
            fora, 0,
            "{nome}: {fora} de {total} amostras da curva caem FORA da malha do domínio — a cerca \
             da célula deixou de cobrir a arte"
        );
    }
}

/// ⭐⭐ **O CONTROLO da régua acima** — ela SABE ver uma amostra fora.
///
/// ⛔ Sem isto, o gate irmão poderia estar verde por a `amostra` devolver `Some` para tudo (um
/// `u`/`v` sem cerca, por exemplo), e ninguém notaria. Aqui pergunta-se por um ponto que está
/// **comprovadamente** fora do domínio — bem longe da caixa — e exige-se `None`.
#[test]
fn a_regua_da_cobertura_sabe_ver_um_ponto_de_fora() {
    let size = [40.0, 40.0];
    let path = cook(ShapeKind::Star, [0.0, 0.0], size, &[]);
    let aneis = contornos_fechados(&path);
    let ossos = ossos_do_membro(size[0], size[1]);
    let (malha, regua) = malha_do_dominio_com_regua(&aneis, &ossos).expect("domínio");
    let para_malha = |p: [f64; 2]| [(p[0] - regua[0]) * regua[2], (p[1] - regua[1]) * regua[2]];
    let campo: Vec<f64> = vec![0.5; malha.rest.len() * 2];
    let longe = para_malha([size[0] * 10.0, size[1] * 10.0]);
    assert!(
        amostra_achatada(&malha, &campo, longe, 2).is_none(),
        "a régua devolveu pesos para um ponto a dez larguras da forma — ela não discrimina nada"
    );
}

/// ⭐ **A cerca da fronteira NÃO paga o que cobre.** O orçamento de triângulos redistribui-se.
///
/// ⚠️ A barra é o próprio orçamento (`ALVO_DE_TRIANGULOS`) com folga, e não um número medido de uma
/// corrida: *uma barra que fosse a contagem de hoje reprovaria no dia em que a grelha graduasse
/// melhor, sobre uma malha melhor.*
#[test]
fn cobrir_a_fronteira_nao_estoura_o_orcamento_de_triangulos() {
    for (nome, kind, size) in [
        ("Star", ShapeKind::Star, [40.0, 40.0]),
        ("Polygon", ShapeKind::Polygon, [40.0, 40.0]),
    ] {
        let path = cook(kind, [0.0, 0.0], size, &[]);
        let aneis = contornos_fechados(&path);
        let ossos = ossos_do_membro(size[0], size[1]);
        let (malha, _) = malha_do_dominio_com_regua(&aneis, &ossos).expect("domínio");
        assert!(
            malha.tris.len() <= ALVO_DE_TRIANGULOS * 2,
            "{nome}: {} triângulos contra um orçamento de {ALVO_DE_TRIANGULOS}",
            malha.tris.len()
        );
        assert!(
            malha.tris.len() >= 100,
            "{nome}: {} triângulos — malha degenerada, o gate irmão passaria por vácuo",
            malha.tris.len()
        );
    }
}

/// ⭐⭐ **A cerca do segmento contra a célula, nos casos que a decidem.**
///
/// ⚠️ O caso que importa é o **terceiro**: um segmento que atravessa a célula de lado a lado não
/// tem nenhum extremo dentro dela, e é exactamente esse que as cinco amostras perdiam.
#[test]
fn o_segmento_toca_a_celula_mesmo_sem_extremo_dentro() {
    let (x0, y0, x1, y1) = (0.0, 0.0, 10.0, 10.0);
    assert!(
        cruza_a_celula([5.0, 5.0], [50.0, 50.0], x0, y0, x1, y1),
        "um extremo DENTRO tem de tocar"
    );
    // ⚠️ **Este caso nasceu de uma MUTAÇÃO SOBREVIVENTE.** Apagar o teste do extremo dentro
    // deixava os outros três verdes, porque um segmento com UM extremo dentro e outro fora
    // **cruza sempre um lado** — a cerca do extremo só é a única testemunha quando o segmento
    // cabe INTEIRO na célula, que é o que um contorno fino sobre uma grelha grossa produz.
    assert!(
        cruza_a_celula([3.0, 3.0], [7.0, 7.0], x0, y0, x1, y1),
        "um segmento INTEIRO dentro da célula não cruza lado nenhum — só a cerca do extremo o vê"
    );
    assert!(
        cruza_a_celula([-5.0, 5.0], [15.0, 5.0], x0, y0, x1, y1),
        "atravessar de lado a lado SEM extremo dentro é o caso que as cinco amostras perdiam"
    );
    assert!(
        !cruza_a_celula([20.0, 20.0], [30.0, 30.0], x0, y0, x1, y1),
        "longe da célula não pode tocar"
    );
    assert!(
        !cruza_a_celula([-5.0, 20.0], [15.0, 20.0], x0, y0, x1, y1),
        "⛔ passar AO LADO (a caixa do segmento não cruza a da célula) não é tocar"
    );
}
