//! Os gates da POSE que vivem **nesta** crate — a fiação, não a lei.
//!
//! ⚠️ A lei está medida contra `69` traços do oráculo na bancada da
//! `ph2d-pose`. O que **só** aqui se pode afirmar é o que a ponte promete: que
//! a simetria do traço chega à lei, e que a cadeia se constrói **uma vez**.

use ph2d_mesh::Mesh;

use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 48, 1.0)
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Pose,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    }
}

fn puxao(centro: [f32; 3], raio: f32, puxao: [f32; 3]) -> Dab {
    let l = (centro[0] * centro[0] + centro[1] * centro[1] + centro[2] * centro[2]).sqrt();
    let olho = [-centro[0] / l, -centro[1] / l, -centro[2] / l];
    Dab::pulling(centro, raio, olho, puxao)
}

/// ⭐ **A simetria do TRAÇO chega à lei.**
///
/// ⛔ Este verbo não entra no censo `every_verb_inherits_symmetry_…` porque ele
/// mede o conjunto **tocado**, um plano por-slot do laço por-vértice que a pose
/// nunca preenche (ela [`Verb::resolve_a_propria_regiao`]). *Tirar um verbo de
/// um censo sem escrever o gate que o substitui é como uma ausência vira
/// permanente* — este é o gate que o substitui, e ele mede o que a ponte de
/// facto promete: que `Symmetry` vira `Controlos::simetria`.
#[test]
fn a_pose_espelha_o_que_move() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let antes = malha.positions().to_vec();
    // Fora do plano `x = 0`, para que o espelho caia noutro sítio.
    for k in 0..2 {
        let d = 0.10 + f32::from(u8::try_from(k).unwrap_or(0)) * 0.05;
        s.dab(
            &mut malha,
            &b,
            &puxao([0.6, 0.0, 0.8], b.radius, [0.0, d, 0.0]),
            Symmetry::MIRROR_X,
        );
    }
    let movidos = s.last_moved();
    assert!(
        movidos.len() > 8,
        "a pose com simetria não moveu nada de jeito ({} vértices) — o gate mediria vácuo",
        movidos.len()
    );
    let pos = malha.positions();
    let (mut esquerda, mut direita, mut no_plano) = (0usize, 0usize, 0usize);
    for &v in movidos {
        // ⚠️ **A banda do PLANO sai da contagem, e a primeira redacção deste
        // gate esqueceu-a.** Uma esfera UV tem um anel inteiro de vértices
        // exactamente em `x = 0`; contá-los num dos lados faz o gate reprovar
        // sobre produto correcto, e foi o que ele fez. *Uma peça simétrica tem
        // três conjuntos, não dois.*
        let x = antes[v as usize][0];
        if x.abs() < 1e-4 {
            no_plano += 1;
        } else if x < 0.0 {
            esquerda += 1;
        } else {
            direita += 1;
        }
    }
    assert!(
        esquerda > 0 && direita > 0,
        "a simetria não chegou à lei: {esquerda} à esquerda e {direita} à direita"
    );
    // ⚠️ Uma esfera UV é simétrica em `x` por construção, logo os dois lados têm
    // de sair com contagens **iguais**. Uma banda larga aqui aceitaria a
    // simetria aplicada **duas** vezes, que é exactamente o defeito que o desvio
    // antes do espelho existe para impedir.
    // ⛔⛔ **A CONTAGEM é a régua FRÁGIL, e ela reprovou sobre produto correcto:**
    // `155` contra `154`. A diferença era **um** vértice que se move um ulp de um
    // lado e exactamente zero do outro — o limiar de escrita é uma igualdade
    // estrita de `f32`, não uma barra. *Uma régua discreta sobre uma grandeza
    // contínua transforma ruído de arredondamento num veredito.*
    assert!(
        esquerda.abs_diff(direita) <= 2,
        "os dois lados moveram contagens muito diferentes ({esquerda} vs {direita}, \
         {no_plano} no plano) — a simetria não é a da lei"
    );
    // ⭐ **A régua que decide é a ENERGIA de cada lado**, que é contínua e imune
    // ao limiar de escrita — e que um espelho aplicado **duas** vezes não pode
    // satisfazer: ali um dos lados receberia a deformação composta consigo
    // mesma e a razão sairia longe de `1`.
    let energia = |sinal: f32| -> f64 {
        movidos
            .iter()
            // ⚠️ **O lado é do REPOUSO, nunca da posição final** — um vértice
            // pode ATRAVESSAR o plano durante a deformação, e classificá-lo
            // onde ele acabou muda-o de balde a meio da medição. A primeira
            // redacção fez isso e leu `1,25e-2` de assimetria numa saída
            // simétrica.
            .filter(|&&v| antes[v as usize][0] * sinal > 1e-4)
            .map(|&v| {
                let (a, b) = (pos[v as usize], antes[v as usize]);
                f64::from(
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
                )
            })
            .sum()
    };
    let (e, d) = (energia(-1.0), energia(1.0));
    assert!(
        e > 0.0 && ((e - d) / e).abs() < 1e-4,
        "a energia dos dois lados difere: {e:.6} à esquerda contra {d:.6} à direita"
    );
}

/// ⭐⭐ **A CADEIA CONSTRÓI-SE UMA VEZ POR TRAÇO — e isto é a nossa vantagem
/// medida sobre o alvo, agora GATEADA em vez de afirmada.**
///
/// O alvo reconstrói a cadeia inteira a cada movimento do rato, **mesmo sem
/// traço nenhum**, só para desenhar o indicador sob o cursor: é a causa
/// registada em quatro relatos públicos de o editor engasgar em malha densa, e
/// com peças desligadas cada movimento de câmara volta a pagar um `O(V²)`.
///
/// ⚠️ *Uma vantagem escrita num cabeçalho é uma promessa; uma vantagem com
/// contador é uma propriedade.* Sem este gate, alguém que movesse a construção
/// para dentro do laço de eventos não partiria teste nenhum — a saída seria a
/// mesma, só mais lenta, e a regressão viajaria até ao dia em que o dono
/// esculpisse uma malha grande.
#[test]
fn a_cadeia_da_pose_constroi_se_uma_vez_por_traco() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    for k in 0..12 {
        let d = 0.02 * f32::from(u8::try_from(k).unwrap_or(0));
        s.dab(
            &mut malha,
            &b,
            &puxao([0.6, 0.0, 0.8], b.radius, [0.0, d, 0.0]),
            Symmetry::default(),
        );
    }
    assert_eq!(
        s.pose_construcoes, 1,
        "doze eventos construíram a cadeia {} vezes — ela é a MESMA do princípio \
         ao fim do traço (espec §10), e reconstruí-la é o defeito do alvo que \
         esta implementação existe para não ter",
        s.pose_construcoes
    );
    // E o controlo do outro lado: um traço NOVO acha o pivô outra vez.
    s.begin(&malha);
    s.dab(
        &mut malha,
        &b,
        &puxao([0.6, 0.0, 0.8], b.radius, [0.0, 0.1, 0.0]),
        Symmetry::default(),
    );
    assert_eq!(
        s.pose_construcoes, 1,
        "o `begin` tem de largar a cadeia: um traço novo constrói a dele"
    );
}
