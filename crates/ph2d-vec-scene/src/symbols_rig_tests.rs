//! Gates do [`super::bone`] e do [`super::rope_segment`].

use super::*;

/// A caixa de autoria: larga e baixa, que é como um osso e um segmento são usados.
const A: [f64; 2] = [-2.0, -0.5];
const B: [f64; 2] = [2.0, 0.5];

/// O `x` de cada vértice, em fracção do comprimento da caixa (`0` = junta, `1` = ponta).
fn us(p: &VecPath) -> Vec<f64> {
    p.verts
        .iter()
        .map(|v| (v.anchor[0] - A[0]) / (B[0] - A[0]))
        .collect()
}

/// ⭐⭐⭐ **O OSSO É AFILADO, e o ombro é o que o separa de um losango.**
///
/// ⚠️ **A metade do CONTROLO é a que importa:** um losango simétrico tem a cintura a meio e
/// também tem quatro vértices, também é «largo no meio e fino nas pontas». *O que faz uma cadeia
/// destes ler-se como um esqueleto — e não como uma fila de pipas — é o ombro estar ENCOSTADO à
/// junta*, e é por isso que o número é medido e não a contagem de vértices.
#[test]
fn o_osso_e_afilado_e_nao_um_losango() {
    let p = super::bone(A, B, 0.0);
    assert_eq!(p.verts.len(), 4, "junta · ombro · ponta · ombro");

    let u = us(&p);
    // ⚠️⚠️ **A JUNTA está em `u = 0` (`−X`), por ordem do dono** (2026-09-19, o SEGUNDO report:
    // *«Shape Bone deveria ser gerado por padrão a 180 graus de rotação do atual pois está
    // invertido»*, na mesma mensagem em que ele pediu o offset). ⛔ **Esta linha já disse `u = 1`
    // e estava errada**: o report da manhã foi obedecido virando a SILHUETA, que não era o
    // defeito — com a forma CENTRADA na junta, as duas orientações lêem-se uma como a outra.
    // *Metade da lei vive na caixa que o `motion_shape_gen` escolhe* (`[0, 2s]`), e só com ela a
    // cabeça do osso cai sobre a posição e o corpo afila para `+X`, que é para onde a cadeia vai.
    assert!((u[0] - 0.0).abs() < 1e-9, "a junta esta' em u = 0: {u:?}");
    assert!((u[2] - 1.0).abs() < 1e-9, "a ponta esta' em u = 1: {u:?}");
    // E os DOIS ombros estão no mesmo `u`, encostados à junta.
    assert!((u[1] - u[3]).abs() < 1e-9, "os dois ombros partilham o u");
    assert!(
        (u[1] - super::OMBRO).abs() < 1e-9,
        "o ombro fica a {} da junta (o numero do gizmo): {u:?}",
        super::OMBRO
    );
    // ⛔ O CONTROLO: bem longe do meio, senão isto é um losango com outro nome.
    assert!(
        u[1] < 0.35,
        "um ombro a meio faz um LOSANGO, nao um osso: u = {}",
        u[1]
    );
}

/// O olho fura de verdade, e sem pedido não há compound — o molde do `gear` e da `tag`.
#[test]
fn o_olho_da_junta_fura_e_sem_pedido_nao_existe() {
    let furado = super::bone(A, B, 1.0);
    let macico = super::bone(A, B, 0.0);
    assert!(furado.is_compound(), "o olho e' um contorno proprio");
    assert_eq!(
        furado.fill_rule,
        crate::FillRule::EvenOdd,
        "sem EvenOdd o olho nao vaza"
    );
    assert!(!macico.is_compound(), "sem olho, sem compound");
}

/// ⛔⛔ **O OLHO NUNCA FURA A SILHUETA, e a régua é a distância PONTO-RECTA e não a caixa.**
///
/// ⚠️ **Uma régua de bounding-box aprovaria um olho partido:** o quadrilátero é muito mais
/// estreito junto da junta do que a caixa que o contém, e um disco que cabe na caixa atravessa
/// as duas arestas curtas com folga. *É a mesma forma do `edge_max` global que este repo já
/// pagou* — medir o envelope quando o que morde é a peça.
///
/// ⛔⛔ **E ele mediu METADE das arestas até 2026-09-19, com o nome a prometer a silhueta
/// inteira:** só as duas que saem da junta. Quem o apanhou foi uma prova de mutação — pôr o olho
/// do lado da PONTA **sobreviveu**, porque ali ele fura as arestas LONGAS, que esta régua não
/// olhava. *Um gate cujo nome é mais largo que a população que ele varre lê-se como cumprido.*
#[test]
fn o_olho_cabe_dentro_da_silhueta() {
    let p = super::bone(A, B, 1.0);
    let olho = p.subpaths.first().expect("com eye = 1 o olho existe");

    // Os QUATRO cantos da silhueta em coordenadas de MUNDO, na ordem do contorno.
    let meio_y = 0.5 * (A[1] + B[1]);
    let ombro_x = A[0] + super::OMBRO * (B[0] - A[0]);
    let cantos = [
        [A[0], meio_y],  // a junta
        [ombro_x, B[1]], // ombro de cima
        [B[0], meio_y],  // a ponta
        [ombro_x, A[1]], // ombro de baixo
    ];
    // O centro do olho é o ponto de referência do «lado de dentro» — ele é, por construção, o
    // sítio mais largo da forma.
    let centro = [ombro_x, meio_y];

    for i in 0..4 {
        let (o, f) = (cantos[i], cantos[(i + 1) % 4]);
        let (dx, dy) = (f[0] - o[0], f[1] - o[1]);
        let n = dx.hypot(dy);
        let lado = |q: [f64; 2]| ((q[0] - o[0]) * dy - (q[1] - o[1]) * dx) / n;
        let dc = lado(centro);
        for v in &olho.verts {
            let d = lado(v.anchor);
            assert!(
                d * dc > 0.0,
                "um vertice do olho saiu pela aresta {i} da silhueta: d = {d:.6}"
            );
        }
    }
}

/// ⭐⭐ **O SEGMENTO DE CORDA É UM CORDÃO COM NÓS — e o controlo é que o nó seja MAIS LARGO.**
///
/// ⚠️ Sem a segunda metade, um `cord` igual a `1` devolve um RECTÂNGULO com doze vértices
/// colineares — *a contagem de quinas fica verde sobre uma forma que já não tem nó nenhum*.
#[test]
fn o_segmento_tem_no_nas_pontas_e_cordao_ao_meio() {
    let p = super::rope_segment(A, B, 0.35, 0.2);
    assert_eq!(p.verts.len(), 12, "o contorno em degrau tem doze quinas");

    let alturas: Vec<f64> = p.verts.iter().map(|v| v.anchor[1]).collect();
    let (lo, hi) = alturas
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), &y| (l.min(y), h.max(y)));
    // O nó ocupa a altura toda da caixa.
    assert!(
        (hi - lo - (B[1] - A[1])).abs() < 1e-9,
        "o no' enche a caixa"
    );

    // ⛔ O CONTROLO: no MEIO do segmento a forma é estritamente mais fina que no nó.
    let meio: Vec<f64> = p
        .verts
        .iter()
        .filter(|v| (v.anchor[0]).abs() < 1e-9)
        .map(|v| v.anchor[1])
        .collect();
    assert_eq!(meio.len(), 0, "nenhuma quina cai exactamente no centro");
    let cordao = p
        .verts
        .iter()
        .map(|v| v.anchor[1].abs())
        .filter(|y| *y < 0.5 * (B[1] - A[1]) - 1e-9)
        .count();
    assert_eq!(
        cordao, 4,
        "as QUATRO quinas do cordao tem de estar dentro da altura do no'"
    );
}

/// ⚠️ **Dois nós nunca se encontram a meio** — se encontrassem, apagavam o cordão, e a forma
/// deixava de mostrar o que existe para mostrar. O tecto é `0,45` e o clamp é que o impõe.
#[test]
fn os_dois_nos_nunca_comem_o_cordao() {
    for pedido in [0.5_f64, 0.9, 5.0] {
        let p = super::rope_segment(A, B, 0.35, pedido);
        let u = us(&p);
        let esq = u
            .iter()
            .filter(|v| **v > 1e-9 && **v < 0.5)
            .fold(0.0_f64, |m, v| m.max(*v));
        assert!(
            esq <= 0.45 + 1e-9,
            "com head = {pedido} o no' foi ate' {esq}, e comia o cordao"
        );
    }
}
