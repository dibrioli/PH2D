//! **Os ids do ESQUELETO** (estudo 42 item 5, doc 47) — módulo irmão de [`super`] pelo teto de 700
//! LOC, com o corte por RESPONSABILIDADE: aqui vive a família que faz um desenho **dobrar** — o
//! modo que autora ossos e a seção que os liga à forma.
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING, então reordenar não quebra nada — mas
//! renomear uma string quebra tudo o que a referencia por nome, e é assim que um widget fica órfão
//! em silêncio.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

/// ⭐⭐⭐ **Osso** — o 17.º modo. Arrastar no vazio faz um osso; o pai é o osso seleccionado, então
/// arrasto-arrasto-arrasto é uma cadeia.
///
/// ⚠️ Ele fica no FIM da fileira, ao lado da Moldura, e a vizinhança diz o porquê: os dois são os
/// únicos modos que produzem algo que **não é uma forma** — aquele um lugar onde as formas moram,
/// este algo que as **move**.
pub const VECTOR_MODE_BONE: NodeId = hash_node_id("vector.mode.bone");

/// O cabeçalho da seção **SKELETON**. ⚠️ Tem de entrar em [`super::VECTOR_SECTIONS`], senão o
/// chevron pinta, clica e **não dobra** (o `dispatch` consulta aquela lista antes de disparar o
/// toggle) — dívida que o Text on Path e o Pattern on Path já pagaram.
pub const VECTOR_SECTION_BONE: NodeId = hash_node_id("vector.section.bone");

/// **Bind** — prende as formas seleccionadas ao esqueleto. Não move um pixel (a pose de repouso é a
/// identidade por construção), e é o gesto que separa um desenho de um personagem.
pub const VECTOR_BONE_BIND: NodeId = hash_node_id("vector.bone.bind");

/// **Keep Pose** — solta as formas e fica com a geometria deformada de AGORA (o *Expand* do
/// envelope). Par do de baixo: adivinhar qual dos dois o artista quer é que não.
pub const VECTOR_BONE_EXPAND: NodeId = hash_node_id("vector.bone.expand");

/// **Release** — solta as formas e devolve o que o artista DESENHOU.
pub const VECTOR_BONE_RELEASE: NodeId = hash_node_id("vector.bone.release");

/// **Length** — o comprimento do osso seleccionado, em unidades locais dele.
pub const VECTOR_BONE_LENGTH: NodeId = hash_node_id("vector.bone.length");

/// **Strength** — o raio de influência, em **comprimentos deste osso** (o *Bone Strength* do Moho).
///
/// ⚠️ Múltiplo e não distância, de propósito: é o que torna a lei adimensional, e o mesmo rig
/// desenhado dez vezes maior deforma-se igual.
pub const VECTOR_BONE_STRENGTH: NodeId = hash_node_id("vector.bone.strength");
