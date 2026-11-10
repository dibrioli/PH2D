//! **A ESCUTA PARTILHADA do Input Map** — os dois `Click` sintéticos que a captura de uma ligação
//! emite (a aba central e o cartão do Trigger usam-na). A janela flutuante saiu na R2a (10/11).
//!
//! ⚠️ **Não são botões**: nenhum pixel os desenha e nenhum hit rect os regista. O despacho de
//! teclado (ou a escuta do comando) só alcança o `WidgetStore` e a ligação precisa do `HeroScreen` —
//! quem captura **guarda** a ligação e emite isto, e o handler de chrome, que tem o hero, **liga**.

use super::{NodeId, hash_node_id};

/// **A LIGAÇÃO FOI CAPTURADA** (tecla ou comando) — drenada por `take_captured_binding`.
pub const INPUT_MAP_BIND_CAPTURED: NodeId = hash_node_id("input_map.bind_captured");
/// **A ESCUTA FOI CANCELADA** (o `Esc`) — emitido para que o `Esc` conste como consumido: sem ele,
/// cairia para o resto da cadeia e fecharia também um painel ou cancelaria uma ferramenta.
pub const INPUT_MAP_LISTEN_CANCELLED: NodeId = hash_node_id("input_map.listen_cancelled");
