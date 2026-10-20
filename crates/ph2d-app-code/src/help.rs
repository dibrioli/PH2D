//! ⭐ **As abas da página de ajuda** (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.4): moram na MESMA
//! família dos scripts (no Godot a página é irmã do script na lista). O id da aba é
//! [`HELP_TAG`]` | <índice da página no catálogo>` — um bit que nenhum [`ScriptId`] usa. Uma página
//! abre-se uma vez: o Ctrl+clique seguinte na mesma biblioteca reaproveita a aba e rola-a.
//!
//! [`ScriptId`]: ph2d_script::ScriptId

use crate::docs::CodeDocs;
use crate::help_doc::{Target, build};
use crate::nav::Place;
use ph2d_code_buffer::{Buffer, FindOptions, Text, find_all};
use ph2d_code_help::{Anchor, catalog};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::help::HelpDoc;
use ph2d_panel_code::{FindBar, FindCount, FindField, HelpFrame, HelpHit, HelpProbe, HelpView};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;
use std::ops::Range;

/// O bit das abas de ajuda no id do hero (um `ScriptId` nunca chega lá).
pub const HELP_TAG: u64 = 1 << 62;

/// O id da aba da página `page` (o índice no catálogo).
#[must_use]
pub fn tab_id(page: usize) -> u64 {
    HELP_TAG | page as u64
}

/// A página de uma aba de ajuda; `None` = é um script.
#[must_use]
pub fn page_of(id: u64) -> Option<usize> {
    (id & HELP_TAG != 0).then_some((id & !HELP_TAG) as usize)
}

/// O bloco dos ids do leitor de ecrã das páginas (os dos scripts são `0xC0DE… ^ (id << 32)`).
const A11Y_SALT: u64 = 0x4E1F_0000_0000_0000;

/// O nó `Document` da página `page`.
#[must_use]
pub fn help_a11y_base(page: usize) -> ph2d_a11y::NodeId {
    ph2d_a11y::NodeId(A11Y_SALT ^ ((page as u64) << 32))
}

/// O índice da página `id` no catálogo.
#[must_use]
pub fn page_index(id: &str) -> Option<usize> {
    catalog().iter().position(|p| p.id == id)
}

/// O número com que a página `page` ancora o membro `member` (o `top_member` da sonda).
#[must_use]
pub fn member_id(page: &str, member: &str) -> Option<u32> {
    let p = ph2d_code_help::page(page)?;
    p.member(member).map(|i| i as u32)
}

/// ⚠️ SONDA (a bancada): a página `page` do catálogo como a aba a dispõe e pinta.
#[must_use]
pub fn help_doc(page: &str) -> Option<HelpDoc> {
    Some(build(&catalog()[page_index(page)?]).doc)
}

/// A busca na página (Ctrl+F): o campo, as ocorrências no texto da página e a corrente.
#[derive(Default)]
pub(crate) struct HelpFind {
    pub open: bool,
    /// O campo tem o teclado.
    pub focus: bool,
    pub query: Buffer,
    pub case: bool,
    pub word: bool,
    pub matches: Vec<Range<usize>>,
    pub current: Option<usize>,
    /// O que as `matches` seguem: (busca, opções, texto da página).
    key: Option<(String, bool, bool, u64, usize)>,
}

impl HelpFind {
    /// Refaz as ocorrências se a busca, as opções ou o texto da página mudaram.
    pub fn refresh(&mut self, doc_key: u64, plain: &str) {
        let key = (
            self.query.text().to_string(),
            self.case,
            self.word,
            doc_key,
            plain.len(),
        );
        if self.key.as_ref() == Some(&key) {
            return;
        }
        let opts = FindOptions {
            case_sensitive: self.case,
            whole_word: self.word,
        };
        self.matches = if key.0.is_empty() {
            Vec::new()
        } else {
            find_all(&Text::new(plain), &key.0, opts)
        };
        self.current = (!self.matches.is_empty()).then_some(0);
        self.key = Some(key);
    }

    pub fn count(&self) -> Option<FindCount> {
        (!self.query.text().is_empty()).then_some(FindCount {
            current: self.current,
            total: self.matches.len(),
        })
    }

    pub fn bar(&self) -> FindBar<'_> {
        FindBar {
            query: &self.query,
            replace: None,
            goto: None,
            focus: self.focus.then_some(FindField::Query),
            case_sensitive: self.case,
            whole_word: self.word,
            count: self.count(),
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.focus = false;
        self.matches.clear();
        self.current = None;
        self.key = None;
    }
}

/// Uma página aberta numa aba.
pub(crate) struct HelpTab {
    pub page: usize,
    pub view: HelpView,
    pub doc: HelpDoc,
    /// O destino de cada número de link / botão de copiar.
    pub targets: Vec<Target>,
    /// O que está sob o rato (o link sublinha).
    pub hover: Option<HelpHit>,
    /// O link com o Tab.
    pub focus: Option<u32>,
    pub selection: Option<Range<usize>>,
    /// O byte onde o arrasto da seleção começou.
    pub drag: Option<usize>,
    pub find: HelpFind,
}

impl HelpTab {
    fn new(page: usize) -> Self {
        let built = build(&catalog()[page]);
        Self {
            page,
            view: HelpView::new(),
            doc: built.doc,
            targets: built.targets,
            hover: None,
            focus: None,
            selection: None,
            drag: None,
            find: HelpFind::default(),
        }
    }

    /// Rola para o sítio `a` desta página (o membro no topo, ou o topo).
    pub fn show(&mut self, a: Anchor) {
        let m = a
            .member
            .and_then(|m| catalog()[self.page].member(m))
            .map(|i| i as u32);
        match m {
            Some(m) => self.view.scroll_to_member(m),
            None => self.view.home(),
        }
    }

    /// O número do link desta página que leva a `to`.
    pub fn link_to(&self, to: Anchor) -> Option<u32> {
        self.targets
            .iter()
            .position(|t| *t == Target::Anchor(to))
            .map(|i| i as u32)
    }
}

/// As páginas abertas, pelo índice no catálogo.
#[derive(Default)]
pub(crate) struct HelpTabs {
    pub open: BTreeMap<usize, HelpTab>,
}

impl CodeDocs {
    /// ⭐ Abre (ou reaproveita) a página de `a`, rola-a até ao membro e pede o foco para ela. `from` =
    /// de onde se veio (empilha no histórico, [`crate::nav`]).
    pub(crate) fn open_help(&mut self, a: Anchor, from: Option<Place>) {
        let Some(page) = page_index(a.page) else {
            return;
        };
        let tab = self
            .help
            .open
            .entry(page)
            .or_insert_with(|| HelpTab::new(page));
        tab.show(a);
        let scroll = tab.view.scroll();
        if let Some(from) = from {
            self.nav.go(from, Place::Help { page, scroll });
        }
        self.focus = Some(tab_id(page));
    }

    /// Os pedidos de página que o Ctrl+clique/F12 de cada aba deixou (depois das respostas do
    /// assistente): abrem-se, e o sítio de partida entra no histórico.
    pub(crate) fn follow_help_requests(&mut self) {
        let asked: Vec<(Anchor, Place)> = self
            .docs
            .iter_mut()
            .filter_map(|d| {
                let a = d.help_request.take()?;
                let caret = d.buffer.selection().primary().head;
                Some((
                    a,
                    Place::Script {
                        id: d.script,
                        caret,
                    },
                ))
            })
            .collect();
        for (a, from) in asked {
            self.open_help(a, Some(from));
        }
    }

    /// O sítio em que a aba `id` está agora (o cursor, a rolagem).
    pub(crate) fn place_of(&self, id: u64) -> Option<Place> {
        if let Some(page) = page_of(id) {
            let t = self.help.open.get(&page)?;
            return Some(Place::Help {
                page,
                scroll: t.view.scroll(),
            });
        }
        let d = self.docs.iter().find(|d| d.script.0 == id)?;
        Some(Place::Script {
            id: d.script,
            caret: d.buffer.selection().primary().head,
        })
    }

    /// Vai para `p`: o script com o cursor onde estava (à vista), ou a página na rolagem guardada.
    pub(crate) fn go_place(&mut self, p: Place) {
        match p {
            Place::Script { id, caret } => {
                if let Some(d) = self.doc(id.0) {
                    let c = caret.min(d.buffer.text().len());
                    d.buffer.set_selection(ph2d_code_buffer::Selection::single(
                        ph2d_code_buffer::Range::cursor(c),
                    ));
                    d.buffer.seal_history();
                    d.reveal();
                }
            }
            Place::Help { page, scroll } => {
                if let Some(t) = self.help.open.get_mut(&page) {
                    t.view.set_scroll(scroll);
                }
            }
        }
        self.focus = Some(p.tab());
    }

    /// ‹ (`back`) ou › a partir da aba `id`. `true` = andou.
    pub(crate) fn nav_step(&mut self, id: u64, back: bool) -> bool {
        let Some(here) = self.place_of(id) else {
            return false;
        };
        let alive = |s: &Self, p: &Place| s.place_of(p.tab()).is_some();
        loop {
            let to = if back {
                self.nav.back(here)
            } else {
                self.nav.forward(here)
            };
            let Some(to) = to else { return false };
            if alive(self, &to) {
                self.go_place(to);
                return true;
            }
            self.nav.remove_tab(to.tab());
        }
    }

    /// Fecha a página `page` (o × da aba ou o Ctrl+W); as entradas dela saem do histórico.
    pub(crate) fn close_help(&mut self, page: usize) {
        self.help.open.remove(&page);
        let _ = self.nav.remove_tab(tab_id(page));
    }

    /// ⭐ Ctrl+W: fecha a aba ACTIVA da família e pede o foco para o sítio anterior do histórico (no
    /// Godot, a página fecha e volta ao script); sem histórico, a última aba que fica.
    pub(crate) fn close_active(&mut self, id: u64) {
        if let Some(here) = self.place_of(id) {
            self.nav.record(here);
        }
        ph2d_editor_core::doc_surface::DocumentSurface::close(self, id);
        let next = self
            .nav
            .current()
            .map(Place::tab)
            .filter(|t| self.place_of(*t).is_some())
            .or_else(|| self.tab_ids().last().copied());
        match next.and_then(|t| self.place_of(t)) {
            Some(p) => self.go_place(p),
            None => self.focus = None,
        }
    }

    /// As abas da família, pela ordem da fila: os scripts, depois as páginas por título (o Godot).
    pub(crate) fn tab_ids(&self) -> Vec<u64> {
        let mut pages: Vec<usize> = self.help.open.keys().copied().collect();
        pages.sort_by_key(|p| catalog()[*p].title.to_lowercase());
        self.docs
            .iter()
            .map(|d| d.script.0)
            .chain(pages.into_iter().map(tab_id))
            .collect()
    }

    /// Pinta a página da aba `page` em `area`.
    pub(crate) fn paint_help(
        &mut self,
        page: usize,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let (back, forward) = self.nav.can(tab_id(page));
        let Some(t) = self.help.open.get_mut(&page) else {
            return;
        };
        t.find.refresh(t.doc.key, t.view.plain());
        let HelpTab {
            view,
            doc,
            hover,
            focus,
            selection,
            find,
            ..
        } = t;
        let frame = HelpFrame {
            doc,
            back,
            forward,
            hover: *hover,
            focus_link: *focus,
            selection: selection.clone(),
            find: find.open.then(|| find.bar()),
            matches: &find.matches,
            current: find.current,
        };
        view.paint(&frame, area, scene, ts, theme);
    }

    fn help_tab(&self, page: &str) -> Option<&HelpTab> {
        self.help.open.get(&page_index(page)?)
    }

    /// O leitor de ecrã da página `page`: os nós da vista, num bloco de ids só das páginas.
    pub(crate) fn help_a11y(&self, page: usize) -> Vec<(ph2d_a11y::NodeId, ph2d_a11y::Node)> {
        self.help
            .open
            .get(&page)
            .map(|t| t.view.a11y(help_a11y_base(page)))
            .unwrap_or_default()
    }

    /// As páginas abertas (o id do catálogo e o da aba), pela ordem da fila.
    #[must_use]
    pub fn help_tabs(&self) -> Vec<(&'static str, u64)> {
        self.tab_ids()
            .into_iter()
            .filter_map(|id| Some((catalog()[page_of(id)?].id, id)))
            .collect()
    }

    /// O id da aba da página `page`, se está aberta.
    #[must_use]
    pub fn help_tab_id(&self, page: &str) -> Option<u64> {
        self.help_tab(page).map(|t| tab_id(t.page))
    }

    /// ⚠️ SONDA: o que a página `page` pintou no último quadro.
    #[must_use]
    pub fn help_probe(&self, page: &str) -> Option<HelpProbe> {
        self.help_tab(page).map(|t| t.view.probe())
    }

    /// Onde a página `page` pintou `hit` no último quadro (os testes clicam aí).
    #[must_use]
    pub fn help_target(&self, page: &str, hit: HelpHit) -> Option<Rect> {
        self.help_tab(page)?.view.target_rect(hit)
    }

    /// O alvo sob `(x, y)` na página `page` (os testes procuram texto por ele).
    pub fn help_hit(&self, page: &str, ts: &mut TextSystem, x: f32, y: f32) -> Option<HelpHit> {
        self.help_tab(page)?.view.hit(ts, x, y)
    }

    /// O número do link da página `page` que leva a `to`.
    #[must_use]
    pub fn help_link(&self, page: &str, to: Anchor) -> Option<u32> {
        self.help_tab(page)?.link_to(to)
    }

    /// Para onde leva o link `n` da página `page`.
    #[must_use]
    pub fn help_link_target(&self, page: &str, n: u32) -> Option<Anchor> {
        match self.help_tab(page)?.targets.get(n as usize)? {
            Target::Anchor(a) => Some(*a),
            Target::Example(_) => None,
        }
    }

    /// O número do botão de copiar o exemplo do membro `member` da página `page`.
    #[must_use]
    pub fn help_copy_of(&self, page: &str, member: &str) -> Option<u32> {
        let t = self.help_tab(page)?;
        let i = catalog()[t.page].member(member)?;
        t.targets
            .iter()
            .position(|x| *x == Target::Example(i))
            .map(|n| n as u32)
    }

    /// O texto da página `page` (as faixas da seleção e da busca indexam-no).
    #[must_use]
    pub fn help_plain(&self, page: &str) -> Option<String> {
        self.help_tab(page).map(|t| t.view.plain().to_owned())
    }

    /// A seleção e o link com o Tab da página `page`.
    #[must_use]
    pub fn help_state(&self, page: &str) -> Option<(Option<Range<usize>>, Option<u32>)> {
        self.help_tab(page).map(|t| (t.selection.clone(), t.focus))
    }

    /// O texto da contagem da busca da página («1 of 4» / «No results»).
    #[must_use]
    pub fn help_find_count_text(&self, page: &str) -> Option<String> {
        let t = self.help_tab(page).filter(|t| t.find.open)?;
        ph2d_panel_code::count_text(t.find.count())
    }

    /// O ‹ e o › acesos na aba `id`.
    #[must_use]
    pub fn nav_can(&self, id: u64) -> (bool, bool) {
        self.nav.can(id)
    }

    /// O texto que um clique mandou para a área de transferência (o botão de copiar um exemplo); a
    /// família entrega-o ao `WidgetStore` no quadro ([`crate::frame`]).
    pub fn take_copy(&mut self) -> Option<String> {
        self.copy_out.take()
    }
}
