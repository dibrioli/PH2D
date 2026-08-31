//! ⭐⭐ **ONDE CADA PAINEL ESTÁ, e o arrasto que o move** — o estado que faz da decisão **D4** um
//! valor em vez de uma constante.
//!
//! > *«Lugares pré-definidos. O artista escolhe **QUAL painel vai em cada lugar**, e arrasta a
//! > divisória — mas não inventa lugares novos.»* — `00_DECISOES_DO_ENIO.md`, D4
//!
//! ⚠️ **O `Panel::DEFAULT_SLOT` continua a ser a resposta de omissão, e só isso.** Este mapa guarda
//! **as excepções** — quem o artista mudou. Um mapa que guardasse todos os painéis teria de ser
//! semeado no arranque e reconciliado a cada painel novo; guardando só as excepções, um painel que
//! nasce amanhã vai para onde ele próprio declara, sem uma linha de migração.
//!
//! # ⛔ Por que o arrasto tem TRÊS estados e não um booleano
//!
//! | estado | o que significa |
//! |---|---|
//! | `tab_drag` com o cursor **perto** do início | o dedo desceu e ainda não decidiu: isto ainda pode ser um **clique** que troca de aba |
//! | `tab_drag` com o cursor **longe** | é um arrasto; as zonas de largada pintam-se |
//! | `tab_drop` | largou; o **hero** resolve, porque só ele tem o layout e os encaixes permitidos |
//!
//! ⚠️ **A resolução NÃO pode viver aqui**: ela precisa do rect de cada encaixe (o layout) e do
//! `ALLOWED_SLOTS` de cada painel (o registry) — e o `WidgetStore` não conhece nenhum dos dois. Ele
//! guarda **onde o dedo largou**; quem decide é `screens::hero::slot_tabs::resolve_tab_drop`.
//!
//! ⭐ E resolver no quadro seguinte é o **correcto**, não um compromisso: o drop tem de ser julgado
//! contra a geometria que o artista estava a ver quando largou.

use super::WidgetStore;
use crate::screens::slot::Slot;
use ph2d_a11y::NodeId;

/// Quanto o dedo tem de andar para um toque numa aba deixar de ser um clique.
///
/// ⚠️ É o mesmo número do arrasto de um campo numérico
/// ([`crate::interaction::drag::NUMBER_INPUT_DRAG_THRESHOLD_PX`]) e por a mesma razão: *um terceiro
/// limiar no mesmo app é onde a mão aprende que cada widget tem regras próprias.*
pub const TAB_DRAG_THRESHOLD_PX: f32 = crate::interaction::drag::NUMBER_INPUT_DRAG_THRESHOLD_PX;

/// **Uma aba com o dedo em cima** — o painel, onde o dedo desceu, e onde ele está.
///
/// ⚠️ O início fica guardado porque é a **distância a ele** que separa um clique (trocar de aba) de
/// um arrasto (mudar de encaixe). Um booleano *«está a arrastar»* teria de ser escrito por quem
/// mede a distância, e aí o limiar viveria no despacho em vez de viver aqui.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TabDragAnchor {
    pub panel: NodeId,
    pub start: (f32, f32),
    pub cursor: (f32, f32),
}

impl WidgetStore {
    /// **Em que encaixe este painel está** — `None` quando o artista nunca o moveu, e nesse caso
    /// quem responde é o `Panel::DEFAULT_SLOT`. Ver `slot_tabs::slot_of`.
    #[must_use]
    pub fn panel_slot(&self, panel: NodeId) -> Option<Slot> {
        self.panel_slot.get(&panel).copied()
    }

    /// Move um painel para um encaixe. ⚠️ **Não valida nada** — a legalidade é do `ALLOWED_SLOTS`,
    /// e quem a conhece é o registry; ver `slot_tabs::resolve_tab_drop`, que é o único chamador do
    /// produto.
    pub fn set_panel_slot(&mut self, panel: NodeId, slot: Slot) {
        self.panel_slot.insert(panel, slot);
    }

    /// ⭐ **Apaga as excepções** — o encaixe de cada painel e a largura de cada coluna voltam ao
    /// que o produto declara. Ver `slot_tabs::reset`, que é a porta do produto.
    pub fn reset_panel_layout(&mut self) {
        self.reset_panel_slots();
        self.dock_w_left = None;
        self.dock_w_right = None;
        self.dock_h_bottom = None;
    }

    /// Só os encaixes. ⚠️ **A largura fica**, e a distinção é o que trocar de layout precisa: ela é
    /// a medida da MÃO de quem usa o ecrã, não da tarefa — ver `hero::layout_switch`.
    pub fn reset_panel_slots(&mut self) {
        self.panel_slot.clear();
    }

    /// **Qual layout por tarefa está activo** (decisão D7).
    #[must_use]
    pub fn active_layout(&self) -> crate::screens::task_layout::TaskLayout {
        self.active_layout
    }

    /// Escreve-o. ⚠️ A porta do produto é `hero::layout_switch::apply`, que também arruma a tela —
    /// escrever só isto deixaria a barra a dizer uma coisa e o ecrã a mostrar outra.
    pub fn set_active_layout(&mut self, layout: crate::screens::task_layout::TaskLayout) {
        self.active_layout = layout;
    }

    /// O dedo desceu sobre a aba de `panel`, em `(x, y)`.
    pub fn begin_tab_drag(&mut self, panel: NodeId, x: f32, y: f32) {
        self.tab_drag = Some(TabDragAnchor {
            panel,
            start: (x, y),
            cursor: (x, y),
        });
    }

    /// O dedo moveu-se. No-op sem arrasto em curso.
    pub fn update_tab_drag(&mut self, x: f32, y: f32) {
        if let Some(a) = self.tab_drag.as_mut() {
            a.cursor = (x, y);
        }
    }

    /// **A aba que está a ser ARRASTADA**, e onde o cursor está — `None` enquanto o dedo não passou
    /// o limiar, que é o que mantém o clique simples a funcionar.
    #[must_use]
    pub fn tab_being_dragged(&self) -> Option<(NodeId, (f32, f32))> {
        let a = self.tab_drag?;
        let (dx, dy) = (a.cursor.0 - a.start.0, a.cursor.1 - a.start.1);
        if dx.hypot(dy) < TAB_DRAG_THRESHOLD_PX {
            return None;
        }
        Some((a.panel, a.cursor))
    }

    /// O dedo subiu. Se havia um ARRASTO (e não um toque parado), publica a largada para o hero
    /// resolver.
    ///
    /// ⚠️ **Não devolve nada, e isso é uma decisão medida:** a versão anterior devolvia *«foi um
    /// arrasto?»* para o despacho suprimir o clique — e a supressão matava o **empurrão de 5 px**,
    /// fazendo a troca de aba depender da firmeza da mão. Ver o comentário no `pointer_up`.
    pub fn end_tab_drag(&mut self) {
        if let Some((panel, cursor)) = self.tab_being_dragged() {
            self.tab_drop = Some((panel, cursor));
        }
        self.tab_drag = None;
    }

    /// O hero consome a largada — **uma vez**. ⚠️ `take`, não um leitor: uma largada relida no
    /// quadro seguinte moveria o painel outra vez para onde o cursor já não está.
    pub fn take_tab_drop(&mut self) -> Option<(NodeId, (f32, f32))> {
        self.tab_drop.take()
    }
}
