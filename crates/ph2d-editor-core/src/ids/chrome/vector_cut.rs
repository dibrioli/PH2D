//! **Os ids do CORTE** (plano 25 §7, a W4) — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_contour` e o do `vector_textpath`: estes são
//! os controles da família que muda a **TOPOLOGIA** de um caminho — parti-lo, soldá-lo, virá-lo —,
//! e o irmão fica com os ids do estilo, das formas e das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING, então reordenar não quebra nada — mas
//! renomear uma string quebra tudo o que a referencia por nome, e é assim que um widget fica órfão
//! em silêncio.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

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

// ── As quatro operações NOVAS do Pathfinder (bloco APPEND-ONLY, plano 25 §8, W5) ──
// Elas vivem no módulo do CORTE e não no do estilo pela mesma razão que o Join e o Reverse: são
// a família que muda a TOPOLOGIA de um caminho. As quatro antigas (Union/Subtract/Intersect/
// Exclude) ficam onde estavam — mover ids seria renomear strings, e um id é o hash de uma string.

// ── A FORMA do marquee (bloco APPEND-ONLY, plano 25 §9, o LAÇO) ──────────────
// Vive no módulo da topologia por vizinhança de assunto: como o Corte, é um par de chips colado
// na fileira TOOL que qualifica o gesto que o modo em mãos executa — não um estilo, não uma forma
// do catálogo.
