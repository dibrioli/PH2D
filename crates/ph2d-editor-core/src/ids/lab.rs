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
pub const LAB_DRAG_HANDLE: NodeId = hash_node_id("lab_drag_handle");
pub const LAB_RESIZE_HANDLE: NodeId = hash_node_id("lab_resize_handle");
pub const LAB_RESIZE_HANDLE_BL: NodeId = hash_node_id("lab_resize_handle_bl");
pub const LAB_CLOSE: NodeId = hash_node_id("lab_close");

// ── Os controlos da bancada ────────────────────────────────────────────────
// ⚠️ Cada um destes tem de ser LIDO por alguém que decide, senão é um knob morto — a espécie
// que o `CLAUDE.md` §5.0 descreve e que esta linha já apanhou uma vez. O consumidor de todos
// eles é o `study::paint_study`, e há gate a prová-lo.
/// Anda para o desenho seguinte da lista de variantes.
pub const LAB_VARIANT_NEXT: NodeId = hash_node_id("lab_variant_next");
/// Anda para o desenho anterior.
pub const LAB_VARIANT_PREV: NodeId = hash_node_id("lab_variant_prev");
/// Cicla o acento (a cor com que o preenchimento é pintado).
pub const LAB_ACCENT_CYCLE: NodeId = hash_node_id("lab_accent_cycle");
/// Cicla a densidade (altura de linha) — `Compact` · `Cozy` · `Comfortable`.
pub const LAB_DENSITY_CYCLE: NodeId = hash_node_id("lab_density_cycle");
/// Liga/desliga a coluna de animação (o *decorator* do Blender).
/// ⭐ Enio, 2026-09-01: *"Sim. Em todas as propriedades que podem ser animadas e nessa engine vou
/// querer animar tudo."*
pub const LAB_DECORATOR_TOGGLE: NodeId = hash_node_id("lab_decorator_toggle");
/// Liga/desliga a fileira de comparação com o widget de HOJE (rótulo | trilho | caixa).
pub const LAB_COMPARE_TOGGLE: NodeId = hash_node_id("lab_compare_toggle");
/// Cicla o raio do canto — `16` (hoje) · `8` · `4` (Godot) · `0`.
pub const LAB_RADIUS_CYCLE: NodeId = hash_node_id("lab_radius_cycle");

/// **A caixa VIVA** — a única do laboratório que aceita arrasto, para se sentir o gesto.
///
/// ⚠️ Todas as outras amostras são pintura sem interacção: elas existem para se **ver**. Uma
/// bancada em que 40 caixas competem pelo ponteiro mede a confusão, não o desenho.
pub const LAB_LIVE_BOX: NodeId = hash_node_id("lab_live_box");
