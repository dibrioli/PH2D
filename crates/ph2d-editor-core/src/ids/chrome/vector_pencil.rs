//! **Os ids da seção PENCIL** — a mão livre (plano 25, W1) — irmão de `vector` pelo teto de 700
//! LOC, e o corte é por responsabilidade: aqui moram os controles do GESTO (que detalhe se
//! guarda, que mão se escuta, de onde vem a espessura), e não os do documento.
//!
//! ⚠️ O `VECTOR_MODE_PENCIL` fica no irmão, com os outros pills da fileira TOOL: ele é a escolha
//! de MODO, e a fileira é uma coisa só.

use ph2d_a11y::NodeId;

use super::hash_node_id;

/// **A seção PENCIL** — os dois controles da mão livre. Existe porque as duas perguntas do gesto
/// são independentes e cada uma tem o seu slider: *que detalhe eu guardo?* (Fidelity, na SAÍDA — a
/// tolerância do decimador) e *que mão eu escuto?* (Stabilizer, na ENTRADA — o lazy mouse). Um
/// controle só teria de mentir sobre uma das duas.
pub const VECTOR_SECTION_PENCIL: NodeId = hash_node_id("vector.section.pencil");

/// **Fidelity** — a tolerância do decimador do lápis, em px de TELA (a mesma grandeza que o slider
/// homónimo do Illustrator expõe), logo invariante ao zoom.
pub const VECTOR_PENCIL_FIDELITY: NodeId = hash_node_id("vector.pencil.fidelity");
/// O chip numérico ligado ao [`VECTOR_PENCIL_FIDELITY`].
pub const VECTOR_PENCIL_FIDELITY_NUM: NodeId = hash_node_id("vector.pencil.fidelity.num");

/// **Stabilizer** — a força do lazy mouse (0 = ponteiro cru). Filtra o TREMOR na entrada, que é o
/// que o decimador não pode fazer: ele preserva extremos locais de propósito, e um tremor é um
/// extremo local.
pub const VECTOR_PENCIL_STABILIZER: NodeId = hash_node_id("vector.pencil.stabilizer");
/// O chip numérico ligado ao [`VECTOR_PENCIL_STABILIZER`].
pub const VECTOR_PENCIL_STABILIZER_NUM: NodeId = hash_node_id("vector.pencil.stabilizer.num");

/// **A FONTE da largura** do traço de lápis (W1d) — três chips exclusivos: a largura do estilo,
/// a velocidade do gesto, ou a pressão da caneta.
pub const VECTOR_PENCIL_W_UNIFORM: NodeId = hash_node_id("vector.pencil.w_uniform");
/// Ver [`VECTOR_PENCIL_W_UNIFORM`].
pub const VECTOR_PENCIL_W_SPEED: NodeId = hash_node_id("vector.pencil.w_speed");
/// Ver [`VECTOR_PENCIL_W_UNIFORM`].
pub const VECTOR_PENCIL_W_PRESSURE: NodeId = hash_node_id("vector.pencil.w_pressure");
