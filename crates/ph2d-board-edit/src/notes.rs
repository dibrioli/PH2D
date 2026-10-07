//! ⭐ **As NOTAS** (W3, o coração do brainstorm) — no idioma do MIRO, o produto de referência do dono
//! (`docs/MiroClone/02_plano.md` §3 W3; as fontes de cada lei em `ferramentas/miro_api_notas.txt` e
//! no handoff da W3):
//!
//! - a nota é uma forma (`ShapeType::Sticky` / `StickyWide`): cor das 16 do Miro, sem contorno; a
//!   ÚLTIMA cor e tamanho escolhidos valem para a próxima (help «Colors»);
//! - CRESCE para baixo com o texto e volta à altura de nascença quando ele sai (decisão 2 do dono —
//!   o idioma do FigJam; o Miro encolhe a letra);
//! - redimensionar mantém a proporção e leva a letra (o Miro escala a nota inteira);
//! - escreve-se ao SELECCIONAR (help: «select it and start typing»), e `Tab` / `Ctrl+D` a escrever
//!   cria a seguinte à direita com a mesma cor e tamanho, já a escrever (staff do Miro, 2020);
//! - modo EM MASSA: uma ideia por linha, e cada linha vira uma nota em fila (help «Bulk mode»);
//! - a PILHA: arrastar dela tira uma nota nova (help «Sticky Stack»);
//! - colar uma PLANILHA = uma nota por célula, na grelha das células (help «Paste as sticky notes»);
//! - a pega de quatro pontos de uma selecção de notas arruma-as em linhas e colunas.

use ph2d_board_geom::{height_for_text, text_rect};
use ph2d_board_model::{
    BoardDoc, BoardOp, Dash, Element, ElementId, History, Mark, Rgba, RichText, STICKY_COLORS,
    STICKY_DEFAULT_COLOR, STICKY_SIDE, STICKY_WIDE, Shape, ShapeType, Style,
};
use ph2d_text::TextSystem;

use crate::{Editor, live};

pub use crate::notes_layout::{grid_layout, parse_cells, reading_order};

/// Os tamanhos P · M · G de uma nota, em múltiplos da de nascença do Miro ([`STICKY_SIDE`]).
/// ⚠️ **P e G por medir**: o Miro tem o menu de tamanho (moderador, 09/11/2024) e não publica as
/// medidas; o M é o oficial (`199`). Uma captura do dono com as três lado a lado fecha-os.
pub const NOTE_SCALES: [f64; 3] = [0.75, 1.0, 1.5];
/// O tamanho de nascença (o M).
pub const NOTE_DEFAULT_SIZE: usize = 1;
/// O vão entre duas notas (`Tab`, colar uma planilha, arrumar em grelha, o modo em massa), em
/// fracção do lado da nota. ⚠️ Por medir numa captura do Miro.
pub const NOTE_GAP_K: f64 = 0.1;

/// Como nasce a PRÓXIMA nota — a última cor, tamanho e forma escolhidos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteStyle {
    pub color: Rgba,
    /// Índice em [`NOTE_SCALES`].
    pub size: usize,
    pub wide: bool,
}

impl Default for NoteStyle {
    fn default() -> Self {
        Self {
            color: Rgba(STICKY_COLORS[STICKY_DEFAULT_COLOR]),
            size: NOTE_DEFAULT_SIZE,
            wide: false,
        }
    }
}

impl NoteStyle {
    /// O tipo da próxima nota.
    #[must_use]
    pub fn kind(&self) -> ShapeType {
        if self.wide {
            ShapeType::StickyWide
        } else {
            ShapeType::Sticky
        }
    }
}

/// A largura de nascença (escala 1) de uma nota do tipo `kind`.
fn base_width(kind: ShapeType) -> f64 {
    if kind == ShapeType::StickyWide {
        STICKY_WIDE
    } else {
        STICKY_SIDE
    }
}

/// O tamanho de letra de uma nota do tipo `kind` com largura `w`: a de nascença do quadro na M.
pub(crate) fn note_font(kind: ShapeType, w: f64) -> f64 {
    ph2d_board_model::DEFAULT_FONT_SIZE * w / base_width(kind)
}

/// A escala de `el` (largura sobre a de nascença do tipo dele).
#[must_use]
pub fn note_scale(el: &Element) -> Option<f64> {
    let kind = el.shape()?.kind;
    kind.is_note().then(|| el.w / base_width(kind))
}

/// O tamanho P/M/G mais perto da escala de `el`.
#[must_use]
pub fn note_size(el: &Element) -> Option<usize> {
    let s = note_scale(el)?;
    (0..NOTE_SCALES.len()).min_by(|a, b| {
        (NOTE_SCALES[*a] - s)
            .abs()
            .total_cmp(&(NOTE_SCALES[*b] - s).abs())
    })
}

/// O vão entre notas da largura de `el`.
fn gap_of(el: &Element) -> f64 {
    let k = note_scale(el).unwrap_or(1.0);
    STICKY_SIDE * k * NOTE_GAP_K
}

/// Uma nota escreve-se (a pilha não).
fn writable(el: &Element) -> bool {
    el.shape()
        .is_some_and(|s| matches!(s.kind, ShapeType::Sticky | ShapeType::StickyWide))
}

/// A altura que `el` pede para o texto caber: numa nota, a de nascença (largura × proporção) ou
/// mais; numa forma, a que tem ou mais (uma forma só cresce).
pub(crate) fn fitted_height(ts: &mut TextSystem, el: &Element) -> f64 {
    let Some(s) = el.shape() else {
        return el.h;
    };
    let base = s.kind.note_aspect().map_or(el.h, |a| el.w * a);
    if s.text.is_empty() {
        return base;
    }
    let tw = text_rect(s.kind, el.w, base)[2];
    let block = ph2d_board_layout::text_height(ts, &s.text, s.style.font_size as f32, tw as f32);
    height_for_text(s.kind, base, f64::from(block))
}

impl Editor {
    /// Uma nota NOVA (do estilo das próximas) do tipo `kind` na caixa `bx`, à frente de tudo. A
    /// letra escala com a largura (a M tem a de nascença do quadro).
    pub(crate) fn new_note(&self, doc: &mut BoardDoc, kind: ShapeType, bx: [f64; 4]) -> Element {
        let c = self.notes.color;
        let mut style = Style::new(Some(c), None, c.readable_ink());
        style.font_size = note_font(kind, bx[2]);
        let shape = Shape {
            kind,
            style,
            text: RichText::default(),
        };
        Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx)
    }

    /// A caixa de uma nota nova do tipo `kind`, centrada em `c`, do tamanho das próximas.
    #[must_use]
    pub fn note_box(&self, kind: ShapeType, c: [f64; 2]) -> [f64; 4] {
        let w = base_width(kind) * NOTE_SCALES[self.notes.size];
        let h = w * kind.note_aspect().unwrap_or(1.0);
        [c[0] - w / 2.0, c[1] - h / 2.0, w, h]
    }

    /// A ÚNICA nota seleccionada que se escreve, se é isso que está seleccionado.
    fn the_note(&self, doc: &BoardDoc) -> Option<ElementId> {
        let [id] = self.selection.iter().copied().collect::<Vec<_>>()[..] else {
            return None;
        };
        doc.get(id).filter(|el| writable(el)).map(|el| el.id)
    }

    /// ⭐ **Escrever com uma nota seleccionada** começa a escrever nela, no FIM do texto (o que lá
    /// está não se perde por uma tecla). `false` = não há nota seleccionada (a tecla é atalho).
    pub fn type_into_note(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem, s: &str) -> bool {
        if self.editing.is_some() || self.gesture.is_some() {
            return false;
        }
        let Some(id) = self.the_note(doc) else {
            return false;
        };
        if !self.begin_text(doc, ts, id, None) {
            return false;
        }
        self.text_key_motion(ts, crate::Move::TextEnd);
        self.text_input(doc, ts, s)
    }

    /// ⭐ **`Tab` (ou `Ctrl+D`) a escrever numa nota**: termina-a e cria a seguinte à DIREITA — mesma
    /// cor, tamanho e forma, sem texto, no primeiro sítio livre — já a escrever. `false` = não se
    /// escrevia numa nota.
    pub fn next_note(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let Some(id) = self.editing.as_ref().map(|e| e.id) else {
            return false;
        };
        if self.bulk == Some(id) {
            return false;
        }
        let Some(src) = doc.get(id).filter(|el| writable(el)).cloned() else {
            return false;
        };
        self.commit_text(doc, history);
        let Some(src) = doc.get(id).cloned().or(Some(src)) else {
            return false;
        };
        let mut next = src.clone();
        next.id = doc.mint_id();
        next.z = doc.z_on_top();
        next.version = 0;
        next.h = src.w
            * src
                .shape()
                .and_then(|s| s.kind.note_aspect())
                .unwrap_or(1.0);
        if let Some(s) = next.shape_mut() {
            s.text = RichText::default();
        }
        let step = src.w + gap_of(&src);
        let notes = doc.live().filter(|o| o.shape().is_some()).count();
        for k in 1..=notes + 1 {
            next.x = src.x + step * k as f64;
            if free(&next, doc) {
                break;
            }
        }
        let nid = next.id;
        live(doc, next);
        history.record(vec![BoardOp::Delete(nid)]);
        self.open_text(doc, nid)
    }

    /// ⭐ **O modo EM MASSA** — um rascunho tracejado (uma nota larga, fora do desfazer) centrado em
    /// `c`, já a escrever: uma ideia por linha. Ao terminar (`Esc`, `Ctrl+Enter`, clicar fora), cada
    /// linha com texto vira uma nota, em fila, e o rascunho sai — UM passo.
    pub fn begin_bulk(&mut self, doc: &mut BoardDoc, history: &mut History, c: [f64; 2]) -> bool {
        self.commit_text(doc, history);
        self.cancel_gesture(doc);
        let kind = ShapeType::StickyWide;
        let bx = self.note_box(kind, c);
        let mut style = self.style.clone();
        style.fill = Some(ph2d_board_model::Rgba(ph2d_board_model::DEFAULT_PAPER));
        style.stroke = Some(style.text_color);
        style.dash = Dash::Dashed;
        style.font_size = ph2d_board_model::DEFAULT_FONT_SIZE * bx[2] / STICKY_WIDE;
        let shape = Shape {
            kind,
            style,
            text: RichText::default(),
        };
        let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx);
        let id = el.id;
        live(doc, el);
        self.bulk = Some(id);
        self.open_text(doc, id)
    }

    /// O rascunho em massa terminou: as linhas viram notas (UM passo), o rascunho sai. A altura das
    /// notas ajusta-se no próximo desenho (quem termina pode não ter o moldador — trocar de aba).
    pub(crate) fn finish_bulk(&mut self, doc: &mut BoardDoc, history: &mut History, id: ElementId) {
        self.bulk = None;
        let Some(draft) = doc.get(id).cloned() else {
            return;
        };
        let _ = BoardOp::Delete(id).apply(doc);
        let lines: Vec<String> = draft
            .shape()
            .map(|s| {
                s.text
                    .as_str()
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let cells: Vec<Vec<String>> = vec![lines];
        let made = self.place_cells(doc, None, &cells, [draft.x, draft.y]);
        self.unfitted.extend(made.iter().copied());
        self.finish_placed(history, made);
    }

    /// ⭐ **Colar uma planilha**: uma nota por célula com texto, na grelha das células, centrada em
    /// `c`, seleccionadas — UM passo. `false` = o texto não tinha célula nenhuma.
    pub fn paste_cells(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        text: &str,
        c: [f64; 2],
    ) -> bool {
        self.commit_text(doc, history);
        let cells = parse_cells(text);
        let kind = self.notes.kind();
        let bx = self.note_box(kind, [0.0, 0.0]);
        let cols = cells.iter().map(Vec::len).max().unwrap_or(0);
        if cols == 0 || cells.iter().flatten().all(|t| t.trim().is_empty()) {
            return false;
        }
        let gap = STICKY_SIDE * NOTE_SCALES[self.notes.size] * NOTE_GAP_K;
        let w = cols as f64 * (bx[2] + gap) - gap;
        let h = cells.len() as f64 * (bx[3] + gap) - gap;
        let made = self.place_cells(doc, Some(ts), &cells, [c[0] - w / 2.0, c[1] - h / 2.0]);
        self.finish_placed(history, made)
    }

    /// Notas novas para as células com texto de `cells` (linha `r`, coluna `k` da grelha) a partir
    /// de `origin`; cada linha da grelha tem a altura da nota mais alta dela (sem o moldador, a de
    /// nascença).
    fn place_cells(
        &mut self,
        doc: &mut BoardDoc,
        mut ts: Option<&mut TextSystem>,
        cells: &[Vec<String>],
        origin: [f64; 2],
    ) -> Vec<ElementId> {
        let kind = self.notes.kind();
        let bx = self.note_box(kind, [0.0, 0.0]);
        let gap = STICKY_SIDE * NOTE_SCALES[self.notes.size] * NOTE_GAP_K;
        let mut made = Vec::new();
        let mut y = origin[1];
        for row in cells {
            let mut row_h: f64 = bx[3];
            for (k, t) in row.iter().enumerate() {
                if t.trim().is_empty() {
                    continue;
                }
                let x = origin[0] + k as f64 * (bx[2] + gap);
                let mut el = self.new_note(doc, kind, [x, y, bx[2], bx[3]]);
                if let Some(s) = el.shape_mut() {
                    s.text = RichText::plain(t.trim());
                }
                if let Some(ts) = ts.as_deref_mut() {
                    el.h = fitted_height(ts, &el);
                }
                row_h = row_h.max(el.h);
                made.push(el.id);
                live(doc, el);
            }
            y += row_h + gap;
        }
        made
    }

    fn finish_placed(&mut self, history: &mut History, made: Vec<ElementId>) -> bool {
        if made.is_empty() {
            return false;
        }
        history.record(made.iter().rev().map(|id| BoardOp::Delete(*id)).collect());
        self.selection = made.into_iter().collect();
        true
    }

    /// A cor das notas seleccionadas (e das próximas) — UM passo. A letra acompanha (a tinta que
    /// se lê nela). As formas da selecção não mudam: a cor de uma nota é das 16 do Miro.
    pub fn set_note_color(&mut self, doc: &mut BoardDoc, history: &mut History, c: Rgba) {
        self.notes.color = c;
        let ops = self
            .selected(doc)
            .into_iter()
            .filter(|el| el.shape().is_some_and(|s| s.kind.is_note()))
            .map(|el| {
                let mut el = el.clone();
                let st = el.style_mut();
                st.fill = Some(c);
                st.text_color = c.readable_ink();
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
    }

    /// O tamanho P/M/G das notas seleccionadas (e das próximas): a nota escala à volta do centro
    /// do topo, com a letra — UM passo.
    pub fn set_note_size(&mut self, doc: &mut BoardDoc, history: &mut History, size: usize) {
        let size = size.min(NOTE_SCALES.len() - 1);
        self.notes.size = size;
        let ops = self
            .selected(doc)
            .into_iter()
            .filter_map(|el| {
                let s = note_scale(el)?;
                let k = NOTE_SCALES[size] / s;
                let mut el = el.clone();
                let cx = el.x + el.w / 2.0;
                el.w *= k;
                el.h *= k;
                el.x = cx - el.w / 2.0;
                el.style_mut().font_size *= k;
                Some(BoardOp::Put(el))
            })
            .collect();
        history.apply(doc, ops);
    }

    /// Quadrada ↔ larga, nas notas seleccionadas (e nas próximas), com a mesma escala: a largura
    /// muda, a altura volta a caber o texto — UM passo.
    pub fn set_note_wide(&mut self, doc: &mut BoardDoc, history: &mut History, wide: bool) {
        self.notes.wide = wide;
        let kind = self.notes.kind();
        let ops = self
            .selected(doc)
            .into_iter()
            .filter(|el| writable(el))
            .filter_map(|el| {
                let s = note_scale(el)?;
                let mut el = el.clone();
                let cx = el.x + el.w / 2.0;
                el.shape_mut()?.kind = kind;
                el.w = base_width(kind) * s;
                el.x = cx - el.w / 2.0;
                el.h = el.w * kind.note_aspect().unwrap_or(1.0);
                Some(BoardOp::Put(el))
            })
            .collect::<Vec<_>>();
        self.refit(&ops);
        history.apply(doc, ops);
    }

    /// Os elementos de `ops` ajustam a altura ao texto no próximo desenho (quem mudou o estilo não
    /// tinha o moldador à mão — um clique na barra).
    fn refit(&mut self, ops: &[BoardOp]) {
        self.unfitted.extend(ops.iter().filter_map(|op| match op {
            BoardOp::Put(el) => Some(el.id),
            BoardOp::Delete(_) => None,
        }));
    }

    /// O texto em edição mudou de estilo: vai já para a forma; a altura ajusta-se no próximo
    /// desenho.
    fn restyled_while_editing(&mut self, doc: &mut BoardDoc) {
        let Some(e) = self.editing.as_ref() else {
            return;
        };
        if let Some(mut el) = doc.get(e.id).cloned()
            && let Some(s) = el.shape_mut()
        {
            s.text = e.edit.rich().clone();
            live(doc, el);
        }
        self.resync = true;
    }

    /// ⭐ Liga ou desliga `mark` — no texto seleccionado se se está a escrever (ao vivo: o passo de
    /// desfazer é o da edição); senão no texto INTEIRO das formas e notas seleccionadas, UM passo
    /// (se todas já a têm em tudo, desliga).
    pub fn toggle_mark(&mut self, doc: &mut BoardDoc, history: &mut History, mark: Mark) -> bool {
        if let Some(e) = self.editing.as_mut() {
            e.edit.toggle(mark);
            self.restyled_while_editing(doc);
            return true;
        }
        let texts: Vec<&Element> = self
            .selected(doc)
            .into_iter()
            .filter(|el| el.shape().is_some_and(|s| !s.text.is_empty()))
            .collect();
        if texts.is_empty() {
            return false;
        }
        let on = !texts.iter().all(|el| {
            let t = &el.shape().expect("filtrado").text;
            t.all(0..t.len(), mark)
        });
        let ops = texts
            .into_iter()
            .map(|el| {
                let mut el = el.clone();
                let t = &mut el.shape_mut().expect("filtrado").text;
                let n = t.len();
                t.restyle(0..n, |m| mark.set(m, on));
                BoardOp::Put(el)
            })
            .collect::<Vec<_>>();
        self.refit(&ops);
        history.apply(doc, ops);
        true
    }

    /// A marca está ligada? (no texto em edição; senão em TODO o texto das formas seleccionadas)
    #[must_use]
    pub fn has_mark(&self, doc: &BoardDoc, mark: Mark) -> bool {
        if let Some(e) = self.editing.as_ref() {
            return e.edit.has(mark);
        }
        let mut texts = self
            .selected(doc)
            .into_iter()
            .filter_map(|el| el.shape())
            .filter(|s| !s.text.is_empty())
            .peekable();
        texts.peek().is_some() && texts.all(|s| s.text.all(0..s.text.len(), mark))
    }

    /// A cor do texto (`None` = a tinta da forma) — no texto seleccionado se se está a escrever;
    /// senão no texto inteiro das FORMAS seleccionadas (as notas do Miro não mudam a cor da letra).
    pub fn set_text_color(&mut self, doc: &mut BoardDoc, history: &mut History, c: Option<Rgba>) {
        if let Some(e) = self.editing.as_mut() {
            e.edit.set_color(c);
            self.restyled_while_editing(doc);
            return;
        }
        let ops = self
            .selected(doc)
            .into_iter()
            .filter(|el| el.shape().is_some_and(|s| !s.kind.is_note()))
            .map(|el| {
                let mut el = el.clone();
                let t = &mut el.shape_mut().expect("filtrado").text;
                let n = t.len();
                t.restyle(0..n, |m| m.color = c);
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
    }

    /// A cor única do texto (em edição, ou da 1.ª forma seleccionada); `None` = várias.
    #[must_use]
    pub fn text_color(&self, doc: &BoardDoc) -> Option<Option<Rgba>> {
        if let Some(e) = self.editing.as_ref() {
            return e.edit.color();
        }
        let s = self.selected(doc).into_iter().find_map(|el| el.shape())?;
        s.text.color(0..s.text.len()).or(Some(None))
    }

    /// O elemento em edição de texto, se há um.
    #[must_use]
    pub fn editing_id(&self) -> Option<ElementId> {
        self.editing.as_ref().map(|e| e.id)
    }

    /// As notas seleccionadas (a pega de arrumar em grelha só aparece com duas ou mais, só notas).
    pub(crate) fn selected_notes(&self, doc: &BoardDoc) -> Option<Vec<Element>> {
        let sel = self.selected(doc);
        (sel.len() >= 2 && sel.iter().all(|el| writable(el)))
            .then(|| sel.into_iter().cloned().collect())
    }

    /// Onde fica a pega de arrumar em grelha (mundo): por fora do canto superior direito da moldura.
    #[must_use]
    pub fn grid_handle(&self, doc: &BoardDoc, px: f64) -> Option<[f64; 2]> {
        if self.editing.is_some() {
            return None;
        }
        self.selected_notes(doc)?;
        let f = self.frame(doc)?;
        let off = (self.metrics.handle + self.metrics.rotate_offset / 2.0) * px;
        Some([f.center[0] + f.w / 2.0 + off, f.center[1] - f.h / 2.0 - off])
    }

    /// As notas em `cols` colunas a partir do canto de cima à esquerda da selecção, em ordem de
    /// leitura (ao vivo — o gesto guarda o passo ao largar).
    pub(crate) fn arrange(&self, doc: &mut BoardDoc, ordered: &[Element], cols: usize) {
        let x0 = ordered.iter().map(|e| e.x).fold(f64::INFINITY, f64::min);
        let y0 = ordered.iter().map(|e| e.y).fold(f64::INFINITY, f64::min);
        let gap = ordered.first().map_or(0.0, gap_of);
        let sizes: Vec<[f64; 2]> = ordered.iter().map(|e| [e.w, e.h]).collect();
        for (el, p) in ordered.iter().zip(grid_layout(&sizes, cols, [x0, y0], gap)) {
            let mut el = el.clone();
            el.x = p[0];
            el.y = p[1];
            live(doc, el);
        }
    }

    /// Quantas colunas pede o ponteiro em `x` (mundo) para `ordered`, a partir da esquerda deles.
    pub(crate) fn grid_columns(ordered: &[Element], x: f64) -> usize {
        let x0 = ordered.iter().map(|e| e.x).fold(f64::INFINITY, f64::min);
        let col_w = ordered.iter().map(|e| e.w).fold(0.0, f64::max);
        let gap = ordered.first().map_or(0.0, gap_of);
        let n = ((x - x0 + gap) / (col_w + gap)).round();
        (n.max(1.0) as usize).min(ordered.len())
    }

    /// Uma nota tirada da pilha `stack`: da cor e tamanho dela, no sítio dela, à frente de tudo.
    pub(crate) fn peel(&self, doc: &mut BoardDoc, stack: &Element) -> Element {
        let mut el = stack.clone();
        el.id = doc.mint_id();
        el.z = doc.z_on_top();
        el.version = 0;
        if let Some(s) = el.shape_mut() {
            s.kind = ShapeType::Sticky;
            s.text = RichText::default();
        }
        el
    }
}

/// `el` não toca em nenhuma forma viva?
fn free(el: &Element, doc: &BoardDoc) -> bool {
    let b = el.aabb();
    !doc.live().filter(|o| o.shape().is_some()).any(|o| {
        let a = o.aabb();
        a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
    })
}
