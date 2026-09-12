//! **Os ids da MOLDURA** (plano UI/UX W0) — irmão de [`super::vector`] pelo teto de 700 LOC, e o
//! corte é por assunto: aqui mora tudo o que o contêiner precisa e nada mais.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_frame.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

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
