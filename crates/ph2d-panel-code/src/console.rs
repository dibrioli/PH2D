//! ⭐ **E10 — o console dentro da aba** (em baixo, recolhível): o `print` do script, os erros de
//! runtime e as recusas de recarga. A família filtra pelo script da aba e entrega as linhas já
//! compostas; aqui pinta-se e responde-se ao clique (uma linha com número põe o cursor nela).

use crate::chrome::{self, Ui};
use crate::status::Tone;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{LineHeight, ROW_H_PX, Spacing, Theme};
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

/// Onde caiu um clique no console.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleHit {
    /// O cabeçalho: abre/fecha.
    Toggle,
    Clear,
    /// A linha `i` (índice nas linhas entregues).
    Row(usize),
    /// Dentro do console, fora de um alvo.
    Body,
}

/// O estado de vista do console: a rolagem (em linhas a partir do fim) e os alvos do último quadro.
pub struct ConsoleView {
    area: Rect,
    header: Rect,
    clear: Rect,
    rows: Vec<(usize, Rect)>,
    /// Quantas linhas acima do fim (0 = a mais recente à vista).
    back: usize,
}

impl Default for ConsoleView {
    fn default() -> Self {
        Self {
            area: chrome::NO_RECT,
            header: chrome::NO_RECT,
            clear: chrome::NO_RECT,
            rows: Vec::new(),
            back: 0,
        }
    }
}

/// O título do cabeçalho: «Console», ou com a contagem de erros («1 error», «2 errors»).
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

impl ConsoleView {
    /// A altura do console numa área de altura `area_h`: só o cabeçalho, ou um terço da área.
    #[must_use]
    pub fn height(open: bool, area_h: f32) -> f32 {
        if open {
            (area_h / 3.0).max(ROW_H_PX * 2.0) // LITERAL-PX-OK: um terço da área do editor
        } else {
            ROW_H_PX
        }
    }

    /// Rola `dy` px (positivo = para trás no tempo).
    pub fn scroll_by(&mut self, dy: f32, total: usize) {
        let n = (dy / row_h()).round() as isize;
        self.back = ph2d_editor_core::math::safe_clamp_ord(
            self.back as isize + n,
            0,
            total.saturating_sub(1) as isize,
        ) as usize;
    }

    /// Volta a mostrar a linha mais recente.
    pub fn follow(&mut self) {
        self.back = 0;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        rows: &dyn ConsoleSource,
        open: bool,
        errors: usize,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        self.area = area;
        self.rows.clear();
        chrome::panel_bg(scene, area, &ui, false);
        let pad = Spacing::Sm.px();
        self.header = Rect::new(area.x, area.y, area.w, ROW_H_PX.min(area.h));
        let s = ph2d_tokens::INLINE_ICON_PX;
        let chev = Rect::new(area.x + pad, area.y, s, self.header.h);
        chrome::icon_button(
            scene,
            if open {
                IconId::ChevronDown
            } else {
                IconId::ChevronRight
            },
            chev,
            &ui,
        );
        let title = title_text(errors);
        let tx = chev.x + chev.w + pad;
        let tone = if errors > 0 { ui.error } else { ui.text2 };
        chrome::label_in(
            scene,
            ts,
            &title,
            Rect::new(tx, area.y, (area.w - tx).max(0.0), self.header.h),
            tone,
        );
        let clear_text = tr("code.console.clear");
        let cw = chrome::label_w(ts, clear_text) + Spacing::Md.px() * 2.0;
        self.clear = Rect::new(
            area.x + area.w - pad - cw,
            area.y + Spacing::Xxs.px(),
            cw,
            self.header.h - Spacing::Xxs.px() * 2.0,
        );
        chrome::text_button(scene, ts, clear_text, self.clear, false, &ui);
        if !open {
            return;
        }
        let body = Rect::new(
            area.x,
            area.y + self.header.h,
            area.w,
            (area.h - self.header.h).max(0.0),
        );
        let h = row_h();
        let fit = ((body.h - pad) / h).max(0.0) as usize;
        let end = rows
            .len()
            .saturating_sub(self.back.min(rows.len().saturating_sub(1)));
        let start = end.saturating_sub(fit);
        scene.push_clip(&rect_to_vello(body));
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
                scene.fill_rect(rect_to_vello(Rect::new(r.x, r.y, r.w, r.h)), ui.hover);
            }
            chrome::label_in(
                scene,
                ts,
                row.text,
                Rect::new(r.x + pad, r.y, (r.w - pad * 2.0).max(0.0), r.h),
                color,
            );
            self.rows.push((i, r));
        }
        scene.pop_layer();
    }

    /// Onde caiu `(x, y)`; `None` fora do console.
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<ConsoleHit> {
        if !self.area.contains(x, y) {
            return None;
        }
        if self.clear.contains(x, y) {
            return Some(ConsoleHit::Clear);
        }
        if self.header.contains(x, y) {
            return Some(ConsoleHit::Toggle);
        }
        Some(
            self.rows
                .iter()
                .find(|(_, r)| r.contains(x, y))
                .map_or(ConsoleHit::Body, |(i, _)| ConsoleHit::Row(*i)),
        )
    }

    /// O retângulo pintado da linha `i` (os testes clicam nele).
    #[must_use]
    pub fn row_rect(&self, i: usize) -> Option<Rect> {
        self.rows.iter().find(|(k, _)| *k == i).map(|(_, r)| *r)
    }

    #[must_use]
    pub fn area(&self) -> Rect {
        self.area
    }
}
