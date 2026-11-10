//! ⭐ **E11 — a lista de scripts do projeto**, estreita e recolhível, à esquerda dentro da aba de
//! código: o filtro, «New script», um script por linha (o aberto realçado, ● com alterações por
//! salvar), renomear (duplo clique) e apagar (o lixo da linha escolhida). Aqui só se pinta e se
//! responde «onde caiu o clique»; a lei (nome único, recusa de apagar um script em uso) é da família.

use crate::chrome::{self, Ui};
use crate::peso::{paint_text_block, paint_text_elided};
use ph2d_code_buffer::Buffer;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ICON_BTN_SIZE_PX, PANEL_MIN_W_PX, ROW_H_PX, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Um campo de texto da lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListField {
    Filter,
    Rename,
}

/// Onde caiu um clique na lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListHit {
    /// Recolher / mostrar a lista.
    Toggle,
    New,
    /// No campo, no byte mais próximo.
    Field(ListField, usize),
    /// Na linha do script `id` (o id da aba).
    Row(u64),
    /// No lixo da linha do script `id`.
    Delete(u64),
    /// Na lista, fora de qualquer alvo.
    Body,
}

/// Uma linha: um script do projeto.
pub struct ListRow<'a> {
    pub id: u64,
    pub name: &'a str,
    pub dirty: bool,
    /// O script da aba activa (realçado, com o lixo).
    pub current: bool,
    /// Quantos objectos correm o script (à direita: «2», ou «unused» apagado).
    pub users: usize,
}

/// O que a lista mostra neste quadro.
pub struct ScriptList<'a> {
    pub open: bool,
    pub filter: &'a Buffer,
    /// O campo com o teclado (`None` = o texto do editor).
    pub focus: Option<ListField>,
    /// As linhas que passam o filtro.
    pub rows: &'a [ListRow<'a>],
    /// O script a ser renomeado e o campo com o nome novo.
    pub renaming: Option<(u64, &'a Buffer)>,
    /// A frase de uma recusa (apagar um script em uso), no fundo da lista.
    pub notice: Option<&'a str>,
    /// O script cuja contagem está sob o rato (o balão «Used by N objects»).
    pub hover_count: Option<u64>,
}

/// Uma contagem pintada: o script, o texto e onde.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintedCount {
    pub id: u64,
    pub users: usize,
    pub text: String,
    pub rect: Rect,
}

struct Painted {
    field: ListField,
    rect: Rect,
    text: String,
    x0: f32,
}

/// O estado de vista da lista: os alvos do último quadro e a rolagem das linhas.
pub struct ScriptListView {
    area: Rect,
    targets: Vec<(ListHit, Rect)>,
    fields: Vec<Painted>,
    rows_rect: Rect,
    scroll: f32,
    content_h: f32,
    counts: Vec<PaintedCount>,
    tip: Option<String>,
    /// A largura em que o cabeçalho e cada linha cabem inteiros (nome, contagem, lixo).
    fit_w: f32,
}

impl Default for ScriptListView {
    fn default() -> Self {
        Self {
            area: chrome::NO_RECT,
            targets: Vec::new(),
            fields: Vec::new(),
            rows_rect: chrome::NO_RECT,
            scroll: 0.0,
            content_h: 0.0,
            counts: Vec::new(),
            tip: None,
            fit_w: 0.0,
        }
    }
}

impl ScriptListView {
    /// A largura que a lista toma: a do painel mais estreito aberta, a de um botão recolhida.
    #[must_use]
    pub fn width(open: bool) -> f32 {
        if open {
            PANEL_MIN_W_PX
        } else {
            ICON_BTN_SIZE_PX.min(ROW_H_PX) + Spacing::Sm.px() * 2.0
        }
    }

    /// A altura que a lista aberta pede para `rows` linhas: os vãos, o título, o filtro e as linhas
    /// (a mesma conta do [`Self::paint`]).
    #[must_use]
    pub fn height_for(rows: usize) -> f32 {
        let g = Spacing::Sm.px();
        g + ROW_H_PX + g + ROW_H_PX + g + ROW_H_PX * rows.max(1) as f32 + g
    }

    /// A largura mínima em que o cabeçalho (`head`: os dois botões e as folgas) e cada linha de
    /// `list` cabem sem cortar: folga · nome · folga · contagem · folga (+ o lixo na actual),
    /// mais a folga de fora da linha — a mesma conta de [`Self::paint`].
    fn fit_of(ts: &mut TextSystem, list: &ScriptList<'_>, head: f32) -> f32 {
        let gap = Spacing::Sm.px();
        let trash = ICON_BTN_SIZE_PX.min(ROW_H_PX);
        list.rows.iter().fold(head, |w, row| {
            let name = if row.dirty {
                tr_with("doc.tab.dirty_title", &[("name", &row.name)])
            } else {
                row.name.to_owned()
            };
            let count = if row.users == 0 {
                tr("code.list.unused").to_owned()
            } else {
                row.users.to_string()
            };
            let lixo = if row.current { trash } else { 0.0 };
            let folgas = gap * 4.0; // LITERAL-PX-OK: fora · nome · contagem · fim
            let need = folgas + chrome::label_w(ts, &name) + chrome::label_w(ts, &count) + lixo;
            w.max(need)
        })
    }

    /// A largura em que a lista do último quadro cabe inteira ([`Self::fit_of`]).
    #[must_use]
    pub fn fit_w(&self) -> f32 {
        self.fit_w
    }

    /// A área do último quadro.
    #[must_use]
    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn paint(
        &mut self,
        list: &ScriptList<'_>,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        self.area = area;
        self.targets.clear();
        self.fields.clear();
        self.counts.clear();
        self.tip = None;
        // A lista é um cartão (dono 08/11): o fundo dele, sem a linha de borda à direita.
        scene.fill_rect(rect_to_vello(area), ui.bg);
        let gap = Spacing::Sm.px();
        let right = area.x + area.w - gap;
        let y = area.y + gap;
        let toggle_icon = if list.open {
            IconId::ChevronLeft
        } else {
            IconId::ChevronRight
        };
        let toggle = self.icon_button(scene, toggle_icon, ListHit::Toggle, right, y, &ui);
        if !list.open {
            self.rows_rect = chrome::NO_RECT;
            return;
        }
        let new = self.icon_button(scene, IconId::Plus, ListHit::New, toggle.x, y, &ui);
        self.fit_w = Self::fit_of(ts, list, area.x + area.w - new.x + gap * 2.0);
        let title = tr("code.list.title");
        let caixa = Rect::new(
            area.x + gap,
            y,
            (new.x - gap - area.x - gap).max(0.0),
            ROW_H_PX,
        );
        if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
            // O título da coluna no idioma da lista de cartões, como HIERARCHY e INSPECTOR.
            use ph2d_editor_core::panel::cards::banda::metrics as m;
            ph2d_editor_core::paint::paint_text_title_elided_tracked(
                ts,
                scene,
                &title.to_uppercase(),
                [
                    caixa.x,
                    caixa.y + ph2d_editor_core::paint::folga_do_texto(caixa.h, m::LIST_TITLE_PX),
                ],
                m::LIST_TITLE_PX,
                caixa.w,
                ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::KanbanListFg, *theme),
                m::LIST_TITLE_TRACKING_EM,
            );
        } else {
            chrome::label(scene, ts, title, caixa, &ui);
        }
        let y = y + ROW_H_PX + gap;
        let field = Rect::new(area.x + gap, y, (area.w - gap * 2.0).max(0.0), ROW_H_PX);
        let focus = list.focus == Some(ListField::Filter);
        self.field(
            scene,
            ts,
            ListField::Filter,
            list.filter,
            tr("code.list.filter"),
            field,
            focus,
            &ui,
        );
        let top = y + ROW_H_PX + gap;
        let notice_h = list.notice.map_or(0.0, |n| {
            let w = (area.w - gap * 2.0).max(0.0);
            let mut probe = VectorScene::new();
            paint_text_block(ts, &mut probe, n, 0.0, 0.0, chrome::font(), w, ui.error) + gap * 2.0
        });
        let bottom = area.y + area.h - notice_h;
        self.rows_rect = Rect::new(area.x, top, area.w, (bottom - top).max(0.0));
        self.content_h = ROW_H_PX * list.rows.len().max(1) as f32;
        self.clamp_scroll();
        scene.push_clip(&rect_to_vello(self.rows_rect));
        if list.rows.is_empty() {
            chrome::label_in(
                scene,
                ts,
                tr("code.list.no_match"),
                Rect::new(area.x + gap, top, (area.w - gap * 2.0).max(0.0), ROW_H_PX),
                ui.text3,
            );
        }
        for (i, row) in list.rows.iter().enumerate() {
            let ry = top + ROW_H_PX * i as f32 - self.scroll;
            if ry + ROW_H_PX < top || ry > bottom {
                continue;
            }
            let r = Rect::new(area.x + gap * 0.5, ry, (area.w - gap).max(0.0), ROW_H_PX);
            self.row(scene, ts, list, row, r, &ui, *theme);
        }
        scene.pop_layer();
        self.count_tip(scene, ts, list, &ui);
        if let Some(n) = list.notice {
            let w = (area.w - gap * 2.0).max(0.0);
            paint_text_block(
                ts,
                scene,
                n,
                area.x + gap,
                bottom + gap,
                chrome::font(),
                w,
                ui.error,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn row(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        list: &ScriptList<'_>,
        row: &ListRow<'_>,
        r: Rect,
        ui: &Ui,
        theme: Theme,
    ) {
        if row.current {
            let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, r, radius, ui.button_on);
        }
        let gap = Spacing::Sm.px();
        let mut text_right = r.x + r.w - gap;
        if row.current {
            let del = self.icon_button(
                scene,
                IconId::Trash,
                ListHit::Delete(row.id),
                r.x + r.w,
                r.y,
                ui,
            );
            text_right = del.x - gap;
        }
        if list.renaming.is_none_or(|(id, _)| id != row.id) {
            text_right = self.count(scene, ts, row, r, text_right, ui);
        }
        if let Some((_, buf)) = list.renaming.filter(|(id, _)| *id == row.id) {
            let field = Rect::new(r.x + gap, r.y, (text_right - r.x - gap).max(0.0), r.h);
            let focus = list.focus == Some(ListField::Rename);
            self.field(
                scene,
                ts,
                ListField::Rename,
                buf,
                row.name,
                field,
                focus,
                ui,
            );
            return;
        }
        let name = if row.dirty {
            tr_with("doc.tab.dirty_title", &[("name", &row.name)])
        } else {
            row.name.to_owned()
        };
        let f = chrome::font();
        let color = if row.current { ui.text1 } else { ui.text2 };
        let w = (text_right - r.x - gap).max(0.0);
        paint_text_elided(
            ts,
            scene,
            &name,
            r.x + gap,
            r.y + ph2d_editor_core::paint::folga_do_texto(r.h, f),
            f,
            w,
            color,
        );
        // A linha inteira é alvo — depois do lixo, que ganha por vir primeiro.
        self.targets.push((ListHit::Row(row.id), r));
    }

    /// A contagem à direita da linha, encostada a `right`; devolve onde o nome tem de parar.
    fn count(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        row: &ListRow<'_>,
        r: Rect,
        right: f32,
        ui: &Ui,
    ) -> f32 {
        let text = if row.users == 0 {
            tr("code.list.unused").to_owned()
        } else {
            row.users.to_string()
        };
        let color = if row.users == 0 { ui.text3 } else { ui.text2 };
        let w = chrome::label_w(ts, &text);
        let rect = Rect::new(right - w, r.y, w, r.h);
        chrome::label_in(scene, ts, &text, rect, color);
        self.counts.push(PaintedCount {
            id: row.id,
            users: row.users,
            text,
            rect,
        });
        rect.x - Spacing::Sm.px()
    }

    /// O balão da contagem sob o rato, por baixo dela, dentro da lista.
    fn count_tip(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        list: &ScriptList<'_>,
        ui: &Ui,
    ) {
        let Some(c) = list
            .hover_count
            .and_then(|id| self.counts.iter().find(|c| c.id == id))
            .filter(|c| self.rows_rect.contains(c.rect.x, c.rect.y + c.rect.h * 0.5))
        else {
            return;
        };
        let tip = if c.users == 0 {
            tr("code.list.used_by_none").to_owned()
        } else {
            tr_with("code.list.used_by", &[("count", &c.users)])
        };
        let gap = Spacing::Sm.px();
        let max_w = (self.area.w - gap * 2.0).max(0.0);
        let w = (chrome::label_w(ts, &tip) + gap * 2.0).min(max_w);
        let mut probe = VectorScene::new();
        let h = paint_text_block(
            ts,
            &mut probe,
            &tip,
            0.0,
            0.0,
            chrome::font(),
            w - gap * 2.0,
            ui.text1,
        ) + gap * 2.0;
        let x = (c.rect.x + c.rect.w - w).max(self.area.x + gap);
        let below = c.rect.y + c.rect.h;
        let y = if below + h > self.area.y + self.area.h {
            c.rect.y - h
        } else {
            below
        };
        let b = Rect::new(x, y, w, h);
        let radius = ph2d_editor_core::paint::frame_radius(ui.theme, Radius::Sm.px());
        fill_rounded_rect(scene, b, radius, ui.field_edge);
        let t = ph2d_tokens::StrokeToken::Thin.px();
        fill_rounded_rect(
            scene,
            Rect::new(
                b.x + t,
                b.y + t,
                (b.w - t * 2.0).max(0.0),
                (b.h - t * 2.0).max(0.0),
            ),
            radius,
            ui.field,
        );
        paint_text_block(
            ts,
            scene,
            &tip,
            b.x + gap,
            b.y + gap,
            chrome::font(),
            w - gap * 2.0,
            ui.text1,
        );
        self.tip = Some(tip);
    }

    /// O script cuja contagem está em `(x, y)` (o rato pede o balão por aqui).
    #[must_use]
    pub fn count_at(&self, x: f32, y: f32) -> Option<u64> {
        if !self.rows_rect.contains(x, y) {
            return None;
        }
        self.counts
            .iter()
            .find(|c| c.rect.contains(x, y))
            .map(|c| c.id)
    }

    /// As contagens pintadas no último quadro (os testes lêem-nas).
    #[must_use]
    pub fn painted_counts(&self) -> &[PaintedCount] {
        &self.counts
    }

    /// O balão da contagem pintado no último quadro.
    #[must_use]
    pub fn painted_tip(&self) -> Option<&str> {
        self.tip.as_deref()
    }

    fn clamp_scroll(&mut self) {
        let max = (self.content_h - self.rows_rect.h).max(0.0);
        self.scroll = ph2d_editor_core::math::safe_clamp(self.scroll, 0.0, max);
    }

    /// A roda sobre as linhas. `true` = tomada.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        if !self.area.contains(x, y) {
            return false;
        }
        self.scroll -= dy;
        self.clamp_scroll();
        true
    }

    /// Onde caiu `(x, y)`; `None` fora da lista.
    #[must_use]
    pub fn hit(&self, ts: &mut TextSystem, x: f32, y: f32) -> Option<ListHit> {
        if !self.area.contains(x, y) {
            return None;
        }
        for p in &self.fields {
            if p.rect.contains(x, y) {
                return Some(ListHit::Field(
                    p.field,
                    chrome::byte_at_x(ts, &p.text, x - p.x0),
                ));
            }
        }
        if let Some((h, _)) = self.targets.iter().find(|(h, r)| {
            r.contains(x, y) && (!matches!(h, ListHit::Row(_)) || self.rows_rect.contains(x, y))
        }) {
            return Some(*h);
        }
        Some(ListHit::Body)
    }

    /// O retângulo de um alvo do último quadro (os testes clicam nele).
    #[must_use]
    pub fn target_rect(&self, hit: ListHit) -> Option<Rect> {
        match hit {
            ListHit::Field(f, _) => self.fields.iter().find(|p| p.field == f).map(|p| p.rect),
            h => self.targets.iter().find(|(t, _)| *t == h).map(|(_, r)| *r),
        }
    }

    fn icon_button(
        &mut self,
        scene: &mut VectorScene,
        icon: IconId,
        hit: ListHit,
        right: f32,
        y: f32,
        ui: &Ui,
    ) -> Rect {
        let s = ICON_BTN_SIZE_PX.min(ROW_H_PX);
        let r = Rect::new(right - s, y, s, ROW_H_PX);
        chrome::icon_button(scene, icon, r, ui);
        self.targets.push((hit, r));
        r
    }

    #[allow(clippy::too_many_arguments)]
    fn field(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        field: ListField,
        buf: &Buffer,
        placeholder: &str,
        rect: Rect,
        focused: bool,
        ui: &Ui,
    ) {
        let x0 = chrome::text_field(scene, ts, buf, placeholder, rect, focused, ui);
        self.fields.push(Painted {
            field,
            rect,
            text: buf.text().to_string(),
            x0,
        });
    }
}
