//! Ids do **Laboratório de Widgets** — a bancada onde os desenhos novos se estudam ANTES de
//! tocarem no app (pedido do Enio, 2026-09-01: *"criar um painel de testes como fizemos com
//! Widget Gallery … vamos fazer nossos estudos num painel antes de sair mudando tudo"*).
//!
//! ⚠️ **Ficheiro PRÓPRIO, e a razão é de isolamento (ADR-0107).** Uma linha paralela que
//! acrescente ids ao `gallery.rs` colide textualmente com outra que faça o mesmo; um módulo irmão
//! append-only não colide com ninguém. É o mesmo motivo pelo qual o `ph2d-panel-widget-lab` é
//! crate nova em vez de uma secção da galeria.
//!
//! ⛔ **A galeria e o laboratório NÃO são o mesmo painel, de propósito.** A galeria mostra o que o
//! editor **é hoje** — é a fonte única de verdade que os agentes periféricos copiam. O laboratório
//! mostra o que ele **pode vir a ser**, com variantes lado a lado que a maioria vai ser deitada
//! fora. *Misturá-los faria a fonte de verdade passar a conter propostas.*

use super::*;

// ── A janela ───────────────────────────────────────────────────────────────
/// A janela flutuante do laboratório. Irmã de [`GAL_PANEL`](super::GAL_PANEL).
pub const LAB_PANEL: NodeId = hash_node_id("lab_panel");
/// Abre/fecha o laboratório — a linha do menu *Window*.
pub const TOPBAR_WIDGET_LAB: NodeId = hash_node_id("topbar_widget_lab");
