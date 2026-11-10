//! ⭐ **A prévia Align do editor de animações** (GEF 28, onda B; decisões do dono 09/11): com «Align» ligado, a área da
//! folha mostra o quadro aberto em grande, o quadro ANTERIOR ténue e um quadro de REFERÊNCIA fixado noutra tinta por
//! baixo, a cruz do Pivot e a linha do chão. O rato move a folha (arrasto), muda-lhe o tamanho (o canto) e acerta só
//! o quadro (Alt + arrasto, ou as setas: 1 px, Shift 10 px). Cada gesto vira um `ImageUseEdit` pela porta das folhas;
//! o undo junta o arrasto inteiro num passo (botão em baixo = gesto em curso).

use super::doc::AnimDoc;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimFrameRef, ImageUseEdit, row_cells};
use std::sync::Arc;

/// A fração da prévia que o desenho ocupa ao encaixar (o resto é ar à volta).
const FIT: f32 = 0.8; // LITERAL-PX-OK: fração de layout
/// O passo das setas com Shift, em pixels da imagem.
pub const BIG_STEP_PX: f32 = 10.0; // LITERAL-PX-OK: passo em pixels da imagem, ordem do plano 28
/// A menor escala que um arrasto do canto pede (relativa ao início): o quad não some debaixo do rato.
const MIN_DRAG_SCALE: f32 = 0.05; // LITERAL-PX-OK: razão adimensional
/// O alcance da pega da linha do chão e do canto, em frações da altura de uma linha.
const GRAB_ROWS: f32 = 0.35; // LITERAL-PX-OK: fração do token de linha

/// Onde fica cada peça da prévia (vazia com «Align» desligado).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AlignLayout {
    /// A moldura própria, embaixo à direita (dono 09/11), e a fila do título (com o nome da referência).
    pub frame: Rect,
    pub title: Rect,
    /// O botão «Align» e o conta-gotas da referência.
    pub toggle: Rect,
    pub pick: Rect,
    /// Os botões «Previous» e «Reference».
    pub show_prev: Rect,
    pub show_ref: Rect,
    /// «By feet» e «By centre» (onda C): cada quadro da animação vai aos pés ou ao centro da referência.
    pub by_feet: Rect,
    pub by_centre: Rect,
    /// A coluna do Pivot (GEF 29): o rótulo, a grade 5 × 5, o «By hand» e o «Follow».
    pub pivot_title: Rect,
    pub pivot_grid: Rect,
    pub by_hand: Rect,
    pub pivot_follow: Rect,
    /// A prévia.
    pub preview: Rect,
}

/// A transformação da prévia: o ecrã de `(0, 0)` local (o Pivot) e os pixels de ecrã por metro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub origin: (f32, f32),
    pub k: f32,
}

impl View {
    /// Um ponto local (metros, Y para cima) no ecrã.
    #[must_use]
    pub fn to_screen(&self, p: [f32; 2]) -> (f32, f32) {
        (self.origin.0 + p[0] * self.k, self.origin.1 - p[1] * self.k)
    }

    /// O quad `(centro, tamanho)` no ecrã.
    #[must_use]
    pub fn quad(&self, (c, s): ([f32; 2], [f32; 2])) -> Rect {
        let (x, y) = self.to_screen(c);
        let (w, h) = (s[0].abs() * self.k, s[1].abs() * self.k);
        Rect::new(x - w / 2.0, y - h / 2.0, w, h)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum AlignDrag {
    #[default]
    None,
    /// A folha anda; `sent` = os pixels da imagem já enviados desde o Down.
    Move { from: (f32, f32), sent: [f32; 2] },
    /// Só o quadro `cell` anda (Alt).
    Nudge {
        from: (f32, f32),
        sent: [f32; 2],
        cell: AnimFrameRef,
    },
    /// O canto: a escala total pedida em relação à distância do Down ao Pivot.
    Scale { d0: f32, applied: f32 },
    /// A linha do chão.
    Floor,
}

/// O que a última pintura pôs na prévia (os gates leem-no).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AlignPaint {
    pub current: Option<Rect>,
    pub previous: Option<Rect>,
    pub reference: Option<Rect>,
    /// A cruz do Pivot.
    pub pivot: Option<(f32, f32)>,
    /// A caixa do desenho da referência (no ecrã) e quantas arestas da silhueta dela foram pintadas por cima.
    pub reference_box: Option<Rect>,
    pub wireframe: usize,
    /// A altura da linha do chão no ecrã.
    pub floor: Option<f32>,
    /// As pegas das quatro quinas (o tamanho da folha).
    pub handles: [Option<Rect>; 4],
    /// O item «Use as reference» do menu aberto.
    pub menu_item: Option<Rect>,
    /// A caixa do desenho de todos os quadros (no ecrã), sobre a qual os 25 pontos do Pivot se pintam.
    pub all_frames: Option<Rect>,
}

/// O ponto do desenho que os botões de alinhar levam ao da referência.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignBy {
    Feet,
    Centre,
}

/// O estado da prévia numa aba.
#[derive(Debug)]
pub struct AlignUi {
    pub on: bool,
    pub show_prev: bool,
    pub show_ref: bool,
    /// A referência escolhida; `None` = o 1.º quadro da animação de entrada.
    pub reference: Option<AnimFrameRef>,
    /// A linha do chão arrastada, em metros locais; `None` = a base do desenho da referência.
    pub floor: Option<f32>,
    /// O menu do clique direito sobre o quadro `i` da tira, aberto em `(x, y)`.
    pub menu: Option<(usize, (f32, f32))>,
    /// O conta-gotas armado: o próximo clique num quadro da tira escolhe a referência.
    pub picking: bool,
    pub view: Option<View>,
    fit_key: Option<FitKey>,
    drag: AlignDrag,
    /// A caixa do desenho de uma célula, em frações dela `[x0, y0, x1, y1]` a partir do canto de cima (chave: folha,
    /// célula, corte, imagem).
    floor_cache: Option<((u8, u32, u32, u32, usize), Option<Arc<Silhueta>>)>,
    pub painted: AlignPaint,
    /// O Pivot do objeto (GEF 29): a grade de 25 pontos e o «By hand».
    pub pivot: super::align_pivot::PivotUi,
}

/// O desenho de uma célula (os píxeis com alfa), em frações dela a partir do canto de cima: a caixa `[x0, y0, x1,
/// y1]` e as arestas da silhueta `[xa, ya, xb, yb]` (as bordas entre um píxel com alfa e um sem), fundidas em corridas.
#[derive(Debug, PartialEq)]
pub struct Silhueta {
    pub caixa: [f32; 4],
    pub arestas: Vec<[f32; 4]>,
}

impl Silhueta {
    /// `None` = a célula não tem desenho.
    #[must_use]
    pub fn of(px: &[u8], w: usize, h: usize) -> Option<Self> {
        let on = |x: isize, y: isize| {
            x >= 0
                && y >= 0
                && (x as usize) < w
                && (y as usize) < h
                && px[(y as usize * w + x as usize) * 4 + 3] > 0
        };
        let mut caixa: Option<[usize; 4]> = None;
        for y in 0..h {
            for x in (0..w).filter(|x| on(*x as isize, y as isize)) {
                let o = caixa.get_or_insert([x, y, x, y]);
                *o = [o[0].min(x), o[1].min(y), o[2].max(x), o[3].max(y)];
            }
        }
        let [a, t, r, d] = caixa?;
        let (fw, fh) = (w.max(1) as f32, h.max(1) as f32);
        let mut arestas = Vec::new();
        // As bordas horizontais (por cima de cada linha `y`, entre `y - 1` e `y`) e as verticais, em corridas.
        for y in 0..=h as isize {
            let mut run: Option<usize> = None;
            for x in 0..=w {
                let borda = x < w && on(x as isize, y) != on(x as isize, y - 1);
                match (borda, run) {
                    (true, None) => run = Some(x),
                    (false, Some(x0)) => {
                        arestas.push([x0 as f32 / fw, y as f32 / fh, x as f32 / fw, y as f32 / fh]);
                        run = None;
                    }
                    _ => {}
                }
            }
        }
        for x in 0..=w as isize {
            let mut run: Option<usize> = None;
            for y in 0..=h {
                let borda = y < h && on(x, y as isize) != on(x - 1, y as isize);
                match (borda, run) {
                    (true, None) => run = Some(y),
                    (false, Some(y0)) => {
                        arestas.push([x as f32 / fw, y0 as f32 / fh, x as f32 / fw, y as f32 / fh]);
                        run = None;
                    }
                    _ => {}
                }
            }
        }
        Some(Self {
            caixa: [
                a as f32 / fw,
                t as f32 / fh,
                (r + 1) as f32 / fw,
                (d + 1) as f32 / fh,
            ],
            arestas,
        })
    }
}

type FitKey = ([u32; 4], usize, Option<AnimFrameRef>, usize, u8);

impl Default for AlignUi {
    fn default() -> Self {
        Self {
            on: false,
            show_prev: true,
            show_ref: true,
            reference: None,
            floor: None,
            menu: None,
            picking: false,
            view: None,
            fit_key: None,
            drag: AlignDrag::None,
            floor_cache: None,
            painted: AlignPaint::default(),
            pivot: super::align_pivot::PivotUi::default(),
        }
    }
}

pub(crate) fn hit(r: Rect, x: f32, y: f32) -> bool {
    r.w > 0.0 && r.h > 0.0 && x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

impl AnimDoc {
    /// O quadro aberto na prévia: `(posição na animação, quadro)` — o escolhido na tira, senão o que toca, senão o 1.º.
    #[must_use]
    pub fn align_current(&self) -> Option<(usize, AnimFrameRef)> {
        let frames = self.authored_frames();
        let info = self.snap.info.as_ref()?;
        let on_screen = info
            .rows
            .get(self.selected)
            .is_some_and(|r| r.name == info.current);
        let i = self
            .marked_frame()
            .or_else(|| on_screen.then_some(info.card.step as usize))
            .filter(|i| *i < frames.len())
            .unwrap_or(0);
        frames.get(i).map(|f| (i, *f))
    }

    /// O quadro anterior ao aberto (o último, quando o aberto é o 1.º: a animação dá a volta).
    #[must_use]
    pub fn align_previous(&self) -> Option<AnimFrameRef> {
        let frames = self.authored_frames();
        let (i, _) = self.align_current()?;
        if frames.len() < 2 {
            return None;
        }
        frames.get((i + frames.len() - 1) % frames.len()).copied()
    }

    /// A referência: a escolhida, ou o 1.º quadro da animação de entrada («Starts with»), ou o da 1.ª animação.
    #[must_use]
    pub fn align_reference(&self) -> Option<AnimFrameRef> {
        if let Some(r) = self.align.reference {
            return Some(r);
        }
        let info = self.snap.info.as_ref()?;
        let row = info
            .rows
            .iter()
            .find(|r| r.name == info.card.starts_with)
            .or_else(|| info.rows.first())?;
        row_cells(row, &info.sheets).first().copied()
    }

    /// O quad `(centro, tamanho)` de um quadro em metros locais (`None` = quadro vazio ou folha sem geometria).
    #[must_use]
    pub fn frame_quad(&self, f: AnimFrameRef) -> Option<([f32; 2], [f32; 2])> {
        let g = self.snap.geo.get(usize::from(f.sheet))?;
        Some((g.center_of(f.cell), g.size))
    }

    /// Os metros locais de um pixel da imagem da folha `k` (na largura).
    #[must_use]
    pub fn meters_per_px(&self, k: u8) -> Option<f32> {
        let s = self.snap.sheets.get(usize::from(k))?;
        let img = s.image.as_ref()?;
        let g = self.snap.geo.get(usize::from(k))?;
        let cell_w = img.w as f32 / s.hframes.max(1) as f32;
        (cell_w > 0.0 && g.size[0] != 0.0).then(|| g.size[0].abs() / cell_w)
    }

    /// Os quadros que a prévia encaixa: os da animação aberta e a referência.
    fn fit_frames(&self) -> Vec<AnimFrameRef> {
        let mut v = self.authored_frames();
        v.extend(self.align_reference());
        v
    }

    /// A transformação da prévia: a guardada, refeita quando a animação, a referência ou a área mudam, ou quando o
    /// desenho já não cabe — nunca a meio de um arrasto (o quad fugia do rato).
    pub(super) fn align_view(&mut self, preview: Rect) -> Option<View> {
        let frames = self.fit_frames();
        let quads: Vec<_> = frames.iter().filter_map(|f| self.frame_quad(*f)).collect();
        let key: FitKey = (
            [preview.x, preview.y, preview.w, preview.h].map(f32::to_bits),
            self.selected,
            self.align.reference,
            frames.len(),
            self.snap.shown_sheet,
        );
        let dragging = self.align.drag != AlignDrag::None;
        if let Some(v) = self.align.view
            && (dragging
                || (self.align.fit_key == Some(key)
                    && quads.iter().all(|q| {
                        let r = v.quad(*q);
                        r.x >= preview.x
                            && r.y >= preview.y
                            && r.x + r.w <= preview.x + preview.w
                            && r.y + r.h <= preview.y + preview.h
                    })))
        {
            return Some(v);
        }
        if preview.w <= 0.0 || preview.h <= 0.0 {
            return None;
        }
        // A caixa de tudo, com o Pivot dentro (a cruz tem de se ver).
        let (mut lo, mut hi) = ([0.0_f32; 2], [0.0_f32; 2]);
        for (c, s) in &quads {
            for a in 0..2 {
                lo[a] = lo[a].min(c[a] - s[a].abs() / 2.0);
                hi[a] = hi[a].max(c[a] + s[a].abs() / 2.0);
            }
        }
        let (bw, bh) = (hi[0] - lo[0], hi[1] - lo[1]);
        let k = if bw > 0.0 && bh > 0.0 {
            (preview.w / bw).min(preview.h / bh) * FIT
        } else {
            preview.h * FIT
        };
        let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        let v = View {
            origin: (
                preview.x + preview.w / 2.0 - mid[0] * k,
                preview.y + preview.h / 2.0 + mid[1] * k,
            ),
            k,
        };
        self.align.view = Some(v);
        self.align.fit_key = Some(key);
        Some(v)
    }

    /// A caixa do desenho (os píxeis com alfa) da referência, em metros locais: `(baixo-esquerda, cima-direita)`. A
    /// base dela é a linha do chão.
    pub fn reference_box(&mut self) -> Option<([f32; 2], [f32; 2])> {
        let (f, sil) = self.reference_silhouette()?;
        self.drawing_box(f, sil.caixa)
    }

    /// A caixa do desenho do quadro `f` (`frac` = [`Silhueta::caixa`]), em metros locais: `(baixo-esquerda,
    /// cima-direita)`.
    pub(crate) fn drawing_box(
        &self,
        f: AnimFrameRef,
        frac: [f32; 4],
    ) -> Option<([f32; 2], [f32; 2])> {
        let (c, s) = self.frame_quad(f)?;
        let (sw, sh) = (s[0].abs(), s[1].abs());
        let (left, top) = (c[0] - sw / 2.0, c[1] + sh / 2.0);
        Some((
            [left + frac[0] * sw, top - frac[3] * sh],
            [left + frac[2] * sw, top - frac[1] * sh],
        ))
    }

    /// A silhueta do desenho da referência (guardada enquanto a célula, o corte e a imagem forem os mesmos).
    pub fn reference_silhouette(&mut self) -> Option<(AnimFrameRef, Arc<Silhueta>)> {
        let f = self.align_reference()?;
        let sheet = self.snap.sheets.get(usize::from(f.sheet))?;
        let img = sheet.image.as_ref()?;
        let (hf, vf) = (sheet.hframes.max(1), sheet.vframes.max(1));
        let key = (f.sheet, f.cell, hf, vf, Arc::as_ptr(&img.rgba) as usize);
        let sil = match &self.align.floor_cache {
            Some((k, v)) if *k == key => v.clone(),
            _ => {
                let v = self.silhouette_of(f).map(Arc::new);
                self.align.floor_cache = Some((key, v.clone()));
                v
            }
        }?;
        Some((f, sil))
    }

    /// O desenho do quadro `f` (`None` = célula vazia ou folha sem imagem).
    fn silhouette_of(&self, f: AnimFrameRef) -> Option<Silhueta> {
        let sheet = self.snap.sheets.get(usize::from(f.sheet))?;
        let img = sheet.image.as_ref()?;
        let (hf, vf) = (sheet.hframes.max(1), sheet.vframes.max(1));
        let [x0, y0, x1, y1] = img.limites_da_celula(hf, vf, f.cell);
        let px = img.pixels_da_celula(hf, vf, f.cell);
        Silhueta::of(&px, (x1 - x0) as usize, (y1 - y0) as usize)
    }

    /// O ponto do desenho que «By feet» (o meio da base) ou «By centre» (o meio da caixa) alinha, em metros locais.
    #[must_use]
    pub fn align_point(&self, f: AnimFrameRef, by: AlignBy) -> Option<[f32; 2]> {
        let (lo, hi) = self.drawing_box(f, self.silhouette_of(f)?.caixa)?;
        let y = match by {
            AlignBy::Feet => lo[1],
            AlignBy::Centre => (lo[1] + hi[1]) / 2.0,
        };
        Some([(lo[0] + hi[0]) / 2.0, y])
    }

    /// ⭐ **By feet / By centre** (onda C): cada quadro da animação aberta anda, pelo acerto da célula dele, até o
    /// ponto da REFERÊNCIA — os pés dela (na linha do chão, se foi arrastada) ou o centro. A referência fica onde está,
    /// e o objeto no jogo também; o passo é em pixels inteiros da folha de cada quadro. Tudo numa edição: um Ctrl+Z.
    /// Devolve se algum quadro andou.
    pub fn align_frames(&mut self, by: AlignBy) -> bool {
        let Some(mut target) = self.align_reference().and_then(|r| self.align_point(r, by)) else {
            return false;
        };
        if by == AlignBy::Feet
            && let Some(floor) = self.align.floor
        {
            target[1] = floor;
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut moved = false;
        for f in self.authored_frames() {
            if !seen.insert((f.sheet, f.cell)) {
                continue;
            }
            let (Some(p), Some(mpp)) = (self.align_point(f, by), self.meters_per_px(f.sheet))
            else {
                continue;
            };
            let px = |a: usize| ((target[a] - p[a]) / mpp).round() * mpp;
            let by = [px(0), px(1)];
            if by != [0.0, 0.0] {
                self.sheet_ui.pending.push(ImageUseEdit::NudgeCell {
                    sheet: f.sheet,
                    cell: f.cell,
                    by,
                });
                moved = true;
            }
        }
        moved
    }

    /// Um gesto da prévia em curso (arrasto da folha, do quadro, do canto, do chão ou da cruz).
    #[must_use]
    pub fn align_gesture_active(&self) -> bool {
        self.align.drag != AlignDrag::None || self.align.pivot.hand.is_some()
    }

    /// O quadro `f` tem acerto próprio (a marca na tira).
    #[must_use]
    pub fn frame_nudged(&self, f: AnimFrameRef) -> bool {
        self.snap
            .geo
            .get(usize::from(f.sheet))
            .and_then(|g| g.offsets.get(&f.cell))
            .is_some_and(|d| *d != [0.0, 0.0])
    }

    /// Antes da primeira edição de um gesto: a folha do quadro aberto passa a ser a visível (os verbos agem nela).
    fn target_sheet(&mut self) -> Option<AnimFrameRef> {
        let (_, f) = self.align_current()?;
        if f.sheet != self.snap.shown_sheet {
            self.sheet_ui.pending.push(ImageUseEdit::ShowSheet(f.sheet));
            self.snap.shown_sheet = f.sheet;
        }
        Some(f)
    }

    /// Um botão premido: o botão Align, os da barra, o menu, e os gestos da prévia. Devolve se o tomou.
    pub(super) fn align_press(&mut self, x: f32, y: f32) -> bool {
        if let Some((i, _)) = self.align.menu.take() {
            if self.align.painted.menu_item.is_some_and(|r| hit(r, x, y)) {
                self.align.reference = self.authored_frames().get(i).copied();
                (self.align.floor, self.align.on) = (None, true);
            }
            return true;
        }
        let l = self.layout.align;
        if hit(l.toggle, x, y) {
            self.align.on = !self.align.on;
            (self.align.drag, self.align.picking) = (AlignDrag::None, false);
            return true;
        }
        if hit(l.pick, x, y) {
            self.align.picking = !self.align.picking;
            self.align.on |= self.align.picking;
            return true;
        }
        // O conta-gotas: um quadro da tira (de qualquer animação aberta na lista) vira a referência.
        if self.align.picking
            && let Some(i) = self.layout.frames.iter().position(|f| hit(f.0, x, y))
            && let Some(f) = self.authored_frames().get(i).copied()
        {
            (self.align.reference, self.align.floor, self.align.picking) = (Some(f), None, false);
            return true;
        }
        if !self.align.on {
            return false;
        }
        if hit(l.show_prev, x, y) {
            self.align.show_prev = !self.align.show_prev;
            return true;
        }
        if hit(l.show_ref, x, y) {
            self.align.show_ref = !self.align.show_ref;
            return true;
        }
        for (r, by) in [(l.by_feet, AlignBy::Feet), (l.by_centre, AlignBy::Centre)] {
            if hit(r, x, y) {
                self.align_frames(by);
                return true;
            }
        }
        if self.pivot_press(x, y) {
            return true;
        }
        if !hit(l.preview, x, y) {
            return false;
        }
        let grab = ph2d_tokens::ROW_H_PX * GRAB_ROWS;
        let p = self.align.painted;
        let na_pega = |h: &Option<Rect>| {
            h.is_some_and(|h| {
                let r = Rect::new(h.x - grab, h.y - grab, h.w + 2.0 * grab, h.h + 2.0 * grab); // LITERAL-PX-OK: as duas margens
                hit(r, x, y)
            })
        };
        self.align.drag = if p.handles.iter().any(na_pega) {
            let Some((ox, oy)) = p.pivot else {
                return true;
            };
            let d0 = (x - ox).hypot(y - oy);
            if d0 <= 0.0 || self.target_sheet().is_none() {
                return true;
            }
            AlignDrag::Scale { d0, applied: 1.0 }
        } else if p.floor.is_some_and(|f| (y - f).abs() <= grab) && self.align.show_ref {
            AlignDrag::Floor
        } else if self.mods.alt {
            let Some(f) = self.target_sheet() else {
                return true;
            };
            AlignDrag::Nudge {
                from: (x, y),
                sent: [0.0; 2],
                cell: f,
            }
        } else {
            if self.target_sheet().is_none() {
                return true;
            }
            AlignDrag::Move {
                from: (x, y),
                sent: [0.0; 2],
            }
        };
        true
    }

    /// O rato moveu-se com um gesto da prévia em curso.
    pub(super) fn align_motion(&mut self, x: f32, y: f32) -> bool {
        if self.pivot_motion(x, y) {
            return true;
        }
        let (Some(v), Some((_, f))) = (self.align.view, self.align_current()) else {
            return self.align.drag != AlignDrag::None;
        };
        let mpp = self.meters_per_px(f.sheet).unwrap_or(0.0);
        // Os pixels da imagem inteiros que o arrasto já percorreu (o ecrã é Y para baixo; o local, para cima).
        let px = |from: (f32, f32)| {
            let to_px = |d: f32| {
                if mpp > 0.0 {
                    (d / v.k / mpp).round()
                } else {
                    0.0
                }
            };
            [to_px(x - from.0), to_px(from.1 - y)]
        };
        match self.align.drag {
            AlignDrag::None => return false,
            AlignDrag::Move { from, sent } => {
                let now = px(from);
                let d = [(now[0] - sent[0]) * mpp, (now[1] - sent[1]) * mpp];
                if d != [0.0, 0.0] {
                    self.sheet_ui.pending.push(ImageUseEdit::MoveSheet(d));
                }
                self.align.drag = AlignDrag::Move { from, sent: now };
            }
            AlignDrag::Nudge { from, sent, cell } => {
                let now = px(from);
                let by = [(now[0] - sent[0]) * mpp, (now[1] - sent[1]) * mpp];
                if by != [0.0, 0.0] {
                    self.sheet_ui.pending.push(ImageUseEdit::NudgeCell {
                        sheet: cell.sheet,
                        cell: cell.cell,
                        by,
                    });
                }
                self.align.drag = AlignDrag::Nudge {
                    from,
                    sent: now,
                    cell,
                };
            }
            AlignDrag::Scale { d0, applied } => {
                let (ox, oy) = v.to_screen([0.0, 0.0]);
                let total = ((x - ox).hypot(y - oy) / d0).max(MIN_DRAG_SCALE);
                let step = total / applied;
                if (step - 1.0).abs() > f32::EPSILON {
                    self.sheet_ui.pending.push(ImageUseEdit::ScaleSheet(step));
                    self.align.drag = AlignDrag::Scale { d0, applied: total };
                }
            }
            AlignDrag::Floor => {
                self.align.floor = Some((v.origin.1 - y) / v.k);
            }
        }
        true
    }

    /// Botão solto: o gesto da prévia acaba.
    pub(super) fn align_release(&mut self, x: f32, y: f32) -> bool {
        if self.pivot_release(x, y) {
            return true;
        }
        if self.align.drag == AlignDrag::None {
            return false;
        }
        self.align_motion(x, y);
        self.align.drag = AlignDrag::None;
        true
    }

    /// As setas com a prévia ligada: o quadro aberto anda 1 px da imagem (`big` = 10). Devolve se a tomou.
    pub fn align_arrow(&mut self, dx: f32, dy: f32, big: bool) -> bool {
        if !self.align.on || self.editing.is_some() {
            return false;
        }
        let Some(f) = self.target_sheet() else {
            return false;
        };
        let Some(mpp) = self.meters_per_px(f.sheet) else {
            return false;
        };
        let n = if big { BIG_STEP_PX } else { 1.0 };
        self.sheet_ui.pending.push(ImageUseEdit::NudgeCell {
            sheet: f.sheet,
            cell: f.cell,
            by: [dx * n * mpp, dy * n * mpp],
        });
        true
    }

    /// O clique direito: sobre um quadro da tira abre o menu «Use as reference». Devolve se o tomou.
    pub fn align_secondary(&mut self, x: f32, y: f32) -> bool {
        self.align.menu = None;
        let Some(i) = self.layout.frames.iter().position(|f| hit(f.0, x, y)) else {
            return false;
        };
        if i >= self.authored_frames().len() {
            return false;
        }
        self.align.menu = Some((i, (x, y)));
        true
    }
}
