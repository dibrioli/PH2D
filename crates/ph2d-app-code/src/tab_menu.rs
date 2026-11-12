//! ⭐ **A barra da própria aba** (13 §4 itens 11 e 13b, D3): os menus ([`TabMenu::ALL`]) à esquerda, o
//! caminho até o cursor à direita e a engrenagem de «Script options» no fim; as linhas saem da tabela
//! de [`crate::commands`], e um clique corre o comando pelo MESMO caminho do `Ctrl+K` e da tecla.

use crate::commands::{CMD_RESTART_ASSIST, TabMenu, menu_rows_in, row_state};
use crate::docs::CodeDocs;
use crate::help::page_of;
use ph2d_editor_core::doc_surface::Key;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{Leaf, MenuItem, MenuPress};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

pub use ph2d_panel_code::tab_bar::{Crumb, CrumbFunc, height, label, split, width_of};

/// A barra da aba: os seis menus de [`TabMenu`].
pub type Bar = ph2d_panel_code::tab_bar::TabBar<TabMenu>;
/// O que a barra pintou no último quadro.
pub type BarPainted = ph2d_panel_code::tab_bar::BarPainted<TabMenu>;

/// A linha `View ▸ Card Style` (o submenu do estilo do quadro do depurador).
pub const ROW_CARD_STYLE: ph2d_a11y::NodeId =
    ph2d_tool_registry::hash_node_id("code_menu_card_style");

/// A linha `View ▸ Text rendering` (a do submenu dos modos da letra do código).
pub const ROW_TEXT_RENDERING: ph2d_a11y::NodeId =
    ph2d_tool_registry::hash_node_id("code_menu_text_rendering");

/// Os títulos da barra, pela ordem.
#[must_use]
pub fn menu_titles() -> [(TabMenu, &'static str); 6] {
    TabMenu::ALL.map(|m| (m, m.title()))
}

/// As linhas do menu `m` (interruptores em `state`, o jogo em `stop`): o Run leva a nota do Restart
/// Code Assistant; o View acaba em `Card Style ▸` e `Text rendering ▸` (dono 10/11 e 11/11).
#[must_use]
pub fn menu_items(
    m: TabMenu,
    state: crate::commands::CommandState,
    stop: crate::debug::GameStop,
) -> Vec<MenuItem> {
    let mut out = Vec::new();
    for (id, name, keys) in menu_rows_in(m, state) {
        let (live, show_keys) = row_state(id, stop);
        let keys = if show_keys { keys } else { String::new() };
        out.push(MenuItem::Cmd {
            id,
            name,
            keys,
            live,
        });
        if m == TabMenu::Run && id == CMD_RESTART_ASSIST {
            out.push(MenuItem::Note(
                ph2d_i18n::tr("code.menu.restart_hint").to_owned(),
            ));
        }
    }
    if m == TabMenu::View {
        use crate::commands::{CMD_CARD_CARDS, CMD_CARD_SHEET, CMD_CARD_STRIP};
        use ph2d_code_debugcard::CardStyle::{Cards, Sheet, Strip};
        let styles = [
            (CMD_CARD_SHEET, "code.menu.card.sheet", Sheet),
            (CMD_CARD_CARDS, "code.menu.card.cards", Cards),
            (CMD_CARD_STRIP, "code.menu.card.strip", Strip),
        ];
        out.push(MenuItem::Sub {
            id: ROW_CARD_STYLE,
            name: ph2d_i18n::tr("code.menu.card_style").to_owned(),
            rows: MenuItem::choices(
                styles.map(|(id, k, s)| (id, ph2d_i18n::tr(k).to_owned(), state.card_style == s)),
            ),
        });
        out.push(MenuItem::Sub {
            id: ROW_TEXT_RENDERING,
            name: ph2d_i18n::tr("code.menu.text_rendering").to_owned(),
            rows: MenuItem::choices(ph2d_panel_code::code_text::choices()),
        });
    }
    out
}

/// As linhas do menu aberto da barra: o de um título, o do transbordo, ou nenhuma.
pub(crate) fn open_items(
    bar: &Bar,
    state: crate::commands::CommandState,
    stop: crate::debug::GameStop,
) -> Vec<MenuItem> {
    let more = |m: TabMenu| (m.more_row(), m.title().into());
    bar.open_items(|m| menu_items(m, state, stop), more)
}

/// O que um carregar na barra (fora do menu) pede.
enum BarPress {
    Menu(MenuPress),
    Crumb(usize, usize),
    Gear,
}

impl CodeDocs {
    /// O cartão do código da aba `id` partido em barra e corpo (uma página de ajuda não tem barra).
    pub(crate) fn code_body(&self, id: u64, doc_r: Rect) -> Rect {
        if page_of(id).is_some() {
            doc_r
        } else {
            ph2d_panel_code::bancada::phrase_split(split(doc_r).1).0
        }
    }

    /// A faixa da frase do pé da Bancada, no fundo do cartão do código (uma ajuda não a tem).
    pub(crate) fn phrase_rect(&self, id: u64, doc_r: Rect) -> Option<Rect> {
        (page_of(id).is_none()).then(|| ph2d_panel_code::bancada::phrase_split(split(doc_r).1).1)
    }

    /// O caminho até o cursor da aba `id` (chrome de cada quadro): nunca reparsa e só procura a função
    /// quando a árvore ou a linha mudam, com as dobras das funções candidatas.
    fn crumb(&mut self, id: u64) -> Option<Crumb> {
        let d = self.docs.iter_mut().find(|d| d.script.0 == id)?;
        let t = d.buffer.text();
        let line = t.line_of_byte(d.buffer.selection().primary().head);
        let key = (d.syntax.generation(), line);
        let cached = d.crumb_func.as_ref();
        if !d.syntax.is_stale() && cached.is_none_or(|c| (c.0, c.1) != key) {
            let func = d
                .syntax
                .symbols(t)
                .into_iter()
                .filter(|s| s.kind == ph2d_code_syntax::SymbolKind::Function)
                .map(|s| (t.line_of_byte(s.byte), s))
                .filter(|(l, _)| *l <= line)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .find(|(l, _)| {
                    d.syntax.fold_at(t, *l).is_some_and(|f| line <= f.end_line) || *l == line
                })
                .map(|(l, s)| {
                    let col = t.slice(t.byte_of_line(l)..s.byte).chars().count();
                    (s.name, l, col)
                });
            d.crumb_func = Some((key.0, key.1, func));
        }
        Some(Crumb {
            script: d.title.clone(),
            func: d.crumb_func.as_ref().and_then(|c| c.2.clone()),
            line: line + 1,
        })
    }

    /// Pinta a barra no topo de `doc_r` (código, nunca ajuda): o fundo de cartão desce um raio por
    /// baixo do cartão do código, que se lê como a mesma moldura.
    pub(crate) fn paint_bar(
        &mut self,
        id: u64,
        doc_r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let Some(crumb) = self.crumb(id) else { return };
        let (head, _) = split(doc_r);
        crate::cards::open_bar(scene, head, doc_r.h, theme);
        self.bar
            .paint(head, &menu_titles(), &crumb, scene, ts, theme);
        crate::cards::close_bar(scene, head, theme);
    }

    /// O que flutua por cima da aba: o cartão das opções e o menu aberto (este por último).
    pub(crate) fn paint_over(
        &mut self,
        id: u64,
        doc_r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        if page_of(id).is_some() {
            return;
        }
        let doc = self.docs.iter().find(|d| d.script.0 == id);
        let state = doc.map(|d| self.command_state(d)).unwrap_or_default();
        if let Some(title) = doc.map(|d| d.title.clone()) {
            let rows = crate::options_card::rows(state);
            let f = crate::options_card::frame(&title, &rows, self.intel.assist());
            let under = self.code_body(id, doc_r);
            self.options.paint(under, &f, scene, ts, theme);
        }
        if self.options.open
            && let Some(pin) = self.options.painted.pin
        {
            let tip = ph2d_i18n::tr("code.options.pin").to_owned();
            self.bar.painted.tips.push((pin, tip));
        }
    }

    /// Um carregar primário: o menu aberto, a barra, o cartão das opções. `Some(true)` = tomado.
    pub(crate) fn bar_press(&mut self, id: u64, x: f32, y: f32) -> Option<bool> {
        if page_of(id).is_some() {
            return None;
        }
        let press = if let Some(p) = self.bar.press(x, y) {
            BarPress::Menu(p)
        } else if let Some((r, l, c)) = self.bar.painted.crumb_fn
            && r.contains(x, y)
        {
            BarPress::Crumb(l, c)
        } else if self.bar.painted.gear.is_some_and(|g| g.contains(x, y)) {
            BarPress::Gear
        } else if let Some(cmd) = self.options.press(x, y) {
            BarPress::Menu(MenuPress::Run(cmd))
        } else if self.options.contains(x, y) {
            BarPress::Menu(MenuPress::Taken)
        } else {
            self.options.click_outside();
            return self.bar.painted.header.contains(x, y).then_some(true);
        };
        match press {
            BarPress::Menu(p) => self.bar_run(id, p),
            BarPress::Crumb(l, c) => {
                let _ = self.tab_go_to(id, l, c);
            }
            BarPress::Gear => self.options.toggle(),
        }
        Some(true)
    }

    /// O rato sem botão: a dica, e no menu aberto a linha (e o submenu) sob ele.
    pub(crate) fn bar_move(&mut self, x: f32, y: f32) -> bool {
        self.bar.track_tip(x, y, self.clock.0);
        self.bar.hover_move_at(x, y, self.clock.0) || self.options.hover(x, y)
    }

    /// Uma tecla no menu aberto (o teclado dos menus de sistema); sem menu, o Esc fecha o cartão
    /// «Script options» sem alfinete. `true` = tomada.
    pub(crate) fn bar_key(&mut self, id: u64, key: Key) -> bool {
        if let Some(p) = self.bar.key(&self.tab_items(id), key) {
            self.bar_run(id, p);
            return true;
        }
        let close = key == Key::Escape && self.options.open && !self.options.pinned;
        self.options.open &= !close;
        close
    }

    fn bar_run(&mut self, id: u64, p: MenuPress) {
        if let MenuPress::Run(cmd) = p {
            let _ = ph2d_editor_core::doc_surface::DocumentSurface::command(self, id, cmd);
        }
    }

    pub(crate) fn tab_items(&self, id: u64) -> Vec<MenuItem> {
        let doc = self.docs.iter().find(|d| d.script.0 == id);
        let state = doc.map(|d| self.command_state(d)).unwrap_or_default();
        open_items(&self.bar, state, self.debug.game_stop)
    }

    /// A `k`-ésima escolha do menu aberto (o smoke), pela ordem de [`MenuItem::leaves`].
    pub(crate) fn tab_leaf(&self, id: u64, k: usize) -> Option<Leaf> {
        MenuItem::leaves(&self.tab_items(id)).get(k).copied()
    }

    /// O que a barra pintou (os testes clicam por aqui).
    #[must_use]
    pub fn bar_painted(&self) -> &BarPainted {
        &self.bar.painted
    }

    /// Abre o menu `m` (o passo da foto do smoke; o clique faz o mesmo).
    pub fn open_tab_menu(&mut self, m: Option<TabMenu>) {
        self.bar.open = m;
        self.bar.more = false;
    }

    /// Abre o menu de transbordo `…` e, com `sub`, o submenu do título escondido (a foto do smoke).
    pub fn open_more_menu(&mut self, sub: Option<TabMenu>) {
        (self.bar.open, self.bar.more) = (None, true);
        self.bar.sub = sub.map(TabMenu::more_row);
    }
}

impl CodeDocs {
    /// O cartão «Script options» (os testes e a sonda da foto lêem-no).
    #[must_use]
    pub fn options_card(&self) -> &crate::options_card::OptionsCard {
        &self.options
    }

    /// Abre/fecha o cartão «Script options» (o passo da foto do smoke; a engrenagem faz o mesmo).
    pub fn set_options_open(&mut self, open: bool) {
        self.options.open = open;
    }
}
