//! ⭐ Os números de FÁBRICA de um nó recém-solto (ciclo 18, doc 127): um `Tint`, um `Morph` e um
//! `Mixer` NOVOS misturam a cor em OKLab; um nó que JÁ estava no documento (um gravado antes do
//! ciclo, sem o param) continua no default do manifesto — RGB linear, ao bit.

use super::*;
use crate::motion_state::MotionState;

#[test]
fn a_fresh_colour_node_is_born_in_oklab_and_an_old_one_keeps_linear() {
    let m = MotionState::new();
    let mut g = Graph::new();
    let velho = g.add_node("motion.tint".to_string());
    let antes = g.clone();
    let novos = [
        ("motion.tint", "space"),
        ("motion.morph", "tint_space"),
        ("motion.mixer", "tint_space"),
    ]
    .map(|(tipo, p)| (g.add_node(tipo.to_string()), p));
    seed_text_defaults(&mut g, &m.registry, &antes);
    for (no, p) in novos {
        assert_eq!(
            g.node_param_overrides(no).and_then(|o| o.get(p)).copied(),
            Some(1.0),
            "um nó novo nasce em OKLab ({p})"
        );
    }
    assert!(
        g.node_param_overrides(velho)
            .is_none_or(|o| !o.contains_key("space")),
        "o nó que já existia não é tocado"
    );
}
