//! ⭐ **E9 — a barra de buscar/substituir e o «Go to line»**, no topo da área do editor. Os campos
//! são `Buffer`s de uma linha da família; aqui só se pinta e se responde «onde caiu o clique».

use crate::chrome::{self, Ui};
use ph2d_code_buffer::Buffer;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ICON_BTN_SIZE_PX, ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Um campo da barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindField {
    Query,
    Replace,
    GoTo,
}

/// Onde caiu um clique na barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindHit {
    /// No campo, no byte mais próximo.
    Field(FindField, usize),
    Prev,
    Next,
    Case,
    Word,
    ReplaceOne,
    ReplaceAll,
    Close,
    /// Na barra, fora de qualquer controlo.
    Bar,
}

/// A contagem das ocorrências: `current` 0-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindCount {
    pub current: Option<usize>,
    pub total: usize,
}

/// O que a barra mostra neste quadro.
pub struct FindBar<'a> {
    pub query: &'a Buffer,
    /// `Some` = a linha de substituir está aberta.
    pub replace: Option<&'a Buffer>,
    /// `Some` = modo «Go to line» (só este campo).
    pub goto: Option<&'a Buffer>,
    /// O campo com o teclado (`None` = o texto do editor).
    pub focus: Option<FindField>,
    pub case_sensitive: bool,
    pub whole_word: bool,
    /// `None` = a busca está vazia.
    pub count: Option<FindCount>,
}

/// O texto da contagem: «3 of 12» ou «No results» (`None` com a busca vazia).
#[must_use]
pub fn count_text(count: Option<FindCount>) -> Option<String> {
    let c = count?;
    Some(match (c.total, c.current) {
        (0, _) => tr("code.find.none").to_owned(),
        (n, Some(i)) => tr_with("code.find.count", &[("current", &(i + 1)), ("total", &n)]),
        (n, None) => tr_with("code.find.total", &[("total", &n)]),
    })
}

/// Um campo pintado: o retângulo, o texto e onde ele começa (o clique volta ao byte por eles).
struct Painted {
    field: FindField,
    rect: Rect,
    text: String,
    x0: f32,
}

/// O estado de vista da barra: os alvos do último quadro.
pub struct FindBarView {
    area: Rect,
    targets: Vec<(FindHit, Rect)>,
    fields: Vec<Painted>,
}

impl Default for FindBarView {
    fn default() -> Self {
        Self {
            area: chrome::NO_RECT,
            targets: Vec::new(),
            fields: Vec::new(),
        }
    }
}

impl FindBarView {
    /// A altura da barra: uma linha (buscar · ir para linha) ou duas (com substituir).
    #[must_use]
    pub fn height(bar: &FindBar<'_>) -> f32 {
        let rows = if bar.goto.is_none() && bar.replace.is_some() {
            2.0
        } else {
            1.0
        };
        rows * ROW_H_PX + Spacing::Sm.px() * (rows + 1.0)
    }

    pub fn paint(
        &mut self,
        bar: &FindBar<'_>,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        self.area = area;
        self.targets.clear();
        self.fields.clear();
        chrome::panel_bg(scene, area, &ui, true);
        let gap = Spacing::Sm.px();
        let mut y = area.y + gap;
        let right = area.x + area.w - gap;
        if let Some(goto) = bar.goto {
            let label = tr("code.goto.label");
            let lw = chrome::label_w(ts, label);
            chrome::label(
                scene,
                ts,
                label,
                Rect::new(area.x + gap, y, lw, ROW_H_PX),
                &ui,
            );
            let close = self.icon_button(scene, IconId::Close, FindHit::Close, right, y, &ui);
            let x = area.x + gap * 2.0 + lw;
            let field = Rect::new(x, y, (close.x - gap - x).max(0.0), ROW_H_PX);
            self.field(
                scene,
                ts,
                FindField::GoTo,
                goto,
                tr("code.goto.placeholder"),
                field,
                bar.focus,
                &ui,
            );
            return;
        }
        // 1.ª linha: o campo, a contagem, ▲ ▼, Aa, W, ×.
        let mut x = right;
        let close = self.icon_button(scene, IconId::Close, FindHit::Close, x, y, &ui);
        x = close.x - gap;
        let word = self.toggle(
            scene,
            ts,
            tr("code.find.word"),
            FindHit::Word,
            bar.whole_word,
            x,
            y,
            &ui,
        );
        x = word.x - gap;
        let case = self.toggle(
            scene,
            ts,
            tr("code.find.case"),
            FindHit::Case,
            bar.case_sensitive,
            x,
            y,
            &ui,
        );
        x = case.x - gap;
        let next = self.icon_button(scene, IconId::ChevronDown, FindHit::Next, x, y, &ui);
        let prev = self.icon_button(scene, IconId::ChevronUp, FindHit::Prev, next.x, y, &ui);
        x = prev.x - gap;
        if let Some(text) = count_text(bar.count) {
            let w = chrome::label_w(ts, &text);
            x -= w;
            let tone = if bar.count.is_some_and(|c| c.total == 0) {
                ui.error
            } else {
                ui.text2
            };
            chrome::label_in(scene, ts, &text, Rect::new(x, y, w, ROW_H_PX), tone);
            x -= gap;
        }
        let x0 = area.x + gap;
        let field = Rect::new(x0, y, (x - x0).max(0.0), ROW_H_PX);
        self.field(
            scene,
            ts,
            FindField::Query,
            bar.query,
            tr("code.find.placeholder"),
            field,
            bar.focus,
            &ui,
        );
        // 2.ª linha: substituir.
        if let Some(rep) = bar.replace {
            y += ROW_H_PX + gap;
            let all = self.text_button(
                scene,
                ts,
                tr("code.find.replace_all"),
                FindHit::ReplaceAll,
                right,
                y,
                &ui,
            );
            let one = self.text_button(
                scene,
                ts,
                tr("code.find.replace_one"),
                FindHit::ReplaceOne,
                all.x - gap,
                y,
                &ui,
            );
            let field = Rect::new(x0, y, (one.x - gap - x0).max(0.0), ROW_H_PX);
            self.field(
                scene,
                ts,
                FindField::Replace,
                rep,
                tr("code.find.replace_placeholder"),
                field,
                bar.focus,
                &ui,
            );
        }
    }

    /// Onde caiu `(x, y)`; `None` fora da barra.
    #[must_use]
    pub fn hit(&self, ts: &mut TextSystem, x: f32, y: f32) -> Option<FindHit> {
        if !self.area.contains(x, y) {
            return None;
        }
        if let Some((h, _)) = self.targets.iter().find(|(_, r)| r.contains(x, y)) {
            return Some(*h);
        }
        for p in &self.fields {
            if p.rect.contains(x, y) {
                return Some(FindHit::Field(
                    p.field,
                    chrome::byte_at_x(ts, &p.text, x - p.x0),
                ));
            }
        }
        Some(FindHit::Bar)
    }

    /// O retângulo de um alvo do último quadro (os testes clicam nele).
    #[must_use]
    pub fn target_rect(&self, hit: FindHit) -> Option<Rect> {
        match hit {
            FindHit::Field(f, _) => self.fields.iter().find(|p| p.field == f).map(|p| p.rect),
            h => self.targets.iter().find(|(t, _)| *t == h).map(|(_, r)| *r),
        }
    }

    fn icon_button(
        &mut self,
        scene: &mut VectorScene,
        icon: IconId,
        hit: FindHit,
        right: f32,
        y: f32,
        ui: &Ui,
    ) -> Rect {
        let r = Rect::new(
            right - ICON_BTN_SIZE_PX.min(ROW_H_PX),
            y,
            ICON_BTN_SIZE_PX.min(ROW_H_PX),
            ROW_H_PX,
        );
        chrome::icon_button(scene, icon, r, ui);
        self.targets.push((hit, r));
        r
    }

    #[allow(clippy::too_many_arguments)]
    fn text_button(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        text: &str,
        hit: FindHit,
        right: f32,
        y: f32,
        ui: &Ui,
    ) -> Rect {
        let w = chrome::label_w(ts, text) + Spacing::Md.px() * 2.0;
        let r = Rect::new(right - w, y, w, ROW_H_PX);
        chrome::text_button(scene, ts, text, r, false, ui);
        self.targets.push((hit, r));
        r
    }

    #[allow(clippy::too_many_arguments)]
    fn toggle(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        text: &str,
        hit: FindHit,
        on: bool,
        right: f32,
        y: f32,
        ui: &Ui,
    ) -> Rect {
        let w = (chrome::label_w(ts, text) + Spacing::Sm.px() * 2.0).max(ROW_H_PX);
        let r = Rect::new(right - w, y, w, ROW_H_PX);
        chrome::text_button(scene, ts, text, r, on, ui);
        self.targets.push((hit, r));
        r
    }

    #[allow(clippy::too_many_arguments)]
    fn field(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        field: FindField,
        buf: &Buffer,
        placeholder: &str,
        rect: Rect,
        focus: Option<FindField>,
        ui: &Ui,
    ) {
        let x0 = chrome::text_field(scene, ts, buf, placeholder, rect, focus == Some(field), ui);
        self.fields.push(Painted {
            field,
            rect,
            text: buf.text().to_string(),
            x0,
        });
    }
}
