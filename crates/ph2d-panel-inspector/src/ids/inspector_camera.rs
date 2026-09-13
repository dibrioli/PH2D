//! **Os ids da secção CAMERA** (TOP-20 #7, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_audio`] e do [`super::inspector_timer`].
//!
//! # ⚠️ Uma secção, TRÊS corpos — e não três secções
//!
//! `GameCamera`, `CameraFollow` e `CameraLimits` são três componentes registados, e a separação
//! deles é deliberada (ver o catálogo). Mas para o artista é **uma** pergunta — *«o que este objecto
//! tem a ver com o enquadramento?»* —, e o ADR-0166 diz que o Inspector mostra **o que o objecto
//! tem**. Três secções fariam uma câmera fixa, que é o caso comum, pagar dois cabeçalhos vazios.
//!
//! # ⚠️ Não há lista, e por isso não há linha aberta
//!
//! Um objecto tem **uma** câmera, **um** seguidor e **uma** cerca. ⇒ esta secção não precisa de
//! estado de painel nenhum: ela lê o snapshot e pinta os campos, como a do áudio.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector_camera.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// ⭐⭐⭐ **Olhar pela câmera da cena** — o interruptor da pré-visualização.
///
/// ⚠️ **É estado de VISTA, e não documento**: ele não escreve componente nenhum. Ligado, a câmera
/// que MANDA passa a ser dona do pan e do zoom; desligado, o enquadramento volta a ser o do editor.
/// *É a metade que a W2 deixou nomeada e por entregar — sem ela, a câmera é um motor vivo cujo
/// único botão era uma variável de ambiente.*
pub const INSP_CAMERA_PREVIEW: NodeId = hash_node_id("insp_camera_preview");

/// Altura do mundo visível, em metros.
pub const INSP_CAMERA_HEIGHT: NodeId = hash_node_id("insp_camera_height");

/// Deslocamento fixo do enquadramento — X.
pub const INSP_CAMERA_OFFSET_X: NodeId = hash_node_id("insp_camera_offset_x");

/// Deslocamento fixo do enquadramento — Y.
pub const INSP_CAMERA_OFFSET_Y: NodeId = hash_node_id("insp_camera_offset_y");

/// Quem manda quando há várias — maior ganha.
pub const INSP_CAMERA_PRIORITY: NodeId = hash_node_id("insp_camera_priority");

/// Desligada, ela não concorre a activa.
pub const INSP_CAMERA_ACTIVE: NodeId = hash_node_id("insp_camera_active");

/// O NOME do objecto seguido. ⚠️ Vazio = não segue ninguém.
pub const INSP_CAMERA_TARGET: NodeId = hash_node_id("insp_camera_target");

/// Amortecimento por eixo, em `1/s` — X.
pub const INSP_CAMERA_DAMP_X: NodeId = hash_node_id("insp_camera_damp_x");

/// Amortecimento por eixo, em `1/s` — Y.
pub const INSP_CAMERA_DAMP_Y: NodeId = hash_node_id("insp_camera_damp_y");

/// A janela morta, em fracção da meia-janela — X.
pub const INSP_CAMERA_DEAD_X: NodeId = hash_node_id("insp_camera_dead_x");

/// A janela morta, em fracção da meia-janela — Y.
pub const INSP_CAMERA_DEAD_Y: NodeId = hash_node_id("insp_camera_dead_y");

/// Antecipação, em SEGUNDOS — X.
pub const INSP_CAMERA_LOOK_X: NodeId = hash_node_id("insp_camera_look_x");

/// Antecipação, em SEGUNDOS — Y.
pub const INSP_CAMERA_LOOK_Y: NodeId = hash_node_id("insp_camera_look_y");

/// Deslocamento sobre o alvo, em metros — X.
pub const INSP_CAMERA_FOLLOW_OFF_X: NodeId = hash_node_id("insp_camera_follow_off_x");

/// Deslocamento sobre o alvo, em metros — Y.
pub const INSP_CAMERA_FOLLOW_OFF_Y: NodeId = hash_node_id("insp_camera_follow_off_y");

/// O canto mínimo da cerca — X.
pub const INSP_CAMERA_MIN_X: NodeId = hash_node_id("insp_camera_min_x");

/// O canto mínimo da cerca — Y.
pub const INSP_CAMERA_MIN_Y: NodeId = hash_node_id("insp_camera_min_y");

/// O canto máximo da cerca — X.
pub const INSP_CAMERA_MAX_X: NodeId = hash_node_id("insp_camera_max_x");

/// O canto máximo da cerca — Y.
pub const INSP_CAMERA_MAX_Y: NodeId = hash_node_id("insp_camera_max_y");

/// **A máscara de camadas — os 32 bits**, na grade `4×8` do [`crate::widget::BitmaskGrid32`].
///
/// ⚠️ **A grade é a MESMA do `VisibilityLayer`**, e é por isso que ela cabe aqui por quinze linhas:
/// o widget já recebe o array de ids. ⛔ **Os ids é que não podem ser os mesmos** — dois donos no
/// mesmo id fariam um clique na camada da sprite mexer na máscara da câmera.
pub const INSP_CAMERA_CULL_BIT: [NodeId; 32] = [
    hash_node_id("insp_camera_cull_0"),
    hash_node_id("insp_camera_cull_1"),
    hash_node_id("insp_camera_cull_2"),
    hash_node_id("insp_camera_cull_3"),
    hash_node_id("insp_camera_cull_4"),
    hash_node_id("insp_camera_cull_5"),
    hash_node_id("insp_camera_cull_6"),
    hash_node_id("insp_camera_cull_7"),
    hash_node_id("insp_camera_cull_8"),
    hash_node_id("insp_camera_cull_9"),
    hash_node_id("insp_camera_cull_10"),
    hash_node_id("insp_camera_cull_11"),
    hash_node_id("insp_camera_cull_12"),
    hash_node_id("insp_camera_cull_13"),
    hash_node_id("insp_camera_cull_14"),
    hash_node_id("insp_camera_cull_15"),
    hash_node_id("insp_camera_cull_16"),
    hash_node_id("insp_camera_cull_17"),
    hash_node_id("insp_camera_cull_18"),
    hash_node_id("insp_camera_cull_19"),
    hash_node_id("insp_camera_cull_20"),
    hash_node_id("insp_camera_cull_21"),
    hash_node_id("insp_camera_cull_22"),
    hash_node_id("insp_camera_cull_23"),
    hash_node_id("insp_camera_cull_24"),
    hash_node_id("insp_camera_cull_25"),
    hash_node_id("insp_camera_cull_26"),
    hash_node_id("insp_camera_cull_27"),
    hash_node_id("insp_camera_cull_28"),
    hash_node_id("insp_camera_cull_29"),
    hash_node_id("insp_camera_cull_30"),
    hash_node_id("insp_camera_cull_31"),
];

/// O cabeçalho da sub-secção da máscara — ela nasce RECOLHIDA, como a irmã da visibilidade.
pub const INSP_CAMERA_CULL_HEADER: NodeId = hash_node_id("insp_camera_cull_header");
