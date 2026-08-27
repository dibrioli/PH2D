//! **Os ids da secção Texture Pattern** — módulo irmão de [`super`] pelo teto de LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_patternpath`: estes são os controles da TINTA
//! de uma forma quando ela é um padrão de textura (plano 33) — qual arte, que reticulado, que
//! tamanho, onde.
//!
//! ⚠️⚠️ **NÃO confundir com o `vector_patternpath`.** Aquele é o *Pattern Along Path* (plano 23): um
//! MOTIVO copiado ao longo de uma guia, com alças e picker. Este é o preenchimento. Os dois têm a
//! palavra *pattern* no nome e são coisas diferentes — a linha já se enganou uma vez, ao chamar o
//! módulo novo de `pattern_live` e sobrescrever o que já existia.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os irmãos: um id é o hash de uma STRING, então reordenar não
//! quebra nada — mas renomear uma string quebra tudo o que a referencia por nome, e é assim que um
//! widget fica órfão em silêncio.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

/// Secção **PATTERN** — a tinta de uma forma quando ela é um padrão de textura.
pub const VECTOR_SECTION_TEXPAT: NodeId = hash_node_id("vector.section.texpat");

/// **Source…** — troca a ARTE do padrão (abre o diálogo de ficheiro).
pub const VECTOR_TEXPAT_SOURCE: NodeId = hash_node_id("vector.texpat.source");

// ── O RETICULADO: como as cópias se arrumam ──────────────────────────────────────
/// **Grid** — cada cópia debaixo da de cima. O ponto neutro.
pub const VECTOR_TEXPAT_TILE_GRID: NodeId = hash_node_id("vector.texpat.tile.grid");
/// **Brick** — as LINHAS desfasam-se horizontalmente.
pub const VECTOR_TEXPAT_TILE_BRICK: NodeId = hash_node_id("vector.texpat.tile.brick");
/// **Column** — as COLUNAS desfasam-se verticalmente (o *half-drop* têxtil, com Offset 1/2).
pub const VECTOR_TEXPAT_TILE_COLUMN: NodeId = hash_node_id("vector.texpat.tile.column");
/// **Hex** — a colmeia: meio passo **mais** o espaçamento `√3/2` que põe os seis vizinhos à mesma
/// distância. ⚠️ O assado é o mesmo do Brick de meio passo; o que a torna colmeia é o espaçamento.
pub const VECTOR_TEXPAT_TILE_HEX: NodeId = hash_node_id("vector.texpat.tile.hex");

/// **Offset** — o desfasamento é `1/n` de uma célula. Só aparece com Brick/Column: na grade ele
/// não tem sentido, e na colmeia ele é **fixo** em meio passo.
pub const VECTOR_TEXPAT_OFFSET: NodeId = hash_node_id("vector.texpat.offset");
/// O campo numérico gémeo do [`VECTOR_TEXPAT_OFFSET`].
pub const VECTOR_TEXPAT_OFFSET_NUM: NodeId = hash_node_id("vector.texpat.offset.num");

/// **Size** — o lado maior de uma cópia, em unidades de MUNDO. ⚠️ O aspecto da arte é preservado.
pub const VECTOR_TEXPAT_SIZE: NodeId = hash_node_id("vector.texpat.size");
/// O campo numérico gémeo do [`VECTOR_TEXPAT_SIZE`].
pub const VECTOR_TEXPAT_SIZE_NUM: NodeId = hash_node_id("vector.texpat.size.num");

/// **Gap** — o vão acrescentado a cada célula, em unidades de MUNDO. **Bipolar**: negativo é a
/// SOBREPOSIÇÃO (o *Overlap* do Illustrator).
pub const VECTOR_TEXPAT_GAP: NodeId = hash_node_id("vector.texpat.gap");
/// O campo numérico gémeo do [`VECTOR_TEXPAT_GAP`].
pub const VECTOR_TEXPAT_GAP_NUM: NodeId = hash_node_id("vector.texpat.gap.num");

/// **Angle** — a rotação do padrão, em GRAUS. ⚠️ Do PADRÃO, não da forma.
pub const VECTOR_TEXPAT_ANGLE: NodeId = hash_node_id("vector.texpat.angle");
/// O campo numérico gémeo do [`VECTOR_TEXPAT_ANGLE`].
pub const VECTOR_TEXPAT_ANGLE_NUM: NodeId = hash_node_id("vector.texpat.angle.num");

// ── A REPETIÇÃO: como o ladrilho preenche o que sobra ────────────────────────────
/// **Tile** — repete (`Extend::Repeat`). O caminho comum.
pub const VECTOR_TEXPAT_MODE_TILE: NodeId = hash_node_id("vector.texpat.mode.tile");
/// **Mirror** — espelha a cada repetição; a costura desaparece mesmo em arte não periódica.
pub const VECTOR_TEXPAT_MODE_MIRROR: NodeId = hash_node_id("vector.texpat.mode.mirror");
/// **Clamp** — uma cópia só, e o resto é a orla dela esticada.
pub const VECTOR_TEXPAT_MODE_CLAMP: NodeId = hash_node_id("vector.texpat.mode.clamp");
