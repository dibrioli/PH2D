//! ⭐ **O botão direito no texto** (13 §4 item 14, maquete C painel 4): um menu no ponto do clique,
//! com o MESMO aspecto do menu da barra ([`crate::tab_menu`]). Por ordem: os consertos daquele sítio
//! (a lista do Alt+Enter, [`CodeDoc::actions_at`]: as nossas já, as do assistente quando chegam, sem
//! esperar) · Go to Definition, Find References, Rename Symbol e «Help for <nome>» (só se o nome tem
//! página, a porta do Ctrl+clique) · Toggle Comment. Cada linha corre pelo caminho do seu atalho.
//!
//! Cut/Copy/Paste ficam de fora: a área de transferência só se lê numa tecla (`KeyInput::paste`).

use crate::commands::{
    CMD_GO_TO_DEFINITION, CMD_REFERENCES, CMD_RENAME, CMD_TOGGLE_COMMENT, TabMenu, menu_rows,
};
use crate::doc::CodeDoc;
use crate::docs::CodeDocs;
use crate::tab_menu::{height, label, width_of};
use ph2d_a11y::NodeId;
use ph2d_code_buffer::{Range, Selection};
use ph2d_code_help::Anchor;
use ph2d_editor_core::doc_surface::Key;
use ph2d_editor_core::widget::{
    Divider, Popover, paint_divider, paint_popover, paint_popover_row_highlight, pop_popover_clip,
    push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::Hit;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

const CONTEXT_MENU_POPOVER_ID: NodeId = hash_node_id("code_context_menu_popover");
const DIVIDER_ID: NodeId = hash_node_id("code_context_menu_divider");

/// O que uma linha do menu faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    /// O conserto `i` da lista do Alt+Enter.
    Fix(usize),
    Cmd(NodeId),
    Help,
    /// Mostrar/esconder uma coluna da tabela do quadro do depurador (o cabeçalho, plano 14).
    Column(&'static str),
}

/// O que o menu pintou no último quadro (o rato e os testes lêem-no).
#[derive(Clone, Debug, Default)]
pub struct ContextPainted {
    /// O rótulo de cada linha e a caixa dela.
    pub rows: Vec<(String, Rect)>,
    pub popup: Option<Rect>,
}

/// O menu aberto na aba `tab`, no ponto `at`.
#[derive(Debug)]
pub(crate) struct ContextMenu {
    tab: u64,
    at: (f32, f32),
    /// A página do nome clicado: o rótulo («ph2d.set») e a âncora.
    help: Option<(String, Anchor)>,
    /// A linha acesa (rato ou setas).
    hot: Option<usize>,
    /// Aberto no cabeçalho da tabela do depurador: as colunas `(chave, título, à vista)`.
    columns: Vec<(&'static str, String, bool)>,
    rows: Vec<Row>,
    painted: ContextPainted,
}

/// O rótulo e o atalho de um comando, da tabela única dos menus.
fn cmd_label(id: NodeId) -> (String, String) {
    TabMenu::ALL
        .into_iter()
        .flat_map(menu_rows)
        .find(|r| r.0 == id)
        .map(|r| (r.1, r.2))
        .unwrap_or_default()
}

/// O nome com o qualificador que o antecede (`ph2d.set`, `s:upper`).
fn qualified(doc: &CodeDoc, word: &std::ops::Range<usize>) -> String {
    let t = doc.buffer.text();
    let line_start = t.byte_of_line(t.line_of_byte(word.start));
    let before = t.slice(line_start..word.start);
    let from = ph2d_panel_code::doc_rules::tail_start(&before, |c| {
        c.is_alphanumeric() || matches!(c, '_' | '.' | ':')
    });
    format!("{}{}", &before[from..], t.slice(word.clone()))
}

impl ContextMenu {
    /// As secções do menu, pela ordem: `(linha, rótulo, atalho)`.
    fn sections(&self, doc: &CodeDoc) -> Vec<Vec<(Row, String, String)>> {
        if !self.columns.is_empty() {
            let check = ph2d_i18n::tr("code.ctx.column_on");
            return vec![
                self.columns
                    .iter()
                    .map(|(k, t, on)| {
                        let keys = if *on { check.to_owned() } else { String::new() };
                        (Row::Column(k), t.clone(), keys)
                    })
                    .collect(),
            ];
        }
        let fixes = doc.menu_fixes.as_ref().map_or_else(Vec::new, |a| {
            a.list()
                .enumerate()
                .map(|(i, f)| {
                    // O atalho na 1.ª: o Alt+Enter abre esta mesma lista (maquete C, painel 4).
                    let keys = if i == 0 {
                        ph2d_i18n::tr("code.ctx.fix_keys").to_owned()
                    } else {
                        String::new()
                    };
                    (Row::Fix(i), f.title.clone(), keys)
                })
                .collect()
        });
        let mut nav: Vec<(Row, String, String)> =
            [CMD_GO_TO_DEFINITION, CMD_REFERENCES, CMD_RENAME]
                .into_iter()
                .map(|c| {
                    let (n, k) = cmd_label(c);
                    (Row::Cmd(c), n, k)
                })
                .collect();
        if let Some((name, _)) = &self.help {
            nav.push((
                Row::Help,
                ph2d_i18n::tr_with("code.ctx.help_for", &[("name", name)]),
                String::new(),
            ));
        }
        let (n, k) = cmd_label(CMD_TOGGLE_COMMENT);
        let edit = vec![(Row::Cmd(CMD_TOGGLE_COMMENT), n, k)];
        [fixes, nav, edit]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn paint(
        &mut self,
        doc: &CodeDoc,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let sections = self.sections(doc);
        let pad = Spacing::Sm.px();
        let gap = Spacing::Lg.px();
        let all = || sections.iter().flatten();
        let name_w = all().map(|r| width_of(ts, &r.1)).fold(0.0, f32::max);
        let keys_w = all().map(|r| width_of(ts, &r.2)).fold(0.0, f32::max);
        let w = name_w + gap + keys_w + pad * 4.0;
        let rh = height();
        let n = all().count();
        let h = rh * n as f32 + pad * (sections.len() - 1) as f32 + pad;
        let (mut x, mut y) = self.at;
        if x + w > area.x + area.w {
            x = (area.x + area.w - w).max(area.x);
        }
        if y + h > area.y + area.h {
            y = (y - h).max(area.y);
        }
        let popup = Rect::new(x, y, w, h);
        paint_popover(&Popover::new(CONTEXT_MENU_POPOVER_ID), popup, scene, *theme);
        push_popover_clip(popup, scene);
        self.rows.clear();
        self.painted.rows.clear();
        let inset = Spacing::Xs.px();
        let mut y = popup.y + pad * 0.5;
        for (s, rows) in sections.iter().enumerate() {
            if s > 0 {
                let line = Rect::new(popup.x + inset, y, popup.w - inset * 2.0, pad);
                paint_divider(&Divider::new(DIVIDER_ID), line, scene, *theme);
                y += pad;
            }
            for (row, name, keys) in rows {
                let rr = Rect::new(popup.x + inset, y, popup.w - inset * 2.0, rh);
                let hot = self.hot == Some(self.rows.len());
                if hot {
                    paint_popover_row_highlight(scene, rr, *theme);
                }
                let (ink, dim) = if hot {
                    (ColorToken::AccentFg, ColorToken::AccentFg)
                } else {
                    (ColorToken::Text1, ColorToken::Text3)
                };
                label(
                    ts,
                    scene,
                    name,
                    Rect::new(popup.x + pad, y, name_w, rh),
                    ink,
                    theme,
                );
                let kw = width_of(ts, keys);
                let kr = Rect::new(popup.x + popup.w - pad * 2.0 - kw, y, kw, rh);
                label(ts, scene, keys, kr, dim, theme);
                self.rows.push(*row);
                self.painted.rows.push((name.clone(), rr));
                y += rh;
            }
        }
        pop_popover_clip(scene);
        self.painted.popup = Some(popup);
    }

    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        self.painted.rows.iter().position(|r| r.1.contains(x, y))
    }
}

impl CodeDocs {
    /// O botão direito em `(x, y)` do código da aba `id`: o cursor vai para lá (salvo dentro da
    /// selecção) e o menu abre. `false` = não era no texto.
    pub(crate) fn context_open(&mut self, id: u64, x: f32, y: f32) -> bool {
        if let Some(columns) = self
            .doc(id)
            .filter(|d| d.debug.view.header_at(x, y))
            .map(|d| d.debug.view.column_menu())
        {
            (self.bar.open, self.bar.more) = (None, false);
            self.ctx = Some(ContextMenu {
                tab: id,
                at: (x, y),
                help: None,
                hot: None,
                columns,
                rows: Vec::new(),
                painted: ContextPainted::default(),
            });
            return true;
        }
        let area = self.code_body(id, self.split_area(self.area).1);
        if !area.contains(x, y) {
            return false;
        }
        let Some(d) = self.touched(id) else {
            return false;
        };
        let Hit::Text(b) = d.view.hit(&d.buffer, x, y) else {
            return false;
        };
        d.complete.close();
        d.actions = None;
        d.refs = None;
        d.goto = None;
        d.hints.hover = None;
        let inside = d
            .buffer
            .selection()
            .ranges()
            .iter()
            .any(|r| !r.is_empty() && r.from() <= b && b < r.to());
        if !inside {
            d.buffer.set_selection(Selection::single(Range::cursor(b)));
            d.buffer.seal_history();
        }
        d.syntax.fresh(d.buffer.text());
        d.menu_fixes = Some(d.actions_at(&[b]));
        let help = d.goto_word(b).and_then(|w| {
            let a = d.help_anchor(&w, None)?;
            Some((qualified(d, &w), a))
        });
        (self.bar.open, self.bar.more) = (None, false);
        self.ctx = Some(ContextMenu {
            tab: id,
            at: (x, y),
            help,
            hot: None,
            columns: Vec::new(),
            rows: Vec::new(),
            painted: ContextPainted::default(),
        });
        true
    }

    /// Fecha o menu (e larga os consertos que ele esperava).
    pub(crate) fn context_close(&mut self) -> bool {
        let Some(m) = self.ctx.take() else {
            return false;
        };
        if let Some(d) = self.doc(m.tab) {
            d.menu_fixes = None;
        }
        true
    }

    /// Pinta o menu por cima de tudo (chame-a por último).
    pub(crate) fn context_paint(
        &mut self,
        id: u64,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let area = self.area;
        let (Some(m), Some(d)) = (
            self.ctx.as_mut().filter(|m| m.tab == id),
            self.docs.iter().find(|d| d.script.0 == id),
        ) else {
            return;
        };
        m.paint(d, area, scene, ts, theme);
    }

    /// Um carregar com o menu aberto: numa linha corre-a; fora fecha. `None` = sem menu.
    pub(crate) fn context_press(&mut self, x: f32, y: f32) -> Option<bool> {
        let m = self.ctx.as_ref()?;
        match m.row_at(x, y) {
            Some(k) => self.context_run(k),
            None if m.painted.popup.is_some_and(|p| p.contains(x, y)) => {}
            None => {
                self.context_close();
            }
        }
        Some(true)
    }

    /// O rato sem botão: a linha sob ele acende. `true` = o menu está aberto.
    pub(crate) fn context_move(&mut self, x: f32, y: f32) -> bool {
        let Some(m) = self.ctx.as_mut() else {
            return false;
        };
        if let Some(k) = m.row_at(x, y) {
            m.hot = Some(k);
        }
        true
    }

    /// As teclas com o menu aberto: ↑/↓ andam, Enter corre, Esc fecha. Outra tecla fecha-o e segue.
    pub(crate) fn context_key(&mut self, key: Key) -> bool {
        let Some(m) = self.ctx.as_mut() else {
            return false;
        };
        let n = m.painted.rows.len();
        match key {
            Key::Down | Key::Up if n > 0 => {
                let down = key == Key::Down;
                m.hot = Some(match m.hot {
                    None if down => 0,
                    None => n - 1,
                    Some(k) if down => (k + 1) % n,
                    Some(k) => (k + n - 1) % n,
                });
            }
            Key::Enter => match m.hot {
                Some(k) => self.context_run(k),
                None => {
                    self.context_close();
                }
            },
            Key::Escape => {
                self.context_close();
            }
            // Um modificador sozinho (o Shift de um Shift+F12) não fecha.
            Key::Other => return false,
            _ => {
                self.context_close();
                return false;
            }
        }
        true
    }

    /// Corre a linha `k` do menu pelo caminho do seu atalho, e fecha-o.
    fn context_run(&mut self, k: usize) {
        let Some(m) = self.ctx.take() else { return };
        let (tab, row) = (m.tab, m.rows.get(k).copied());
        let fixes = self.doc(tab).and_then(|d| d.menu_fixes.take());
        match row {
            Some(Row::Fix(i)) => {
                if let (Some(d), Some(a)) = (self.touched(tab), fixes) {
                    d.apply(&a, i);
                    d.reveal();
                }
                self.submit();
            }
            Some(Row::Cmd(cmd)) => {
                let _ = ph2d_editor_core::doc_surface::DocumentSurface::command(self, tab, cmd);
            }
            Some(Row::Column(key)) => {
                if let Some(d) = self.doc(tab) {
                    let t = &mut d.debug.view.table;
                    let shown = !t.hidden.contains(key);
                    t.set_hidden(key, shown);
                }
            }
            Some(Row::Help) => {
                if let (Some(d), Some((_, a))) = (self.doc(tab), m.help) {
                    d.help_request = Some(a);
                }
            }
            None => {}
        }
    }

    /// O que o menu pintou no último quadro (`None` = fechado).
    #[must_use]
    pub fn context_painted(&self) -> Option<&ContextPainted> {
        self.ctx.as_ref().map(|m| &m.painted)
    }

    /// Abre o menu sobre o byte `at` da aba `id`, como o botão direito no centro dessa letra (o
    /// passo da foto do smoke). `false` = a letra não está à vista.
    pub fn context_open_at(&mut self, id: u64, at: usize) -> bool {
        let Some((x, y)) = self.glyph_center(ph2d_script::ScriptId(id), at) else {
            return false;
        };
        self.context_open(id, x, y)
    }
}
