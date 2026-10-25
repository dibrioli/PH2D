//! **Uma aba do editor de animações** — o estado, a disposição e o que cada gesto escreve.
//!
//! O documento não toca no mundo: cada gesto vira uma [`AnimFieldEdit`] em [`AnimDoc::pending`], e a
//! família ([`super::frame`]) aplica-as pela MESMA porta do Inspector (`inspector::apply_anim_edit`),
//! então o undo regista por diff como lá. A disposição ([`Layout`]) é função pura da área e do snapshot;
//! a pintura e o rato leem a mesma, guardada no último `paint`.

use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimCycleMode, AnimFieldEdit, InspectorAnimInfo, SheetImage, row_cells};
use ph2d_sprite_sheet::GridGuess;
use ph2d_tokens::ROW_H_PX;

pub use super::layout::Layout;

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
    /// Arrastar um quadro da faixa para outro intervalo (`gap`, `0..=n`); `at` é a mão.
    Frame {
        from: usize,
        x0: f32,
        y0: f32,
        moved: bool,
        gap: usize,
        at: (f32, f32),
    },
}

/// Um arrasto de quadro já em curso (passou o limiar), como a pintura o lê.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameDrag {
    pub from: usize,
    pub gap: usize,
    /// Onde a mão pegou, e onde está.
    pub grab: (f32, f32),
    pub at: (f32, f32),
}

/// O que a última pintura desenhou de um arrasto na faixa (os gates leem-no).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragPaint {
    /// O intervalo (`0..=n`) onde o marcador de inserção foi pintado.
    pub marker: Option<usize>,
    /// O rectângulo (antes do afim do fantasma) do quadro que segue a mão.
    pub ghost: Option<Rect>,
}

/// A posição final de um quadro tirado de `from` e largado no intervalo `gap` (`0..=n`).
#[must_use]
pub fn gap_to_index(from: usize, gap: usize) -> usize {
    if gap > from { gap - 1 } else { gap }
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
    /// O que a última pintura desenhou do arrasto.
    pub painted: DragPaint,
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
            } else if self.snap.info.as_ref().map(|i| &i.current) != Some(&info.current)
                && let Some(i) = info.current_index()
            {
                // Quem pôs outra no ecrã (o cartão) mudou também a aberta: os dois mostram a mesma.
                self.selected = i;
                self.frame_selected = None;
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

    /// ⭐ **A lei única do marcador e da largada**: o intervalo (`0..=n`) onde um quadro largado em
    /// `x` entra — antes do primeiro quadro pintado cujo centro está à direita de `x`.
    #[must_use]
    pub fn drop_gap(&self, x: f32) -> usize {
        self.layout
            .frames
            .iter()
            .filter(|f| f.0.x + f.0.w / 2.0 <= x)
            .count()
    }

    /// O arrasto de quadro em curso, se já passou o limiar.
    #[must_use]
    pub fn frame_drag(&self) -> Option<FrameDrag> {
        match self.drag {
            Drag::Frame {
                from,
                x0,
                y0,
                moved: true,
                gap,
                at,
            } => Some(FrameDrag {
                from,
                gap,
                grab: (x0, y0),
                at,
            }),
            _ => None,
        }
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
                gap: i,
                at: (x, y),
            };
        } else if let Some(cell) = self.cell_at(x, y)
            && self.row().is_some()
        {
            self.drag = Drag::Cells {
                base: self.click_base(),
                added: vec![cell],
            };
        }
        true
    }

    /// O rato moveu-se (com ou sem botão).
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let cell = self.cell_at(x, y);
        let slot = self.drop_gap(x);
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
                x0,
                y0,
                moved,
                gap,
                at,
                ..
            } => {
                let slop = ROW_H_PX * DRAG_SLOP_ROWS;
                if (x - *x0).abs() > slop || (y - *y0).abs() > slop {
                    *moved = true;
                }
                if *moved {
                    *gap = slot;
                    *at = (x, y);
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
                from, moved, gap, ..
            } => {
                let to = gap_to_index(from, gap);
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
