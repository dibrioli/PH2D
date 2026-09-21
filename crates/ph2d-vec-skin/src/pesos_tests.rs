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
                if amostra_achatada(&malha, &campo, para_malha(p), 2, None).is_none() {
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
        amostra_achatada(&malha, &campo, longe, 2, None).is_none(),
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

/// ⭐⭐⭐ **GATE — O ÍNDICE DÁ A MESMA RESPOSTA QUE A VARREDURA, e a grelha não é uma célula só.**
///
/// O [`super::IndiceDoCampo`] existe para tirar a consulta de `O(triângulos)` — e a única coisa
/// que ele não pode fazer é mudar a resposta.
///
/// # As três metades
///
/// 1. **A resposta é a mesma AO BIT** em toda a caixa da malha, dentro e fora — medido, `0e0`
///    sobre `229` amostras interiores. ⚠️ Em teoria ela podia diferir no arredondamento: num
///    ponto sobre uma ARESTA os dois caminhos escolhem triângulos diferentes (a varredura toma o
///    primeiro na ordem de `tris`, o índice o primeiro do balde) e os dois estão certos. *A
///    medição diz que aqui isso não acontece, e por isso a barra é a mais apertada que há.*
/// 2. **Ele responde `Some` onde a varredura responde `Some`** — um índice que perdesse o
///    triângulo devolveria `None` e a lei cairia na mistura, que é arte errada em silêncio.
/// 3. ⛔ **A grelha tem de DIVIDIR** — sem esta metade um índice de uma célula só passa nas duas
///    primeiras e não indexa nada. A régua é o maior balde contra o total de triângulos.
#[test]
fn o_indice_da_a_mesma_resposta_que_a_varredura() {
    let size = [200.0_f64, 80.0];
    let path = cook(ShapeKind::Star, [0.0, 0.0], size, &[]);
    let aneis = contornos_fechados(&path);
    let ossos = ossos_do_membro(size[0], size[1]);
    let (malha, _) = malha_do_dominio_com_regua(&aneis, &ossos).expect("domínio");
    // ⚠️ Pesos DISTINTOS por vértice: um campo constante faria a metade da igualdade passar mesmo
    // que o índice escolhesse outro triângulo — a régua tem de conseguir ver a diferença.
    #[expect(clippy::cast_precision_loss, reason = "índice de vértice")]
    let campo: Vec<f64> = (0..malha.rest.len())
        .flat_map(|i| {
            let t = (i as f64 * 0.37).fract();
            [t, 1.0 - t]
        })
        .collect();
    let idx = super::IndiceDoCampo::novo(&malha).expect("índice");
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &malha.rest {
        lo = [lo[0].min(v[0]), lo[1].min(v[1])];
        hi = [hi[0].max(v[0]), hi[1].max(v[1])];
    }
    // Uma grelha de amostras que TRANSBORDA a caixa em 20 % de cada lado.
    let (mut dentro, mut pior) = (0usize, 0.0_f64);
    const N: usize = 60;
    for iy in 0..=N {
        for ix in 0..=N {
            let f = |i: usize, a: f64, b: f64| {
                #[expect(clippy::cast_precision_loss, reason = "i <= N")]
                let t = i as f64 / N as f64;
                (b - a).mul_add(0.2f64.mul_add(1.4, t * 1.4) - 0.2, a)
            };
            // ⚠️⚠️ **As amostras vivem em coordenadas de MALHA**, que é o espaço da `lo`/`hi` e o
            // que a [`super::amostra_achatada`] recebe. A 1.ª redacção passava-as pela `regua`
            // outra vez e lia **`27` de `3 721`** dentro — *uma conversão aplicada duas vezes lê-se
            // como uma fixtura sem fenómeno*, e foi o piso de população que a apanhou.
            let pm = [f(ix, lo[0], hi[0]), f(iy, lo[1], hi[1])];
            let p = pm;
            let varre = super::amostra_achatada(&malha, &campo, pm, 2, None);
            let com = super::amostra_achatada(&malha, &campo, pm, 2, Some(&idx));
            assert_eq!(
                varre.is_some(),
                com.is_some(),
                "em {p:?} a varredura diz {:?} e o índice diz {:?} — um índice que PERDE o \
                 triângulo faz a lei cair na mistura, e isso é arte errada em silêncio",
                varre.is_some(),
                com.is_some()
            );
            if let (Some(a), Some(b)) = (varre, com) {
                dentro += 1;
                for (x, y) in a.iter().zip(&b) {
                    pior = pior.max((x - y).abs());
                }
            }
        }
    }
    assert!(
        dentro > 150,
        "só {dentro} amostras caíram DENTRO da malha (medido: 229 de 3 721 — uma estrela enche \
         pouco da caixa dela, e a grelha transborda 40 %) — a fixtura deixou de conter o fenómeno"
    );
    println!("  índice vs varredura: pior diferença de peso {pior:e} em {dentro} amostras");
    // ⭐⭐ **AO BIT, e isso é medição e não esperança:** `0e0` sobre as 229 amostras. ⚠️ Se um dia
    // isto reprovar, o suspeito é um ponto sobre uma ARESTA — ali os dois caminhos podem escolher
    // triângulos diferentes e **os dois estão certos** —, e a cura é afrouxar para `1e-12`,
    // ***nunca*** para mais: acima disso deixa de ser arredondamento e passa a ser outro peso.
    assert_eq!(
        pior, 0.0,
        "o índice mudou a resposta em {pior} — ele pode escolher outro triângulo numa aresta, mas \
         não pode dar outro PESO"
    );
    // (3) A grelha divide mesmo?
    let maior = idx.maior_balde();
    assert!(
        maior * 4 < malha.tris.len(),
        "o maior balde tem {maior} dos {} triângulos — esta grelha não indexa nada, e as duas \
         metades acima passam por vácuo",
        malha.tris.len()
    );
}

/// ⭐⭐⭐ **GATE — A LEI DO PRODUTO DERIVA O ÍNDICE, UMA VEZ POR FORMA.**
///
/// ⛔⛔ **Gate de TEXTO, e a razão está medida:** o índice não muda a resposta (é isso que o irmão
/// [`o_indice_da_a_mesma_resposta_que_a_varredura`] afirma), logo *nenhuma régua de VALOR o vê*.
/// O que ele muda é o RELÓGIO — `0,839 µs` por ponto para `0,077` —, e um gate de relógio nesta
/// casa é um membro da família de flakes de carga. ⇒ o que se afirma é o FIO.
///
/// ⚠️ **E ele exige as duas coisas:** que a lei da curva o derive, e que o derive **fora** do laço
/// dos segmentos — derivá-lo por segmento seria construir a grelha `54` vezes por forma, que é
/// pior do que não a ter.
#[test]
fn a_lei_da_curva_deriva_o_indice_uma_vez_por_forma() {
    let fonte = include_str!("curva.rs");
    let n = fonte.matches("IndiceDoCampo::novo").count();
    assert_eq!(
        n, 2,
        "a [`crate::curva`] deriva o índice {n} vezes e devia derivá-lo DUAS (uma por lei: a que \
         ship e o refit medido) — se for 0, cada amostra voltou a varrer os 878 triângulos e o \
         recook triplica em silêncio"
    );
    // ⛔ FORA do laço: a linha que o deriva não pode estar dentro de um `for` de segmentos.
    for lei in ["pub fn aplica_pela_curva_com(", "pub fn refit_pela_curva("] {
        let i = fonte.find(lei).expect("a lei existe");
        let corpo = &fonte[i..];
        let idx = corpo.find("IndiceDoCampo::novo").expect("deriva o índice");
        let laco = corpo
            .find("for k in 0..segs")
            .expect("o laço dos segmentos");
        assert!(
            idx < laco,
            "em `{lei}` o índice é derivado DENTRO do laço dos segmentos — construir a grelha uma \
             vez por segmento é mais caro do que não a ter"
        );
    }
}
