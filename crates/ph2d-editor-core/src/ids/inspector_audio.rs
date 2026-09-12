//! **Os ids da secção AUDIO** (TOP-20 #4, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_timer`] e do [`super::inspector_action`].
//!
//! # ⚠️ Uma secção, DOIS corpos — e não duas secções
//!
//! Um objecto pode ter a FONTE, as ORELHAS, ou as duas. São dois componentes registados, mas uma
//! pergunta só para o artista — *«o que é que este objecto tem a ver com som?»* —, e o ADR-0166 diz
//! que o Inspector mostra **o que o objecto tem**. Duas secções fariam o caso comum (só a fonte)
//! pagar um cabeçalho vazio, e o caso das orelhas — um componente **sem campo nenhum** — teria uma
//! secção inteira para dizer uma linha.
//!
//! # ⚠️ Não há lista, e por isso não há linha aberta
//!
//! Ao contrário do `Timers` e do `SignalActions`, um objecto tem **UMA** fonte de som (ver o doc do
//! [`ph2d_ecs::AudioSource2D`]). ⇒ esta secção não precisa de estado de painel nenhum: ela lê o
//! snapshot e pinta os campos. É por isso que ela vive com as seções compartilhadas e não com as
//! que têm estado.

use super::*;

/// A secção AUDIO — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_AUDIO_SECTION: NodeId = hash_node_id("insp_live_audio_section");
/// AUDIO — ponto de cor do cabeçalho.
pub const INSP_LIVE_AUDIO_COLOR: NodeId = hash_node_id("insp_live_audio_color");
