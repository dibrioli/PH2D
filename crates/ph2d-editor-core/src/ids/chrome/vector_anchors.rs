//! **Os ids das ÂNCORAS** (plano UI/UX W3) — irmão de [`super::vector_layout`] pelo teto de LOC, e
//! o corte é o assunto: aqui mora a regra do filho que NÃO está num fluxo.
//!
//! # Duas fileiras, e elas são a mesma pergunta feita a dois eixos
//!
//! O Figma tem exactamente isto: um menu por eixo, e nada mais. Cada fileira responde *"quando a
//! moldura muda de largura (ou de altura), o que este filho faz?"*, e as quatro respostas são
//! todas as que existem — **seguir a aresta mínima · ficar no meio · seguir a máxima · ESTICAR**
//! (as duas pontas seguem arestas diferentes).
//!
//! ⚠️ **Não há chip de *Off*, e a ausência é deliberada.** No auto layout, *"esta moldura empilha?"*
//! e *"em que direção?"* são a MESMA pergunta (o `display` do CSS), então o `Off` é o primeiro chip
//! do rádio. Aqui não: um filho está sempre em algum lugar quando a moldura cresce, e *"colado na
//! aresta mínima"* já É a resposta neutra — a que a ausência do componente produz. Um `Off` ao lado
//! dela seria um segundo chip com o mesmo efeito, e o artista teria de descobrir por tentativa qual
//! dos dois usar.
//!
//! # A vertical é nomeada pelo que se VÊ, não pelo sinal
//!
//! ⚠️ O documento é Y-up, então a aresta MÍNIMA é a de BAIXO — mas o artista lê *"Top"* e
//! *"Bottom"*, não *"máximo"* e *"mínimo"*. A tradução mora **UMA vez**, na tabela de chips da
//! shell (`vec_anchor_edit::V`), e é por isso que estes ids não carregam número nenhum.

use ph2d_a11y::NodeId;

use crate::ids::hash_node_id;

/// O cabeçalho da seção **Constraints** (só com um filho de moldura que NÃO flui selecionado).
pub const VECTOR_SECTION_ANCHORS: NodeId = hash_node_id("vector.section.anchors");

/// Horizontal — segue a aresta ESQUERDA (a mínima em X).
pub const VECTOR_ANCHOR_H_START: NodeId = hash_node_id("vector.anchor.h.start");
/// Horizontal — fica no meio.
pub const VECTOR_ANCHOR_H_CENTER: NodeId = hash_node_id("vector.anchor.h.center");
/// Horizontal — segue a aresta DIREITA.
pub const VECTOR_ANCHOR_H_END: NodeId = hash_node_id("vector.anchor.h.end");
/// Horizontal — **estica**: a esquerda fica, a direita acompanha.
pub const VECTOR_ANCHOR_H_STRETCH: NodeId = hash_node_id("vector.anchor.h.stretch");

/// Vertical — segue a aresta de CIMA (a máxima em Y, porque o documento é Y-up).
pub const VECTOR_ANCHOR_V_START: NodeId = hash_node_id("vector.anchor.v.start");
/// Vertical — fica no meio.
pub const VECTOR_ANCHOR_V_CENTER: NodeId = hash_node_id("vector.anchor.v.center");
/// Vertical — segue a aresta de BAIXO.
pub const VECTOR_ANCHOR_V_END: NodeId = hash_node_id("vector.anchor.v.end");
/// Vertical — **estica**: uma ponta fica, a outra acompanha.
pub const VECTOR_ANCHOR_V_STRETCH: NodeId = hash_node_id("vector.anchor.v.stretch");
