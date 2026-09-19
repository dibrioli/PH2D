//! **OS SÍMBOLOS DO RIG** — o **OSSO** e o **SEGMENTO DE CORDA**.
//!
//! ⭐⭐⭐ **Eles existem por ordem do dono** (2026-09-19): *«no caso dos ossos e segmentos de corda,
//! criaremos no nó shape formas similares para isso»*, dita quando ele mandou **retirar** os
//! gizmos de osso e de corda dos nós que só passam posições. ⇒ *o que era CHROME desenhado pelo
//! editor passa a ser CONTEÚDO que o artista compõe* — uma forma no `source.shape`, que um
//! `motion.duplicator` carimba em cada posição e que a rotação do grafo orienta.
//!
//! ⚠️⚠️ **A silhueta não é inventada: é a do gizmo apagado, lida do commit que o retirou**
//! (`38ac64852`). O que lá estava escrito em prosa — *«a silhueta de armadura: larga na junta que
//! manda, afilada para a ponta, com um anel na junta»* e *«a polilinha dos consecutivos, com uma
//! marca em cada nó»* — está aqui em geometria, com as **mesmas proporções**.
//!
//! ⚠️ **Irmão do [`super::symbols`] pelo tecto de LOC** (ele está a `678` de `700`) e por
//! PERGUNTA: ali moram os símbolos de DIAGRAMA (setas, balões, fluxograma), aqui os de RIG.

use crate::VecPath;
use crate::space::{Unit, Uv, add_sub, fit, poly, punch};

// ─────────────────────────────────────────────────────────────────────────────
// OSSO
// ─────────────────────────────────────────────────────────────────────────────

/// **Onde fica o OMBRO**, em fracção do comprimento, contado da JUNTA para a ponta.
///
/// ⚠️ **`0,2` é o número do gizmo que esta forma substitui** — lá a linha era
/// `a + d·0.2`, com o comentário *«o ombro fica a um quinto do caminho: é onde a armadura do
/// referencial o põe, e é o que dá a direcção sem engordar a cadeia inteira»*.
///
/// ⛔ **Ele NÃO é um knob, e isso é uma decisão:** o dono pediu *«formas similares»* às que
/// tinha, não um editor de ossos. Um param novo aqui aumenta a superfície do cartão sem ninguém
/// a pedir — e o `aspect`, que já existe, é a alavanca que muda o que o olho de facto lê (quão
/// gordo o osso é face ao comprimento).
const OMBRO: f64 = 0.2;

/// **O OSSO** — a silhueta de armadura: larga na junta, afilada para a ponta, com o olho na junta.
///
/// A caixa `a..b` é lida com a **JUNTA à esquerda** e a **PONTA à direita**, que é a convenção de
/// toda forma direccional deste catálogo (`ArrowRight`, `Chevron`): a rotação do grafo orienta-a
/// a partir daí, e um `align = Normal` num `motion.path` aponta-a ao longo do caminho de graça.
///
/// `eye` é o olho da junta como fracção do MÁXIMO que lá cabe (`0` = maciço). ⚠️ **O máximo é
/// DERIVADO e não escolhido** — ver [`raio_do_olho`].
#[must_use]
pub fn bone(a: [f64; 2], b: [f64; 2], eye: f64) -> VecPath {
    let u = Unit::of(a, b);
    // O quadrilátero do gizmo, vértice a vértice: junta · ombro de cima · ponta · ombro de baixo.
    let mut p = poly(&u, &[(0.0, 0.5), (OMBRO, 0.0), (1.0, 0.5), (OMBRO, 1.0)]);

    let r = raio_do_olho(eye);
    if r > 1e-6 {
        // ⚠️ **O olho é centrado no OMBRO e não na junta**, e a diferença é geométrica: na junta
        // o quadrilátero tem largura ZERO (é um vértice), logo qualquer disco ali atravessa as
        // duas arestas e parte a forma. No ombro é onde ele é mais largo.
        let mut olho = u.arc((OMBRO, 0.5), r, r, 0.0, -360.0);
        olho.pop();
        add_sub(&mut p, olho, true);
        punch(&mut p);
    }
    fit(&u, &mut p);
    p
}

/// **O raio do olho, DERIVADO da folga que a silhueta de facto tem.**
///
/// ⛔⛔ **Um número cravado aqui seria uma cerca sobre a geometria de OUTRA pessoa:** a folga
/// depende do [`OMBRO`], e quem o mudasse deixava o olho a furar a aresta **sem um erro**. A
/// distância do centro do ombro à aresta curta (de `(0, ½)` a `(OMBRO, 0)`) é
/// `½·OMBRO / √(¼ + OMBRO²)` — a fórmula ponto-recta, e é ela o tecto.
///
/// ⚠️ **A aresta CURTA é sempre a que morde:** a longa (do ombro à ponta) fica a `0,424` com o
/// `OMBRO` de fábrica contra `0,186` desta. *Medir só a longa deixaria o olho a sair pelo lado
/// da junta.*
fn raio_do_olho(eye: f64) -> f64 {
    let folga = 0.5 * OMBRO / (0.25 + OMBRO * OMBRO).sqrt();
    // A margem impede que o olho ENCOSTE na aresta: dois contornos tangentes deixam um fio de
    // largura nula, que o triangulador lê como uma quina degenerada.
    eye.clamp(0.0, 1.0) * folga * 0.9
}

// ─────────────────────────────────────────────────────────────────────────────
// SEGMENTO DE CORDA
// ─────────────────────────────────────────────────────────────────────────────

/// **O SEGMENTO DE CORDA** — o cordão com um NÓ em cada ponta.
///
/// O gizmo que esta forma substitui desenhava *«a polilinha dos consecutivos, com uma marca em
/// cada nó»*. Uma forma unitária não vê os vizinhos, então o que ela carrega é **um segmento**: o
/// cordão fino ao meio e o nó nas duas pontas. Carimbada ao longo de uma corda, a cadeia lê-se
/// como o gizmo lia — com a diferença de que agora ela **é** o desenho que o artista entrega.
///
/// `cord` = espessura do cordão em fracção da altura · `head` = largura de cada nó em fracção do
/// comprimento.
///
/// ⚠️ **Doze quinas e nenhum arco, de propósito.** Um disco escrito em UV vira ELIPSE quando a
/// caixa é alongada (e esta forma é alongada por construção — é um segmento), logo os nós
/// sairiam esmagados exactamente no uso normal. *O `cross` desta casa já resolve a mesma
/// pergunta com doze quinas.*
#[must_use]
pub fn rope_segment(a: [f64; 2], b: [f64; 2], cord: f64, head: f64) -> VecPath {
    let u = Unit::of(a, b);
    // ⚠️ O tecto do nó é `½`: dois nós que se encontrassem a meio apagavam o cordão, e a forma
    // deixava de mostrar o que ela existe para mostrar.
    let h = head.clamp(0.05, 0.45);
    let c = cord.clamp(0.05, 0.95);
    let (lo, hi) = (0.5 - 0.5 * c, 0.5 + 0.5 * c);
    let pts: [Uv; 12] = [
        (0.0, 0.0),
        (h, 0.0),
        (h, lo),
        (1.0 - h, lo),
        (1.0 - h, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (1.0 - h, 1.0),
        (1.0 - h, hi),
        (h, hi),
        (h, 1.0),
        (0.0, 1.0),
    ];
    let mut p = poly(&u, &pts);
    fit(&u, &mut p);
    p
}

#[cfg(test)]
#[path = "symbols_rig_tests.rs"]
mod tests;
