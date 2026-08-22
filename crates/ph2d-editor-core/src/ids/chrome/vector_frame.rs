//! **Os ids da MOLDURA** (plano UI/UX W0) — irmão de [`super::vector`] pelo teto de 700 LOC, e o
//! corte é por assunto: aqui mora tudo o que o contêiner precisa e nada mais.

use ph2d_a11y::NodeId;

use crate::ids::hash_node_id;

/// O 14º pill do rail: **Frame**. Arrasta e nasce um contêiner (um retângulo vivo com
/// `ph2d_ecs::VecFrame`).
///
/// ⚠️ É um MODO, e não um botão *"transformar em moldura"*, pela mesma razão que o Shape é um
/// modo: o gesto é **produzir**, e o artista quer desenhar a tela onde ela vai ficar. Converter
/// uma forma existente é a outra metade (a de Figma, `Frame selection`) e ainda não existe.
pub const VECTOR_MODE_FRAME: NodeId = hash_node_id("vector.mode.frame");

/// O cabeçalho da seção **Frame** (só com uma moldura selecionada).
pub const VECTOR_SECTION_FRAME: NodeId = hash_node_id("vector.section.frame");

/// O cabeçalho da seção **Clip** — o recorte, que desde 2026-08-21 vale para qualquer forma
/// vetorial FECHADA e não só para a moldura (Enio: *"coloque a feature Clip Content para qualquer
/// forma vetorial fechada"*).
///
/// ⚠️ **Seção própria, e não uma linha da seção Frame**, porque a Frame não pode aparecer sobre
/// uma estrela: ela carrega também o *Show as Panel* e os presets de dispositivo, que são
/// perguntas sobre uma MOLDURA. Separar o cabeçalho é o que deixa o recorte alcançar toda forma
/// fechada sem levar consigo três controles que não significam nada ali.
pub const VECTOR_SECTION_CLIP: NodeId = hash_node_id("vector.section.clip");

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

/// Os quatro **presets de dispositivo**. Cada um escreve W/H pela MESMA porta que os campos
/// numéricos da seção Transform usam (`apply_vec_transform`) — um preset é uma segunda forma de
/// PEDIR a mesma edição, nunca um segundo caminho de fazê-la.
pub const VECTOR_FRAME_PRESET_PHONE: NodeId = hash_node_id("vector.frame.preset.phone");
/// Ver [`VECTOR_FRAME_PRESET_PHONE`].
pub const VECTOR_FRAME_PRESET_TABLET: NodeId = hash_node_id("vector.frame.preset.tablet");
/// Ver [`VECTOR_FRAME_PRESET_PHONE`].
pub const VECTOR_FRAME_PRESET_DESKTOP: NodeId = hash_node_id("vector.frame.preset.desktop");
/// Ver [`VECTOR_FRAME_PRESET_PHONE`].
pub const VECTOR_FRAME_PRESET_SQUARE: NodeId = hash_node_id("vector.frame.preset.square");
