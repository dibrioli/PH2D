//! **Os ids da MOLDURA** (plano UI/UX W0) — irmão de [`super::vector`] pelo teto de 700 LOC, e o
//! corte é por assunto: aqui mora tudo o que o contêiner precisa e nada mais.

use ph2d_a11y::NodeId;

use crate::ids::hash_node_id;

/// Os dois chips de **Clip content** (o par segmentado Off/On).
///
/// ⚠️ O nome guarda a época em que só a moldura recortava; eles são hoje os chips da seção
/// [`VECTOR_SECTION_CLIP`], sobre qualquer forma fechada. Renomeá-los mudaria o `hash_node_id` —
/// que é o id que o a11y e os testes de costura conhecem — para não mudar comportamento nenhum.
pub const VECTOR_FRAME_CLIP_OFF: NodeId = hash_node_id("vector.frame.clip.off");
/// Ver [`VECTOR_FRAME_CLIP_OFF`].
pub const VECTOR_FRAME_CLIP_ON: NodeId = hash_node_id("vector.frame.clip.on");

/// **Mostrar esta moldura como PAINEL** (plano UI/UX W8b.2) — o interruptor do painel autorado.
///
/// ⚠️ Ele mora na seção FRAME, e não numa seção própria, porque *"que painel esta moldura
/// descreve?"* é uma pergunta sobre a MOLDURA. E é um par de chips (Off/On) em vez de um botão
/// porque o painel é uma coisa PERSISTENTE: um botão diria *"abra"* e não teria o que dizer com
/// ele já aberto, que é o clique-que-não-faz-nada deste repo.
pub const VECTOR_FRAME_PANEL_OFF: NodeId = hash_node_id("vector.frame.panel.off");
/// Ver [`VECTOR_FRAME_PANEL_OFF`].
pub const VECTOR_FRAME_PANEL_ON: NodeId = hash_node_id("vector.frame.panel.on");
