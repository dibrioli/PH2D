//! ⭐ **A lista de cartões de um painel** (`docs/CartoesUI/02_especificacao.md` §4): o painel é uma
//! lista do Kanban e cada secção um cartão que se arrasta pelo cabeçalho. O DESPACHO só regista a
//! mão (`CardDrag`); a PINTURA (`panel::cards`) conduz o `ph2d_card_stack::Stack` — é ela que
//! conhece os rects — e grava a ordem nova na mesma lista das secções (`sections.txt`, `order=`).

use super::WidgetStore;
use ph2d_a11y::NodeId;
use ph2d_card_stack::{Item, Stack};
use std::collections::BTreeMap;

/// A mão sobre o cabeçalho de um cartão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CardDrag {
    pub section: NodeId,
    pub down: (f32, f32),
    pub at: (f32, f32),
    /// Passou o limiar — antes disso um Down+Up é um clique (dobra o cartão).
    pub active: bool,
    /// A mão caiu no cartão SOLTO desta secção (T40: a lista pode mostrar a mesma secção ao lado).
    pub loose: bool,
    /// T39: a mão agarrou uma BORDA do solto — muda a largura, não o lugar.
    pub edge: Option<super::LooseEdge>,
}

/// Como acabou um arrasto activo (consumido pela pintura do painel do cartão).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CardDrop {
    pub section: NodeId,
    pub cancelled: bool,
    pub loose: bool,
    pub edge: Option<super::LooseEdge>,
}

/// O estado VISTO de uma lista (por painel): as molas, as alturas do último quadro, o relógio.
#[derive(Debug, Default)]
pub struct CardList {
    pub stack: Stack<NodeId>,
    pub items: Vec<Item<NodeId>>,
    pub last: Option<std::time::Instant>,
    /// A altura das notas SEM cartão (no fim da lista) no quadro anterior.
    pub loose_notes_h: f32,
}

/// O que os cartões guardam no store.
#[derive(Debug, Default)]
pub struct CardLists {
    pub(super) lists: BTreeMap<NodeId, CardList>,
    pub(super) drag: Option<CardDrag>,
    pub(super) dropped: Option<CardDrop>,
    /// As secções que são CARTÕES neste quadro (registadas pela pintura) — o despacho distingue-as
    /// das secções de pega do Inspector de hoje.
    pub(super) sections: std::collections::BTreeSet<NodeId>,
    /// O ícone de menu ao lado do título de cada lista (D21): botão → (painel, rect do botão).
    pub(super) menus: BTreeMap<NodeId, (NodeId, crate::zones::Rect)>,
    /// As linhas da lista de estilos de cor ABERTA (D28/D30): linha → estilo. A lista publica-as
    /// antes de pintar; o botão direito numa delas abre o menu dela sem o despacho ver o painel.
    pub(super) style_rows: BTreeMap<NodeId, u32>,
    /// C5: os cartões soltos no canvas (secção → onde), a profundidade e os rects publicados.
    pub(super) loose: BTreeMap<NodeId, super::LooseCard>,
    pub(super) loose_z: u32,
    pub(super) loose_rects: std::collections::BTreeSet<NodeId>,
    /// O painel escondido por uma aba que pinta SÓ os soltos neste passo (`panel_walk`).
    pub(super) loose_only: Option<NodeId>,
    /// T7: o sujeito de agora por painel, os nomes por sujeito, as secções com sujeito, o título
    /// de cada solto e o sujeito preso sob a última pressão (`card_subject_ops`).
    pub(super) subjects: BTreeMap<NodeId, u64>,
    pub(super) names: BTreeMap<u64, String>,
    pub(super) pinnable: std::collections::BTreeSet<NodeId>,
    pub(super) titles: BTreeMap<NodeId, String>,
    pub(super) press_subject: Option<u64>,
    /// T41: de onde (ecrã) regressa cada solto devolvido à lista — a pintura consome-o e a mola parte daí.
    pub(super) arrivals: BTreeMap<NodeId, (f32, f32)>,
    /// T40: os números do sujeito PRESO de cada solto (o mesmo id mostra outro valor na lista).
    pub(super) pinned_values: BTreeMap<NodeId, Vec<(NodeId, f64)>>,
    /// T61: os estados do sujeito PRESO de cada solto do Inspector (`card_ensaio_ops`).
    pub(super) pinned_states: BTreeMap<NodeId, super::EstadosPresos>,
    /// T44: a corda de cada solto preso com corda — pontos em ecrã, do cartão ao objecto.
    pub(super) ropes: BTreeMap<NodeId, Vec<[f32; 2]>>,
    /// O alfinete DURÁVEL de cada solto preso: `(StableId, nome)` do sujeito (`card_subject_ops`).
    pub(super) pin_keys: BTreeMap<NodeId, (u64, String)>,
    /// T41: a corda que se soltou a recolher-se — `(painel, pontos em ecrã)`.
    pub(super) reels: BTreeMap<NodeId, (NodeId, Vec<[f32; 2]>)>,
    /// T53: o solto com alfinete que a mão trouxe de volta à coluna — suspenso (alfinete, corda e
    /// chave guardados) até largar: encaixou ⇒ esquece; largou fora ⇒ volta igual.
    pub(super) suspended: Option<(NodeId, super::LooseCard)>,
    /// T60: o ↺ de um cartão de componente — `(entidade, nome canónico)` à espera da app.
    pub(super) resets: Vec<(u64, &'static str)>,
}

impl WidgetStore {
    pub fn card_list_mut(&mut self, panel: NodeId) -> &mut CardList {
        self.card_lists.lists.entry(panel).or_default()
    }

    /// T60: o ↺ de um cartão de componente pede que `component` volte ao de fábrica em `entity`.
    pub fn request_component_reset(&mut self, entity: u64, component: &'static str) {
        self.card_lists.resets.push((entity, component));
    }

    /// Os pedidos de reposição deste quadro (quem os atende é a app de componentes).
    pub fn take_component_resets(&mut self) -> Vec<(u64, &'static str)> {
        std::mem::take(&mut self.card_lists.resets)
    }

    pub fn mark_card_section(&mut self, section: NodeId) {
        self.card_lists.sections.insert(section);
    }

    #[must_use]
    pub fn is_card_section(&self, section: NodeId) -> bool {
        self.card_lists
            .sections
            .contains(&self.loose_section(section))
    }

    #[must_use]
    pub fn card_drag(&self) -> Option<CardDrag> {
        self.card_lists.drag
    }

    pub fn begin_card_drag(&mut self, section: NodeId, x: f32, y: f32) {
        let loose = self
            .panel_rect(super::loose_rect_of(section))
            .is_some_and(|r| r.contains(x, y));
        self.card_lists.drag = Some(CardDrag {
            section,
            down: (x, y),
            at: (x, y),
            active: false,
            loose,
            edge: None,
        });
    }

    /// A mão anda; `true` = há um arrasto de cartão (o despacho não passa o movimento adiante).
    pub fn update_card_drag(&mut self, x: f32, y: f32) -> bool {
        let Some(d) = self.card_lists.drag.as_mut() else {
            return false;
        };
        d.at = (x, y);
        let (dx, dy) = (x - d.down.0, y - d.down.1);
        let thr = crate::interaction::drag::DRAG_THRESHOLD_PX;
        d.active |= dx * dx + dy * dy >= thr * thr;
        true
    }

    /// Solta (ou cancela, Esc). Devolve o arrasto; um activo fica à espera da pintura.
    pub fn end_card_drag(&mut self, cancelled: bool) -> Option<CardDrag> {
        let d = self.card_lists.drag.take()?;
        if d.active {
            self.card_lists.dropped = Some(CardDrop {
                section: d.section,
                cancelled,
                loose: d.loose,
                edge: d.edge,
            });
        }
        Some(d)
    }

    /// O arrasto `d` é de `panel`: a secção está na lista dele, ou a mão leva um SOLTO dele — que anda
    /// com qualquer seleção e com a aba atrás (smoke do dono 02/11, T55).
    #[must_use]
    pub fn card_drag_is_of(&self, d: &CardDrag, panel: NodeId, sections: &[NodeId]) -> bool {
        (d.loose && self.is_loose_in(d.section, panel))
            || sections.contains(&self.loose_section(d.section))
    }

    /// Regista o ícone de menu de `panel` (a pintura, todo quadro).
    pub fn mark_card_list_menu(&mut self, button: NodeId, panel: NodeId, at: crate::zones::Rect) {
        self.card_lists.menus.insert(button, (panel, at));
    }

    /// Publica as linhas da lista de estilos de cor aberta (vazia = fechada), todo quadro.
    pub fn set_color_style_rows(&mut self, rows: impl IntoIterator<Item = (NodeId, u32)>) {
        self.card_lists.style_rows = rows.into_iter().collect();
    }

    /// O estilo da linha `row` da lista de estilos aberta, se é uma.
    #[must_use]
    pub fn color_style_row(&self, row: NodeId) -> Option<u32> {
        self.card_lists.style_rows.get(&row).copied()
    }

    /// O painel e o rect do ícone de menu `button`, se é um.
    #[must_use]
    pub fn card_list_menu(&self, button: NodeId) -> Option<(NodeId, crate::zones::Rect)> {
        self.card_lists.menus.get(&button).copied()
    }

    /// ⭐ **Repõe os cartões de `panel`** (D21): `order` = a ordem natural do painel; `colors` = sem
    /// tema nem contorno; `folds` = cada cartão volta a dobrar como o painel o semeia. As notas são
    /// conteúdo do artista e não se tocam.
    pub fn reset_card_list(&mut self, panel: NodeId, order: bool, colors: bool, folds: bool) {
        let cartoes: Vec<NodeId> = self
            .card_lists
            .lists
            .get(&panel)
            .map(|l| l.items.iter().map(|i| i.key).collect())
            .unwrap_or_default();
        if order {
            self.return_loose_cards_of(panel);
            let resto: Vec<NodeId> = self
                .section_order()
                .iter()
                .copied()
                .filter(|k| !cartoes.contains(k))
                .collect();
            self.set_section_order(resto);
        }
        let faces = self.loose_cards_of(panel).into_iter();
        let faces: Vec<NodeId> = faces.map(|(k, _)| super::loose_face_of(k)).collect();
        for s in cartoes
            .iter()
            .chain(colors.then_some(&faces).into_iter().flatten())
        {
            if colors {
                self.set_section_theme(*s, None);
                self.set_section_outline_color(*s, None);
            }
            if folds {
                self.collapsed.remove(s);
            }
        }
    }

    /// A queda de `section`, se ela é desta lista (a pintura consome-a).
    pub fn take_card_drop(&mut self, panel: NodeId, sections: &[NodeId]) -> Option<CardDrop> {
        let d = self.card_lists.dropped?;
        let solto = d.loose && self.is_loose_in(d.section, panel);
        if !solto && !sections.contains(&self.loose_section(d.section)) {
            return None;
        }
        self.card_lists.dropped = None;
        Some(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_tokens::Theme;

    /// ⭐ D21: cada linha do menu do painel repõe só o que diz, e só nos cartões DAQUELE painel.
    #[test]
    fn the_panel_menu_resets_only_what_it_names() {
        let (painel, a, b, alheio) = (NodeId(1), NodeId(10), NodeId(11), NodeId(99));
        let mut s = WidgetStore::default();
        s.card_list_mut(painel).items = vec![Item { key: a, h: 1.0 }, Item { key: b, h: 1.0 }];
        let de_novo = |s: &mut WidgetStore| {
            s.set_section_order(vec![alheio, b, a]);
            s.set_section_theme(a, Some(Theme::Dark));
            s.set_section_theme(alheio, Some(Theme::Dark));
            s.set_section_outline_color(b, Some(2));
            s.set_collapsed(a, true);
        };
        de_novo(&mut s);
        s.reset_card_list(painel, false, true, false);
        assert_eq!(s.section_theme(a), None);
        assert_eq!(s.section_outline_color(b), None);
        assert_eq!(s.section_theme(alheio), Some(Theme::Dark));
        assert_eq!(s.section_order(), &[alheio, b, a]);
        assert!(s.is_collapsed(a));

        de_novo(&mut s);
        s.reset_card_list(painel, true, false, false);
        assert_eq!(s.section_order(), &[alheio]);
        assert_eq!(s.section_theme(a), Some(Theme::Dark));

        de_novo(&mut s);
        s.reset_card_list(painel, true, true, true);
        assert_eq!(s.section_order(), &[alheio]);
        assert_eq!(s.section_theme(a), None);
        assert_eq!(s.collapsed_choice(a), None);
    }
}
