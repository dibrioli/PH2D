//! ⭐⭐ **OS NOMES DAS SECÇÕES do painel de params de um nó** — a fronteira dos MANIFESTOS.
//!
//! ⛔⛔ **Eram inglês cru em 229 sítios de 21 crates**, e o report do dono apontou-lhes o dedo com
//! duas setas vermelhas: *«nomes de seção não mudaram»*. O doc do campo que os carrega dizia
//! *«o título da seção, em inglês (HR-15: a face do artista sai por i18n no painel)»* — e o
//! painel pintava-o **cru**. *Uma nota que descreve onde a tradução aconteceria lê-se como se ela
//! acontecesse.*
//!
//! # ⚠️⚠️ A chave é IDENTIDADE, e é por isso que ela não se resolve na ponte
//!
//! O título faz DUAS coisas: é o que se lê **e** é o que o `section_id` hasheia para o hit-rect e
//! para a memória de dobra (`motion_param/section/{título}`). ⇒ resolvê-lo na ponte mudaria o
//! **id** com o idioma: a secção que o artista deixou dobrada abria sozinha ao trocar de língua, e
//! o clique caía noutro sítio.
//!
//! ⇒ a chave viaja inteira até ao PINTOR, que a resolve só para desenhar. *Uma coisa que é ao
//! mesmo tempo identidade e legenda resolve-se no último instante possível.*
//!
//! # ⭐ Uma chave por NOME, e não por (nó, secção)
//!
//! `Falloff` aparece em doze nós e é a mesma palavra em todos — é vocabulário de painel, não
//! propriedade de um nó. ⛔ Uma chave por par daria **229** entradas onde bastam **38**, e poria o
//! tradutor a escrever a mesma palavra doze vezes, com doze oportunidades de divergir.

/// A tradução de uma chave `node.group.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "node.group.bottom_edge" => "Bottom Edge",
        "node.group.burst" => "Burst",
        "node.group.collision" => "Collision",
        "node.group.colour" => "Colour",
        "node.group.corners" => "Corners",
        "node.group.curve" => "Curve",
        "node.group.decay" => "Decay",
        "node.group.envelope" => "Envelope",
        "node.group.falloff" => "Falloff",
        "node.group.field" => "Field",
        "node.group.flocking" => "Flocking",
        "node.group.grammar" => "Grammar",
        "node.group.growth" => "Growth",
        "node.group.gust" => "Gust",
        "node.group.lean_and_look" => "Lean & Look",
        "node.group.leaves" => "Leaves",
        "node.group.left_edge" => "Left Edge",
        "node.group.look" => "Look",
        "node.group.mesh" => "Mesh",
        "node.group.origin" => "Origin",
        "node.group.output" => "Output",
        "node.group.particle_size" => "Particle Size",
        "node.group.physics" => "Physics",
        "node.group.pin" => "Pin",
        "node.group.placement" => "Placement",
        "node.group.randomness" => "Randomness",
        "node.group.range" => "Range",
        "node.group.response" => "Response",
        "node.group.right_edge" => "Right Edge",
        "node.group.shape" => "Shape",
        "node.group.source" => "Source",
        "node.group.space" => "Space",
        "node.group.spawn" => "Spawn",
        "node.group.steering" => "Steering",
        "node.group.timing" => "Timing",
        "node.group.top_edge" => "Top Edge",
        "node.group.trigger" => "Trigger",
        "node.group.values" => "Values",
        "node.group.velocity" => "Velocity",
        _ => return None,
    })
}
