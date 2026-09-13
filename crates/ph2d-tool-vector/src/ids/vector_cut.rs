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
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_cut.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Corte** — o 13º modo (plano 25 §7, W4): desenha-se a LINHA DE CORTE com a caneta, e o botão
/// [`VECTOR_CUT_APPLY`] corta com ela. Fica ao lado dos pills de quina e do Width: os quatro
/// editam uma forma que JÁ existe, apontando-a no canvas.
///
/// ⚠️ Ele substituiu **dois** pills (Tesoura e Faca), cujas strings de id morreram com eles. Um id
/// registrado que nada pinta é a podridão que os Rake/Random do Paper viraram no Painter.
pub const VECTOR_MODE_CUT: NodeId = hash_node_id("vector.mode.cut");

/// ⭐⭐⭐ **Aparar** — o 15º modo (plano 38). Vizinho do Corte de propósito: os dois removem
/// geometria, e a diferença é quem manda no CORTE. O Corte quer uma lâmina autorada; o Trim usa o
/// que já está na tela e só pede que se aponte o pedaço.
pub const VECTOR_MODE_TRIM: NodeId = hash_node_id("vector.mode.trim");

/// ⭐⭐⭐ **Balde** — o 16º modo (plano 40). Vizinho do Trim e do Corte porque os três apontam uma
/// REGIÃO que já está na tela em vez de a autorar: aqueles removem, este preenche.
pub const VECTOR_MODE_BUCKET: NodeId = hash_node_id("vector.mode.bucket");

/// **Minus Back** — a forma da FRENTE menos a união de tudo o que está atrás.
pub const VECTOR_BOOL_MINUS_BACK: NodeId = hash_node_id("vector.bool.minus_back");

/// **Trim** — cada forma menos a união do que está ACIMA dela; todas sobrevivem, sem sobreposição.
pub const VECTOR_BOOL_TRIM: NodeId = hash_node_id("vector.bool.trim");

/// **Crop** — cada forma ∩ a do TOPO, e o topo é descartado (ele foi a moldura).
pub const VECTOR_BOOL_CROP: NodeId = hash_node_id("vector.bool.crop");

/// **Merge** — Trim, e depois as de MESMO preenchimento que se tocam viram uma.
pub const VECTOR_BOOL_MERGE: NodeId = hash_node_id("vector.bool.merge");

/// **Marquee: Box** — a região do arrasto no vazio é o retângulo entre os dois cantos.
pub const VECTOR_MARQUEE_BOX: NodeId = hash_node_id("vector.marquee.box");

/// **Marquee: Lasso** — a região é o caminho que a mão desenhou, fechado da ponta ao começo.
///
/// ⚠️ Os dois são a metade PEGAJOSA da resposta; o **Ctrl** troca a de um gesto só
/// (`MarqueeShape::for_gesture`). O chip existe porque um atalho que ninguém descobre é uma
/// feature que não existe — a mesma razão pela qual o *Select Subpath* é botão e não tecla.
pub const VECTOR_MARQUEE_LASSO: NodeId = hash_node_id("vector.marquee.lasso");
