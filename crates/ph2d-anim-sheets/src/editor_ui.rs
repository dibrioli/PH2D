//! ⭐ **As folhas no editor de animações** (plano GEF 26 §3.4, V4; maquete 26, painéis 4 e 6): o dropdown da folha
//! no título (só com 2 ou mais), a tira que pinta cada quadro com a imagem da SUA folha (e a marca da folha quando a
//! animação mistura), o quadro vazio destacado, as linhas em alerta e a limpeza ao pé da lista.
//!
//! O editor (`ph2d_app_sprite_anim::editor`) chama isto; a folha escolhida é a MESMA do cartão Image
//! (`ph2d_ecs::ShownSheet`), escrita pela porta `ImageUseEdit::ShowSheet`.

use crate::sheet_cache::{SheetCache, SheetKey};
use ph2d_ecs::{Entity, SpriteGrid, SpriteSheets, World};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, label_column_width, paint_text, paint_text_centered,
    paint_text_title, rect_for_label, resolve, stroke_rounded_rect_dashed,
};
use ph2d_editor_core::widget::sheet_cell::paint_sheet_cell;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{
    AnimFrameRef, ImageUseEdit, InspectorAnimInfo, InspectorAnimRow, SheetImage,
};
use ph2d_i18n::{tr, tr_with};
use ph2d_sprite_sheet::GridGuess;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};
use std::cell::RefCell;

/// O fundo de uma linha em alerta: o token de aviso, transparente o bastante para o texto ler-se por cima.
const ALERT_BACK_ALPHA: f32 = 0.35; // LITERAL-PX-OK: opacidade
/// O corpo do texto do editor: o dos campos dos cartões.
const FONT: TypeToken = TypeToken::Base;

/// A cor do editor: em cartões, o tom da lista; `PH2D_CARDS=0`, o de antes.
#[must_use]
pub fn tom(t: ColorToken, theme: Theme) -> Color {
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        return resolve(t, theme);
    }
    let cartao = match t {
        ColorToken::PanelBg => ColorToken::KanbanList,
        ColorToken::Bg1 => ColorToken::KanbanCard,
        ColorToken::Bg2 => ColorToken::CardField,
        ColorToken::Bg3 => ColorToken::KanbanCardRaised,
        ColorToken::Text1 => ColorToken::KanbanCardFg,
        ColorToken::Text2 | ColorToken::Text3 => ColorToken::CardLabel,
        o => return resolve(o, theme),
    };
    ph2d_editor_core::widget::card_ink::CardInk::of_list(theme)
        .color(cartao)
        .with_alpha(1.0)
}

/// Uma folha como o editor a mostra: a imagem (se descodificada), o corte e o nome.
#[derive(Clone, Debug, Default)]
pub struct EditorSheet {
    pub image: Option<SheetImage>,
    pub guess: Option<GridGuess>,
    pub hframes: u32,
    pub vframes: u32,
    pub name: String,
}

/// ⭐ **O nome que se mostra da folha `k`**, no editor e no cartão Image: o da biblioteca, ou «Sheet N» quando não há
/// (auditoria 08/11: o cartão mostrava um item em branco onde o editor dizia «Sheet 2»).
#[must_use]
pub fn sheet_label(name: Option<&str>, k: usize) -> String {
    name.filter(|n| !n.is_empty()).map_or_else(
        || tr_with("anim_editor.sheet_n", &[("n", &(k + 1))]),
        str::to_owned,
    )
}

/// As folhas de `bits` (a `0` primeiro, nunca vazio) e a visível (`ph2d_ecs::shown_sheet`). Lê a [`SheetCache`]
/// que `sheet_cache::refresh` resolveu neste quadro.
#[must_use]
pub fn editor_sheets(world: &World, bits: u64) -> (Vec<EditorSheet>, u8) {
    let e = Entity::from_bits(bits);
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let extra = world.get::<SpriteSheets>(e).map_or(&[][..], |s| &s.0[..]);
    let grids =
        std::iter::once([g.hframes, g.vframes]).chain(extra.iter().map(|x| [x.hframes, x.vframes]));
    let cache = world.get_resource::<SheetCache>();
    let sheets = grids
        .enumerate()
        .map(|(k, [hframes, vframes])| {
            let k8 = u8::try_from(k).ok();
            let entry = k8.and_then(|k| cache?.of_sheet(bits, k));
            // ⭐ Uma fonte só para o nome (o do cartão Image): a biblioteca; a arte de fábrica tem o seu.
            let name = match k8.and_then(|k| cache?.key_of(bits, k)) {
                Some(SheetKey::Asset(id)) => ph2d_ecs::sheet_name(world, id).map(str::to_owned),
                _ => entry.and_then(|x| x.name.clone()),
            };
            EditorSheet {
                image: entry.map(|x| x.sheet.clone()),
                guess: entry.and_then(|x| x.guess),
                hframes,
                vframes,
                name: sheet_label(name.as_deref(), k),
            }
        })
        .collect();
    (sheets, ph2d_ecs::shown_sheet(world, e))
}

/// Pinta o quadro `f` em `r` com a imagem da folha DELE. Devolve a folha pintada (`None`: vazio ou sem pixels).
pub fn paint_cell(
    scene: &mut VectorScene,
    sheets: &[EditorSheet],
    f: AnimFrameRef,
    r: Rect,
) -> Option<u8> {
    if f.is_empty() {
        return None;
    }
    let k = f.sheet;
    let s = sheets.get(usize::from(k))?;
    let image = s.image.as_ref()?;
    paint_sheet_cell(scene, image, s.hframes, s.vframes, f.cell, r);
    Some(k)
}

/// A animação usa quadros de mais de uma folha (os vazios não contam).
#[must_use]
pub fn mixes_sheets(frames: &[AnimFrameRef]) -> bool {
    let mut real = frames.iter().filter(|f| !f.is_empty());
    real.next()
        .is_some_and(|a| real.any(|b| b.sheet != a.sheet))
}

/// Quantos quadros vazios (de uma folha apagada) a animação tem.
#[must_use]
pub fn empty_frames(row: &InspectorAnimRow) -> usize {
    row.frames.iter().filter(|f| f.is_empty()).count()
}

/// A animação é TODA vazia (nenhum quadro de verdade)?
#[must_use]
pub fn all_empty(row: &InspectorAnimRow) -> bool {
    !row.frames.is_empty() && row.frames.iter().all(|f| f.is_empty())
}

/// Os botões de limpeza ao pé da lista: `0` sem vazios, `1` só Remove empty frames, `2` com uma animação toda vazia.
#[must_use]
pub fn clean_buttons(info: Option<&InspectorAnimInfo>) -> usize {
    let rows = info.map_or(&[][..], |i| &i.rows[..]);
    if !rows.iter().any(|r| empty_frames(r) > 0) {
        0
    } else if rows.iter().any(all_empty) {
        2
    } else {
        1
    }
}

/// O que a tira pintou num quadro (os gates leem-no).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramePaint {
    /// A folha cuja imagem foi pintada (`None` = nenhuma).
    pub sheet: Option<u8>,
    /// A marca da folha, quando a animação mistura.
    pub mark: Option<String>,
    /// Pintado como vazio (contorno tracejado de aviso e ⚠).
    pub empty: bool,
}

/// O estado das folhas numa aba do editor: o dropdown, os botões e o que a última pintura pôs.
#[derive(Debug, Default)]
pub struct SheetUi {
    pub open: bool,
    /// O botão «idle ▾» (só com 2 ou mais folhas).
    pub picker: Option<Rect>,
    /// Os itens do dropdown aberto, um por folha.
    pub items: Vec<Rect>,
    /// **Remove empty frames** · **Delete empty animations**, se à vista.
    pub clean: [Option<Rect>; 2],
    /// As edições à espera da família (`ImageUseEdit`, pela porta do cartão Image).
    pub pending: Vec<ImageUseEdit>,
    /// Por quadro da tira, o que a última pintura pôs.
    pub painted: RefCell<Vec<FramePaint>>,
    /// As linhas da lista pintadas em alerta.
    pub alerts: RefCell<Vec<usize>>,
}

fn hit(r: Option<Rect>, x: f32, y: f32) -> bool {
    r.is_some_and(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
}

fn button(scene: &mut VectorScene, text: &mut TextSystem, theme: Theme, r: Rect, label: &str) {
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, Radius::Sm.px()),
        tom(ColorToken::Bg3, theme),
    );
    paint_text_centered(
        text,
        scene,
        label,
        r,
        FONT.px(),
        tom(ColorToken::Text1, theme),
    );
}

impl SheetUi {
    /// Um botão premido em `(x, y)`: o dropdown e a limpeza. Devolve se o tomou (com o dropdown aberto, qualquer
    /// clique o fecha e fica aqui, como o véu do cartão Image).
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        if self.open {
            self.open = false;
            if let Some(k) = self.items.iter().position(|r| hit(Some(*r), x, y))
                && let Ok(k) = u8::try_from(k)
            {
                self.pending.push(ImageUseEdit::ShowSheet(k));
            }
            return true;
        }
        if hit(self.picker, x, y) {
            self.open = true;
            return true;
        }
        let edits = [
            ImageUseEdit::RemoveEmptyFrames,
            ImageUseEdit::DeleteEmptyAnimations,
        ];
        for (r, edit) in self.clean.iter().zip(edits) {
            if hit(*r, x, y) {
                self.pending.push(edit);
                return true;
            }
        }
        false
    }

    /// O título da folha: «SHEET» em destaque (dono 08/11), com 2 ou mais folhas o botão «nome ▾», e a dica ao lado
    /// enquanto couber.
    pub fn paint_title(
        &mut self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
        (title, hint): (Rect, &str),
        sheets: &[EditorSheet],
        shown: u8,
    ) {
        self.painted.borrow_mut().clear();
        let font = FONT.px();
        let y = title.y + (title.h - ROW_H_PX).max(0.0) / 2.0;
        let mut x = title.x;
        self.picker = None;
        if sheets.len() >= 2 {
            let name = sheets
                .get(usize::from(shown))
                .map_or("", |s| s.name.as_str());
            let label = tr_with("anim_editor.sheet_pick", &[("name", &name)]);
            let w = rect_for_label(label_column_width(text, font, [label.as_str()]));
            let r = Rect::new(x, title.y, w.min(title.w), title.h);
            button(scene, text, theme, r, &label);
            self.picker = Some(r);
            x += r.w + Spacing::Md.px();
        } else {
            self.open = false;
        }
        let head = tr("anim_editor.sheet");
        let fim = title.x + title.w;
        paint_text_title(
            text,
            scene,
            head,
            x,
            y,
            font,
            (fim - x).max(0.0),
            tom(ColorToken::Text1, theme),
        );
        x += label_column_width(text, font, [head]) + Spacing::Xs.px();
        if label_column_width(text, font, [hint]) <= fim - x {
            let c = tom(ColorToken::Text2, theme);
            paint_text(text, scene, hint, x, y, font, fim - x, c);
        }
    }

    /// O dropdown aberto, por cima de tudo: um item por folha (`nome · C × R`), a visível com ✓.
    pub fn paint_menu(
        &mut self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
        sheets: &[EditorSheet],
        shown: u8,
    ) {
        self.items.clear();
        let Some(p) = self.picker.filter(|_| self.open) else {
            return;
        };
        let font = FONT.px();
        let labels: Vec<String> = sheets
            .iter()
            .enumerate()
            .map(|(k, s)| {
                let key = if k == usize::from(shown) {
                    "anim_editor.sheet_item_on"
                } else {
                    "anim_editor.sheet_item"
                };
                tr_with(
                    key,
                    &[
                        ("name", &s.name),
                        ("cols", &s.hframes),
                        ("rows", &s.vframes),
                    ],
                )
            })
            .collect();
        let w = rect_for_label(label_column_width(
            text,
            font,
            labels.iter().map(String::as_str),
        ))
        .max(p.w);
        let back = Rect::new(p.x, p.y + p.h, w, ROW_H_PX * labels.len() as f32);
        fill_rounded_rect(
            scene,
            back,
            frame_radius(theme, Radius::Sm.px()),
            tom(ColorToken::Bg3, theme),
        );
        for (k, label) in labels.iter().enumerate() {
            let r = Rect::new(back.x, back.y + k as f32 * ROW_H_PX, w, ROW_H_PX);
            if k == usize::from(shown) {
                fill_rounded_rect(
                    scene,
                    r,
                    frame_radius(theme, Radius::Sm.px()),
                    tom(ColorToken::Bg2, theme),
                );
            }
            let at = r.x + Spacing::Sm.px();
            paint_text(
                text,
                scene,
                label,
                at,
                r.y,
                font,
                r.w,
                tom(ColorToken::Text1, theme),
            );
            self.items.push(r);
        }
    }

    /// O quadro `i` da tira: a imagem da folha dele; o vazio, tracejado de aviso com ⚠; a marca da folha quando
    /// a animação mistura.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_frame(
        &self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
        sheets: &[EditorSheet],
        frames: &[AnimFrameRef],
        i: usize,
        thumb: Rect,
    ) {
        let Some(&f) = frames.get(i) else {
            return;
        };
        let font = FONT.px();
        let mut out = FramePaint {
            sheet: None,
            mark: None,
            empty: f.is_empty(),
        };
        if out.empty {
            let warn = tom(ColorToken::Warn, theme);
            let dash = (f64::from(Spacing::Xs.px()), f64::from(Spacing::Xxs.px()));
            let radius = frame_radius(theme, Radius::Sm.px());
            stroke_rounded_rect_dashed(scene, thumb, radius, StrokeToken::Thick.px(), dash, warn);
            paint_text_centered(
                text,
                scene,
                tr("anim_editor.empty_frame"),
                thumb,
                font,
                warn,
            );
        } else {
            out.sheet = paint_cell(scene, sheets, f, thumb);
            if mixes_sheets(frames)
                && let Some(s) = sheets.get(usize::from(f.sheet))
            {
                let small = TypeToken::Sm.px();
                let w =
                    rect_for_label(label_column_width(text, small, [s.name.as_str()])).min(thumb.w);
                let r = Rect::new(
                    thumb.x + thumb.w - w,
                    thumb.y + thumb.h - ROW_H_PX,
                    w,
                    ROW_H_PX,
                );
                fill_rounded_rect(
                    scene,
                    r,
                    frame_radius(theme, Radius::Sm.px()),
                    tom(ColorToken::Bg2, theme),
                );
                paint_text_centered(
                    text,
                    scene,
                    &s.name,
                    r,
                    small,
                    tom(ColorToken::Text1, theme),
                );
                out.mark = Some(s.name.clone());
            }
        }
        self.painted.borrow_mut().push(out);
    }

    /// A linha `i` da lista em alerta, se a animação tem vazios: «⚠ nome» e «N empty» sobre o tom de aviso.
    /// Devolve se a pintou (o editor salta então a linha de sempre).
    #[allow(clippy::too_many_arguments)]
    pub fn paint_alert_row(
        &self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
        r: Rect,
        i: usize,
        row: &InspectorAnimRow,
    ) -> bool {
        let n = empty_frames(row);
        if n == 0 {
            return false;
        }
        let font = FONT.px();
        let warn = tom(ColorToken::Warn, theme);
        fill_rounded_rect(
            scene,
            r,
            frame_radius(theme, Radius::Sm.px()),
            warn.with_alpha(ALERT_BACK_ALPHA),
        );
        let count = tr_with("panel.inspector.anim_card.empty", &[("n", &n)]);
        let count_w = label_column_width(text, font, [count.as_str()]);
        let inset = Spacing::Sm.px();
        let name = tr_with(
            "panel.inspector.anim_card.alert_name",
            &[("name", &row.name)],
        );
        let name_w = (r.w - count_w - 2.0 * inset).max(0.0); // LITERAL-PX-OK: as duas margens
        paint_text(
            text,
            scene,
            &name,
            r.x + inset,
            r.y,
            font,
            name_w,
            tom(ColorToken::Text1, theme),
        );
        paint_text(
            text,
            scene,
            &count,
            r.x + r.w - count_w - inset,
            r.y,
            font,
            count_w,
            warn,
        );
        self.alerts.borrow_mut().push(i);
        true
    }

    /// Ao pé da lista (pinta-se ANTES dela: limpa os alertas do quadro), sob `foot` (o último botão), um por linha da
    /// largura `w`: **Remove empty frames** e, com uma animação toda vazia, **Delete empty animations**.
    pub fn paint_clean(
        &mut self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
        (foot, w): (Rect, f32),
        info: Option<&InspectorAnimInfo>,
    ) {
        self.clean = [None; 2];
        self.alerts.borrow_mut().clear();
        let keys = [
            "panel.inspector.anim_card.remove_empty_frames",
            "panel.inspector.anim_card.delete_empty_anims",
        ];
        let step = foot.h + ph2d_tokens::control_gap_px();
        for (k, key) in keys.iter().enumerate().take(clean_buttons(info)) {
            let r = Rect::new(foot.x, foot.y + step * (k + 1) as f32, w, foot.h);
            button(scene, text, theme, r, tr(key));
            self.clean[k] = Some(r);
        }
    }
}

#[cfg(test)]
#[path = "editor_ui_tests.rs"]
mod tests;
