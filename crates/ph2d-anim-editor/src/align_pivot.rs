//! ⭐ **O Pivot da Animated Sprite mora no editor de animações** ([pedido do dono, GEF 29](../../../docs/GameEngineFuncional/29_pivot_no_editor_pedido.md)):
//! na moldura do Align, uma grade de 5 × 5 pontos contados sobre o desenho de TODOS os quadros de TODAS as animações,
//! já alinhados (o Pivot não salta ao trocar de animação), e o «By hand», que arrasta a cruz na prévia. Cada escolha é
//! um [`ImageUseEdit::MovePivot`]: um Ctrl+Z, e o objeto no jogo não anda.

use super::align::{View, hit};
use super::doc::AnimDoc;
use ph2d_editor_core::{AnimFrameRef, ImageUseEdit, row_cells};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Pontos por lado da grade (dono 09/11: 25 pontos).
pub const SIDE: u8 = 5; // LITERAL-PX-OK: contagem, não geometria
/// O alcance do encaixe do «By hand» num dos 25 pontos, em alturas de linha.
const SNAP_ROWS: f32 = 0.35; // LITERAL-PX-OK: fração do token de linha
/// Sem folha com imagem, o Pivot conta em milímetros locais (o arredondamento de `f32`, não geometria).
const NO_PIXEL_M: f32 = 1.0e-3; // LITERAL-PX-OK: metros
/// O passo do Pivot que o mundo ignora (`ph2d_sprite_edit::pivot::move_origin`, `MESMO_PONTO_M`): a prévia também.
const SAME_POINT_M: f32 = 1.0e-3; // LITERAL-PX-OK: metros

/// `p` é um passo que o mundo aplica (finito e fora do mesmo ponto).
fn world_moves(p: [f32; 2]) -> bool {
    p.iter().all(|v| v.is_finite()) && (p[0].abs() >= SAME_POINT_M || p[1].abs() >= SAME_POINT_M)
}

/// A chave da caixa de uma célula: folha, célula, corte e a imagem.
type CellKey = (u8, u32, u32, u32, usize);

/// O estado do Pivot numa aba.
#[derive(Debug, Default)]
pub struct PivotUi {
    /// «By hand» ligado: arrastar na prévia move a cruz, e não a folha.
    pub by_hand: bool,
    /// A cruz a meio do arrasto, em metros locais desde o Pivot de agora.
    pub hand: Option<[f32; 2]>,
    /// «Follow» ligado (dono 09/11): o Pivot acompanha o desenho quando a folha muda de tamanho ou anda.
    pub follow: bool,
    /// Onde o Pivot está na caixa de todos os quadros, em frações dela desde baixo-esquerda (medido fora de gestos).
    frac: Option<[f32; 2]>,
    /// O desenho mexeu desde a última fração medida: um arrasto espalha as edições por vários quadros, e o último
    /// pode não trazer nenhuma (smoke do dono 09/11: arrastou a folha e o Pivot não seguiu).
    dirty: bool,
    boxes: BTreeMap<CellKey, Option<[f32; 4]>>,
}

/// O ponto `[coluna, linha]` da grade (`0..SIDE`, a linha de cima é a 0) sobre a caixa `(lo, hi)`, em metros locais.
#[must_use]
pub fn grid_point((lo, hi): ([f32; 2], [f32; 2]), [c, r]: [u8; 2]) -> [f32; 2] {
    let t = |i: u8| f32::from(i.min(SIDE - 1)) / f32::from(SIDE - 1);
    [
        lo[0] + (hi[0] - lo[0]) * t(c),
        hi[1] - (hi[1] - lo[1]) * t(r),
    ]
}

/// Os 25 pontos, linha a linha.
pub fn grid() -> impl Iterator<Item = [u8; 2]> {
    (0..SIDE).flat_map(|r| (0..SIDE).map(move |c| [c, r]))
}

/// A caixa dos píxeis com alfa de uma célula `w × h`, em frações dela desde o canto de cima. `None` = vazia.
fn cell_box(px: &[u8], w: usize, h: usize) -> Option<[f32; 4]> {
    let mut b: Option<[usize; 4]> = None;
    for y in 0..h {
        for x in 0..w {
            if px.get((y * w + x) * 4 + 3).copied().unwrap_or(0) > 0 {
                let o = b.get_or_insert([x, y, x, y]);
                *o = [o[0].min(x), o[1].min(y), o[2].max(x), o[3].max(y)];
            }
        }
    }
    let [a, t, r, d] = b?;
    let (fw, fh) = (w.max(1) as f32, h.max(1) as f32);
    Some([
        a as f32 / fw,
        t as f32 / fh,
        (r + 1) as f32 / fw,
        (d + 1) as f32 / fh,
    ])
}

impl AnimDoc {
    /// Cada quadro de cada animação, uma vez.
    fn every_frame(&self) -> Vec<AnimFrameRef> {
        let Some(info) = &self.snap.info else {
            return Vec::new();
        };
        let mut seen = BTreeSet::new();
        info.rows
            .iter()
            .flat_map(|r| row_cells(r, &info.sheets))
            .filter(|f| seen.insert((f.sheet, f.cell)))
            .collect()
    }

    /// A caixa do desenho do quadro `f`, em frações da célula (guardada enquanto a célula, o corte e a imagem forem
    /// os mesmos).
    fn cell_box_of(&mut self, f: AnimFrameRef) -> Option<[f32; 4]> {
        let sheet = self.snap.sheets.get(usize::from(f.sheet))?;
        let img = sheet.image.as_ref()?;
        let (hf, vf) = (sheet.hframes.max(1), sheet.vframes.max(1));
        let key = (f.sheet, f.cell, hf, vf, Arc::as_ptr(&img.rgba) as usize);
        if let Some(b) = self.align.pivot.boxes.get(&key) {
            return *b;
        }
        let [x0, y0, x1, y1] = img.limites_da_celula(hf, vf, f.cell);
        let px = img.pixels_da_celula(hf, vf, f.cell);
        let b = cell_box(&px, (x1 - x0) as usize, (y1 - y0) as usize);
        self.align.pivot.boxes.insert(key, b);
        b
    }

    /// ⭐ **O desenho de todos os quadros de todas as animações**, já alinhados (com os acertos de folha e de célula),
    /// em metros locais: `(baixo-esquerda, cima-direita)`. É sobre ele que os 25 pontos contam.
    pub fn all_frames_box(&mut self) -> Option<([f32; 2], [f32; 2])> {
        let live: BTreeSet<usize> = self
            .snap
            .sheets
            .iter()
            .filter_map(|s| s.image.as_ref().map(|i| Arc::as_ptr(&i.rgba) as usize))
            .collect();
        // Uma imagem que saiu do projeto leva as caixas dela.
        self.align.pivot.boxes.retain(|k, _| live.contains(&k.4));
        let mut out: Option<([f32; 2], [f32; 2])> = None;
        for f in self.every_frame() {
            let Some(frac) = self.cell_box_of(f) else {
                continue;
            };
            let Some((lo, hi)) = self.drawing_box(f, frac) else {
                continue;
            };
            out = Some(out.map_or((lo, hi), |(a, b)| {
                (
                    [a[0].min(lo[0]), a[1].min(lo[1])],
                    [b[0].max(hi[0]), b[1].max(hi[1])],
                )
            }));
        }
        out
    }

    /// Os metros locais de um pixel da folha do quadro aberto (a medida do «By hand» e do «está neste ponto»).
    fn pivot_pixel(&self) -> f32 {
        let k = self.align_current().map_or(0, |(_, f)| f.sheet);
        self.meters_per_px(k)
            .filter(|m| *m > 0.0)
            .unwrap_or(NO_PIXEL_M)
    }

    /// ⭐ O ponto da grade em que o Pivot está agora; `None` = fora dos 25, ou sem desenho.
    pub fn pivot_on_grid(&mut self) -> Option<[u8; 2]> {
        let caixa = self.all_frames_box()?;
        let tol = self.pivot_pixel() / 2.0;
        grid().find(|q| {
            let p = grid_point(caixa, *q);
            p[0].abs() <= tol && p[1].abs() <= tol
        })
    }

    /// ⭐ O Pivot vai a `p` (metros locais desde o de agora): uma edição, um Ctrl+Z. A prévia e a linha do chão seguem
    /// o desenho, que não anda no ecrã. Devolve se pediu a edição.
    pub fn move_pivot(&mut self, p: [f32; 2]) -> bool {
        if !world_moves(p) {
            return false;
        }
        self.sheet_ui.pending.push(ImageUseEdit::MovePivot(p));
        // O mundo só responde no próximo quadro: a geometria guardada anda já, pela mesma lei.
        for g in &mut self.snap.geo {
            g.center = [g.center[0] - p[0], g.center[1] - p[1]];
        }
        self.pivot_moved(p);
        true
    }

    /// A prévia e a linha do chão seguem o Pivot que andou `p`: o desenho não salta no ecrã.
    fn pivot_moved(&mut self, p: [f32; 2]) {
        if let Some(f) = self.align.floor.as_mut() {
            *f -= p[1];
        }
        if let Some(v) = self.align.view.as_mut() {
            v.origin = v.to_screen(p);
        }
    }

    /// ⭐ **Follow** (dono 09/11): chamado pela família depois de aplicar as edições do quadro e refazer a geometria.
    /// `moved` = alguma delas mexeu no desenho (tamanho, folha, quadro). Com o Follow ligado devolve, a cada quadro em
    /// que o desenho mexeu (a meio do arrasto também), o passo do Pivot que o leva ao mesmo lugar da caixa de todos os
    /// quadros onde estava antes do gesto: a família aplica-o no mesmo quadro, e o Ctrl+Z desfaz tudo junto.
    pub fn pivot_follow(&mut self, moved: bool) -> Option<[f32; 2]> {
        self.align.pivot.dirty |= moved;
        let gesture = self.align_gesture_active();
        // A quina mede a escala contra o Pivot no ecrã e escala em volta dele: segui-lo a meio realimentava o gesto.
        // O Follow da escala espera pelo quadro em que o gesto acaba.
        let live = !self.align_scaling();
        let (lo, hi) = self.all_frames_box()?;
        let size = [hi[0] - lo[0], hi[1] - lo[1]];
        if size[0] <= 0.0 || size[1] <= 0.0 {
            return None;
        }
        // Em tempo real (smoke do dono 09/11): a cada quadro em que o desenho mexeu, mesmo a meio do arrasto.
        if live
            && self.align.pivot.dirty
            && self.align.pivot.follow
            && let Some(f) = self.align.pivot.frac
        {
            self.align.pivot.dirty = false;
            let p = [lo[0] + f[0] * size[0], lo[1] + f[1] * size[1]];
            let tol = self.pivot_pixel() / 2.0;
            if (p[0].abs() > tol || p[1].abs() > tol) && world_moves(p) {
                self.pivot_moved(p);
                return Some(p);
            }
        }
        if gesture {
            return None; // a meio do gesto, a fração de antes dele fica guardada
        }
        self.align.pivot.dirty = false;
        // Num dos 25 pontos guarda-se o PONTO, não a medida: dentro do meio pixel de folga a medida pode estar torta, e o
        // erro cresce com a caixa no gesto seguinte (sonda do smoke 09/11: 0,5096 em vez de 0,5, e o ponto apagava).
        let t = |i: u8| f32::from(i) / f32::from(SIDE - 1);
        self.align.pivot.frac = Some(match self.pivot_on_grid() {
            Some([c, r]) => [t(c), 1.0 - t(r)],
            None => [-lo[0] / size[0], -lo[1] / size[1]],
        });
        None
    }

    /// Onde a cruz do «By hand» cai com o rato em `(x, y)`: num dos 25 pontos, se perto; senão no pixel inteiro do
    /// desenho.
    fn hand_point(&mut self, v: View, x: f32, y: f32) -> [f32; 2] {
        let caixa = self.all_frames_box();
        if let Some(c) = caixa {
            let reach = ph2d_tokens::ROW_H_PX * SNAP_ROWS;
            if let Some(q) = grid().find(|q| {
                let (sx, sy) = v.to_screen(grid_point(c, *q));
                (sx - x).hypot(sy - y) <= reach
            }) {
                return grid_point(c, q);
            }
        }
        let m = self.pivot_pixel();
        let base = caixa.map_or([0.0, 0.0], |(lo, _)| lo);
        let raw = [(x - v.origin.0) / v.k, (v.origin.1 - y) / v.k];
        [0, 1].map(|a| base[a] + ((raw[a] - base[a]) / m).round() * m)
    }

    /// O «By hand», a grade e o arrasto da cruz. Devolve se tomou o clique.
    pub(super) fn pivot_press(&mut self, x: f32, y: f32) -> bool {
        let l = self.layout.align;
        if hit(l.by_hand, x, y) {
            self.align.pivot.by_hand = !self.align.pivot.by_hand;
            self.align.pivot.hand = None;
            return true;
        }
        if hit(l.pivot_follow, x, y) {
            self.align.pivot.follow = !self.align.pivot.follow;
            return true;
        }
        if hit(l.pivot_grid, x, y) {
            let g = l.pivot_grid;
            let at = |d: f32, w: f32| ((d / w * f32::from(SIDE)) as u8).min(SIDE - 1);
            let q = [at(x - g.x, g.w), at(y - g.y, g.h)];
            if let Some(c) = self.all_frames_box() {
                self.move_pivot(grid_point(c, q));
            }
            return true;
        }
        if self.align.pivot.by_hand
            && hit(l.preview, x, y)
            && let Some(v) = self.align.view
        {
            self.align.pivot.hand = Some(self.hand_point(v, x, y));
            return true;
        }
        false
    }

    /// O rato com a cruz na mão.
    pub(super) fn pivot_motion(&mut self, x: f32, y: f32) -> bool {
        if self.align.pivot.hand.is_none() {
            return false;
        }
        if let Some(v) = self.align.view {
            self.align.pivot.hand = Some(self.hand_point(v, x, y));
        }
        true
    }

    /// A cruz largada: o Pivot vai para lá.
    pub(super) fn pivot_release(&mut self, x: f32, y: f32) -> bool {
        if !self.pivot_motion(x, y) {
            return false;
        }
        if let Some(p) = self.align.pivot.hand.take() {
            self.move_pivot(p);
        }
        true
    }
}
