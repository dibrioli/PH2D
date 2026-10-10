//! **Os nós do EDIT SHAPE** (F7e) — os nós PRÓPRIOS da forma que o QuickShape acabou de fazer (os
//! cantos, os extremos dos eixos, o ápice do arco). A gramática é a dos pinos do Envelope
//! ([`crate::draw_envelope_pins`]): um ponto cheio, aceso enquanto agarrado — uma alça que se
//! arrasta, distinta do quadrado de âncora que está por baixo.

use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

/// `nodes` em MUNDO, `camera` mundo→ecrã; `grabbed` acende o agarrado.
pub fn draw_shape_nodes(
    nodes: &[[f64; 2]],
    grabbed: Option<usize>,
    camera: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let pins: Vec<[[f64; 2]; 2]> = nodes.iter().map(|&p| [p, p]).collect();
    crate::draw_envelope_pins(&pins, grabbed, camera, theme, target);
}
