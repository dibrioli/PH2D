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
    // A junta e a ponta ocupam as duas extremidades.
    assert!((u[0] - 0.0).abs() < 1e-9, "a junta esta' em u = 0: {u:?}");
    assert!((u[2] - 1.0).abs() < 1e-9, "a ponta esta' em u = 1: {u:?}");
    // E os DOIS ombros estão no mesmo `u`, encostados à junta.
    assert!((u[1] - u[3]).abs() < 1e-9, "os dois ombros partilham o u");
    assert!(
        (u[1] - super::OMBRO).abs() < 1e-9,
        "o ombro fica a {} do caminho (o numero do gizmo): {u:?}",
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

/// ⛔⛔ **O OLHO NUNCA FURA A ARESTA, e a régua é a distância PONTO-RECTA e não a caixa.**
///
/// ⚠️ **Uma régua de bounding-box aprovaria um olho partido:** o quadrilátero é muito mais
/// estreito junto da junta do que a caixa que o contém, e um disco que cabe na caixa atravessa
/// as duas arestas curtas com folga. *É a mesma forma do `edge_max` global que este repo já
/// pagou* — medir o envelope quando o que morde é a peça.
#[test]
fn o_olho_cabe_dentro_da_silhueta() {
    let p = super::bone(A, B, 1.0);
    let olho = p.subpaths.first().expect("com eye = 1 o olho existe");

    // As duas arestas que saem da junta, em coordenadas de MUNDO.
    let (x0, y0) = (A[0], 0.5 * (A[1] + B[1]));
    let ombro_x = A[0] + super::OMBRO * (B[0] - A[0]);
    for (ax, ay) in [(ombro_x, A[1]), (ombro_x, B[1])] {
        let (dx, dy) = (ax - x0, ay - y0);
        let n = dx.hypot(dy);
        for v in &olho.verts {
            // Distância com sinal do vértice à recta junta→ombro; o interior é o lado do centro.
            let (px, py) = (v.anchor[0] - x0, v.anchor[1] - y0);
            let d = (px * dy - py * dx) / n;
            let (cx, cy) = (ombro_x - x0, 0.0_f64);
            let dc = (cx * dy - cy * dx) / n;
            assert!(
                d * dc > 0.0,
                "um vertice do olho saiu pela aresta da junta: d = {d:.6}"
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
