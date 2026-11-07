//! **A disposição de uma aba do editor de animações** — função da área, do texto medido e do
//! snapshot. Irmã do [`super::doc`] por responsabilidade (e pelo tecto de 700 linhas): lá os gestos,
//! aqui onde cada coisa fica. A lista ocupa a esquerda; à direita, a folha em cima e a animação aberta
//! embaixo ([estudo](../../../../docs/GameEngineFuncional/estudo_editor_de_animacoes_2026-11-07.md) §4).

use super::doc::{AnimDoc, Field, format_factor, format_fps};
use ph2d_editor_core::paint::{label_column_width, rect_for_label};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

/// A lista de animações à esquerda, como fração da largura (a da maquete, 150 de 820).
const LIST_FRAC: f32 = 0.18; // LITERAL-PX-OK: fração de layout
/// A largura mínima da lista, em alturas de linha.
const FIELD_ROWS: f32 = 3.0; // LITERAL-PX-OK: múltiplo do token de linha
/// O piso da miniatura, em alturas de linha.
const THUMB_ROWS: f32 = 2.0; // LITERAL-PX-OK: múltiplo do token de linha
/// A miniatura da faixa é a célula da folha pintada vezes isto (dono 25/10: «um pouco maior»).
const THUMB_OVER_CELL: f32 = 1.25; // LITERAL-PX-OK: razão adimensional
/// O tecto da miniatura: a fração da altura que ela e a folha repartem. A folha guarda sempre a outra
/// metade, porque é nela que se escolhem as células.
const THUMB_MAX_FRAC: f32 = 0.5; // LITERAL-PX-OK: fração de layout
/// As colunas da fila dos campos: Frames/s, This frame e as duas do marcador.
const FIELD_COLUMNS: f32 = 4.0; // LITERAL-PX-OK: contagem de colunas
/// O que a tira rola por pintura com a mão na borda, em alturas de linha.
const EDGE_SCROLL_ROWS: f32 = 0.25; // LITERAL-PX-OK: múltiplo do token de linha

/// A disposição de um quadro pintado.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub area: Rect,
    /// Os três cartões: a lista, a folha e a animação aberta.
    pub list: Rect,
    pub sheet_card: Rect,
    pub frames_card: Rect,
    /// O título de cada cartão (o da lista, o da folha e o «run · 8 frames», com o total à direita).
    pub list_title: Rect,
    pub sheet_title: Rect,
    pub frames_title: Rect,
    pub list_rows: Vec<Rect>,
    pub add: Rect,
    pub remove: Rect,
    /// **Split by rows**; vazio sem folha.
    pub split: Rect,
    pub cols: Rect,
    pub rows: Rect,
    /// A faixa «Parece 8 × 4 — Usar»: (texto, botão).
    pub banner: Option<(Rect, Rect)>,
    /// A folha encaixada (a imagem inteira).
    pub sheet: Rect,
    pub strip: Rect,
    /// Por quadro da faixa: (miniatura, ×).
    pub frames: Vec<(Rect, Rect)>,
    /// Durante o arrasto de um quadro: (o intervalo `0..=n`, a barra de inserção).
    pub marker: Option<(usize, Rect)>,
    pub fps: Rect,
    pub this_frame: Rect,
    /// O lugar do marcador, e, com um quadro escolhido, o botão e o campo do sinal.
    pub mark_bar: Rect,
    pub mark_toggle: Rect,
    pub mark_signal: Rect,
    pub modes: [Rect; 3],
}

impl Default for Layout {
    /// Nada pintado ainda: tudo vazio, nenhum clique acerta.
    fn default() -> Self {
        let z = Rect::new(0.0, 0.0, 0.0, 0.0);
        Self {
            area: z,
            list: z,
            sheet_card: z,
            frames_card: z,
            list_title: z,
            sheet_title: z,
            frames_title: z,
            list_rows: Vec::new(),
            add: z,
            remove: z,
            split: z,
            cols: z,
            rows: z,
            banner: None,
            sheet: z,
            strip: z,
            frames: Vec::new(),
            marker: None,
            fps: z,
            this_frame: z,
            mark_bar: z,
            mark_toggle: z,
            mark_signal: z,
            modes: [z; 3],
        }
    }
}

impl Layout {
    /// O passo de um quadro na faixa (miniatura mais o intervalo); `0` sem quadros.
    #[must_use]
    pub fn strip_pitch(&self) -> f32 {
        self.frames
            .first()
            .map_or(0.0, |f| f.0.w + ph2d_tokens::control_gap_px())
    }
}

impl AnimDoc {
    /// O valor do campo Frames/s; a unidade mora no rótulo (vazio sem animação aberta).
    #[must_use]
    pub fn fps_label(&self) -> String {
        self.snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected))
            .map(|r| format_fps(ph2d_ecs::fps_of_ms(r.frame_ms)))
            .unwrap_or_default()
    }

    /// O valor do campo This frame, ou o traço sem quadro escolhido.
    #[must_use]
    pub fn this_frame_label(&self) -> String {
        self.marked_frame().map_or_else(
            || ph2d_i18n::tr("anim_editor.no_frame").to_owned(),
            |f| format_factor(self.frame_factor_at(f)),
        )
    }

    /// **O lado da miniatura da faixa**: [`THUMB_OVER_CELL`] × a célula que a folha vai pintar.
    ///
    /// A folha e as miniaturas repartem `shared_h` (a folha fica com o que a miniatura deixa), então
    /// a célula depende da miniatura: com a folha presa pela altura, `t = f·c·(H − t)/sh` dá
    /// `t = f·c·H/(sh + f·c)`; presa pela largura, `t = f·c·W/sw` (`c` = o maior lado da célula em
    /// pixels da imagem). Vale o menor dos dois, depois o tecto [`THUMB_MAX_FRAC`] × `shared_h`, e
    /// por fim `min_side`.
    fn strip_thumb_side(&self, mid_w: f32, shared_h: f32, min_side: f32) -> f32 {
        let f = THUMB_OVER_CELL;
        let by_cell = match &self.snap.sheet {
            Some(s) if s.w > 0 && s.h > 0 => {
                let (sw, sh) = (s.w as f32, s.h as f32);
                let c = (sw / self.snap.hframes.max(1) as f32)
                    .max(sh / self.snap.vframes.max(1) as f32);
                (f * c * shared_h / (sh + f * c)).min(f * c * mid_w / sw)
            }
            _ => 0.0,
        };
        by_cell.min(shared_h * THUMB_MAX_FRAC).max(min_side)
    }

    /// O rótulo de um campo, ou o texto a ser escrito nele (com o cursor).
    fn shown(&self, f: &Field, label: String) -> String {
        match &self.editing {
            Some((g, buf)) if g == f => ph2d_i18n::tr_with("anim_editor.caret", &[("text", buf)]),
            _ => label,
        }
    }

    /// Calcula a disposição para `area` (e guarda-a: o rato lê a do último quadro pintado). Tudo fica
    /// acima da pílula de estado da janela (`HUD_H_PX + HUD_BOTTOM_PAD_PX`), que se pinta por cima da área.
    pub fn lay_out(&mut self, area: Rect, text: &mut TextSystem) -> &Layout {
        let pad = Spacing::Md.px();
        let inner = Spacing::Sm.px();
        let gap = ph2d_tokens::control_gap_px();
        let row = ROW_H_PX;
        let font = TypeToken::Sm.px();
        let close_side = rect_for_label(label_column_width(
            text,
            font,
            [ph2d_i18n::tr("anim_editor.remove_frame")],
        ));
        let mut fit = |labels: &[&str]| {
            rect_for_label(label_column_width(text, font, labels.iter().copied())).max(row)
        };
        let cols_w = fit(&[&self.shown(&Field::Cols, self.cols_label())]);
        let rows_w = fit(&[&self.shown(&Field::Rows, self.rows_label())]);
        let button_w = fit(&[
            ph2d_i18n::tr("anim_editor.add"),
            ph2d_i18n::tr("anim_editor.remove"),
        ]);
        let split_w = fit(&[ph2d_i18n::tr("anim_editor.split_by_rows")]);
        let n_frames = self.shown_frames().len();

        let mut l = Layout {
            area,
            ..Layout::default()
        };
        let (left, right) = (area.x + pad, area.x + area.w - pad);
        let top = area.y + pad;
        let bottom = area.y + area.h - pad - ph2d_tokens::HUD_H_PX - ph2d_tokens::HUD_BOTTOM_PAD_PX;
        let list_w = (area.w * LIST_FRAC)
            .max(row * FIELD_ROWS)
            .max(split_w + 2.0 * inner) // LITERAL-PX-OK: as duas margens
            .max(2.0 * (button_w + inner) + gap); // LITERAL-PX-OK: dois botões e as margens
        let mid_x = left + list_w + pad;
        let mid_w = (right - mid_x).max(0.0);

        // ── A lista ──
        l.list = Rect::new(left, top, list_w, (bottom - top).max(0.0));
        let list_in = Rect::new(
            left + inner,
            top + inner,
            (list_w - 2.0 * inner).max(0.0), // LITERAL-PX-OK: as duas margens
            row,
        );
        l.list_title = list_in;
        let n = self.snap.info.as_ref().map_or(0, |i| i.rows.len());
        l.list_rows = (0..n)
            .map(|i| Rect::new(list_in.x, list_in.y + (i + 1) as f32 * row, list_in.w, row))
            .collect();
        let split_h = if self.snap.sheet.is_some() {
            row + gap
        } else {
            0.0
        };
        let list_end = bottom - inner - row - split_h;
        let buttons_y = (list_in.y + (n + 1) as f32 * row + gap).min(list_end);
        // O piso da lista já dá `button_w` a cada um; o `max` tira o arredondamento que cortava «Remove».
        let half = ((list_in.w - gap) / 2.0).max(button_w); // LITERAL-PX-OK: dois botões iguais
        l.add = Rect::new(list_in.x, buttons_y, half, row);
        l.remove = Rect::new(list_in.x + half + gap, buttons_y, half, row);
        if split_h > 0.0 {
            l.split = Rect::new(list_in.x, buttons_y + split_h, list_in.w, row);
        }

        // ── A altura que a folha e as miniaturas repartem ──
        let card_w = (mid_w - 2.0 * inner).max(0.0); // LITERAL-PX-OK: as duas margens
        // Os campos numa fila só, ou, se os rótulos não couberem, o marcador na fila de baixo.
        let col_need = fit(&[
            ph2d_i18n::tr("anim_editor.this_frame"),
            ph2d_i18n::tr("anim_editor.marker_add"),
            ph2d_i18n::tr("anim_editor.marker_remove"),
        ]);
        let one_line = (card_w - (FIELD_COLUMNS - 1.0) * gap) / FIELD_COLUMNS >= col_need;
        let mark_line = if one_line { 0.0 } else { row + row + gap };
        let frames_fixed = 2.0 * inner + row + gap + gap + row + row + gap + row + mark_line; // LITERAL-PX-OK: as margens
        let banner_h = if self.proposal().is_some() {
            row + gap
        } else {
            0.0
        };
        let sheet_fixed = 2.0 * inner + row + gap + banner_h; // LITERAL-PX-OK: as margens
        let shared_h = (bottom - top - pad - frames_fixed - sheet_fixed).max(0.0);
        let thumb = self.strip_thumb_side(card_w, shared_h, row * THUMB_ROWS);

        // ── A animação aberta, embaixo ──
        let frames_h = frames_fixed + thumb;
        l.frames_card = Rect::new(mid_x, bottom - frames_h, mid_w, frames_h);
        let cx = mid_x + inner;
        let mut y = l.frames_card.y + inner;
        l.frames_title = Rect::new(cx, y, card_w, row);
        y += row + gap;
        l.strip = Rect::new(cx, y, card_w, thumb);
        y += thumb + gap + row;
        let columns = if one_line { FIELD_COLUMNS } else { 2.0 }; // LITERAL-PX-OK: duas colunas
        let col_w = ((card_w - (columns - 1.0) * gap) / columns).max(0.0);
        let col = |i: f32| cx + i * (col_w + gap);
        l.fps = Rect::new(col(0.0), y, col_w, row);
        l.this_frame = Rect::new(col(1.0), y, col_w, row);
        let (mark_x, mark_y) = if one_line {
            (col(2.0), y)
        } else {
            (cx, y + row + gap + row)
        };
        l.mark_bar = Rect::new(mark_x, mark_y, (cx + card_w - mark_x).max(0.0), row);
        if self.marked_frame().is_some() {
            l.mark_toggle = Rect::new(mark_x, mark_y, col_w, row);
            l.mark_signal = Rect::new(mark_x + col_w + gap, mark_y, col_w, row);
        }
        y = mark_y + row + gap;
        let mode_w = card_w / l.modes.len() as f32;
        for (i, m) in l.modes.iter_mut().enumerate() {
            *m = Rect::new(cx + i as f32 * mode_w, y, mode_w, row);
        }

        // ── A folha, em cima ──
        l.sheet_card = Rect::new(mid_x, top, mid_w, (l.frames_card.y - pad - top).max(0.0));
        let mut y = top + inner;
        let rows_x = cx + card_w - rows_w;
        l.rows = Rect::new(rows_x, y, rows_w, row);
        l.cols = Rect::new(rows_x - gap - cols_w, y, cols_w, row);
        l.sheet_title = Rect::new(cx, y, (l.cols.x - gap - cx).max(0.0), row);
        y += row + gap;
        if self.proposal().is_some() {
            let use_w = fit(&[ph2d_i18n::tr("anim_editor.use_grid")]);
            let text = Rect::new(cx, y, (card_w - use_w - gap).max(0.0), row);
            l.banner = Some((text, Rect::new(text.x + text.w + gap, y, use_w, row)));
            y += row + gap;
        }
        let mid = Rect::new(
            cx,
            y,
            card_w,
            (l.sheet_card.y + l.sheet_card.h - inner - y).max(0.0),
        );
        l.sheet = match &self.snap.sheet {
            Some(s) if s.w > 0 && s.h > 0 && mid.w > 0.0 && mid.h > 0.0 => {
                let k = (mid.w / s.w as f32).min(mid.h / s.h as f32);
                let (w, h) = (s.w as f32 * k, s.h as f32 * k);
                Rect::new(mid.x + (mid.w - w) / 2.0, mid.y + (mid.h - h) / 2.0, w, h)
            }
            _ => mid,
        };

        // ── Os quadros da tira ──
        self.scroll_at_edge(l.strip, row * EDGE_SCROLL_ROWS, row);
        let max_scroll = (n_frames as f32 * (thumb + gap) - gap - l.strip.w).max(0.0);
        self.strip_scroll = self.strip_scroll.clamp(0.0, max_scroll);
        l.frames = (0..n_frames)
            .map(|i| {
                let fx = l.strip.x + i as f32 * (thumb + gap) - self.strip_scroll;
                (
                    Rect::new(fx, l.strip.y, thumb, thumb),
                    Rect::new(fx + thumb - close_side, l.strip.y, close_side, close_side),
                )
            })
            .collect();
        self.drag_points_at(&l.frames);
        l.marker = self.frame_drag().and_then(|d| {
            let centre = match (l.frames.get(d.gap), l.frames.last()) {
                (Some(f), _) => f.0.x - gap / 2.0,
                (None, Some(f)) => f.0.x + f.0.w + gap / 2.0,
                (None, None) => return None,
            };
            let w = gap.max(ph2d_tokens::StrokeToken::Thick.px());
            Some((d.gap, Rect::new(centre - w / 2.0, l.strip.y, w, l.strip.h)))
        });
        self.layout = l;
        &self.layout
    }

    /// O rótulo do campo das colunas da grelha.
    #[must_use]
    pub fn cols_label(&self) -> String {
        ph2d_i18n::tr_with("anim_editor.cols", &[("n", &self.snap.hframes)])
    }

    /// O rótulo do campo das linhas da grelha.
    #[must_use]
    pub fn rows_label(&self) -> String {
        ph2d_i18n::tr_with("anim_editor.rows", &[("n", &self.snap.vframes)])
    }

    /// O multiplicador do quadro `i`, como a miniatura o mostra quando ele não é `1`.
    #[must_use]
    pub fn ms_label(&self, i: usize) -> String {
        ph2d_i18n::tr_with(
            "anim_editor.frame_factor",
            &[("x", &format_factor(self.frame_factor_at(i)))],
        )
    }

    /// Quanto dura um ciclo da animação aberta, em ms, sem as pausas de Hold e Repeat delay.
    #[must_use]
    pub fn total_ms(&self) -> u32 {
        (0..self.shown_frames().len())
            .map(|i| self.frame_ms_at(i))
            .sum()
    }
}
