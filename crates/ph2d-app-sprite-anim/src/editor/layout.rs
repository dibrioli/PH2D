//! **A disposição de uma aba do editor de animações** — função da área, do texto medido e do
//! snapshot. Irmã do [`super::doc`] por responsabilidade (e pelo tecto de 700 linhas): lá os gestos,
//! aqui onde cada coisa fica.

use super::doc::{AnimDoc, Field, format_fps};
use ph2d_editor_core::paint::{label_column_width, rect_for_label};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

/// A lista de animações à esquerda, como fração da largura.
const LIST_FRAC: f32 = 0.22; // LITERAL-PX-OK: fração de layout
/// A largura mínima da lista, em alturas de linha.
const FIELD_ROWS: f32 = 3.0; // LITERAL-PX-OK: múltiplo do token de linha
/// A faixa de quadros: miniatura de duas linhas mais a linha do tempo.
const THUMB_ROWS: f32 = 2.0; // LITERAL-PX-OK: múltiplo do token de linha
/// A miniatura da faixa é a célula da folha pintada vezes isto (dono 25/10: «um pouco maior»).
const THUMB_OVER_CELL: f32 = 1.25; // LITERAL-PX-OK: razão adimensional
/// O tecto da miniatura: a fração da altura que ela e a folha repartem. A folha guarda sempre a outra
/// metade, porque é nela que se escolhem as células.
const THUMB_MAX_FRAC: f32 = 0.5; // LITERAL-PX-OK: fração de layout
/// A disposição de um quadro pintado.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub area: Rect,
    pub play: Rect,
    pub fps: Rect,
    pub modes: [Rect; 3],
    pub cols: Rect,
    pub rows: Rect,
    /// A faixa «Parece 8 × 4 — Usar»: (texto, botão).
    pub banner: Option<(Rect, Rect)>,
    pub list: Rect,
    pub list_rows: Vec<Rect>,
    pub add: Rect,
    pub remove: Rect,
    /// A folha encaixada (a imagem inteira), dentro da área do meio.
    pub sheet: Rect,
    pub strip: Rect,
    /// Por quadro da faixa: (miniatura, tempo, ×).
    pub frames: Vec<(Rect, Rect, Rect)>,
    /// Durante o arrasto de um quadro: (o intervalo `0..=n`, a barra de inserção).
    pub marker: Option<(usize, Rect)>,
}

impl Default for Layout {
    /// Nada pintado ainda: tudo vazio, nenhum clique acerta.
    fn default() -> Self {
        let z = Rect::new(0.0, 0.0, 0.0, 0.0);
        Self {
            area: z,
            play: z,
            fps: z,
            modes: [z; 3],
            cols: z,
            rows: z,
            banner: None,
            list: z,
            list_rows: Vec::new(),
            add: z,
            remove: z,
            sheet: z,
            strip: z,
            frames: Vec::new(),
            marker: None,
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
    /// O rótulo do campo dos quadros por segundo (vazio sem animação aberta).
    #[must_use]
    pub fn fps_label(&self) -> String {
        self.snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected))
            .map(|r| {
                ph2d_i18n::tr_with(
                    "anim_editor.fps",
                    &[("fps", &format_fps(ph2d_ecs::fps_of_ms(r.frame_ms)))],
                )
            })
            .unwrap_or_default()
    }

    /// **O lado da miniatura da faixa**: [`THUMB_OVER_CELL`] × a célula que a folha vai pintar.
    ///
    /// A folha e as miniaturas repartem `shared_h` (a folha fica com o que a miniatura deixa), então
    /// a célula depende da miniatura: com a folha presa pela altura, `t = f·c·(H − t)/sh` dá
    /// `t = f·c·H/(sh + f·c)`; presa pela largura, `t = f·c·W/sw` (`c` = o maior lado da célula em
    /// pixels da imagem). Vale o menor dos dois, depois o tecto [`THUMB_MAX_FRAC`] × `shared_h`, e
    /// por fim `min_side` — o número e o «NNN ms» legíveis ganham a tudo.
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

    /// Calcula a disposição para `area` (e guarda-a: o rato lê a do último quadro pintado).
    ///
    /// Cada controlo da barra mede o próprio texto (o respiro é o do pintor, `rect_for_label`) e a
    /// barra quebra para a linha de baixo quando não cabe na largura. A faixa de quadros fica acima
    /// da pílula de estado da janela (`HUD_H_PX + HUD_BOTTOM_PAD_PX`), que se pinta por cima da área.
    pub fn lay_out(&mut self, area: Rect, text: &mut TextSystem) -> &Layout {
        let pad = Spacing::Md.px();
        let gap = ph2d_tokens::control_gap_px();
        let row = ROW_H_PX;
        let font = TypeToken::Sm.px();
        // O «×» de um quadro: um quadrado do tamanho que o sinal pede.
        let close_side = rect_for_label(label_column_width(
            text,
            font,
            [ph2d_i18n::tr("anim_editor.remove_frame")],
        ));
        let mut fit = |labels: &[&str]| {
            rect_for_label(label_column_width(text, font, labels.iter().copied())).max(row)
        };
        let play_w = fit(&[
            ph2d_i18n::tr("anim_editor.play"),
            ph2d_i18n::tr("anim_editor.pause"),
        ]);
        let fps_w = fit(&[&self.shown(&Field::Fps, self.fps_label())]);
        let mode_w: Vec<f32> = [
            "anim_editor.repeat",
            "anim_editor.once",
            "anim_editor.ping_pong",
        ]
        .iter()
        .map(|k| fit(&[ph2d_i18n::tr(k)]))
        .collect();
        let cols_w = fit(&[&self.shown(&Field::Cols, self.cols_label())]);
        let rows_w = fit(&[&self.shown(&Field::Rows, self.rows_label())]);
        let ms: Vec<String> = (0..self.shown_frames().len())
            .map(|i| self.shown(&Field::FrameMs(i), self.ms_label(i)))
            .collect();
        let ms_w = fit(&ms.iter().map(String::as_str).collect::<Vec<_>>());

        let mut l = Layout {
            area,
            ..Layout::default()
        };
        let (left, right) = (area.x + pad, area.x + area.w - pad);
        let (mut x, mut y) = (left, area.y + pad);
        // Uma peça da barra de largura `w`: na linha em curso, ou no começo da seguinte.
        let mut place = |w: f32| {
            if x > left && x + w > right {
                x = left;
                y += row + gap;
            }
            let at = (x, y);
            x += w + gap;
            at
        };
        let (px, py) = place(play_w);
        l.play = Rect::new(px, py, play_w, row);
        let (fx, fy) = place(fps_w);
        l.fps = Rect::new(fx, fy, fps_w, row);
        // Os três modos são um segmento só: quebram juntos.
        let (mx, my) = place(mode_w.iter().sum());
        let mut mx = mx;
        for (m, w) in l.modes.iter_mut().zip(&mode_w) {
            *m = Rect::new(mx, my, *w, row);
            mx += w;
        }
        let (cx, cy) = place(cols_w);
        l.cols = Rect::new(cx, cy, cols_w, row);
        let (rx, ry) = place(rows_w);
        l.rows = Rect::new(rx, ry, rows_w, row);
        y += row + gap;
        if self.proposal().is_some() {
            let use_w = fit(&[ph2d_i18n::tr("anim_editor.use_grid")]);
            let text = Rect::new(left, y, (right - left - use_w - gap).max(0.0), row);
            l.banner = Some((text, Rect::new(text.x + text.w + gap, y, use_w, row)));
            y += row + gap;
        }
        let bottom = area.y + area.h - pad - ph2d_tokens::HUD_H_PX - ph2d_tokens::HUD_BOTTOM_PAD_PX;
        // A lista é pelo menos os seus dois botões lado a lado, cada um do tamanho do maior.
        let button_w = fit(&[
            ph2d_i18n::tr("anim_editor.add"),
            ph2d_i18n::tr("anim_editor.remove"),
        ]);
        let list_w = (area.w * LIST_FRAC)
            .max(row * FIELD_ROWS)
            .max(2.0 * button_w + gap); // LITERAL-PX-OK: dois botões
        let mid_x = left + list_w + pad;
        let mid_w = (right - mid_x).max(0.0);
        // A altura que a folha e as miniaturas repartem (o resto da faixa é a linha do tempo).
        let shared_h = (bottom - y - row - gap).max(0.0);
        let thumb = self.strip_thumb_side(mid_w, shared_h, (row * THUMB_ROWS).max(ms_w));
        let strip_h = thumb + row;
        l.strip = Rect::new(left, bottom - strip_h, right - left, strip_h);
        let mid_bottom = l.strip.y - gap;
        l.list = Rect::new(left, y, list_w, (mid_bottom - y).max(0.0));
        let n = self.snap.info.as_ref().map_or(0, |i| i.rows.len());
        l.list_rows = (0..n)
            .map(|i| Rect::new(l.list.x, l.list.y + i as f32 * row, l.list.w, row))
            .collect();
        let buttons_y = (l.list.y + n as f32 * row + gap).min(mid_bottom - row);
        let half = (l.list.w - gap) / 2.0; // LITERAL-PX-OK: dois botões iguais
        l.add = Rect::new(l.list.x, buttons_y, half, row);
        l.remove = Rect::new(l.list.x + half + gap, buttons_y, half, row);
        let mid = Rect::new(mid_x, y, mid_w, (mid_bottom - y).max(0.0));
        l.sheet = match &self.snap.sheet {
            Some(s) if s.w > 0 && s.h > 0 && mid.w > 0.0 && mid.h > 0.0 => {
                let k = (mid.w / s.w as f32).min(mid.h / s.h as f32);
                let (w, h) = (s.w as f32 * k, s.h as f32 * k);
                Rect::new(mid.x + (mid.w - w) / 2.0, mid.y + (mid.h - h) / 2.0, w, h)
            }
            _ => mid,
        };
        let max_scroll = (ms.len() as f32 * (thumb + gap) - gap - l.strip.w).max(0.0);
        self.strip_scroll = self.strip_scroll.clamp(0.0, max_scroll);
        l.frames = (0..ms.len())
            .map(|i| {
                let fx = l.strip.x + i as f32 * (thumb + gap) - self.strip_scroll;
                let t = Rect::new(fx, l.strip.y, thumb, thumb);
                (
                    t,
                    Rect::new(fx, l.strip.y + thumb, thumb, row),
                    Rect::new(fx + thumb - close_side, l.strip.y, close_side, close_side),
                )
            })
            .collect();
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

    /// O rótulo do tempo do quadro `i` da faixa.
    #[must_use]
    pub fn ms_label(&self, i: usize) -> String {
        ph2d_i18n::tr_with("anim_editor.ms", &[("ms", &self.frame_ms_at(i))])
    }
}
