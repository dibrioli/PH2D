//! **Uma aba do editor de animações** — o estado, a disposição e o que cada gesto escreve.
//!
//! O documento não toca no mundo: cada gesto vira uma [`AnimFieldEdit`] em [`AnimDoc::pending`], e a
//! família ([`super::frame`]) aplica-as pela MESMA porta do Inspector (`inspector::apply_anim_edit`),
//! então o undo regista por diff como lá. A disposição ([`Layout`]) é função pura da área e do snapshot;
//! a pintura e o rato leem a mesma, guardada no último `paint`.

use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimCycleMode, AnimFieldEdit, InspectorAnimInfo, SheetImage, row_cells};
use ph2d_sprite_sheet::GridGuess;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// A lista de animações à esquerda, como fração da largura.
const LIST_FRAC: f32 = 0.22; // LITERAL-PX-OK: fração de layout
/// Larguras dos controlos da barra de cima, em alturas de linha.
const FIELD_ROWS: f32 = 3.0; // LITERAL-PX-OK: múltiplo do token de linha
const MODE_ROWS: f32 = 4.0; // LITERAL-PX-OK: múltiplo do token de linha
/// A faixa de quadros: miniatura de duas linhas mais a linha do tempo.
const THUMB_ROWS: f32 = 2.0; // LITERAL-PX-OK: múltiplo do token de linha
/// Um arrasto na faixa só conta depois deste caminho (um clique trémulo não reordena).
const DRAG_SLOP_ROWS: f32 = 0.25; // LITERAL-PX-OK: fração do token de linha

/// O que a aba mostra: o snapshot da §11 (com o cartão) e a folha.
#[derive(Clone, Debug, Default)]
pub struct EditorSnapshot {
    /// `None` = a sprite ainda não tem animações (só a folha, a grelha e «+»).
    pub info: Option<InspectorAnimInfo>,
    pub sheet: Option<SheetImage>,
    pub guess: Option<GridGuess>,
    pub hframes: u32,
    pub vframes: u32,
}

/// Um campo de texto aberto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    Fps,
    FrameMs(usize),
    Cols,
    Rows,
    Name(usize),
}

#[derive(Clone, Debug, Default, PartialEq)]
enum Drag {
    #[default]
    None,
    /// Arrastar (ou clicar) na folha: as células juntam-se por ordem a `base`.
    Cells { base: Vec<u32>, added: Vec<u32> },
    /// Arrastar um quadro da faixa para outra posição.
    Frame {
        from: usize,
        x0: f32,
        y0: f32,
        moved: bool,
        to: usize,
    },
}

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
        }
    }
}

fn hit(r: Rect, x: f32, y: f32) -> bool {
    r.w > 0.0 && r.h > 0.0 && x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

/// Uma aba aberta sobre uma sprite.
#[derive(Debug, Default)]
pub struct AnimDoc {
    pub entity: u64,
    pub name: String,
    pub snap: EditorSnapshot,
    /// A animação aberta (índice na biblioteca).
    pub selected: usize,
    /// O quadro escolhido na faixa (o alvo do Delete).
    pub frame_selected: Option<usize>,
    pub editing: Option<(Field, String)>,
    pub layout: Layout,
    pub strip_scroll: f32,
    drag: Drag,
    /// A seleção ainda não leu o snapshot (abre na animação que toca).
    seeded: bool,
    /// Depois de um «+», a seleção vai para a nova (a última) no próximo snapshot.
    select_last: bool,
    /// As edições à espera de [`super::frame`].
    pub pending: Vec<AnimFieldEdit>,
}

impl AnimDoc {
    #[must_use]
    pub fn new(entity: u64, name: String) -> Self {
        Self {
            entity,
            name,
            ..Self::default()
        }
    }

    /// O snapshot do quadro.
    pub fn set_snapshot(&mut self, snap: EditorSnapshot) {
        if let Some(info) = &snap.info {
            if self.select_last && !info.rows.is_empty() {
                self.selected = info.rows.len() - 1;
                self.select_last = false;
            } else if !self.seeded {
                self.selected = info.current_index().unwrap_or(0);
            }
            self.seeded = true;
            self.selected = self.selected.min(info.rows.len().saturating_sub(1));
        }
        self.snap = snap;
    }

    fn row(&self) -> Option<u8> {
        let info = self.snap.info.as_ref()?;
        if self.selected < info.rows.len() {
            u8::try_from(self.selected).ok()
        } else {
            None
        }
    }

    fn cells(&self) -> u32 {
        self.snap.hframes.max(1) * self.snap.vframes.max(1)
    }

    /// Os quadros da animação aberta como estão autorados (sem o arrasto em curso).
    #[must_use]
    pub fn authored_frames(&self) -> Vec<u32> {
        let Some(info) = &self.snap.info else {
            return Vec::new();
        };
        info.rows
            .get(self.selected)
            .map(|r| row_cells(r, self.cells()))
            .unwrap_or_default()
    }

    /// A base a que um clique na folha junta células. ⚠️ Uma animação SEM lista cujo intervalo é a
    /// folha inteira é a que o «+» acabou de criar (o default `0..=células-1`): ninguém a escolheu,
    /// então o primeiro clique começa uma lista nova em vez de juntar a célula a todas as outras.
    fn click_base(&self) -> Vec<u32> {
        let Some(row) = self
            .snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected))
        else {
            return Vec::new();
        };
        if row.frames.is_empty() && row.span(self.cells()) == self.cells() {
            return Vec::new();
        }
        row_cells(row, self.cells())
    }

    /// Os quadros que a faixa mostra agora (com o arrasto na folha, ou a reordenação, em curso).
    #[must_use]
    pub fn shown_frames(&self) -> Vec<u32> {
        match &self.drag {
            Drag::Cells { base, added } => base.iter().chain(added).copied().collect(),
            Drag::Frame {
                from,
                moved: true,
                to,
                ..
            } => reorder(&self.authored_frames(), *from, *to),
            _ => self.authored_frames(),
        }
    }

    /// O tempo do quadro `i` da animação aberta, em ms (o próprio, ou o da animação).
    #[must_use]
    pub fn frame_ms_at(&self, i: usize) -> u32 {
        let Some(r) = self
            .snap
            .info
            .as_ref()
            .and_then(|x| x.rows.get(self.selected))
        else {
            return 0;
        };
        r.per_frame_ms
            .get(i)
            .copied()
            .filter(|v| *v > 0)
            .unwrap_or(r.frame_ms)
    }

    /// O modo do ciclo da animação aberta.
    #[must_use]
    pub fn mode(&self) -> Option<AnimCycleMode> {
        let r = self.snap.info.as_ref()?.rows.get(self.selected)?;
        Some(AnimCycleMode::of_row(r.direction_tag, r.repeat))
    }

    /// A animação aberta está a tocar?
    #[must_use]
    pub fn playing(&self) -> bool {
        self.snap.info.as_ref().is_some_and(|i| {
            i.playing
                && i.rows
                    .get(self.selected)
                    .is_some_and(|r| r.name == i.current)
        })
    }

    /// A grelha proposta, quando difere da que a sprite tem e divide de facto a folha.
    #[must_use]
    pub fn proposal(&self) -> Option<GridGuess> {
        let g = self.snap.guess?;
        (g.cols * g.rows > 1 && (g.cols, g.rows) != (self.snap.hframes, self.snap.vframes))
            .then_some(g)
    }

    // ── A disposição ───────────────────────────────────────────────────────────────────────────

    /// Calcula a disposição para `area` (e guarda-a: o rato lê a do último quadro pintado).
    pub fn lay_out(&mut self, area: Rect) -> &Layout {
        let pad = Spacing::Md.px();
        let gap = ph2d_tokens::control_gap_px();
        let row = ROW_H_PX;
        let mut l = Layout {
            area,
            ..Layout::default()
        };
        let mut y = area.y + pad;
        let mut x = area.x + pad;
        l.play = Rect::new(x, y, row, row);
        x += row + gap;
        l.fps = Rect::new(x, y, row * FIELD_ROWS, row);
        x += row * FIELD_ROWS + gap;
        let mode_w = row * MODE_ROWS / 3.0; // LITERAL-PX-OK: três segmentos iguais
        for (i, m) in l.modes.iter_mut().enumerate() {
            *m = Rect::new(x + i as f32 * mode_w, y, mode_w, row);
        }
        x += row * MODE_ROWS + gap;
        l.cols = Rect::new(x, y, row * FIELD_ROWS / 2.0, row); // LITERAL-PX-OK: meio campo
        x += row * FIELD_ROWS / 2.0 + gap; // LITERAL-PX-OK: meio campo
        l.rows = Rect::new(x, y, row * FIELD_ROWS / 2.0, row); // LITERAL-PX-OK: meio campo
        y += row + gap;
        if self.proposal().is_some() {
            let use_w = row * FIELD_ROWS;
            let text = Rect::new(
                area.x + pad,
                y,
                (area.w - 2.0 * pad - use_w - gap).max(0.0),
                row,
            );
            l.banner = Some((text, Rect::new(text.x + text.w + gap, y, use_w, row)));
            y += row + gap;
        }
        let bottom = area.y + area.h - pad;
        let strip_h = row * (THUMB_ROWS + 1.0);
        l.strip = Rect::new(area.x + pad, bottom - strip_h, area.w - 2.0 * pad, strip_h);
        let mid_bottom = l.strip.y - gap;
        let list_w = (area.w * LIST_FRAC).max(row * FIELD_ROWS);
        l.list = Rect::new(area.x + pad, y, list_w, (mid_bottom - y).max(0.0));
        let n = self.snap.info.as_ref().map_or(0, |i| i.rows.len());
        l.list_rows = (0..n)
            .map(|i| Rect::new(l.list.x, l.list.y + i as f32 * row, l.list.w, row))
            .collect();
        let buttons_y = (l.list.y + n as f32 * row + gap).min(mid_bottom - row);
        let half = (l.list.w - gap) / 2.0; // LITERAL-PX-OK: dois botões iguais
        l.add = Rect::new(l.list.x, buttons_y, half, row);
        l.remove = Rect::new(l.list.x + half + gap, buttons_y, half, row);
        let mid = Rect::new(
            l.list.x + l.list.w + pad,
            y,
            (area.x + area.w - pad - (l.list.x + l.list.w + pad)).max(0.0),
            (mid_bottom - y).max(0.0),
        );
        l.sheet = match &self.snap.sheet {
            Some(s) if s.w > 0 && s.h > 0 && mid.w > 0.0 && mid.h > 0.0 => {
                let k = (mid.w / s.w as f32).min(mid.h / s.h as f32);
                let (w, h) = (s.w as f32 * k, s.h as f32 * k);
                Rect::new(mid.x + (mid.w - w) / 2.0, mid.y + (mid.h - h) / 2.0, w, h)
            }
            _ => mid,
        };
        let thumb = row * THUMB_ROWS;
        let frames = self.shown_frames();
        l.frames = (0..frames.len())
            .map(|i| {
                let fx = l.strip.x + i as f32 * (thumb + gap) - self.strip_scroll;
                let t = Rect::new(fx, l.strip.y, thumb, thumb);
                let close_side = row / 2.0; // LITERAL-PX-OK: meia linha
                (
                    t,
                    Rect::new(fx, l.strip.y + thumb, thumb, row),
                    Rect::new(fx + thumb - close_side, l.strip.y, close_side, close_side),
                )
            })
            .collect();
        self.layout = l;
        &self.layout
    }

    /// A célula da folha sob `(x, y)`.
    #[must_use]
    pub fn cell_at(&self, x: f32, y: f32) -> Option<u32> {
        let s = self.layout.sheet;
        if self.snap.sheet.is_none() || !hit(s, x, y) {
            return None;
        }
        let (hf, vf) = (self.snap.hframes.max(1), self.snap.vframes.max(1));
        let col = (((x - s.x) / s.w) * hf as f32) as u32;
        let row = (((y - s.y) / s.h) * vf as f32) as u32;
        Some(row.min(vf - 1) * hf + col.min(hf - 1))
    }

    /// O rectângulo da célula `cell` sobre a folha pintada.
    #[must_use]
    pub fn cell_rect(&self, cell: u32) -> Rect {
        let s = self.layout.sheet;
        let (hf, vf) = (self.snap.hframes.max(1), self.snap.vframes.max(1));
        let (cw, ch) = (s.w / hf as f32, s.h / vf as f32);
        Rect::new(
            s.x + (cell % hf) as f32 * cw,
            s.y + (cell / hf) as f32 * ch,
            cw,
            ch,
        )
    }

    /// Para onde um quadro largado em `x` vai na faixa.
    fn strip_slot(&self, x: f32) -> usize {
        let n = self.authored_frames().len();
        let thumb = ROW_H_PX * THUMB_ROWS + ph2d_tokens::control_gap_px();
        let i = ((x - self.layout.strip.x + self.strip_scroll) / thumb).floor();
        (i.max(0.0) as usize).min(n.saturating_sub(1))
    }

    // ── Os gestos ──────────────────────────────────────────────────────────────────────────────

    fn push_frames(&mut self, frames: Vec<u32>) {
        if let Some(row) = self.row() {
            self.pending.push(AnimFieldEdit::Frames(row, frames));
        }
    }

    /// Fecha o campo aberto, escrevendo-o (`commit`) ou não.
    pub fn close_field(&mut self, commit: bool) {
        let Some((field, text)) = self.editing.take() else {
            return;
        };
        if !commit {
            return;
        }
        let t = text.trim();
        let row = self.row();
        let edit = match field {
            Field::Fps => t
                .replace(',', ".")
                .parse::<f32>()
                .ok()
                .filter(|v| *v > 0.0)
                .and_then(|v| Some(AnimFieldEdit::FrameMs(row?, ph2d_ecs::ms_of_fps(v)))),
            Field::FrameMs(i) => t
                .parse::<u32>()
                .ok()
                .and_then(|ms| Some(AnimFieldEdit::FrameMsAt(row?, i as u32, ms))),
            Field::Cols => t
                .parse::<u32>()
                .ok()
                .map(|c| AnimFieldEdit::Grid(c, self.snap.vframes)),
            Field::Rows => t
                .parse::<u32>()
                .ok()
                .map(|r| AnimFieldEdit::Grid(self.snap.hframes, r)),
            Field::Name(i) => u8::try_from(i)
                .ok()
                .filter(|_| !t.is_empty())
                .map(|i| AnimFieldEdit::Rename(i, t.to_owned())),
        };
        self.pending.extend(edit);
    }

    fn open_field(&mut self, field: Field) {
        let text = match &field {
            Field::Fps => self
                .snap
                .info
                .as_ref()
                .and_then(|i| i.rows.get(self.selected))
                .map(|r| format_fps(ph2d_ecs::fps_of_ms(r.frame_ms)))
                .unwrap_or_default(),
            Field::FrameMs(i) => self.frame_ms_at(*i).to_string(),
            Field::Cols => self.snap.hframes.to_string(),
            Field::Rows => self.snap.vframes.to_string(),
            Field::Name(i) => self
                .snap
                .info
                .as_ref()
                .and_then(|x| x.rows.get(*i))
                .map(|r| r.name.clone())
                .unwrap_or_default(),
        };
        self.editing = Some((field, text));
    }

    /// Botão premido em `(x, y)`. Devolve se a aba o tomou.
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        self.close_field(true);
        let l = self.layout.clone();
        if !hit(l.area, x, y) {
            return false;
        }
        if hit(l.play, x, y) {
            self.toggle_play();
        } else if hit(l.fps, x, y) && self.row().is_some() {
            self.open_field(Field::Fps);
        } else if let Some(i) = l.modes.iter().position(|r| hit(*r, x, y)) {
            if let Some(row) = self.row() {
                self.pending
                    .push(AnimFieldEdit::CycleMode(row, AnimCycleMode::ALL[i].tag()));
            }
        } else if hit(l.cols, x, y) {
            self.open_field(Field::Cols);
        } else if hit(l.rows, x, y) {
            self.open_field(Field::Rows);
        } else if let Some((_, use_)) = l.banner
            && hit(use_, x, y)
        {
            if let Some(g) = self.proposal() {
                self.pending.push(AnimFieldEdit::Grid(g.cols, g.rows));
            }
        } else if let Some(i) = l.list_rows.iter().position(|r| hit(*r, x, y)) {
            self.pick_row(i);
        } else if hit(l.add, x, y) {
            self.pending.push(AnimFieldEdit::Add);
            self.select_last = true;
        } else if hit(l.remove, x, y) {
            if let Some(row) = self.row() {
                self.pending.push(AnimFieldEdit::Remove(row));
            }
        } else if let Some(i) = l.frames.iter().position(|f| hit(f.2, x, y))
            && self.authored_frames().len() > 1
        {
            let mut frames = self.authored_frames();
            frames.remove(i);
            self.frame_selected = None;
            self.push_frames(frames);
        } else if let Some(i) = l.frames.iter().position(|f| hit(f.1, x, y)) {
            self.open_field(Field::FrameMs(i));
        } else if let Some(i) = l.frames.iter().position(|f| hit(f.0, x, y)) {
            self.drag = Drag::Frame {
                from: i,
                x0: x,
                y0: y,
                moved: false,
                to: i,
            };
        } else if let Some(cell) = self.cell_at(x, y) {
            if self.row().is_some() {
                self.drag = Drag::Cells {
                    base: self.click_base(),
                    added: vec![cell],
                };
            }
        }
        true
    }

    /// O rato moveu-se (com ou sem botão).
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let cell = self.cell_at(x, y);
        let slot = self.strip_slot(x);
        match &mut self.drag {
            Drag::Cells { added, .. } => {
                if let Some(c) = cell
                    && added.last() != Some(&c)
                {
                    added.push(c);
                }
                true
            }
            Drag::Frame {
                x0, y0, moved, to, ..
            } => {
                let slop = ROW_H_PX * DRAG_SLOP_ROWS;
                if (x - *x0).abs() > slop || (y - *y0).abs() > slop {
                    *moved = true;
                }
                if *moved {
                    *to = slot;
                }
                true
            }
            Drag::None => false,
        }
    }

    /// Botão solto.
    pub fn release(&mut self, x: f32, y: f32) -> bool {
        self.motion(x, y);
        match std::mem::take(&mut self.drag) {
            Drag::Cells { base, added } => {
                self.push_frames(base.into_iter().chain(added).collect());
                true
            }
            Drag::Frame {
                from, moved, to, ..
            } => {
                if moved {
                    if to != from {
                        self.frame_selected = Some(to);
                        self.push_frames(reorder(&self.authored_frames(), from, to));
                    }
                } else {
                    self.frame_selected = Some(from);
                    self.pending.push(AnimFieldEdit::SetStep(from as u32));
                }
                true
            }
            Drag::None => false,
        }
    }

    fn pick_row(&mut self, i: usize) {
        if i == self.selected && self.seeded {
            self.open_field(Field::Name(i));
            return;
        }
        self.selected = i;
        self.frame_selected = None;
        if let Some(r) = self.snap.info.as_ref().and_then(|x| x.rows.get(i)) {
            self.pending.push(AnimFieldEdit::SetCurrent(r.name.clone()));
        }
    }

    fn toggle_play(&mut self) {
        let Some(name) = self
            .snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected))
            .map(|r| r.name.clone())
        else {
            return;
        };
        if self.playing() {
            self.pending.push(AnimFieldEdit::Playing(false));
        } else {
            if self.snap.info.as_ref().is_some_and(|i| i.current != name) {
                self.pending.push(AnimFieldEdit::SetCurrent(name));
            }
            self.pending.push(AnimFieldEdit::Playing(true));
        }
    }

    /// Escreve no campo aberto. Devolve se havia campo.
    pub fn type_text(&mut self, text: &str) -> bool {
        let Some((field, buf)) = &mut self.editing else {
            return false;
        };
        for c in text.chars() {
            let ok = match field {
                Field::Name(_) => !c.is_control(),
                Field::Fps => c.is_ascii_digit() || c == '.' || c == ',',
                _ => c.is_ascii_digit(),
            };
            if ok && buf.len() < ph2d_ecs::ANIM_NAME_MAX_BYTES {
                buf.push(c);
            }
        }
        true
    }

    /// Apaga o último carácter do campo, ou o quadro escolhido da faixa.
    pub fn backspace_or_delete(&mut self) -> bool {
        if let Some((_, buf)) = &mut self.editing {
            buf.pop();
            return true;
        }
        let Some(i) = self.frame_selected else {
            return false;
        };
        let mut frames = self.authored_frames();
        if frames.len() <= 1 || i >= frames.len() {
            return false;
        }
        frames.remove(i);
        self.frame_selected = None;
        self.push_frames(frames);
        true
    }
}

/// Mover o elemento `from` para a posição `to`.
#[must_use]
pub fn reorder(frames: &[u32], from: usize, to: usize) -> Vec<u32> {
    let mut v = frames.to_vec();
    if from < v.len() {
        let c = v.remove(from);
        v.insert(to.min(v.len()), c);
    }
    v
}

/// Quadros por segundo para ler: inteiro quando é inteiro, senão uma casa.
#[must_use]
pub fn format_fps(fps: f32) -> String {
    let r = (fps * 10.0).round() / 10.0; // LITERAL-PX-OK: uma casa decimal, não é UI
    if (r - r.round()).abs() < f32::EPSILON {
        format!("{}", r.round() as i64)
    } else {
        format!("{r:.1}")
    }
}
