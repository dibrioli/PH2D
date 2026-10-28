//! **Uma aba do editor de Region** ([plano GEF 16](../../../../docs/GameEngineFuncional/16_uso_region_plano.md) D2):
//! a área escolhida em píxeis da imagem, a grelha de snap, a vista (zoom e arrasto) e os gestos. Nada aqui
//! pinta nem toca o mundo: o OK deixa um [`Outcome`] que o [`super::frame`] aplica pela porta do Inspector.

use ph2d_editor_core::SheetImage;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

/// A imagem encaixada ocupa esta fracção da tela ao abrir (o resto é margem para as alças).
const FIT_FRAC: f32 = 0.9; // LITERAL-PX-OK: fração de layout
/// O passo do zoom por clique da roda.
const ZOOM_STEP: f32 = 1.1; // LITERAL-PX-OK: razão adimensional
/// O zoom máximo: um píxel da imagem a ocupar este número de píxeis do ecrã.
const ZOOM_MAX: f32 = 64.0; // LITERAL-PX-OK: razão adimensional
/// A largura de um campo Columns/Rows, em alturas de linha.
const FIELD_ROWS: f32 = 2.0; // LITERAL-PX-OK: múltiplo do token de linha
/// A largura de um botão OK/Cancel, em alturas de linha.
const BUTTON_ROWS: f32 = 3.0; // LITERAL-PX-OK: múltiplo do token de linha

/// Que campo do topo está a ser escrito.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Cols,
    Rows,
}

/// Como a aba acabou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// OK: a área (`[x, y, w, h]` px) e a grelha.
    Ok { rect: [u32; 4], grid: [u32; 2] },
    /// Cancel / `Esc`: nada muda.
    Cancel,
}

/// O gesto em curso sobre a tela.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Drag {
    #[default]
    None,
    /// Arrastar dentro da área: `grab` é o ponto agarrado (px da imagem), `start` a área ao carregar; sem mexer a
    /// área, é um clique (escolhe a célula).
    Move {
        grab: [f32; 2],
        start: [f32; 4],
        moved: bool,
    },
    /// Uma das 8 alças: `sx`/`sy` dizem que lados ela move (`-1` o esquerdo/de cima, `1` o outro, `0` nenhum).
    Resize { sx: i8, sy: i8, start: [f32; 4] },
    /// Arrastar fora da área: uma área nova de `from` ao ponteiro.
    Draw { from: [f32; 2], moved: bool },
    /// Botão do meio, ou espaço premido: a vista anda com o ponteiro.
    Pan { last: [f32; 2] },
}

/// Onde cada coisa da aba está (ecrã).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub area: Rect,
    pub cols_label: Rect,
    pub cols: Rect,
    pub rows_label: Rect,
    pub rows: Rect,
    /// O botão **Snap** (ligado/desligado).
    pub snap: Rect,
    pub hint: Rect,
    pub cancel: Rect,
    pub ok: Rect,
    /// A tela onde a imagem se pinta.
    pub canvas: Rect,
}

impl Default for Layout {
    /// Nada pintado ainda: tudo vazio, nenhum clique acerta.
    fn default() -> Self {
        let z = Rect::new(0.0, 0.0, 0.0, 0.0);
        Self {
            area: z,
            cols_label: z,
            cols: z,
            rows_label: z,
            rows: z,
            snap: z,
            hint: z,
            cancel: z,
            ok: z,
            canvas: z,
        }
    }
}

/// Uma aba aberta.
#[derive(Debug)]
pub struct RegionDoc {
    pub entity: u64,
    /// A identidade da sprite (o restauro do desfazer muda-lhe os bits).
    pub stable: u64,
    pub name: String,
    pub image: Option<SheetImage>,
    /// Os píxeis da imagem inteira.
    pub px: [u32; 2],
    /// A área em píxeis da imagem, sempre inteira e dentro dela.
    pub rect: [f32; 4],
    pub grid: [u32; 2],
    pub editing: Option<(Field, String)>,
    /// O botão Snap (dono 27/10: «é preciso ter a opção sem snap»): desligado, a área anda ao píxel.
    pub snap_on: bool,
    /// `Alt` premido: inverte o Snap enquanto está premido.
    pub alt: bool,
    space: bool,
    /// Píxeis do ecrã por píxel da imagem.
    pub zoom: f32,
    /// Onde, no ecrã, fica o canto (0, 0) da imagem.
    pub origin: [f32; 2],
    fitted: bool,
    drag: Drag,
    pub layout: Layout,
    pub outcome: Option<Outcome>,
}

fn hit(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

impl RegionDoc {
    /// Uma aba sobre a imagem de `px` píxeis, com a área `rect` (`None` = a imagem inteira) e a grelha `grid`.
    #[must_use]
    pub fn new(
        entity: u64,
        stable: u64,
        name: String,
        px: [u32; 2],
        rect: Option<[f32; 4]>,
        grid: [u32; 2],
    ) -> Self {
        let inteira = [0.0, 0.0, px[0] as f32, px[1] as f32];
        Self {
            entity,
            stable,
            name,
            image: None,
            px,
            rect: rect.unwrap_or(inteira),
            grid: [
                grid[0].clamp(1, px[0].max(1)),
                grid[1].clamp(1, px[1].max(1)),
            ],
            editing: None,
            snap_on: true,
            alt: false,
            space: false,
            zoom: 1.0,
            origin: [0.0, 0.0],
            fitted: false,
            drag: Drag::None,
            layout: Layout::default(),
            outcome: None,
        }
    }

    /// Dispõe a aba em `area` e, na primeira vez, encaixa a imagem na tela.
    pub fn lay_out(&mut self, area: Rect, text: &mut TextSystem) -> Layout {
        let gap = Spacing::Md.px();
        let font = TypeToken::Sm.px();
        let h = ROW_H_PX;
        let y = area.y + gap;
        let label =
            |t: &mut TextSystem, key: &str| {
                ph2d_editor_core::paint::rect_for_label(
                    ph2d_editor_core::paint::label_column_width(t, font, [ph2d_i18n::tr(key)]),
                )
            };
        let mut x = area.x + gap;
        let cols_label = Rect::new(x, y, label(text, "region_editor.columns"), h);
        x += cols_label.w;
        let cols = Rect::new(x, y, h * FIELD_ROWS, h);
        x += cols.w + gap;
        let rows_label = Rect::new(x, y, label(text, "region_editor.rows"), h);
        x += rows_label.w;
        let rows = Rect::new(x, y, h * FIELD_ROWS, h);
        x += rows.w + gap;
        let bw = h * BUTTON_ROWS;
        let snap = Rect::new(x, y, bw, h);
        x += snap.w + gap;
        let ok = Rect::new(area.x + area.w - gap - bw, y, bw, h);
        let cancel = Rect::new(ok.x - gap - bw, y, bw, h);
        let hint = Rect::new(x, y, (cancel.x - gap - x).max(0.0), h);
        let top = y + h + gap;
        let canvas = Rect::new(area.x, top, area.w, (area.y + area.h - top).max(0.0));
        self.layout = Layout {
            area,
            cols_label,
            cols,
            rows_label,
            rows,
            snap,
            hint,
            cancel,
            ok,
            canvas,
        };
        if !self.fitted && canvas.w > 0.0 && canvas.h > 0.0 && self.px[0] > 0 && self.px[1] > 0 {
            self.fit();
        }
        self.layout
    }

    /// A imagem inteira cabe na tela, ao centro.
    pub fn fit(&mut self) {
        let c = self.layout.canvas;
        let (w, h) = (self.px[0].max(1) as f32, self.px[1].max(1) as f32);
        self.zoom = (c.w / w).min(c.h / h) * FIT_FRAC;
        self.origin = [
            c.x + (c.w - w * self.zoom) * 0.5,
            c.y + (c.h - h * self.zoom) * 0.5,
        ];
        self.fitted = true;
    }

    /// Ecrã → píxeis da imagem (sem prender).
    #[must_use]
    pub fn to_image(&self, x: f32, y: f32) -> [f32; 2] {
        [
            (x - self.origin[0]) / self.zoom,
            (y - self.origin[1]) / self.zoom,
        ]
    }

    /// A área no ecrã.
    #[must_use]
    pub fn rect_on_screen(&self) -> Rect {
        let [x, y, w, h] = self.rect;
        Rect::new(
            self.origin[0] + x * self.zoom,
            self.origin[1] + y * self.zoom,
            w * self.zoom,
            h * self.zoom,
        )
    }

    /// A imagem inteira no ecrã.
    #[must_use]
    pub fn image_on_screen(&self) -> Rect {
        Rect::new(
            self.origin[0],
            self.origin[1],
            self.px[0] as f32 * self.zoom,
            self.px[1] as f32 * self.zoom,
        )
    }

    /// A linha `i` da grelha no eixo `axis`, em píxeis da imagem.
    #[must_use]
    pub fn grid_line(&self, axis: usize, i: u32) -> f32 {
        (i as f32 * self.px[axis] as f32 / self.grid[axis] as f32).round()
    }

    /// O snap vale agora: o botão Snap, invertido enquanto o `Alt` está premido.
    #[must_use]
    pub fn snapping(&self) -> bool {
        self.snap_on != self.alt
    }

    /// ⭐ **O snap** (plano GEF 16 D2): `v` (px, eixo `axis`) cai na linha da grelha mais perto; sem [`Self::snapping`]
    /// e com 1 coluna (ou linha), ao píxel. Sempre dentro da imagem.
    #[must_use]
    pub fn snap(&self, axis: usize, v: f32) -> f32 {
        let max = self.px[axis] as f32;
        let n = self.grid[axis];
        if !self.snapping() || n <= 1 {
            return v.round().clamp(0.0, max);
        }
        let passo = max / n as f32;
        let i = (v / passo).round().clamp(0.0, n as f32) as u32;
        self.grid_line(axis, i)
    }

    /// As 8 alças no ecrã: `(sx, sy, quadrado)`.
    #[must_use]
    pub fn handles(&self) -> Vec<(i8, i8, Rect)> {
        let r = self.rect_on_screen();
        let s = Spacing::Sm.px();
        let mut v = Vec::with_capacity(8);
        for sy in [-1_i8, 0, 1] {
            for sx in [-1_i8, 0, 1] {
                if sx == 0 && sy == 0 {
                    continue;
                }
                let cx = r.x + r.w * f32::from(sx + 1) * 0.5;
                let cy = r.y + r.h * f32::from(sy + 1) * 0.5;
                v.push((sx, sy, Rect::new(cx - s, cy - s, s * 2.0, s * 2.0)));
            }
        }
        v
    }

    /// Carregar o botão principal em `(x, y)`.
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        self.close_field(true);
        let l = self.layout;
        if !hit(l.area, x, y) {
            return false;
        }
        if hit(l.ok, x, y) {
            self.ok();
        } else if hit(l.cancel, x, y) {
            self.outcome = Some(Outcome::Cancel);
        } else if hit(l.cols, x, y) {
            self.editing = Some((Field::Cols, self.grid[0].to_string()));
        } else if hit(l.rows, x, y) {
            self.editing = Some((Field::Rows, self.grid[1].to_string()));
        } else if hit(l.snap, x, y) {
            self.snap_on = !self.snap_on;
        } else if hit(l.canvas, x, y) {
            let p = self.to_image(x, y);
            self.drag = if self.space {
                Drag::Pan { last: [x, y] }
            } else if let Some(&(sx, sy, _)) = self.handles().iter().find(|(_, _, r)| hit(*r, x, y))
            {
                Drag::Resize {
                    sx,
                    sy,
                    start: self.rect,
                }
            } else if hit(self.rect_on_screen(), x, y) {
                Drag::Move {
                    grab: p,
                    start: self.rect,
                    moved: false,
                }
            } else {
                Drag::Draw {
                    from: [self.snap(0, p[0]), self.snap(1, p[1])],
                    moved: false,
                }
            };
        }
        true
    }

    /// Carregar o botão do meio: a vista anda.
    pub fn press_middle(&mut self, x: f32, y: f32) -> bool {
        if !hit(self.layout.canvas, x, y) {
            return false;
        }
        self.drag = Drag::Pan { last: [x, y] };
        true
    }

    /// O ponteiro mexeu-se (com o gesto em curso, se houver).
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let p = self.to_image(x, y);
        match self.drag {
            Drag::None => return false,
            Drag::Pan { last } => {
                self.origin[0] += x - last[0];
                self.origin[1] += y - last[1];
                self.drag = Drag::Pan { last: [x, y] };
            }
            Drag::Move { grab, start, moved } => {
                let mut r = start;
                for i in 0..2 {
                    let max = self.px[i] as f32 - start[2 + i];
                    r[i] = self
                        .snap(i, start[i] + p[i] - grab[i])
                        .clamp(0.0, max.max(0.0));
                }
                self.drag = Drag::Move {
                    grab,
                    start,
                    moved: moved || r != start,
                };
                self.rect = r;
            }
            Drag::Resize { sx, sy, start } => {
                let mut lo = [start[0], start[1]];
                let mut hi = [start[0] + start[2], start[1] + start[3]];
                for (i, s) in [sx, sy].into_iter().enumerate() {
                    let v = self.snap(i, p[i]);
                    match s {
                        -1 => lo[i] = v.min(hi[i] - 1.0),
                        1 => hi[i] = v.max(lo[i] + 1.0),
                        _ => {}
                    }
                }
                self.rect = [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]];
            }
            Drag::Draw { from, .. } => {
                let to = [self.snap(0, p[0]), self.snap(1, p[1])];
                if to != from {
                    self.drag = Drag::Draw { from, moved: true };
                    self.rect = [
                        from[0].min(to[0]),
                        from[1].min(to[1]),
                        (from[0] - to[0]).abs().max(1.0),
                        (from[1] - to[1]).abs().max(1.0),
                    ];
                }
            }
        }
        self.keep_inside();
        true
    }

    /// Largar o botão. Um clique sem arrastar (dentro ou fora da área) escolhe a célula da grelha por baixo.
    pub fn release(&mut self, x: f32, y: f32) -> bool {
        let drag = std::mem::take(&mut self.drag);
        if let Drag::Draw { moved: false, .. } | Drag::Move { moved: false, .. } = drag {
            self.pick_cell(x, y);
        }
        drag != Drag::None
    }

    /// A célula da grelha sob `(x, y)` passa a ser a área (com 1 × 1, ou sem snap, não há células: nada muda).
    fn pick_cell(&mut self, x: f32, y: f32) {
        if self.grid == [1, 1] || !self.snapping() {
            return;
        }
        let p = self.to_image(x, y);
        let mut r = [0.0; 4];
        for i in 0..2 {
            if p[i] < 0.0 || p[i] >= self.px[i] as f32 {
                return;
            }
            let c = ((p[i] / self.px[i] as f32) * self.grid[i] as f32).floor() as u32;
            r[i] = self.grid_line(i, c);
            r[2 + i] = self.grid_line(i, c + 1) - r[i];
        }
        self.rect = r;
        self.keep_inside();
    }

    /// A área fica dentro da imagem, inteira, com 1 px pelo menos.
    fn keep_inside(&mut self) {
        for i in 0..2 {
            let max = self.px[i] as f32;
            let lo = self.rect[i].round().clamp(0.0, (max - 1.0).max(0.0));
            let w = self.rect[2 + i].round().clamp(1.0, (max - lo).max(1.0));
            self.rect[i] = lo;
            self.rect[2 + i] = w;
        }
    }

    /// A roda: zoom em volta do ponteiro.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        if !hit(self.layout.canvas, x, y) || dy == 0.0 {
            return false;
        }
        let fit = {
            let c = self.layout.canvas;
            (c.w / self.px[0].max(1) as f32).min(c.h / self.px[1].max(1) as f32) * FIT_FRAC
        };
        let novo = if dy > 0.0 {
            self.zoom * ZOOM_STEP
        } else {
            self.zoom / ZOOM_STEP
        };
        let novo = novo.clamp(fit.min(1.0) * FIT_FRAC, ZOOM_MAX);
        let p = self.to_image(x, y);
        self.zoom = novo;
        self.origin = [x - p[0] * novo, y - p[1] * novo];
        true
    }

    /// `Alt` e o espaço, como estão agora.
    pub fn set_keys(&mut self, alt: bool, space: Option<bool>) {
        self.alt = alt;
        if let Some(s) = space {
            self.space = s;
        }
    }

    /// OK: a área e a grelha de agora.
    pub fn ok(&mut self) {
        self.close_field(true);
        self.outcome = Some(Outcome::Ok {
            rect: self.rect.map(|v| v.round().max(0.0) as u32),
            grid: self.grid,
        });
    }

    /// Escreve no campo aberto (só algarismos).
    pub fn type_text(&mut self, t: &str) -> bool {
        let Some((_, buf)) = &mut self.editing else {
            return false;
        };
        // Nunca mais algarismos do que o lado da imagem tem: uma célula tem pelo menos 1 px.
        let max = self.px[0].max(self.px[1]).max(1).to_string().len();
        buf.extend(
            t.chars()
                .filter(char::is_ascii_digit)
                .take(max.saturating_sub(buf.len())),
        );
        true
    }

    /// Apaga o último algarismo do campo aberto.
    pub fn backspace(&mut self) -> bool {
        self.editing.as_mut().map(|(_, b)| b.pop()).is_some()
    }

    /// Fecha o campo aberto; `commit` aplica-o (de 1 ao lado da imagem em px: uma célula tem 1 px pelo menos).
    pub fn close_field(&mut self, commit: bool) {
        let Some((f, t)) = self.editing.take() else {
            return;
        };
        if let (true, Ok(n)) = (commit, t.trim().parse::<u32>()) {
            let i = match f {
                Field::Cols => 0,
                Field::Rows => 1,
            };
            self.grid[i] = n.clamp(1, self.px[i].max(1));
        }
    }

    /// O texto de um campo: o número, ou o que está a ser escrito.
    #[must_use]
    pub fn field_text(&self, f: Field) -> (String, bool) {
        match &self.editing {
            Some((g, t)) if *g == f => (t.clone(), true),
            _ => {
                let i = usize::from(f == Field::Rows);
                (self.grid[i].to_string(), false)
            }
        }
    }
}
