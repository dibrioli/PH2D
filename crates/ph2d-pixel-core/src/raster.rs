//! Kernels inteiros (`docs/Pixel Art/02_spec_engine_e_porte.md` §8). Não existe meio pixel.
//!
//! Paridade com o MVP web: `tests/fixtures/mvp_draw.json` (`docs/PixelArt/ferramentas/extrai_desenho_mvp.mjs`).

/// Uma célula.
pub type Cell = (i32, i32);

/// O `Math.round` do MVP (meio → +∞), não o `f64::round` (meio → longe de zero).
#[must_use]
pub(crate) fn js_round(v: f64) -> i32 {
    (v + 0.5).floor() as i32
}

/// As células do segmento `(x0,y0)→(x1,y1)`, inclusivo, em `cb(x, y)`.
///
/// **Canonicalizado**: os extremos ordenam-se (menor x; empate → menor y) antes de rasterizar, logo
/// A→B e B→A dão o MESMO conjunto de células (o Bresenham cru desempata pela direção). A ordem do
/// `cb` segue a direção canónica, não a de quem chama — quem acumula um CAMINHO inverte o trecho
/// (ver [`curve_path`]).
pub fn line_cells(x0: i32, y0: i32, x1: i32, y1: i32, mut cb: impl FnMut(i32, i32)) {
    let (mut ax, mut ay, mut bx, mut by) = (x0, y0, x1, y1);
    if bx < ax || (bx == ax && by < ay) {
        std::mem::swap(&mut ax, &mut bx);
        std::mem::swap(&mut ay, &mut by);
    }
    let dx = i64::from(bx - ax).abs();
    let dy = -i64::from(by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (ax, ay);
    loop {
        cb(x, y);
        if x == bx && y == by {
            return;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// As células do segmento NA ORDEM de `a` para `b` (o [`line_cells`] canónico, invertido se preciso).
#[must_use]
pub fn line_path(a: Cell, b: Cell) -> Vec<Cell> {
    let mut seg = Vec::new();
    line_cells(a.0, a.1, b.0, b.1, |x, y| seg.push((x, y)));
    if seg.first() != Some(&a) {
        seg.reverse();
    }
    seg
}

// ── pixel-perfect ─────────────────────────────────────────────────────────────

/// `m` é o cotovelo redundante de um L entre `a` e `c` (Ricky Han, como o Aseprite): os dois vizinhos
/// são ortogonais a `m` e diferem nos dois eixos.
#[must_use]
pub fn is_l_corner(a: Cell, m: Cell, c: Cell) -> bool {
    (a.0 == m.0 || a.1 == m.1) && (c.0 == m.0 || c.1 == m.1) && a.0 != c.0 && a.1 != c.1
}

/// O caminho inteiro sem os cotovelos (a forma em lote: formas e gates).
#[must_use]
pub fn pixel_perfect(path: &[Cell]) -> Vec<Cell> {
    if path.len() < 3 {
        return path.to_vec();
    }
    let mut out = vec![path[0]];
    let mut prev = path[0];
    for i in 1..path.len() - 1 {
        if is_l_corner(prev, path[i], path[i + 1]) {
            continue;
        }
        out.push(path[i]);
        prev = path[i];
    }
    out.push(path[path.len() - 1]);
    out
}

/// O que um ponto do lápis ao vivo fez.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PpStep {
    /// A célula pintada AGORA (sob o cursor); `None` = repetiu a última.
    pub paint: Option<Cell>,
    /// A célula mostrada um passo antes que o novo ponto revelou ser cotovelo: volta ao valor antigo.
    pub retract: Option<Cell>,
}

/// **O pixel-perfect ao vivo** (lei do dono: o lápis pinta sob o cursor, sem atraso): cada ponto
/// pinta na hora; quando os três últimos formam um L, o do meio RETRAI.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PixelPerfect {
    /// Os dois últimos pontos aceites — os únicos que o juízo do próximo lê.
    tail: Vec<Cell>,
}

impl PixelPerfect {
    pub fn push(&mut self, x: i32, y: i32) -> PpStep {
        if self.tail.last() == Some(&(x, y)) {
            return PpStep::default();
        }
        self.tail.push((x, y));
        let mut step = PpStep {
            paint: Some((x, y)),
            retract: None,
        };
        let n = self.tail.len();
        if n >= 3 && is_l_corner(self.tail[n - 3], self.tail[n - 2], self.tail[n - 1]) {
            step.retract = Some(self.tail.remove(n - 2));
        }
        if self.tail.len() > 2 {
            self.tail.drain(..self.tail.len() - 2);
        }
        step
    }
}

// ── curva ─────────────────────────────────────────────────────────────────────

/// **A Bézier quadrática** `(x0,y0) → (x1,y1)` com controlo `(cx,cy)`, em células, com a MESMA lei
/// do lápis: amostras ligadas por [`line_cells`] e o caminho pelo [`pixel_perfect`]. Controlo sobre
/// a corda (desvio < ½ célula e projeção dentro dela) = a reta, célula a célula.
#[must_use]
pub fn curve_path(x0: i32, y0: i32, cx: i32, cy: i32, x1: i32, y1: i32) -> Vec<Cell> {
    let (fx0, fy0, fcx, fcy, fx1, fy1) = (
        f64::from(x0),
        f64::from(y0),
        f64::from(cx),
        f64::from(cy),
        f64::from(x1),
        f64::from(y1),
    );
    let (ex, ey) = (fx1 - fx0, fy1 - fy0);
    let len2 = ex * ex + ey * ey;
    let proj = if len2 > 0.0 {
        ((fcx - fx0) * ex + (fcy - fy0) * ey) / len2
    } else {
        0.0
    };
    let perp = if len2 > 0.0 {
        ((fcx - fx0) * ey - (fcy - fy0) * ex).abs() / len2.sqrt()
    } else {
        0.0
    };
    if perp < 0.5 && (0.0..=1.0).contains(&proj) {
        let mut reta = Vec::new();
        line_cells(x0, y0, x1, y1, |x, y| reta.push((x, y)));
        return pixel_perfect(&reta);
    }
    // Uma amostra por célula do polígono de controlo (que limita o arco); sem tecto: o custo é
    // linear no comprimento, e um tecto deixaria a curva longa grosseira.
    let ctrl = (fcx - fx0).hypot(fcy - fy0) + (fx1 - fcx).hypot(fy1 - fcy);
    let steps = (ctrl.ceil() as u64).max(2);
    let mut raw: Vec<Cell> = Vec::new();
    let push = |c: Cell, raw: &mut Vec<Cell>| {
        if raw.last() != Some(&c) {
            raw.push(c);
        }
    };
    let mut p = (x0, y0);
    push(p, &mut raw);
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let u = 1.0 - t;
        let b = (
            js_round(u * u * fx0 + 2.0 * u * t * fcx + t * t * fx1),
            js_round(u * u * fy0 + 2.0 * u * t * fcy + t * t * fy1),
        );
        if b == p {
            continue;
        }
        for c in line_path(p, b) {
            push(c, &mut raw);
        }
        p = b;
    }
    pixel_perfect(&raw)
}

// ── retângulos ────────────────────────────────────────────────────────────────

fn ordered(x0: i32, y0: i32, x1: i32, y1: i32) -> (i32, i32, i32, i32) {
    (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
}

/// O retângulo da bbox `(x0,y0)-(x1,y1)` (cantos em qualquer ordem), oco ou cheio.
pub fn rect_cells(x0: i32, y0: i32, x1: i32, y1: i32, filled: bool, mut cb: impl FnMut(i32, i32)) {
    let (ax, ay, bx, by) = ordered(x0, y0, x1, y1);
    if filled {
        for y in ay..=by {
            for x in ax..=bx {
                cb(x, y);
            }
        }
        return;
    }
    for x in ax..=bx {
        cb(x, ay);
        cb(x, by);
    }
    for y in ay + 1..by {
        cb(ax, y);
        cb(bx, y);
    }
}

/// **O retângulo de cantos redondos** (Aseprite 1.3.17): cada canto é o quadrante da elipse
/// [`ellipse_cells`] da caixa `(2r+1)²`, as arestas retas ligam-nos. O raio prende-se a metade do
/// lado menor; `r = 0` é o [`rect_cells`]. Cheio = cada linha entre os seus extremos.
pub fn rounded_rect_cells(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    radius: u32,
    filled: bool,
    mut cb: impl FnMut(i32, i32),
) {
    let (ax, ay, bx, by) = ordered(x0, y0, x1, y1);
    let side = (bx - ax).min(by - ay);
    let r = i32::try_from(radius).unwrap_or(i32::MAX).min(side / 2);
    if r <= 0 {
        rect_cells(ax, ay, bx, by, filled, cb);
        return;
    }
    let mut outline = Vec::new();
    // A célula do MEIO de um quadrante (x = r ou y = r) pertence aos dois cantos desse lado.
    let place = |v: i32, lo: i32, hi: i32| -> [Option<i32>; 2] {
        match v.cmp(&r) {
            std::cmp::Ordering::Less => [Some(lo + v), None],
            std::cmp::Ordering::Greater => [Some(hi - 2 * r + v), None],
            std::cmp::Ordering::Equal => [Some(lo + r), Some(hi - r)],
        }
    };
    ellipse_cells(0, 0, 2 * r, 2 * r, |x, y| {
        for px in place(x, ax, bx).into_iter().flatten() {
            for py in place(y, ay, by).into_iter().flatten() {
                outline.push((px, py));
            }
        }
    });
    for x in ax + r + 1..bx - r {
        outline.push((x, ay));
        outline.push((x, by));
    }
    for y in ay + r + 1..by - r {
        outline.push((ax, y));
        outline.push((bx, y));
    }
    if filled {
        fill_rows(ay, by, &outline, cb);
    } else {
        for (x, y) in outline {
            cb(x, y);
        }
    }
}

/// Cada linha de `ay..=by` cheia entre o menor e o maior x que `outline` tem nela.
fn fill_rows(ay: i32, by: i32, outline: &[Cell], mut cb: impl FnMut(i32, i32)) {
    let h = (by - ay + 1).max(0) as usize;
    let mut lo = vec![i32::MAX; h];
    let mut hi = vec![i32::MIN; h];
    for &(x, y) in outline {
        let r = y - ay;
        if r < 0 || r as usize >= h {
            continue;
        }
        lo[r as usize] = lo[r as usize].min(x);
        hi[r as usize] = hi[r as usize].max(x);
    }
    for r in 0..h {
        for x in lo[r]..=hi[r] {
            cb(x, ay + r as i32);
        }
    }
}

// ── elipses ───────────────────────────────────────────────────────────────────

/// **A elipse inscrita na bbox** (Zingl, os quatro quadrantes de UM termo de erro). Trabalhar pela
/// bbox e não por centro+raio é o que a torna certa em largura PAR e ÍMPAR.
pub fn ellipse_cells(x0: i32, y0: i32, x1: i32, y1: i32, mut cb: impl FnMut(i32, i32)) {
    let (mut xa, mut ya, mut xb, mut yb) = ordered(x0, y0, x1, y1);
    let (xa64, xb64) = (i64::from(xa), i64::from(xb));
    let a = xb64 - xa64;
    let b = i64::from(yb) - i64::from(ya);
    let mut b1 = b & 1;
    let mut dx = 4 * (1 - a) * b * b;
    let mut dy = 4 * (b1 + 1) * a * a;
    let mut err = dx + dy + b1 * a * a;
    ya += ((b + 1) / 2) as i32;
    yb = ya - b1 as i32;
    let a8 = 8 * a * a;
    b1 = 8 * b * b;
    loop {
        cb(xb, ya);
        cb(xa, ya);
        cb(xa, yb);
        cb(xb, yb);
        let e2 = 2 * err;
        if e2 <= dy {
            ya += 1;
            yb -= 1;
            dy += a8;
            err += dy;
        }
        if e2 >= dx || 2 * err > dy {
            xa += 1;
            xb -= 1;
            dx += b1;
            err += dx;
        }
        if xa > xb {
            break;
        }
    }
    while i64::from(ya) - i64::from(yb) < b {
        cb(xa - 1, ya);
        cb(xb + 1, ya);
        ya += 1;
        cb(xa - 1, yb);
        cb(xb + 1, yb);
        yb -= 1;
    }
}

/// A elipse cheia: o contorno de [`ellipse_cells`], cada linha entre os seus extremos.
pub fn ellipse_filled_cells(x0: i32, y0: i32, x1: i32, y1: i32, cb: impl FnMut(i32, i32)) {
    let (_, ay, _, by) = ordered(x0, y0, x1, y1);
    let mut outline = Vec::new();
    ellipse_cells(x0, y0, x1, y1, |x, y| outline.push((x, y)));
    fill_rows(ay, by, &outline, cb);
}

/// `(x0,y0)-(x1,y1)` presa a um quadrado (`Shift` nas formas): o lado é o MAIOR dos dois, no
/// sentido em que o ponteiro foi.
#[must_use]
pub fn square_box(x0: i32, y0: i32, x1: i32, y1: i32) -> (i32, i32) {
    let side = (x1 - x0).abs().max((y1 - y0).abs());
    let sx = if x1 < x0 { -1 } else { 1 };
    let sy = if y1 < y0 { -1 } else { 1 };
    (x0 + sx * side, y0 + sy * side)
}

/// O ponto `b` preso ao ângulo de pixel art mais próximo à volta de `a` (`Shift` na Linha): 0°,
/// 90°, 45° e as razões 2:1 e 1:2 — as linhas que um pixel artist desenha sem serrilha.
#[must_use]
pub fn snap_pixel_angle(a: Cell, b: Cell) -> Cell {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    if dx == 0 && dy == 0 {
        return b;
    }
    const DIRS: [(i32, i32); 5] = [(1, 0), (2, 1), (1, 1), (1, 2), (0, 1)];
    let (ux, uy) = (f64::from(dx.abs()), f64::from(dy.abs()));
    let len = ux.hypot(uy);
    let mut best = (f64::MIN, (1, 0));
    for (px, py) in DIRS {
        let (fx, fy) = (f64::from(px), f64::from(py));
        let cos = (ux * fx + uy * fy) / (len * fx.hypot(fy));
        if cos > best.0 {
            best = (cos, (px, py));
        }
    }
    let (px, py) = best.1;
    // Quantas vezes o passo cabe na projeção (o comprimento segue a mão).
    let k =
        js_round((ux * f64::from(px) + uy * f64::from(py)) / f64::from(px * px + py * py)).max(0);
    (a.0 + dx.signum() * px * k, a.1 + dy.signum() * py * k)
}

// ── polígonos ─────────────────────────────────────────────────────────────────

/// **O polígono fechado** pelos vértices `pts`: o contorno ([`line_cells`] entre vértices
/// consecutivos e do último ao primeiro) e o interior (par-ímpar nos CENTROS das células, linha a
/// linha). É o Polygon e o Contour do Aseprite: o contorno leva o pincel, o interior é cheio.
#[must_use]
pub fn polygon_cells(pts: &[Cell]) -> (Vec<Cell>, Vec<Cell>) {
    let mut outline = Vec::new();
    if pts.is_empty() {
        return (outline, Vec::new());
    }
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        for c in line_path(a, b) {
            if outline.last() != Some(&c) {
                outline.push(c);
            }
        }
    }
    let (ymin, ymax) = pts.iter().fold((i32::MAX, i32::MIN), |(lo, hi), p| {
        (lo.min(p.1), hi.max(p.1))
    });
    let mut interior = Vec::new();
    let mut xs: Vec<f64> = Vec::new();
    for y in ymin..=ymax {
        xs.clear();
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            // Meio-aberto: o vértice partilhado por duas arestas conta uma vez.
            if (a.1 <= y && y < b.1) || (b.1 <= y && y < a.1) {
                let t = f64::from(y - a.1) / f64::from(b.1 - a.1);
                xs.push(f64::from(a.0) + t * f64::from(b.0 - a.0));
            }
        }
        xs.sort_by(f64::total_cmp);
        for pair in xs.chunks_exact(2) {
            for x in pair[0].ceil() as i32..=pair[1].floor() as i32 {
                interior.push((x, y));
            }
        }
    }
    (outline, interior)
}
