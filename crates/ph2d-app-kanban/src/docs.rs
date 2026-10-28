//! ⭐ **`KanbanDocs` — a família instalada no hero**: os Kanbans do projecto (o MESMO `Rc` que o
//! `Documents` segura — ADR-0195 §2), as abas abertas (uma por Kanban, mais o início «Kanbans»), e o
//! desfazer de CADA Kanban (§5: `Ctrl+Z` num Kanban nunca desfaz outro nem a cena).

use crate::actions::Doc;
use crate::front::{self, Counts};
use crate::layout::{self, Heights, Layout, Lifted};
use crate::metrics as m;
use crate::paint_card::CardLook;
use crate::tab::{Lift, Tab};
use ph2d_editor_core::documents::SharedKanbans;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, History, Kanban, KanbanId, KanbanSet, ListId};
use ph2d_spring::Spring;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

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
        }
    }
}

/// O medidor de alturas que a disposição pede, sobre o texto e a cache da aba.
struct Measure<'a> {
    ts: &'a mut TextSystem,
    cache: &'a mut BTreeMap<CardId, (u64, f32)>,
    counts: &'a Counts,
    label_text: bool,
}

impl Heights for Measure<'_> {
    fn card(&mut self, k: &Kanban, id: CardId) -> f32 {
        front::front(k, self.counts, id).map_or(m::LINE_H, |f| {
            front::height(self.ts, self.cache, id, &f, self.label_text)
        })
    }

    #[allow(clippy::cast_precision_loss)]
    fn field(&mut self, text: &str, width: f32) -> f32 {
        front::title_lines(self.ts, text, width) as f32 * m::LINE_H
    }

    fn title_w(&mut self, text: &str) -> f32 {
        crate::list_title::width(self.ts, text)
    }

    fn limit_w(&mut self, text: &str) -> f32 {
        title_width(self.ts, text, m::SMALL_PX) + 2.0 * m::COUNT_PAD_X
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

    /// ⭐ Um quadro de imagem da aba `id`: dispõe (alvos → molas → ecrã), segue o arrasto (o alvo e
    /// a rolagem nas bordas) e pinta.
    pub(crate) fn paint_tab(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: Theme,
    ) {
        self.area = area;
        let dt = self.dt();
        let law = self.law;
        let set = self.set.clone();
        let set = set.borrow();
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id() == id) else {
            return;
        };
        let Some(k) = set.get(tab.view.kanban) else {
            crate::paint::missing(scene, ts, theme, area);
            return;
        };
        crate::smoke::hold(tab);
        edge_scroll(tab, dt);
        *tab.painted.borrow_mut() = crate::tab::Painted::default();
        let counts = Counts::of(k).seen_by(Some(self.me.clone())).with_files(&set, k);
        let title_w = title_width(ts, &k.header.name, m::HEADER_TITLE_PX) + 2.0 * m::FIELD_PAD_X;
        let floor = self.over.map(|o| o.y);
        let lifted = lifted(tab, &counts, k, ts);
        let mut cache = std::mem::take(&mut tab.heights);
        let target = {
            let mut measure = Measure {
                ts,
                cache: &mut cache,
                counts: &counts,
                label_text: tab.label_text,
            };
            layout::targets(
                k,
                &tab.view,
                area,
                (title_w, floor),
                tab.field.as_ref(),
                lifted,
                &mut measure,
            )
        };
        tab.heights = cache;
        clamp_scroll(tab, &target);
        let seen = crate::tab::place(tab, target.clone(), dt, law);
        tab.layout = seen;
        retarget(tab, &target);
        let mut looks = |cid: CardId| {
            front::front(k, &counts, cid).map(|f| CardLook {
                front: f,
                hover: false,
                focus: false,
            })
        };
        crate::paint::board(scene, ts, theme, k, tab, &mut looks);
        crate::overlay::paint(scene, ts, theme, k, tab);
        crate::back::paint(
            scene,
            ts,
            theme,
            k,
            tab,
            &set,
            &self.me,
            &self.team,
            crate::actions::now(),
        );
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

/// O que voa sai do fluxo da disposição e abre o espaço dele onde cai.
fn lifted(tab: &mut Tab, counts: &Counts, k: &Kanban, ts: &mut TextSystem) -> Option<Lifted> {
    let d = tab.drag?;
    match d.lift {
        Lift::Card(id) => {
            let h = front::front(k, counts, id).map_or(d.size.1, |f| {
                front::height(ts, &mut tab.heights, id, &f, tab.label_text)
            });
            Some(Lifted::Card {
                id,
                to: d.to_card,
                h,
            })
        }
        Lift::List(id) => Some(Lifted::List {
            id,
            to: d.to_list,
            w: d.size.0,
            h: d.size.1,
        }),
        Lift::Pan { .. } => None,
    }
}

/// A rolagem não passa do fim (uma lista que encolheu, o quadro que estreitou).
fn clamp_scroll(tab: &mut Tab, target: &Layout) {
    let max_x = layout::max_scroll_x(target);
    tab.view.scroll_x = tab.view.scroll_x.clamp(0.0, f64::from(max_x));
    for lb in &target.lists {
        let max_y = f64::from(layout::max_scroll_y(lb));
        if let Some(v) = tab.view.scroll_y.get_mut(&lb.id) {
            *v = v.clamp(0.0, max_y);
        }
    }
}

/// A fundura `0 → 1` de `p` na faixa junto à borda `[lo, hi]` (negativa = a de baixo/esquerda).
fn edge(p: f32, lo: f32, hi: f32) -> f32 {
    let band = m::EDGE_SCROLL_BAND.min((hi - lo) / 3.0).max(1.0);
    if p < lo + band {
        -((lo + band - p) / band).min(1.0)
    } else if p > hi - band {
        ((p - (hi - band)) / band).min(1.0)
    } else {
        0.0
    }
}

/// ⭐ A rolagem nas bordas durante um arrasto (o Trello): o quadro na horizontal, a lista por baixo
/// do ponteiro na vertical; mais depressa quanto mais fundo na faixa.
#[allow(clippy::cast_possible_truncation)]
fn edge_scroll(tab: &mut Tab, dt: f64) {
    let Some(d) = tab.drag else {
        return;
    };
    if matches!(d.lift, Lift::Pan { .. }) {
        return;
    }
    let speed = f64::from(m::EDGE_SCROLL_MAX_PX_S) * dt;
    let b = tab.layout.board;
    let ex = edge(d.at.0, b.x, b.x + b.w);
    tab.view.scroll_x += f64::from(ex) * speed;
    if let Lift::Card(_) = d.lift
        && let Some(lb) = tab
            .layout
            .lists
            .iter()
            .find(|l| !l.collapsed && l.rect.x <= d.at.0 && d.at.0 <= l.rect.x + l.rect.w)
    {
        let ey = edge(d.at.1, lb.body.y, lb.body.y + lb.body.h);
        if ey != 0.0 && (lb.body.y..=lb.body.y + lb.body.h).contains(&d.at.1) {
            *tab.view.scroll_y.entry(lb.id).or_default() += f64::from(ey) * speed;
        }
    }
}

/// ⭐ Onde cai o que voa — medido nos ALVOS sem o espaço aberto (senão o espaço que abre empurra a
/// fronteira e o alvo oscila entre dois sítios).
fn retarget(tab: &mut Tab, target: &Layout) {
    let Some(mut d) = tab.drag else {
        return;
    };
    let ox = target.board.x + m::BOARD_PAD;
    #[allow(clippy::cast_possible_truncation)]
    let sx = tab.view.scroll_x as f32;
    let cx = d.at.0 - d.grab.0 + d.size.0 / 2.0 - ox + sx;
    match d.lift {
        Lift::Card(id) => {
            let open: Vec<_> = target.lists.iter().filter(|l| !l.collapsed).collect();
            let Some(lb) = open
                .iter()
                .find(|l| cx < l.rect.x + l.rect.w + m::LIST_GAP / 2.0)
                .or(open.last())
            else {
                d.to_card = None;
                tab.drag = Some(d);
                return;
            };
            let sy = tab.scroll_y(lb.id);
            let cy = d.at.1 - d.grab.1 + d.size.1 / 2.0 - (lb.body.y + m::LIST_BODY_TOP) + sy;
            let mut acc = 0.0_f32;
            let mut index = 0usize;
            for c in lb.cards.iter().filter(|c| c.id != id) {
                if acc + c.rect.h / 2.0 > cy {
                    break;
                }
                acc += c.rect.h + m::CARD_GAP;
                index += 1;
            }
            d.to_card = Some((lb.id, index));
        }
        Lift::List(id) => {
            let mut acc = 0.0_f32;
            let mut index = 0usize;
            for l in target.lists.iter().filter(|l| l.id != id) {
                if acc + l.rect.w / 2.0 > cx {
                    break;
                }
                acc += l.rect.w + m::LIST_GAP;
                index += 1;
            }
            d.to_list = index;
        }
        Lift::Pan { .. } => {}
    }
    tab.drag = Some(d);
}
