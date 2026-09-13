//! **Os ids do CORTE** (plano 25 §7, a W4) — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_contour` e o do `vector_textpath`: estes são
//! os controles da família que muda a **TOPOLOGIA** de um caminho — parti-lo, soldá-lo, virá-lo —,
//! e o irmão fica com os ids do estilo, das formas e das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING, então reordenar não quebra nada — mas
//! renomear uma string quebra tudo o que a referencia por nome, e é assim que um widget fica órfão
//! em silêncio.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_cut.rs` em 2026-09-13** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── As três operações de NÓ da W4 (bloco APPEND-ONLY, plano 25 §7) ───────────
// Join · Reverse · Average. As duas primeiras são de CAMINHO e vivem na seção PATH, ao lado do
// `Close Path` que já lá estava; a terceira é de NÓ e vive na seção Vertex, com os outros gestos
// que só existem com nós selecionados.
/// **Join** — solda os caminhos selecionados numa cadeia (2+; fechar um só é o `VECTOR_PATH_CLOSE`,
/// que já existia — uma segunda porta para "fechar" divergiria dele no primeiro refino).
pub const VECTOR_PATH_JOIN: NodeId = hash_node_id("vector.path.join");

/// **Reverse** — inverte o sentido de cada caminho selecionado. Decide de que lado uma ponta de
/// seta aponta, para onde um texto-em-caminho corre e qual contorno de um compound é buraco.
pub const VECTOR_PATH_REVERSE: NodeId = hash_node_id("vector.path.reverse");

/// **Average** — colapsa os nós selecionados no centroide deles. Compõe com o Join: *Average +
/// Join* é a solda exata de duas pontas, o par canônico do Illustrator.
pub const VECTOR_VERT_AVERAGE: NodeId = hash_node_id("vector.vert.average");

/// ⭐⭐⭐ **Soldar** (plano 39) — os traços seleccionados partem-se nos cruzamentos e as pontas
/// vizinhas passam a cair no mesmo sítio. Vive ao lado do `VECTOR_PATH_JOIN` de propósito: aquele
/// solda **duas pontas**, este solda **os cruzamentos**, e lidos juntos ensinam a diferença.
pub const VECTOR_PATH_WELD: NodeId = hash_node_id("vector.path.weld");

/// **Cut** — executa o corte com a linha desenhada. Só é oferecido quando ela existe: um botão
/// que não tem lâmina para usar é um botão morto.
pub const VECTOR_CUT_APPLY: NodeId = hash_node_id("vector.cut.apply");

/// **Discard Cut Line** — apaga a linha de corte. O par do de cima, e a razão de a linha poder
/// ser um objeto persistente sem virar lixo na cena: há um gesto explícito para a tirar de lá.
pub const VECTOR_CUT_DISCARD: NodeId = hash_node_id("vector.cut.discard");
