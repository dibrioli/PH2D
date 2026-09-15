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

use super::*;

/// A secção CAMERA — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_CAMERA_SECTION: NodeId = hash_node_id("insp_live_camera_section");
/// CAMERA — ponto de cor do cabeçalho.
pub const INSP_LIVE_CAMERA_COLOR: NodeId = hash_node_id("insp_live_camera_color");

/// A secção FACTORY — o cabeçalho colapsável (TOP-20 #11, W3).
///
/// ⚠️ **Aqui e não na crate do painel, como as irmãs `INSP_LIVE_*`**: o cabeçalho de secção é lido
/// pelo `LIVE_SECTIONS` da fundação (a ordem das secções vivas), e não só por quem pinta.
pub const INSP_LIVE_FACTORY_SECTION: NodeId = hash_node_id("insp_live_factory_section");
/// A secção LIFECYCLE — o cabeçalho colapsável (TOP-20 #12, W3).
pub const INSP_LIVE_LIFECYCLE_SECTION: NodeId = hash_node_id("insp_live_lifecycle_section");
/// Quantas opções o segmentado do ONDE tem — a porta que o painel lê para repartir a largura.
pub const INSP_FACTORY_WHERE_LEN: usize = 3;
