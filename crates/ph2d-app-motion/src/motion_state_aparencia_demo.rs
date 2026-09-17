//! ⭐⭐⭐ **A COR E O RASTO** (`PH2D_GPU_COOK_DEMO=118`) — a cena do **ciclo 7**
//! ([doc 112](../../docs/Motion%20Nodes/112_ciclo_7_aparencia.md) §4-octies).
//!
//! ## O que ela ensina, e por que são TRÊS fileiras
//!
//! O grupo decide **como as peças se parecem**, e isso tem três perguntas diferentes: *de que COR*
//! (o `Tint`, a `Color Ramp`), *que MARCA deixam ao andar* (o `Trail`) e *QUANDO cada uma chega*
//! (o `Slit Scan`, que ganhou o `Delay By` neste ciclo). Uma fileira por pergunta.
//!
//! ```text
//!   CIMA    A COR      Tint                      |  Color Ramp (no lugar dele)
//!   MEIO    O RASTO    cada peça anda em roda    |  + Trail
//!   BAIXO   O TEMPO    Slit Scan por ORDEM       |  Slit Scan por CAMPO (+ Falloff)
//! ```
//!
//! ⚠️⚠️ **Os seis panos são o MESMO pano** (a mesma grade, a mesma peça, e nas duas de baixo o
//! mesmo movimento). Em cima muda **um cartão** (troca), no meio **um cartão a mais**; em baixo
//! mudam **a linha `Delay By` e o campo que ela passa a ler** — e o anúncio diz as duas coisas,
//! porque o modo novo SEM campo não tem nada para mostrar (o pano inteiro atrasa por igual). Há gate
//! a contar os nós e a linha.
//!
//! ⚠️ **A fileira de cima é PARADA de propósito** — a cor não precisa de tempo, e uma cena que a
//! pusesse a mexer misturaria duas perguntas no mesmo par.
//!
//! ⚠️ **Os cartões repetidos têm NOME** (`set_label`, o molde da `=115`): o passo que manda clicar
//! num `Slit Scan` tem de dizer em QUAL, e o grafo tem dois.
//!
//! ⛔ **O brilho, a sombra e a separação RGB não estão aqui:** o brilho é um passe de TELA (acende a
//! cena INTEIRA, os seis panos juntos, e não teria par), e os outros dois são cópias atrás da peça
//! — o tutorial ensina-os no mapa do grupo. *Uma cena de ciclo mostra a lei; o catálogo tem as
//! cenas da conferência.*

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// O lado de cada pano — o mesmo `6` da `=117`.
pub(super) const LADO: f32 = 6.0;
/// O passo entre peças, em unidades de MUNDO.
const VAO: f32 = 0.4;
/// O tamanho da peça (o sink desenha um quad de `1`, e o `motion.scale` põe-no à escala).
/// ⚠️ **Menor que a RODA do meio**, e isto foi a figura do tutorial que o mostrou: com a peça a
/// `0,16` e a roda a `0,1`, o rasto saía uma MANCHA colada à peça em vez de um arco. O anel inteiro
/// (`2·RODA + PECA = 0,38`) cabe no passo (`0,4`), então nenhum toca o vizinho.
const PECA: f32 = 0.1;
/// A meia-largura de um pano, DERIVADA.
pub(super) const MEIO_PANO: f32 = (LADO - 1.0) * 0.5 * VAO;
/// A que distância do centro cada metade vive.
pub(super) const COL_X: f32 = 2.9;
/// A distância vertical entre fileiras. ⚠️ **A câmara abre com `10` unidades de altura** (o
/// cabeçalho da `=111`), e três fileiras mais as fichas têm de caber nela sem zoom.
pub(super) const ROW_GAP: f32 = 3.1;
/// A que altura acima do centro do pano pousa a ficha — acima do pano e do movimento dele.
const FICHA_Y: f32 = 1.45;

/// A cor sólida do `Tint` — um laranja que se distingue de todas as cores do arco-íris juntas.
const LARANJA: [f32; 3] = [1.0, 0.55, 0.15];

/// O raio da roda do meio. ⚠️ Menor que metade do passo, senão os anéis de duas peças vizinhas
/// fundem-se e a cena deixa de mostrar «cada peça».
pub(super) const RODA: f32 = 0.14;
/// Voltas por segundo da roda — uma volta em dois segundos, que se segue a olho.
pub(super) const RODA_HZ: f32 = 0.5;
/// A cauda: ecos, e um eco a cada tantos tiques. ⚠️ `12 × 4 = 48` tiques = 40 % da volta (a
/// `60` tiques por segundo e `RODA_HZ` de `0,5`, a volta são `120`); o `Length` no fim do slider
/// (`32 × 4 = 128`) FECHA o anel — é o passo 6 do anúncio.
pub(super) const ECOS: f32 = 12.0;
pub(super) const ECO_A_CADA: f32 = 4.0;

/// A subida-e-descida de baixo.
pub(super) const ONDA: f32 = 0.22;
pub(super) const ONDA_HZ: f32 = 1.0;
/// O atraso da peça MAIS atrasada, em tiques — perto do tecto (`32`), para a onda atravessar o
/// pano com meia volta de diferença.
pub(super) const ATRASO: f32 = 30.0;
/// O `Delay By` do `motion.slit_scan`: `0` = `Order`, `1` = `Field`.
pub(super) const POR_ORDEM: f32 = 0.0;
pub(super) const POR_CAMPO: f32 = 1.0;

/// Os nomes dos cartões que a cena dá — o anúncio e o tutorial nomeiam-nos.
pub(super) const NOME_ORDEM: &str = "Slit Scan: ORDEM";
pub(super) const NOME_CAMPO: &str = "Slit Scan: CAMPO";

/// O centro da fileira `k` (`0` = cima).
pub(super) fn fileira_y(k: usize) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "tres fileiras")]
    let k = k as f32;
    ROW_GAP * (1.0 - k)
}

/// As legendas que a cena pousa no canvas — uma por metade, à mesma altura em cada fileira.
pub(super) fn captions() -> Vec<Caption> {
    let mut v = Vec::new();
    for (k, (esq, dir)) in [
        ("Tint: UMA cor para o pano", "Color Ramp: uma cor POR PECA"),
        ("cada peca anda em roda", "+ Trail: cada peca deixa RASTO"),
        (
            "atraso pela ORDEM: linha a linha",
            "atraso pelo LUGAR: esq -> dir",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let y = fileira_y(k) + FICHA_Y;
        v.push(Caption::new([-COL_X, y], esq));
        v.push(Caption::new([COL_X, y], dir));
    }
    v
}

/// Liga `de` a `para`, com o `pre` a dizer se a aresta atravessa o tique.
fn liga(doc: &mut MotionDoc, de: NodeId, para: (NodeId, u16), pre: bool) -> Option<()> {
    doc.graph
        .connect(Edge {
            from: (de, 0),
            to: para,
            delayed: pre,
        })
        .ok()
}

/// Um nó na posição `(x, y)` do GRAFO.
fn no(doc: &mut MotionDoc, tipo: &str, x: f32, y: f32) -> NodeId {
    let n = doc.graph.add_node(tipo);
    doc.graph.set_pos(n, Pos { x, y });
    n
}

/// Um pano de `LADO × LADO` centrado em `centro`. Devolve o último nó.
fn pano(doc: &mut MotionDoc, centro: [f32; 2], y: f32) -> Option<NodeId> {
    let g = no(doc, "motion.grid", -760.0, y);
    doc.graph.set_param(g, "rows", LADO);
    doc.graph.set_param(g, "cols", LADO);
    doc.graph.set_param(g, "gap_x", VAO);
    doc.graph.set_param(g, "gap_y", VAO);
    let peca = no(doc, "motion.scale", -580.0, y);
    doc.graph.set_param(peca, "amount", PECA);
    let mv = no(doc, "motion.move", -400.0, y);
    doc.graph.set_param(mv, "dx", centro[0]);
    doc.graph.set_param(mv, "dy", centro[1]);
    liga(doc, g, (peca, 0), false)?;
    liga(doc, peca, (mv, 0), false)?;
    Some(mv)
}

/// Um oscilador sem desfasamento entre peças: o pano mexe-se INTEIRO. ⚠️ O default do nó
/// (`phase_stagger 0,1`) já faria uma onda sozinho, e a de baixo tem de vir do `Slit Scan`.
fn oscila(doc: &mut MotionDoc, canal: f32, amp: f32, hz: f32, fase: f32, x: f32, y: f32) -> NodeId {
    let o = no(doc, "motion.oscillator", x, y);
    doc.graph.set_param(o, "channel", canal);
    doc.graph.set_param(o, "amplitude", amp);
    doc.graph.set_param(o, "frequency", hz);
    doc.graph.set_param(o, "phase", fase);
    doc.graph.set_param(o, "phase_stagger", 0.0);
    o
}

/// Fecha a cadeia num `motion.output` e devolve-o.
fn saida(doc: &mut MotionDoc, de: NodeId, y: f32) -> Option<NodeId> {
    let o = no(doc, "motion.output", 420.0, y);
    liga(doc, de, (o, 0), false)?;
    Some(o)
}

/// Monta a cena. Devolve os seis sinks, na ordem: cor (esq, dir) · rasto (esq, dir) · tempo (esq,
/// dir).
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let mut sinks = Vec::new();
    // As cadeias empilham-se no grafo pela ordem da tela: esquerda antes de direita.
    let grafo_y = |k: usize, lado: usize| -> f32 {
        #[expect(clippy::cast_precision_loss, reason = "seis cadeias")]
        let i = (k * 2 + lado) as f32;
        -600.0 + i * 240.0
    };

    // ── CIMA: A COR — muda UM cartão (troca) ─────────────────────────────────────────────
    for lado in 0..2 {
        let y = grafo_y(0, lado);
        let x_mundo = if lado == 0 { -COL_X } else { COL_X };
        let p = pano(doc, [x_mundo, fileira_y(0)], y)?;
        let cor = if lado == 0 {
            let t = no(doc, "motion.tint", 200.0, y);
            doc.graph.set_param(t, "r", LARANJA[0]);
            doc.graph.set_param(t, "g", LARANJA[1]);
            doc.graph.set_param(t, "b", LARANJA[2]);
            doc.graph.set_param(t, "a", 1.0);
            t
        } else {
            // O arco-íris de omissão do nó, atravessado pelo índice: cada peça, a sua cor.
            no(doc, "motion.color_ramp", 200.0, y)
        };
        liga(doc, p, (cor, 0), false)?;
        sinks.push(saida(doc, cor, y)?);
    }

    // ── MEIO: O RASTO — UM cartão a mais ─────────────────────────────────────────────────
    for lado in 0..2 {
        let y = grafo_y(1, lado);
        let x_mundo = if lado == 0 { -COL_X } else { COL_X };
        let p = pano(doc, [x_mundo, fileira_y(1)], y)?;
        // A roda: X e Y com um quarto de volta de diferença.
        let ox = oscila(doc, 0.0, RODA, RODA_HZ, 0.0, -220.0, y);
        let oy = oscila(doc, 1.0, RODA, RODA_HZ, 0.25, -40.0, y);
        liga(doc, p, (ox, 0), false)?;
        liga(doc, ox, (oy, 0), false)?;
        let ultimo = if lado == 1 {
            let tr = no(doc, "motion.trail", 200.0, y);
            doc.graph.set_param(tr, "length", ECOS);
            doc.graph.set_param(tr, "spacing", ECO_A_CADA);
            liga(doc, oy, (tr, 0), false)?;
            // O `state` do rasto é a memória da cauda.
            liga(doc, tr, (tr, 1), true)?;
            tr
        } else {
            oy
        };
        sinks.push(saida(doc, ultimo, y)?);
    }

    // ── BAIXO: O TEMPO — muda o `Delay By`, e o campo que ele passa a ler ────────────────
    for lado in 0..2 {
        let y = grafo_y(2, lado);
        let x_mundo = if lado == 0 { -COL_X } else { COL_X };
        let p = pano(doc, [x_mundo, fileira_y(2)], y)?;
        let sobe = oscila(doc, 1.0, ONDA, ONDA_HZ, 0.0, -220.0, y);
        liga(doc, p, (sobe, 0), false)?;
        let antes = if lado == 1 {
            // O campo: uma rampa de `0` (borda esquerda) a `1` (borda direita) DESTE pano.
            let f = no(doc, "motion.falloff", -40.0, y);
            doc.graph.set_param(f, "shape", 2.0);
            doc.graph.set_param(f, "curve", 0.0);
            doc.graph.set_param(f, "center_x", x_mundo);
            doc.graph.set_param(f, "center_y", fileira_y(2));
            doc.graph.set_param(f, "radius", MEIO_PANO);
            liga(doc, sobe, (f, 0), false)?;
            f
        } else {
            sobe
        };
        let scan = no(doc, "motion.slit_scan", 200.0, y);
        doc.graph.set_param(scan, "lag", ATRASO);
        doc.graph.set_param(
            scan,
            ph2d_node_motion_slit_scan::RAMP,
            if lado == 0 { POR_ORDEM } else { POR_CAMPO },
        );
        doc.graph
            .set_label(scan, if lado == 0 { NOME_ORDEM } else { NOME_CAMPO });
        liga(doc, antes, (scan, 0), false)?;
        // O `state` do slit-scan é a linha de atraso.
        liga(doc, scan, (scan, 1), true)?;
        sinks.push(saida(doc, scan, y)?);
    }

    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_aparencia_demo_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "motion_state_aparencia_tutorial_tests.rs"]
mod tutorial_tests;
