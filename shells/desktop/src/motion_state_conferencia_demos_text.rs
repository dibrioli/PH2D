//! **O TEXTO** (`PH2D_GPU_COOK_DEMO=39`) — a cena da linha 1 da §3 da folha
//! SOURCE, o P0 mais caro da conferência: *"não há nó de texto … metade do
//! mograph do mundo é texto animado por caractere"*.
//!
//! ## O que a cena põe lado a lado
//!
//! **A MESMA palavra duas vezes**, do mesmo `source.text`, com o mesmo tamanho e
//! a mesma fonte. A da direita tem dois nós a mais:
//! `value.instance_field(Ramp) → motion.drive(Rotation)`.
//!
//! - **EM CIMA** a palavra é uma palavra: as letras assentam na baseline.
//! - **EM BAIXO** cada letra roda um pouco mais que a anterior, abrindo em leque.
//!
//! ⚠️ **É o leque que prova a wave, e não o texto aparecer.** Um bloco emitido
//! como UMA instância desenharia a mesma palavra em cima e, em baixo, giraria
//! **rigidamente como bloco** — a palavra inteira tombada, todas as letras no
//! mesmo ângulo. Uma linha por caractere é o que faz a biblioteca `motion.*`
//! inteira agir por letra sem um nó novo a jusante, e o leque é a assinatura
//! disso na tela.
//!
//! ## O que a cena NÃO prova
//!
//! Ela não julga o pivô (os dois desenham a mesma imagem em repouso — é gate, não
//! olho) nem o kerning (o layout é advance-only, sem shaping complexo: a decisão
//! está escrita no `vec_glyph`).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// A palavra. Letras distintas de propósito — uma repetida esconderia a partilha
/// de geometria, que é justamente o que se quer ver funcionar sem se notar.
pub(crate) const WORD: &str = "MOTION";
/// Quanto o leque abre na última letra (graus).
pub(crate) const FAN_DEG: f32 = 55.0;

/// `text → [instance_field → drive] → transform → output`, duas vezes. Devolve os
/// DOIS sinks (o de cima é o controle).
pub(crate) fn build_text_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for k in 0..2u8 {
        let fanned = k == 1;
        let row = 120.0 + f32::from(k) * 300.0;

        let text = g.add_node("source.text");
        g.set_text_param(text, ph2d_node_source_text::TEXT_KEY, WORD);
        g.set_param(text, ph2d_node_source_text::param::SIZE, 1.6);
        g.set_param(text, ph2d_node_source_text::param::TRACKING, 0.06);
        // Ao CENTRO: as duas palavras partilham o eixo vertical, então o leque é
        // lido contra a palavra reta em vez de contra a margem.
        g.set_param(text, ph2d_node_source_text::param::ALIGN, 1.0);

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", if fanned { -2.2 } else { 1.4 });
        let out = g.add_node("motion.output");

        for (i, n) in [text, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 220.0,
                    y: row,
                },
            );
        }

        let tail = if fanned {
            let field = g.add_node("value.instance_field");
            g.set_param(field, "mode", 1.0); // Ramp: 0..1 pela ordem da letra
            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", 2.0); // Rotation
            g.set_param(drive, "mode", 1.0); // Set — o ângulo É a rotação
            g.set_param(drive, "scale", FAN_DEG);
            g.set_pos(
                field,
                Pos {
                    x: 300.0,
                    y: row + 140.0,
                },
            );
            g.set_pos(
                drive,
                Pos {
                    x: 520.0,
                    y: row + 140.0,
                },
            );
            wire(g, text, 0, field, 0)?;
            wire(g, text, 0, drive, 0)?;
            wire(g, field, 0, drive, 1)?;
            drive
        } else {
            text
        };
        wire(g, tail, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` o empresta
/// até ao fim do escopo, e o corpo do laço ainda precisa dele.
fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
) -> Option<()> {
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_text_tests.rs"]
mod tests;
