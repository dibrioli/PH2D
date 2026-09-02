//! Os gates da [`super`] — e **cada um traz o CONTROLO da régua que ele substitui**.
//!
//! ⚠️ *Sem o controlo, «a régua nova acerta» é uma afirmação sobre a régua nova.* A metade
//! que interessa é a outra: que a **antiga** erra na mesma fixtura, e por quanto.

use super::{TIP_DEVIATION_MAX, TIP_GAP_MAX, area_centroid, point_triangle, reach, tip_deviation};
use ph2d_mesh::{Face, Mesh};

/// ⭐⭐⭐ **GATE — a distância é ao INTERIOR da face, não ao canto mais próximo.**
///
/// ⛔⛔ **Ele existe porque uma mutação SOBREVIVEU:** trocar a região interior do
/// ponto-triângulo pela distância ao canto `a` deixou os quatro gates de ponta **verdes**,
/// porque nenhuma fixtura deles põe uma amostra sobre o meio de uma face. *A cadeia real
/// põe: com o quad a `0,10` e a escultura a `0,03`, quase toda amostra cai no interior de
/// um quad da saída.* ⇒ o gate que faltava é o da **propriedade que a função promete**,
/// medida onde ela é definida.
#[test]
fn a_distancia_e_ao_interior_da_face_e_nao_ao_canto() {
    let t = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]];
    // ⚠️ Sobre o baricentro, a `1` de altura: a resposta certa é `1`, e a distância ao
    // canto mais próximo é `√(1 + 4/9 + 4/9) ≈ 1,374`.
    let p = [4.0 / 3.0, 4.0 / 3.0, 1.0];
    let d = point_triangle(p, &t);
    assert!(
        (d - 1.0).abs() < 1.0e-5,
        "a perpendicular vale 1,0 e deu {d}"
    );
    // E as sete regiões continuam a valer: fora de um canto, a resposta é o canto.
    let fora = point_triangle([-3.0, -4.0, 0.0], &t);
    assert!((fora - 5.0).abs() < 1.0e-5, "o canto vale 5,0 e deu {fora}");
    // Fora de uma aresta, a resposta é a projecção nela.
    let aresta = point_triangle([2.0, -2.0, 0.0], &t);
    assert!(
        (aresta - 2.0).abs() < 1.0e-5,
        "a projeccao na aresta vale 2,0 e deu {aresta}"
    );
}

/// O **mesmo rectângulo** `[0,1] × [0,1]`, cortado em `n` colunas iguais.
fn tira(colunas: usize) -> Mesh {
    corte(
        &(0..=colunas)
            .map(|k| {
                #[expect(clippy::cast_precision_loss, reason = "colunas <= 32 nesta fixtura")]
                let x = k as f32 / colunas as f32;
                x
            })
            .collect::<Vec<f32>>(),
    )
}

/// O mesmo rectângulo, mas com os cortes onde o chamador quiser — é isto que permite
/// **amontoar** vértices numa metade sem mudar a forma.
fn corte(xs: &[f32]) -> Mesh {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for &x in xs {
        verts.push([x, 0.0, 0.0]);
        verts.push([x, 1.0, 0.0]);
    }
    for k in 0..xs.len() - 1 {
        let b = u32::try_from(k * 2).expect("a fixtura e' pequena");
        faces.push(Face::quad(b, b + 2, b + 3, b + 1));
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// A régua **VELHA**, aqui só como controlo: o centroide é a média dos vértices.
fn alcance_por_vertice(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    #[expect(clippy::cast_precision_loss, reason = "fixturas pequenas")]
    let n = pos.len().max(1) as f32;
    let mut c = [0.0f32; 3];
    for q in pos {
        for k in 0..3 {
            c[k] += q[k] / n;
        }
    }
    pos.iter().fold(0.0f32, |acc, q| {
        let d = [q[0] - c[0], q[1] - c[1], q[2] - c[2]];
        acc.max(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt())
    })
}

/// ⭐⭐⭐ **GATE — a mesma FORMA amostrada de duas maneiras tem o mesmo alcance.**
///
/// ⛔⛔ **E o controlo é a outra metade:** a régua velha, na **mesma** fixtura, erra por
/// mais de `5 %`. *Uma retopologia redistribui vértices por construção — é literalmente o
/// que ela faz —, logo uma régua ancorada na média dos vértices muda de valor sem que a
/// forma se mexa um micrómetro.*
#[test]
fn a_mesma_forma_amostrada_de_duas_maneiras_tem_o_mesmo_alcance() {
    let uniforme = tira(4);
    // ⚠️ A **mesma** tira, com 16 cortes na metade esquerda e um só na direita.
    let mut xs: Vec<f32> = (0..=16)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "k <= 16")]
            let x = k as f32 / 32.0;
            x
        })
        .collect();
    xs.push(1.0);
    let amontoada = corte(&xs);

    let (a, b) = (reach(&uniforme), reach(&amontoada));
    assert!(
        (a - b).abs() <= 1.0e-4 * a,
        "a mesma forma tem de dar o mesmo alcance: {a} contra {b}"
    );
    let (va, vb) = (
        alcance_por_vertice(&uniforme),
        alcance_por_vertice(&amontoada),
    );
    assert!(
        (va - vb).abs() > 0.05 * va,
        "CONTROLO: a regua velha tinha de errar nesta fixtura, e deu {va} contra {vb}"
    );
}

/// ⚠️ **GATE — uma malha sem área ainda tem alcance.** Uma régua que devolvesse `0` aqui
/// diria *«esta forma não tem tamanho»*, que é falso — e a chave de amputação do selector
/// lê-o como *«nada a defender»*.
#[test]
fn uma_malha_sem_area_cai_para_a_media_dos_vertices() {
    let degenerada = Mesh::from_parts(
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("a fixtura e' construida aqui");
    assert!(
        (reach(&degenerada) - 2.0).abs() < 1.0e-5,
        "o alcance de tres pontos colineares e' 2,0 e deu {}",
        reach(&degenerada)
    );
    let c = area_centroid(&degenerada);
    assert!((c[0] - 2.0).abs() < 1.0e-5, "centroide {c:?}");
}

/// Um cone de eixo `+Z` com quatro anéis **amontoados junto do ápice** — é assim que uma
/// escultura descreve um espinho, e é o que faz a vizinhança do ápice ter população.
///
/// ⚠️ `amputa` corta o bico ao nível de [`CORTE`] e fecha-o com uma tampa chata. *É o
/// defeito exacto que a foto do dono mostra: curto e gordo.*
///
/// ⚠️ **O anel da BASE também é ápice** para a lei de [`super::super::local::apices`] (num
/// círculo perfeito todos os vértices empatam em raio, e a lei aceita empate). ⭐ Isso é
/// sorte para este gate: eles ficam **intactos** nas duas malhas e provam que a régua é
/// por-ponta e não borra o defeito de um bico por cima dos outros.
///
/// ⛔⛔ **E é por isso que o anel tem `8` vértices e não `12`:** a lei ordena os ápices por
/// raio e corta em `MAX_TIPS = 12`, e o anel da base é **mais longe do centroide** que o
/// bico. Com `12` no anel, os doze empatavam à frente e **o espinho era o 13.º** — a
/// medição saía com `12` pontas, todas a zero, e lia-se como *«a peça está perfeita»*.
/// *Um corte por posto é uma decisão sobre QUEM não é medido, e um empate de doze
/// preenche-o inteiro.*
fn cone(amputa: bool) -> Mesh {
    const N: u32 = 8;
    const ALTURA: f32 = 3.0;
    const CORTE: f32 = 2.2;
    // ⚠️ **O corpo tem de ter MAIS anéis que o bico**, senão a nuvem junto do ápice puxa o
    // centroide para cima e o próprio ápice cai abaixo do piso de `0,55` — a lei deixa de o
    // ver e o gate mede uma peça sem pontas. *Foi o que aconteceu à 1.ª redacção desta
    // fixtura, e é uma propriedade da lei, não um acidente da aritmética.*
    let aneis: &[f32] = if amputa {
        &[CORTE, 2.0, 1.5, 1.0, 0.5, 0.0]
    } else {
        &[2.9, 2.7, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0]
    };
    let raio = |z: f32| (ALTURA - z) / ALTURA;
    let bico = if amputa { CORTE } else { ALTURA };
    let mut verts: Vec<[f32; 3]> = vec![[0.0, 0.0, bico], [0.0, 0.0, 0.0]];
    for &z in aneis {
        for k in 0..N {
            #[expect(clippy::cast_precision_loss, reason = "N = 8")]
            let a = core::f32::consts::TAU * k as f32 / N as f32;
            verts.push([raio(z) * a.cos(), raio(z) * a.sin(), z]);
        }
    }
    let anel = |i: u32, k: u32| 2 + i * N + k % N;
    let ultimo = u32::try_from(aneis.len() - 1).expect("poucos aneis");
    let mut faces: Vec<Face> = Vec::new();
    for k in 0..N {
        faces.push(Face::tri(0, anel(0, k), anel(0, k + 1)));
        for i in 0..ultimo {
            faces.push(Face::quad(
                anel(i, k),
                anel(i + 1, k),
                anel(i + 1, k + 1),
                anel(i, k + 1),
            ));
        }
        faces.push(Face::tri(1, anel(ultimo, k + 1), anel(ultimo, k)));
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — o bico amputado é acusado, e o intacto não.**
///
/// ⛔ **É a régua que o report de 2026-08-31 exigiu.** A função de suporte
/// ([`super::super::tip_survival`]) diz *até onde* a peça vai naquela direcção e **nada**
/// sobre a espessura com que lá chega; esta mede a distância da escultura à superfície que
/// saiu, junto do ápice, em unidades do quad pedido.
#[test]
fn um_bico_amputado_e_acusado_e_o_intacto_nao() {
    let entrada = cone(false);
    let alvo = 0.2;

    let igual = tip_deviation(&entrada, &entrada, alvo);
    assert!(igual.tips >= 1, "nenhuma ponta medida: {igual:?}");
    assert!(
        igual.max < TIP_DEVIATION_MAX,
        "a saida IDENTICA a' entrada nao pode acusar nada: {igual:?}"
    );
    assert_eq!(igual.over, 0, "{igual:?}");

    let cortado = tip_deviation(&entrada, &cone(true), alvo);
    assert_eq!(cortado.tips, igual.tips, "as duas medem as MESMAS pontas");
    assert!(
        cortado.p50 > TIP_DEVIATION_MAX,
        "o bico cortado tem de passar a barra: {cortado:?}"
    );
    assert_eq!(cortado.over, 1, "{cortado:?}");

    // ⭐⭐⭐ **E o ÁPICE sozinho** (2026-09-02, [`super::TIP_GAP_MAX`]): a saída idêntica
    // tem o bico EM CIMA da superfície, e a cortada tem-no a `(3,0 − 2,2) / 0,2 = 4` células.
    assert_eq!(igual.cut, 0, "{igual:?}");
    assert!(igual.apex_max < TIP_GAP_MAX, "{igual:?}");
    assert_eq!(
        cortado.cut, 1,
        "⛔ o bico amputado e' UMA ponta a mais de meia celula: {cortado:?}"
    );
    assert!(
        (cortado.apex_max - 4.0).abs() < 0.05,
        "o gap do apice e' a altura cortada em celulas: {cortado:?}"
    );
}

/// ⭐⭐⭐ **GATE — uma ponta comida POR INTEIRO é o pior caso, não um «não medido».**
///
/// ⛔⛔⛔ **É o gate de um defeito que esta régua teve no dia em que nasceu.** Quando a saída
/// não tem superfície nenhuma junto do ápice, não há amostra — e a 1.ª redacção **saltava** a
/// ponta. Resultado medido no produto: um relatório a dizer `0 de 3 pontas acima da barra`
/// sobre uma peça com um espinho amputado em **`−46,6 %`**, e o selector a escolher
/// exactamente essa candidata porque a régua a dava por limpa.
///
/// ⚠️ *É a família do balde vazio: «não medido» e «perfeito» são o mesmo byte.* A diferença é
/// que aqui o balde vazio **é** o defeito máximo, e não uma ausência de informação.
#[test]
fn uma_ponta_comida_por_inteiro_e_o_pior_caso() {
    let entrada = cone(false);
    // ⚠️ Um alvo pequeno faz o raio de busca (`3 × alvo`) não alcançar a tampa do cone
    // cortado — é assim que se encena «não há saída nenhuma junto do ápice».
    let alvo = 0.05;
    let d = tip_deviation(&entrada, &cone(true), alvo);
    assert!(
        d.over >= 1,
        "⛔ um espinho AMPUTADO tem de contar como partido: {d:?}"
    );
    assert!(
        d.max > TIP_DEVIATION_MAX,
        "⛔ e o desvio registado tem de passar a barra: {d:?}"
    );
    // ⛔ O CONTROLE: com a saída IGUAL à entrada o mesmo alvo pequeno não acusa nada — senão
    // este gate estaria a medir o raio de busca, e não a amputação.
    let igual = tip_deviation(&entrada, &entrada, alvo);
    assert_eq!(
        igual.over, 0,
        "CONTROLO: a saida identica nao pode acusar com o mesmo alvo: {igual:?}"
    );
}

/// ⭐⭐ **GATE — a régua é ADIMENSIONAL no passo da grade.**
///
/// ⚠️ Sem esta propriedade a régua não pode ser comparada entre densidades, e foi
/// exactamente para isso que ela nasceu: a mesma peça a `Detail 0,50` e a `0,85` tem de
/// poder ir na mesma tabela. ⛔ *Uma medida em unidades de mundo diria que a saída mais
/// fina é sempre melhor, o que é verdade por construção e não informa nada.*
#[test]
fn a_regua_nao_muda_quando_a_peca_e_o_alvo_crescem_juntos() {
    let entrada = cone(false);
    let saida = cone(true);
    let pequena = tip_deviation(&entrada, &saida, 0.2);
    // ⚠️ `4` é uma potência de dois: a escala é exacta em `f32` e o gate mede a lei, não o
    // arredondamento.
    let escala = |m: &Mesh| {
        let verts: Vec<[f32; 3]> = m
            .positions()
            .iter()
            .map(|p| [p[0] * 4.0, p[1] * 4.0, p[2] * 4.0])
            .collect();
        Mesh::from_parts(verts, m.faces().to_vec()).expect("a fixtura e' construida aqui")
    };
    let grande = tip_deviation(&escala(&entrada), &escala(&saida), 0.8);
    assert_eq!(pequena.tips, grande.tips, "{pequena:?} contra {grande:?}");
    assert_eq!(pequena.over, grande.over, "{pequena:?} contra {grande:?}");
    assert!(
        (pequena.p50 - grande.p50).abs() < 1.0e-3,
        "{pequena:?} contra {grande:?}"
    );
}

/// ⚠️ **GATE — «não medido» não pode ler-se como «perfeito».** ⛔ São o mesmo byte em toda
/// régua que devolve só a média, e este repo já pagou isso: um balde que ninguém enche lê
/// mediana `0,0`, que é o valor de uma peça impecável.
#[test]
fn uma_medicao_sem_populacao_diz_que_nao_mediu() {
    let entrada = cone(false);
    let vazia = Mesh::from_parts(Vec::new(), Vec::new()).expect("uma malha vazia e' legal");
    assert_eq!(tip_deviation(&entrada, &vazia, 0.2).tips, 0);
    assert_eq!(tip_deviation(&vazia, &entrada, 0.2).tips, 0);
    assert_eq!(
        tip_deviation(&entrada, &entrada, 0.0).tips,
        0,
        "um alvo nao positivo nao tem unidade em que dividir"
    );
    assert_eq!(
        tip_deviation(&entrada, &entrada, f32::NAN).tips,
        0,
        "um alvo NaN tambem nao"
    );
}
