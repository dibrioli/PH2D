//! ⭐ **`KanbanDocs` — a família instalada no hero**: os Kanbans do projecto (o MESMO `Rc` que o
//! `Documents` segura — ADR-0195 §2), as abas abertas (uma por Kanban, mais o início «Kanbans»), e o
//! desfazer de CADA Kanban (§5: `Ctrl+Z` num Kanban nunca desfaz outro nem a cena).

use crate::actions::Doc;
use crate::tab::Tab;
use ph2d_editor_core::documents::SharedKanbans;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, History, KanbanId, KanbanSet, ListId};
use ph2d_spring::Spring;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// ⚠️ SONDA (W4): o que a vista de um elemento do Quadro mostrou no último desenho.
#[derive(Clone, Debug, Default)]
pub struct CanvasProbe {
    /// Os títulos de lista pintados (como se pintam: MAIÚSCULAS).
    pub titles: Vec<String>,
    pub cards: Vec<(CardId, Rect)>,
    pub lists: Vec<(ListId, Rect)>,
    /// O cartão com o verso aberto.
    pub back: Option<CardId>,
    /// O texto do campo aberto no lugar (título, cartão novo).
    pub field: Option<String>,
}

/// O id da aba «Kanbans» (o início): nenhum `KanbanId` chega lá (contam de 1).
pub(crate) const HOME: u64 = u64::MAX;

pub struct KanbanDocs {
    pub(crate) set: SharedKanbans,
    /// Já segue o conjunto do projecto (o 1.º `follow_project` não fecha nada).
    adopted: bool,
    pub(crate) tabs: Vec<Tab>,
    pub(crate) home: Option<crate::home::Home>,
    pub(crate) histories: BTreeMap<KanbanId, History>,
    pub(crate) focus: Option<u64>,
    /// O relógio do chrome (ns) no último quadro, o do quadro anterior, e a lei das molas.
    pub(crate) clock: (u128, u128),
    pub(crate) law: Option<Spring>,
    /// A área da aba no último quadro.
    pub(crate) area: Rect,
    /// O chrome do hero por cima da área (o HUD de baixo): as listas param acima dele.
    pub(crate) over: Option<Rect>,
    /// Quem escreve (a identidade do Quadro; «Me» enquanto não há nenhuma) e as pessoas das equipas
    /// dos quadros do projecto — os membros e as menções do verso.
    pub(crate) me: ph2d_kanban_model::Member,
    pub(crate) team: Vec<String>,
    /// Avisos para o ecrã (um ficheiro que não se leu), servidos com a fila de fora.
    pub(crate) notices: Vec<String>,
    /// W4 — as vistas dos Kanbans no canvas do Quadro (partilhadas com os ganchos dele).
    pub(crate) canvas: crate::canvas::SharedCanvas,
    /// W6 — o cartão do `Ctrl+C` (o `Ctrl+V` cola uma cópia dele).
    pub(crate) clip: Option<ph2d_kanban_model::CardRef>,
    /// A última aba pintada (a activa: o «New Kanban from…» do menu File parte dela).
    pub(crate) active: Option<u64>,
}

impl Default for KanbanDocs {
    fn default() -> Self {
        Self {
            set: SharedKanbans::default(),
            adopted: false,
            tabs: Vec::new(),
            home: None,
            histories: BTreeMap::new(),
            focus: None,
            clock: (0, 0),
            law: Some(Spring::default()),
            area: Rect::new(0.0, 0.0, 0.0, 0.0),
            over: None,
            me: ph2d_kanban_model::Member(ph2d_i18n::tr("kanban.back.me").to_owned()),
            team: Vec::new(),
            notices: Vec::new(),
            canvas: crate::canvas::SharedCanvas::default(),
            clip: None,
            active: None,
        }
    }
}

/// A largura de um texto pintado em `SemiBold` (os títulos e a pílula do limite), medida no MESMO
/// peso em que o pintor o pinta: medir em `Medium` cortava «Short film — producti…» na fronteira.
/// Arredondada para cima, para a reticência nunca nascer de uma fracção.
pub(crate) fn title_width(ts: &mut TextSystem, text: &str, px: f32) -> f32 {
    ts.prefix_width_weighted(text, px, ph2d_text::FontWeight::SEMI_BOLD)
        .ceil()
}

impl KanbanDocs {
    /// ⭐ Segue o conjunto do projecto. Um `Rc` diferente do que segura = outro projecto (os ids
    /// recomeçam): as abas, o desfazer e as vistas do anterior largam-se.
    pub fn follow_project(&mut self, shared: SharedKanbans) {
        if std::rc::Rc::ptr_eq(&self.set, &shared) {
            return;
        }
        if self.adopted {
            self.tabs.clear();
            self.histories.clear();
            self.focus = None;
        }
        self.set = shared;
        self.adopted = true;
    }

    /// O relógio do chrome e a lei das molas (`None` = movimento reduzido).
    pub fn set_clock(&mut self, now_ns: u128, law: Option<Spring>) {
        self.clock = (
            now_ns,
            if self.clock.0 == 0 {
                now_ns
            } else {
                self.clock.0
            },
        );
        self.law = law;
    }

    /// Os segundos desde o quadro anterior.
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn dt(&self) -> f64 {
        self.clock.0.saturating_sub(self.clock.1) as f64 / 1e9
    }

    /// Abre (ou activa) o Kanban `id` numa aba.
    pub fn open(&mut self, id: KanbanId) {
        if self.set.borrow().get(id).is_none() {
            return;
        }
        if !self.tabs.iter().any(|t| t.view.kanban == id) {
            self.tabs.push(Tab::new(id));
        }
        self.focus = Some(id.0);
    }

    /// Cria um Kanban vazio, abre-o e põe o nome em edição (o idioma do Trello: o nome primeiro).
    pub fn create(&mut self) -> KanbanId {
        let n = self.set.borrow().iter().count() + 1;
        let name = ph2d_i18n::tr_with("kanban.default_name", &[("n", &n)]);
        let id = self.set.borrow_mut().create(&name);
        self.open(id);
        if let Some(t) = self.tab_mut(id.0) {
            t.field = Some(crate::edit::Field::open(
                crate::edit::Target::KanbanName,
                &name,
            ));
        }
        id
    }

    /// Abre o início «Kanbans».
    pub fn open_home(&mut self) {
        self.home.get_or_insert_with(Default::default);
        self.focus = Some(HOME);
    }

    pub(crate) fn tab_mut(&mut self, id: u64) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id() == id)
    }

    /// As abas de Kanban abertas (os ids).
    #[must_use]
    pub fn open_kanbans(&self) -> Vec<KanbanId> {
        self.tabs.iter().map(|t| t.view.kanban).collect()
    }

    /// O conjunto que a família segue (o do projecto).
    #[must_use]
    pub fn set(&self) -> SharedKanbans {
        self.set.clone()
    }

    /// Um gesto que escreve: a aba, o Kanban dela e o desfazer dele.
    pub(crate) fn with_doc<R>(
        &mut self,
        id: u64,
        f: impl FnOnce(&mut Tab, &mut Doc<'_>) -> R,
    ) -> Option<R> {
        let tab = self.tabs.iter_mut().find(|t| t.id() == id)?;
        let kid = tab.view.kanban;
        let mut set = self.set.borrow_mut();
        let history = self.histories.entry(kid).or_default();
        let mut doc = Doc {
            set: &mut set,
            history,
            id: kid,
        };
        Some(f(tab, &mut doc))
    }

    /// ⭐ Um quadro de imagem da aba `id`: a VISTA ([`crate::view::paint`]) à escala 1, no ecrã.
    pub(crate) fn paint_tab(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: Theme,
    ) {
        self.area = area;
        self.active = Some(id);
        let cx = self.view_ctx();
        let set = self.set.clone();
        let set = set.borrow();
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id() == id) else {
            return;
        };
        if set.get(tab.view.kanban).is_none() {
            crate::paint::missing(scene, ts, theme, area);
            return;
        }
        crate::smoke::hold(tab);
        let at = crate::view::At {
            to_screen: ph2d_vector::Affine::IDENTITY,
            area,
            lod: ph2d_board_render::KanbanLod::High,
            modal: true,
        };
        crate::view::paint(scene, ts, theme, at, tab, &set, &cx);
    }

    /// O que todas as vistas deste quadro de imagem partilham.
    pub(crate) fn view_ctx(&self) -> crate::view::ViewCtx {
        crate::view::ViewCtx {
            dt: self.dt(),
            law: self.law,
            me: self.me.clone(),
            team: self.team.clone(),
            floor: self.over.map(|o| o.y),
        }
    }

    /// Quem é «eu» (vazio = «Me») e as equipas dos quadros do projecto.
    pub fn follow_people(&mut self, author: &str, mut team: Vec<String>) {
        team.sort();
        team.dedup();
        self.team = team;
        if !author.is_empty() {
            self.me = ph2d_kanban_model::Member(author.to_owned());
        }
    }

    /// Abre o verso do cartão `card` na aba `id` (o clique e o `Enter` no quadro).
    pub fn open_back(&mut self, id: u64, card: CardId) {
        crate::back::open(self, id, card);
    }

    pub(crate) fn kanban_of(&self, id: u64) -> Option<KanbanId> {
        self.tabs
            .iter()
            .find(|t| t.id() == id)
            .map(|t| t.view.kanban)
    }

    /// Os Kanbans do projecto (o início lista-os).
    pub(crate) fn names(&self) -> Vec<(KanbanId, String)> {
        self.set
            .borrow()
            .iter()
            .map(|k| (k.id, k.header.name.clone()))
            .collect()
    }

    /// Fecha os campos e o arrasto de uma aba que deixou de estar activa (o texto escrito grava-se).
    pub(crate) fn leave(&mut self, id: u64) {
        crate::pointer::commit_field(self, id);
        if let Some(t) = self.tab_mut(id) {
            t.press = None;
            t.drag = None;
            t.hover = None;
            t.menu = None;
            if let Some(b) = t.back.as_deref_mut() {
                b.press = None;
            }
        }
    }

    /// A família sobre um conjunto próprio (os testes e as sondas que não montam o hero).
    #[must_use]
    pub fn with_set(set: KanbanSet) -> Self {
        let mut d = Self::default();
        d.follow_project(std::rc::Rc::new(std::cell::RefCell::new(set)));
        d
    }

    /// ⚠️ SONDA: a disposição VISTA da aba `id` no último quadro (os testes clicam aí).
    #[must_use]
    pub fn probe_card_rect(&self, id: u64, card: CardId) -> Option<Rect> {
        let t = self.tabs.iter().find(|t| t.id() == id)?;
        t.layout.card(card).map(|(_, c)| c.rect)
    }

    #[must_use]
    pub fn probe_list_rect(&self, id: u64, list: ListId) -> Option<Rect> {
        self.tabs
            .iter()
            .find(|t| t.id() == id)?
            .layout
            .list(list)
            .map(|l| l.rect)
    }

    /// ⚠️ SONDA: um rect nomeado da disposição da aba (`"add_list"`, `"title"`, `"star"`, `"menu"`).
    #[must_use]
    pub fn probe_rect(&self, id: u64, what: &str) -> Option<Rect> {
        let l = &self.tabs.iter().find(|t| t.id() == id)?.layout;
        Some(match what {
            "add_list" => l.add_list,
            "title" => l.title,
            "star" => l.star,
            "menu" => l.board_menu,
            _ => return None,
        })
    }

    /// ⚠️ SONDA: o «Add a card», o recolher e o «…» da lista `list`.
    #[must_use]
    pub fn probe_list_buttons(&self, id: u64, list: ListId) -> Option<(Rect, Rect, Rect)> {
        let l = self.tabs.iter().find(|t| t.id() == id)?.layout.list(list)?;
        Some((l.add, l.collapse, l.menu))
    }

    /// ⚠️ SONDA (W4): a vista do elemento `el` do Quadro — os títulos que pintou e os rects dos
    /// cartões e das listas, em px da VISTA.
    #[must_use]
    pub fn probe_canvas(&self, el: u64) -> Option<CanvasProbe> {
        let c = self.canvas.borrow();
        let t = c.tabs.get(&ph2d_board_model::ElementId(el))?;
        Some(CanvasProbe {
            titles: t
                .painted
                .borrow()
                .titles
                .iter()
                .map(|x| x.1.clone())
                .collect(),
            cards: t
                .layout
                .lists
                .iter()
                .flat_map(|l| l.cards.iter().map(|c| (c.id, c.rect)))
                .collect(),
            lists: t.layout.lists.iter().map(|l| (l.id, l.rect)).collect(),
            back: t.back.as_ref().map(|b| b.card),
            field: t.field.as_ref().map(|f| f.text.clone()),
        })
    }

    /// ⚠️ SONDA: as molas ainda se mexem?
    #[must_use]
    pub fn probe_moving(&self, id: u64) -> bool {
        self.tabs
            .iter()
            .find(|t| t.id() == id)
            .is_some_and(|t| t.springs.moving)
    }

    /// ⚠️ SONDA: o arrasto em curso cai em `(lista, índice)`.
    #[must_use]
    pub fn probe_drop(&self, id: u64) -> Option<(ListId, usize)> {
        self.tabs.iter().find(|t| t.id() == id)?.drag?.to_card
    }

    /// ⚠️ SONDA: a vista da aba (recolhidas, rolagem).
    #[must_use]
    pub fn probe_view(&self, id: u64) -> Option<ph2d_kanban_model::KanbanView> {
        self.tabs
            .iter()
            .find(|t| t.id() == id)
            .map(|t| t.view.clone())
    }

    /// ⚠️ SONDA: o cartão com o foco do teclado.
    #[must_use]
    pub fn probe_focus(&self, id: u64) -> Option<CardId> {
        self.tabs.iter().find(|t| t.id() == id)?.focus
    }

    /// ⚠️ SONDA: o texto do campo aberto.
    #[must_use]
    pub fn probe_field(&self, id: u64) -> Option<String> {
        self.tabs
            .iter()
            .find(|t| t.id() == id)?
            .field
            .as_ref()
            .map(|f| f.text.clone())
    }

    /// ⚠️ SONDA: os títulos de lista que o último quadro PINTOU — `(lista, texto mostrado, tamanho,
    /// peso)`.
    #[must_use]
    pub fn probe_painted_titles(&self, id: u64) -> Vec<(ListId, String, f32, f32)> {
        self.tabs
            .iter()
            .find(|t| t.id() == id)
            .map(|t| t.painted.borrow().titles.clone())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: a lista em voo que o último quadro PINTOU — `(lista, rect no ecrã antes de
    /// inclinar, inclinação °, cartões pintados dentro dela)`.
    #[must_use]
    pub fn probe_painted_flight(&self, id: u64) -> Option<(ListId, Rect, f64, Vec<CardId>)> {
        self.tabs
            .iter()
            .find(|t| t.id() == id)?
            .painted
            .borrow()
            .flight
            .clone()
    }

    /// ⚠️ SONDA: a inclinação de quem voa (graus).
    #[must_use]
    pub fn probe_tilt(&self, id: u64) -> f64 {
        self.tabs
            .iter()
            .find(|t| t.id() == id)
            .map_or(0.0, |t| t.springs.tilt.pos)
    }

    /// ⚠️ SONDA: o menu e o painel do arquivo estão abertos?
    #[must_use]
    pub fn probe_overlays(&self, id: u64) -> (bool, bool) {
        self.tabs
            .iter()
            .find(|t| t.id() == id)
            .map_or((false, false), |t| (t.menu.is_some(), t.panel.is_some()))
    }

    /// ⚠️ SONDA: as linhas do menu aberto, com o rect de cada uma.
    #[must_use]
    pub fn probe_menu_rows(&self, id: u64) -> Vec<(String, Rect)> {
        let Some(t) = self.tabs.iter().find(|t| t.id() == id) else {
            return Vec::new();
        };
        crate::overlay::menu_placed(t)
            .map(|p| {
                p.rows
                    .iter()
                    .map(|(r, rect)| (format!("{r:?}"), *rect))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: os botões do painel do arquivo aberto (`Close`, `ShowLists`, `Restore(..)`…), com o
    /// rect de cada um.
    #[must_use]
    pub fn probe_panel_rows(&self, id: u64, ts: &mut TextSystem) -> Vec<(String, Rect)> {
        use crate::archive::Row;
        let set = self.set.borrow();
        let Some(t) = self.tabs.iter().find(|t| t.id() == id) else {
            return Vec::new();
        };
        let Some(p) = set
            .get(t.view.kanban)
            .and_then(|k| crate::overlay::panel_placed(t, k, ts))
        else {
            return Vec::new();
        };
        let mut out = vec![
            (format!("{:?}", Row::Close), p.close),
            (format!("{:?}", Row::ShowCards), p.cards),
            (format!("{:?}", Row::ShowLists), p.lists),
        ];
        for i in &p.items {
            out.push((format!("{:?}", Row::Restore(i.id)), i.restore));
            out.push((format!("{:?}", Row::Delete(i.id)), i.delete));
        }
        out
    }
}
