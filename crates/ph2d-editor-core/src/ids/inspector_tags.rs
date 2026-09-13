//! **Os ids da secção TAGS** (TOP-20 #9, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo padrão do [`super::inspector_timer`]
//! e do [`super::inspector_camera`].
//!
//! # ⚠️ Aqui só vivem os ids da SECÇÃO VIVA
//!
//! O cabeçalho colapsável e o ponto de cor entram em [`super::LIVE_SECTIONS`], que é a tabela que
//! faz uma secção nova nascer viva nos quatro sítios que ninguém liga entre si (dobra · ponto ·
//! despacho · menu). Os ids dos CONTROLOS (os chips, o `×` de cada um, a caixa de escolha) vivem na
//! crate do painel, que é quem os lê — a mesma descida que a auditoria de arquitectura A5b fez.

use super::*;

/// A secção TAGS — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_TAGS_SECTION: NodeId = hash_node_id("insp_live_tags_section");
/// TAGS — ponto de cor do cabeçalho.
pub const INSP_LIVE_TAGS_COLOR: NodeId = hash_node_id("insp_live_tags_color");
