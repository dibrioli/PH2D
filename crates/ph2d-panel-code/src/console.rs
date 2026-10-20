//! ⭐ **A faixa de baixo da aba** (recolhível), com dois separadores: o **Console** (E10: o `print`
//! do script, os erros de runtime e as recusas de recarga) e os **Problems** (acabamento, item 4 de
//! `docs/CodeEditor/03_onda2.md`: os MESMOS problemas da calha, um por linha — em [`problems`]). A
//! família entrega as linhas já compostas; aqui pinta-se e responde-se ao clique.

mod problems;

pub use problems::{PaintedProblem, ProblemRow, problem_label, problems_title};

use crate::chrome::{self, Ui};
use crate::status::Tone;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{INLINE_ICON_PX, LineHeight, ROW_H_PX, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Uma linha do console, já composta (emprestada de quem a guarda: pintar não copia nada).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsoleRow<'a> {
    pub text: &'a str,
    pub tone: Tone,
    /// A linha (1-based) do script a que o clique leva.
    pub line: Option<u32>,
}

/// As linhas de um console, por índice: a pintura só pede as que estão à vista (e nenhuma com o
/// console fechado).
pub trait ConsoleSource {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn row(&self, i: usize) -> ConsoleRow<'_>;
}

/// O separador activo da faixa.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BandTab {
    #[default]
    Console,
    Problems,
}

/// O que a faixa pinta num quadro.
pub struct Band<'a> {
    pub open: bool,
    pub tab: BandTab,
    pub console: &'a dyn ConsoleSource,
    /// Os erros do console (o título do separador conta-os).
    pub console_errors: usize,
    /// Quantos problemas o script tem (o separador mostra-o).
    pub problem_count: usize,
    /// As linhas da lista, pela ordem (vazias se a lista não está à mostra).
    pub problems: &'a [ProblemRow<'a>],
    /// A linha realçada (o F8), índice em `problems`.
    pub current: Option<usize>,
}

/// Onde caiu um clique na faixa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleHit {
    /// O cabeçalho fora dos separadores e do «Clear»: abre/fecha.
    Toggle,
    Clear,
    Tab(BandTab),
    /// A linha `i` do console (índice nas linhas entregues).
    Row(usize),
    /// A linha `i` da lista de problemas (índice em [`Band::problems`]).
    Problem(usize),
    /// Dentro da faixa, fora de um alvo.
    Body,
}

/// ⚠️ SONDA: o que a faixa pintou no último quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct BandProbe {
    pub open: bool,
    pub tab: BandTab,
    /// Os rótulos e os alvos dos separadores (Console, Problems).
    pub tabs: [(String, Rect); 2],
    pub problems: Vec<PaintedProblem>,
    /// A frase da lista vazia, se foi pintada.
    pub empty: Option<String>,
}

/// O estado de vista da faixa: a rolagem de cada separador e os alvos do último quadro.
pub struct ConsoleView {
    area: Rect,
    header: Rect,
    clear: Rect,
    tabs: [Rect; 2],
    tab_labels: [String; 2],
    open: bool,
    tab: BandTab,
    /// As linhas do console pintadas: (índice, rect, leva a uma linha do script).
    rows: Vec<(usize, Rect, bool)>,
    problems: Vec<PaintedProblem>,
    empty: Option<String>,
    problem_total: usize,
    /// Quantas linhas do console acima do fim (0 = a mais recente à vista).
    back: usize,
    /// A 1.ª linha da lista de problemas à vista.
    top: usize,
    /// A linha realçada no último quadro (uma nova rola-se para a vista).
    last_current: Option<usize>,
}

impl Default for ConsoleView {
    fn default() -> Self {
        Self {
            area: chrome::NO_RECT,
            header: chrome::NO_RECT,
            clear: chrome::NO_RECT,
            tabs: [chrome::NO_RECT; 2],
            tab_labels: [String::new(), String::new()],
            open: false,
            tab: BandTab::Console,
            rows: Vec::new(),
            problems: Vec::new(),
            empty: None,
            problem_total: 0,
            back: 0,
            top: 0,
            last_current: None,
        }
    }
}

/// Os ids do leitor de ecrã, somados ao do `Document` da aba (longe dos das linhas, da lista de
/// completação e dos balões; os da lista de problemas em [`problems`]).
const A11Y_TABS: u64 = (1 << 31) | (1 << 28);

/// O título do separador do console: «Console», ou com a contagem de erros («1 error», «2 errors»).
#[must_use]
pub fn title_text(errors: usize) -> String {
    match errors {
        0 => tr("code.console.title").to_owned(),
        1 => tr("code.console.title_errors_one").to_owned(),
        n => tr_with("code.console.title_errors", &[("count", &n)]),
    }
}

fn row_h() -> f32 {
    ph2d_editor_core::ui_scale::ao_pixel(chrome::font() * LineHeight::Normal.ratio())
}

/// Os limites de um nó do leitor de ecrã.
fn bounds(r: Rect) -> (f64, f64, f64, f64) {
    (
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.w),
        f64::from(r.h),
    )
}

impl ConsoleView {
    /// A altura da faixa numa área de altura `area_h`: só o cabeçalho, ou um terço da área.
    #[must_use]
    pub fn height(open: bool, area_h: f32) -> f32 {
        if open {
            (area_h / 3.0).max(ROW_H_PX * 2.0) // LITERAL-PX-OK: um terço da área do editor
        } else {
            ROW_H_PX
        }
    }

    /// Rola `dy` px o separador à mostra (positivo = para trás no tempo / para cima na lista).
    pub fn scroll_by(&mut self, dy: f32, total: usize) {
        let n = (dy / row_h()).round() as isize;
        if self.tab == BandTab::Problems {
            self.top = ph2d_editor_core::math::safe_clamp_ord(
                self.top as isize - n,
                0,
                self.problem_total.saturating_sub(1) as isize,
            ) as usize;
            return;
        }
        self.back = ph2d_editor_core::math::safe_clamp_ord(
            self.back as isize + n,
            0,
            total.saturating_sub(1) as isize,
        ) as usize;
    }

    /// Volta a mostrar a linha mais recente do console.
    pub fn follow(&mut self) {
        self.back = 0;
    }

    pub fn paint(
        &mut self,
        band: Band<'_>,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        self.area = area;
        self.open = band.open;
        self.tab = band.tab;
        self.rows.clear();
        self.problems.clear();
        self.empty = None;
        chrome::panel_bg(scene, area, &ui, false);
        let pad = Spacing::Sm.px();
        self.header = Rect::new(area.x, area.y, area.w, ROW_H_PX.min(area.h));
        let chev = Rect::new(area.x + pad, area.y, INLINE_ICON_PX, self.header.h);
        chrome::icon_button(
            scene,
            if band.open {
                IconId::ChevronDown
            } else {
                IconId::ChevronRight
            },
            chev,
            &ui,
        );
        self.paint_tabs(&band, chev.x + chev.w + pad, scene, ts, &ui);
        self.clear = chrome::NO_RECT;
        if band.tab == BandTab::Console {
            let clear_text = tr("code.console.clear");
            let cw = chrome::button_w(ts, clear_text);
            self.clear = Rect::new(
                area.x + area.w - pad - cw,
                area.y + Spacing::Xxs.px(),
                cw,
                self.header.h - Spacing::Xxs.px() * 2.0,
            );
            chrome::text_button(scene, ts, clear_text, self.clear, false, &ui);
        }
        self.problem_total = band.problems.len();
        if !band.open {
            return;
        }
        let body = Rect::new(
            area.x,
            area.y + self.header.h,
            area.w,
            (area.h - self.header.h).max(0.0),
        );
        scene.push_clip(&rect_to_vello(body));
        match band.tab {
            BandTab::Console => self.paint_console(band.console, body, scene, ts, &ui),
            BandTab::Problems => {
                self.paint_problems(band.problems, band.current, body, scene, ts, &ui);
            }
        }
        scene.pop_layer();
    }

    /// Os dois separadores a partir de `x`: o activo no texto forte, sublinhado no acento.
    fn paint_tabs(
        &mut self,
        band: &Band<'_>,
        mut x: f32,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        ui: &Ui,
    ) {
        let labels = [
            title_text(band.console_errors),
            problems_title(band.problem_count),
        ];
        let pad = Spacing::Md.px();
        for (k, (label, tab)) in labels
            .iter()
            .zip([BandTab::Console, BandTab::Problems])
            .enumerate()
        {
            let w = chrome::label_w(ts, label) + pad * 2.0;
            let r = Rect::new(x, self.header.y, w, self.header.h);
            let active = tab == band.tab;
            let color = match tab {
                BandTab::Console if band.console_errors > 0 => ui.error,
                _ if active => ui.text1,
                _ => ui.text2,
            };
            chrome::label_in(
                scene,
                ts,
                label,
                Rect::new(r.x + pad, r.y, (r.w - pad * 2.0).max(0.0), r.h),
                color,
            );
            if active && band.open {
                let t = StrokeToken::Thick.px();
                scene.fill_rect(
                    rect_to_vello(Rect::new(r.x + pad, r.y + r.h - t, r.w - pad * 2.0, t)),
                    ui.focus,
                );
            }
            self.tabs[k] = r;
            x += w;
        }
        self.tab_labels = labels;
    }

    fn paint_console(
        &mut self,
        rows: &dyn ConsoleSource,
        body: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        ui: &Ui,
    ) {
        let pad = Spacing::Sm.px();
        let h = row_h();
        let fit = ((body.h - pad) / h).max(0.0) as usize;
        let end = rows
            .len()
            .saturating_sub(self.back.min(rows.len().saturating_sub(1)));
        let start = end.saturating_sub(fit);
        if rows.is_empty() {
            chrome::label_in(
                scene,
                ts,
                tr("code.console.empty"),
                Rect::new(body.x + pad, body.y, body.w, h),
                ui.text3,
            );
        }
        for (k, i) in (start..end).enumerate() {
            let r = Rect::new(body.x, body.y + k as f32 * h, body.w, h);
            let row = rows.row(i);
            let color = match row.tone {
                Tone::Plain | Tone::Success => ui.text1,
                Tone::Warning => ui.warn,
                Tone::Error => ui.error,
            };
            if row.line.is_some() {
                scene.fill_rect(rect_to_vello(r), ui.hover);
            }
            chrome::label_in(
                scene,
                ts,
                row.text,
                Rect::new(r.x + pad, r.y, (r.w - pad * 2.0).max(0.0), r.h),
                color,
            );
            self.rows.push((i, r, row.line.is_some()));
        }
    }

    /// Onde caiu `(x, y)`; `None` fora da faixa.
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<ConsoleHit> {
        if !self.area.contains(x, y) {
            return None;
        }
        if self.clear.contains(x, y) {
            return Some(ConsoleHit::Clear);
        }
        if let Some(k) = self.tabs.iter().position(|r| r.contains(x, y)) {
            return Some(ConsoleHit::Tab(if k == 0 {
                BandTab::Console
            } else {
                BandTab::Problems
            }));
        }
        if self.header.contains(x, y) {
            return Some(ConsoleHit::Toggle);
        }
        if let Some(p) = self.problems.iter().find(|p| p.rect.contains(x, y)) {
            return Some(ConsoleHit::Problem(p.index));
        }
        Some(
            self.rows
                .iter()
                .find(|(_, r, _)| r.contains(x, y))
                .map_or(ConsoleHit::Body, |(i, _, _)| ConsoleHit::Row(*i)),
        )
    }

    /// Um clique em `hit` leva a algum sítio (o ponteiro é a mão): o cabeçalho, os separadores, o
    /// «Clear», um problema e uma linha do console com número.
    #[must_use]
    pub fn clickable(&self, hit: ConsoleHit) -> bool {
        match hit {
            ConsoleHit::Toggle
            | ConsoleHit::Clear
            | ConsoleHit::Tab(_)
            | ConsoleHit::Problem(_) => true,
            ConsoleHit::Row(i) => self.rows.iter().any(|(k, _, line)| *k == i && *line),
            ConsoleHit::Body => false,
        }
    }

    /// O retângulo pintado da linha `i` do console (os testes clicam nele).
    #[must_use]
    pub fn row_rect(&self, i: usize) -> Option<Rect> {
        self.rows.iter().find(|(k, ..)| *k == i).map(|(_, r, _)| *r)
    }

    #[must_use]
    pub fn area(&self) -> Rect {
        self.area
    }

    /// ⚠️ SONDA: o que o último quadro pintou.
    #[must_use]
    pub fn probe(&self) -> BandProbe {
        BandProbe {
            open: self.open,
            tab: self.tab,
            tabs: [
                (self.tab_labels[0].clone(), self.tabs[0]),
                (self.tab_labels[1].clone(), self.tabs[1]),
            ],
            problems: self.problems.clone(),
            empty: self.empty.clone(),
        }
    }

    /// Os nós da faixa ao leitor de ecrã, filhos do `Document` (`doc`): os dois separadores (`Tab`, o
    /// activo seleccionado) e, com a lista à mostra, a `List` «Problems: N» com um `ListItem` por
    /// linha pintada (a realçada pelo F8 seleccionada). Devolve as raízes e todos os nós.
    #[must_use]
    pub fn a11y(&self, doc: NodeId) -> (Vec<NodeId>, Vec<(NodeId, Node)>) {
        let (mut roots, mut out) = (Vec::new(), Vec::new());
        for (k, tab) in [BandTab::Console, BandTab::Problems]
            .into_iter()
            .enumerate()
        {
            let id = NodeId(doc.0 + A11Y_TABS + k as u64);
            let (x, y, w, h) = bounds(self.tabs[k]);
            let mut n = NodeBuilder::new(Role::Tab)
                .label(self.tab_labels[k].clone())
                .bounds(x, y, w, h)
                .action(Action::Click)
                .build();
            n.set_selected(tab == self.tab);
            roots.push(id);
            out.push((id, n));
        }
        self.problems_a11y(doc, &mut roots, &mut out);
        (roots, out)
    }
}
