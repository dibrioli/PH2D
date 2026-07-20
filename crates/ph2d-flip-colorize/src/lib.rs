#![forbid(unsafe_code)]
//! `ph2d-flip-colorize` — **o Colorize do Flip** (ADR-0114, `docs/Flip/09_colorize.md`):
//! rabiscar cores num line-art em vez de clicar região a região (a feature que só o TVPaint
//! entrega). Clean-room de **LazyBrush** (Sýkora et al., EG 2009).
//!
//! **Front-end novo, back-end INTOCADO** (`09 §2`): entra a line-art + os rabiscos coloridos,
//! e cada área conexa que ganha um rótulo sai como **GEOMETRIA** — pelo MESMO
//! `trace_contours`/`simplify_ring` do balde, então a borda de uma cor e a de um balde não
//! podem divergir, e colorir herda selecionar/mover/animar/undo de graça.
//!
//! ## O modelo (`09 §3` + `§8`) — o que de fato roda
//!
//! O LazyBrush original é um **Potts multiway cut** sobre pixels, cuja fronteira é atraída
//! para a linha. A `§7.1` MEDIU esse corte cru: **3,3 s a 4096²**, e **157 s** quando dois
//! rabiscos se contradizem sobre a mesma linha. Então o pipeline é outro, em três passos
//! (`solve`):
//!
//! 1. **Partição trapped-ball** (`segment.rs`): a arte vira componentes estanques (a bola de
//!    raio `trap_px` fecha os vãos; o flood de papel não atravessa tinta).
//! 2. **Um componente de UMA cor é PREENCHIDO** — o contorno da cor cola na linha de graça.
//!    É o caso comum num line-art de verdade, e custa um flood.
//! 3. **Um componente CONTESTADO (≥2 cores)** é dividido por **Voronoi geodésico POR PIXEL**
//!    (`voronoi.rs`): a frente de cada cor anda só por PAPEL — a tinta é intransponível —,
//!    então cada lado de uma linha pertence a quem está nele (a fronteira visível é a
//!    própria linha), e numa área aberta, sem tinta entre as cores, a fronteira cai no meio
//!    (faixas parelhas). Um vão estreito paga **pedágio de aperto** (quase selado; a
//!    trapped-ball em forma contínua); fechá-lo de vez é o knob **Trap**.
//!
//!
//! ⚠️ **O min-cut de fluxo (`flow.rs`) NÃO é o produto** — é a referência (oráculo `#[cfg(test)]`,
//! provada `BK ≡ Edmonds–Karp`). Ele foi medido e reprovado como solver do produto: o guloso
//! um-contra-todos espreme as cores do meio, e o min-cut de Potts *encolhe* uma cor de semente
//! fina (minimizar a energia de Potts É minimizar fronteira). Detalhe em `solve`.
//!
//! ⚠️ **O custo é a PARTIÇÃO + o Voronoi por pixel** — 4096² ≈ 1,7 s no pior caso (a caixa
//! inteira contestada; 512² = 15 ms, 1024² = 67 ms, 2048² = 348 ms — medido 2026-07-20):
//! EDT, BFS e Dial sobre 16 M pixels (a `§7.1` já apontava para cá). A alavanca nomeada é a
//! exceção `rayon`, decisão do Enio.

use ph2d_core::Vec2;
use ph2d_flip_fill::{
    BOUNDARY, FILLED, FillResult, Grid, RDP_EPSILON_PX, signed_area, simplify_ring, trace_contours,
};

#[cfg(test)]
mod flow;
mod segment;
mod voronoi;
use segment::{NO_REGION, segment};

// Mirram o `fill_at` (`09 §2.1` — MESMO raster, MESMO back-end).
const MARGIN_PX: usize = 20;
const MAX_SIDE: usize = 4096;
const AXIS_COVER_PASSES: usize = 3;
const MIN_SCALE: f32 = 1e-3;

/// Um rabisco: uma polilinha em coordenadas do documento, marcada com um rótulo de paleta.
/// Vários rabiscos podem ter o MESMO rótulo (a mesma cor em lugares diferentes).
#[derive(Clone, Debug)]
pub struct Scribble {
    pub label: u16,
    pub points: Vec<Vec2>,
    /// A ESPESSURA do rabisco (unidades do documento) — a semente é a cápsula, não o eixo.
    ///
    /// O que o artista PINTA tem de ser o que SEMEIA — mas o que está em jogo é
    /// **COBERTURA**, não correção: a largura decide quantas regiões um rabisco reivindica.
    /// `0` = só o eixo.
    ///
    /// ⚠️ **Correção: antes do `§8` isto era load-bearing e não é mais.** Sobre a grade de
    /// pixels, uma semente de 1 px degenerava o corte (o mínimo virava *"cercar aquele
    /// pixel"*), então a espessura era o que fazia um toque curto funcionar. Sobre o grafo de
    /// regiões esse corte nem é representável: um pixel identifica a região dele e a região
    /// inteira é o nó. Pinado por
    /// `the_region_reduction_cures_the_one_pixel_seed_degeneracy`.
    pub width: f32,
}

/// Uma região colorida: uma área conexa que recebeu um rótulo, como GEOMETRIA (`09 §2`). O
/// chamador a materializa como um `FlipStroke` com `hide_stroke` + `fill`, atrás na lista.
#[derive(Clone, Debug)]
pub struct ColorRegion {
    pub label: u16,
    pub fill: FillResult,
}

/// **Colorize:** line-art (as mesmas polilinhas de fronteira do balde) + rabiscos coloridos
/// → uma região de geometria por área conexa, cada uma com o rótulo que o corte LazyBrush
/// lhe deu. Vazio se não há linha OU não há rabisco.
#[must_use]
pub fn colorize(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    scribbles: &[Scribble],
    precision: f32,
    trap_px: f32,
) -> Vec<ColorRegion> {
    if strokes.is_empty() || scribbles.is_empty() {
        return Vec::new();
    }

    // 1. A grade: bbox das linhas + dos rabiscos, com margem — a receita do `fill_at`.
    let mut lo = Vec2::splat(f32::INFINITY);
    let mut hi = Vec2::splat(f32::NEG_INFINITY);
    for (pts, w, _) in strokes {
        for (i, p) in pts.iter().enumerate() {
            let r = w.get(i).copied().unwrap_or(0.0);
            lo = lo.min(Vec2::new(p.x - r, p.y - r));
            hi = hi.max(Vec2::new(p.x + r, p.y + r));
        }
    }
    for s in scribbles {
        for &p in &s.points {
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    if !lo.x.is_finite() || !hi.x.is_finite() {
        return Vec::new();
    }
    let scale = precision.max(MIN_SCALE);
    let mut grid = Grid::new(lo, hi, scale, MARGIN_PX, MAX_SIDE);

    // 2. As fronteiras NO EIXO (raio 0), a mesma cápsula do balde (`09 §2.1`, BUGS #14).
    for (pts, _, closed) in strokes {
        let n = pts.len();
        if n < 2 {
            continue;
        }
        let last = if *closed { n } else { n - 1 };
        for i in 0..last {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            grid.stroke_capsule(a, b, 0.0); // a parede
            grid.ink_capsule(a, b, 0.0); // o eixo (alvo do expand_under_ink)
        }
    }

    // 3. Os pixels dos rabiscos, agrupados por rótulo distinto; pixel de tinta não semeia cor.
    let labels = group_scribbles(&grid, scribbles);
    if labels.is_empty() {
        return Vec::new();
    }

    // 4. O multiway guloso (`09 §3`): índice de rótulo por pixel.
    let assign = solve(&grid, &labels, trap_px);

    // 5. Vetoriza por REGIÃO conexa — o back-end, intocado (`09 §2`).
    let out = regions_to_geometry(&mut grid, &labels, &assign);
    if std::env::var("PH2D_COLORIZE_LOG").is_ok() {
        let assigned = assign.iter().filter(|a| a.is_some()).count();
        eprintln!(
            "[colorize] grid {}x{} scale {:.1} · labels={} seeds={:?} · assigned={}/{} · regions={}",
            grid.w,
            grid.h,
            grid.scale,
            labels.len(),
            labels
                .iter()
                .map(|(l, p)| (*l, p.len()))
                .collect::<Vec<_>>(),
            assigned,
            grid.w * grid.h,
            out.len(),
        );
    }
    out
}

/// The neighbour of `i` in direction `0..4` (E/W/S/N), or `None` at the grid edge.
#[inline]
fn neighbour(grid: &Grid, i: usize, d: usize) -> Option<usize> {
    let x = i % grid.w;
    let y = i / grid.w;
    match d {
        0 if x + 1 < grid.w => Some(i + 1),
        1 if x > 0 => Some(i - 1),
        2 if y + 1 < grid.h => Some(i + grid.w),
        3 if y > 0 => Some(i - grid.w),
        _ => None,
    }
}

/// The grid pixels the scribble COVERS — the capsule of `width` swept along the polyline,
/// skipping ink (a colour can't seed on the line). `width = 0` degenerates to the axis.
fn polyline_pixels(grid: &Grid, points: &[Vec2], width: f32, out: &mut Vec<usize>) {
    let r_px = (width * 0.5 * grid.scale).max(0.0);
    // The swept union of discs IS the capsule; stamping per sample reuses the walk below.
    let sample = |p: Vec2, out: &mut Vec<usize>| {
        if r_px < 0.5 {
            if let Some((x, y)) = grid.pixel_of(p) {
                let i = y * grid.w + x;
                if grid.flags[i] & BOUNDARY == 0 {
                    out.push(i);
                }
            }
            return;
        }
        let (cx, cy) = grid.to_px(p);
        let x0 = (cx - r_px).floor().max(0.0) as usize;
        let x1 = ((cx + r_px).ceil().max(0.0) as usize).min(grid.w.saturating_sub(1));
        let y0 = (cy - r_px).floor().max(0.0) as usize;
        let y1 = ((cy + r_px).ceil().max(0.0) as usize).min(grid.h.saturating_sub(1));
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                if dx * dx + dy * dy > r_px * r_px {
                    continue;
                }
                let i = y * grid.w + x;
                if grid.flags[i] & BOUNDARY == 0 {
                    out.push(i);
                }
            }
        }
    };
    if let Some(&first) = points.first() {
        sample(first, out);
    }
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let d = b - a;
        let len_px = ((d.x * d.x + d.y * d.y).sqrt() * grid.scale)
            .ceil()
            .max(1.0) as usize;
        for s in 1..=len_px {
            let t = s as f32 / len_px as f32;
            sample(a + d * t, out);
        }
    }
}

/// Group scribble pixels by distinct palette label. A label with no non-ink pixel is dropped.
fn group_scribbles(grid: &Grid, scribbles: &[Scribble]) -> Vec<(u16, Vec<usize>)> {
    let mut out: Vec<(u16, Vec<usize>)> = Vec::new();
    for s in scribbles {
        let mut px = Vec::new();
        polyline_pixels(grid, &s.points, s.width, &mut px);
        if px.is_empty() {
            continue;
        }
        if let Some(entry) = out.iter_mut().find(|(l, _)| *l == s.label) {
            entry.1.extend(px);
        } else {
            out.push((s.label, px));
        }
    }
    for (_, px) in &mut out {
        px.sort_unstable();
        px.dedup();
    }
    out
}

/// Colore o line-art (`09 §3`). A espinha é **decidir por COMPONENTE**:
///
/// - Um **componente estanque com UMA cor** é PREENCHIDO inteiro — sem disputa. O contorno da
///   cor já cola na linha de graça, porque o componente nasceu de um flood de papel que não
///   atravessa tinta (`segment` §4a). É o caso comum num line-art de verdade, e custa um
///   flood. Preencher também mata a *degenerescência da semente fina*: não há corte capaz de
///   preferir "cercar a semente".
/// - Um **componente contestado por ≥2 cores** é dividido por **Voronoi geodésico por pixel**
///   (`voronoi::claim`): tinta intransponível, chanfro 5/7, pedágio de aperto. A fronteira
///   cola na linha onde há linha e cai no meio onde não há.
///
/// ⚠️ **Por que Voronoi, e não o min-cut do LazyBrush.** Três solvers foram MEDIDOS no blob
/// aberto de 4 cores e reprovados:
/// - **guloso um-contra-todos** — *espreme as cores do meio* (`[856,128,128,856]`: a externa
///   reivindica primeiro até o vizinho e a do meio fica com uma tira);
/// - **min-cut de Potts** (α-expansion) — ⚠️ **minimizar a energia de Potts É minimizar o
///   comprimento total de fronteira**, e o mínimo de verdade *encolhe uma cor do meio com
///   semente fina* (o perímetro de um blobinho custa menos que duas cordas), dando
///   `[2131,128,2991,909]` — o oposto do que o artista, vendo quatro rabiscos parelhos,
///   espera. Só uma semente GORDA fixaria uma faixa gorda. (E custa os 157 s da `§7.1`.)
/// - **Voronoi de CÉLULAS pesado por `V_pq`** (o 4º smoke, 2026-07-20) — célula > 1 px
///   cavalga a linha (a cor vaza por dentro do nó) e `V_pq` numa métrica é a direção errada
///   (tinta de graça = linha invisível; a fronteira caía no meio dos rabiscos, `max_x`
///   medido `0,575` contra a linha em `0,7`).
fn solve(grid: &Grid, labels: &[(u16, Vec<usize>)], trap_px: f32) -> Vec<Option<usize>> {
    let n = grid.w * grid.h;
    let mut assign: Vec<Option<usize>> = vec![None; n];

    let seg = segment(grid, trap_px);
    if seg.count == 0 {
        return assign;
    }

    // As cores que semeiam cada componente (ordem estável: `labels` preserva a chegada).
    let mut comp_labels: Vec<Vec<usize>> = vec![Vec::new(); seg.count];
    for (k, (_, pixels)) in labels.iter().enumerate() {
        for &p in pixels {
            let c = seg.component[p];
            if c == NO_REGION {
                continue;
            }
            let ls = &mut comp_labels[c as usize];
            if !ls.contains(&k) {
                ls.push(k);
            }
        }
    }

    let mut fill: Vec<Option<usize>> = vec![None; seg.count];
    let mut contested: Vec<u32> = Vec::new();
    for (c, ls) in comp_labels.iter().enumerate() {
        match ls.as_slice() {
            [] => {}
            &[only] => fill[c] = Some(only),
            _ => contested.push(c as u32),
        }
    }

    // UMA cor: preenche o componente inteiro (a linha já é respeitada — §4a). A tinta nunca
    // recebe cor — a linha fica por cima (`09 §2`).
    for (i, a) in assign.iter_mut().enumerate() {
        if grid.flags[i] & BOUNDARY != 0 {
            continue;
        }
        let c = seg.component[i];
        if c != NO_REGION {
            *a = fill[c as usize];
        }
    }

    // ≥2 cores: o Voronoi por pixel, só nos componentes disputados (rascunho reutilizado).
    if !contested.is_empty() {
        let mut scratch = voronoi::Scratch::new(&seg.ink_dist2);
        for &c in &contested {
            voronoi::claim(grid, &seg, c, labels, &mut scratch, &mut assign);
        }
    }

    if std::env::var("PH2D_COLORIZE_LOG").is_ok() {
        let mut px_of = vec![0usize; labels.len()];
        for a in assign.iter().flatten() {
            px_of[*a] += 1;
        }
        for (comp, ls) in comp_labels.iter().enumerate() {
            if !ls.is_empty() {
                eprintln!("[colorize]   componente {comp}: cores {ls:?}");
            }
        }
        eprintln!("[colorize]   pixels por rotulo: {px_of:?}");
    }
    assign
}

/// Split the labelled pixels into connected regions and vectorize each through the untouched
/// back-end (mark `FILLED`, crave onto the axis, trace, clear).
fn regions_to_geometry(
    grid: &mut Grid,
    labels: &[(u16, Vec<usize>)],
    assign: &[Option<usize>],
) -> Vec<ColorRegion> {
    let n = grid.w * grid.h;
    let mut visited = vec![false; n];
    let mut out = Vec::new();
    let eps = RDP_EPSILON_PX / grid.scale;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let Some(k) = assign[start] else { continue };
        if grid.flags[start] & BOUNDARY != 0 {
            continue;
        }
        // Flood the connected component of the same label.
        let mut comp = vec![start];
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            for d in 0..4 {
                if let Some(q) = neighbour(grid, i, d)
                    && !visited[q]
                    && assign[q] == Some(k)
                    && grid.flags[q] & BOUNDARY == 0
                {
                    visited[q] = true;
                    comp.push(q);
                    stack.push(q);
                }
            }
        }
        if let Some(fill) = trace_region(grid, &comp, eps) {
            out.push(ColorRegion {
                label: labels[k].0,
                fill,
            });
        }
    }
    out
}

/// Vectorize one connected region: mark it `FILLED`, crave the border onto the axis
/// (`expand_under_ink`, so two colours meet AT the line — no gap between them), trace, and
/// clear `FILLED` for the next region. The largest ring is the outer, opposite-signed rings
/// are its holes — the exact `fill_at` classification.
fn trace_region(grid: &mut Grid, comp: &[usize], eps: f32) -> Option<FillResult> {
    for f in &mut grid.flags {
        *f &= !FILLED;
    }
    for &i in comp {
        grid.flags[i] |= FILLED;
    }
    grid.expand_under_ink(AXIS_COVER_PASSES);

    let mut rings: Vec<Vec<Vec2>> = trace_contours(grid)
        .into_iter()
        .map(|r| simplify_ring(&r, eps, 2))
        .filter(|r| r.len() >= 3)
        .collect();

    for f in &mut grid.flags {
        *f &= !FILLED;
    }

    if rings.is_empty() {
        return None;
    }
    rings.sort_by(|a, b| {
        signed_area(b)
            .abs()
            .total_cmp(&signed_area(a).abs())
            .then(a[0].x.total_cmp(&b[0].x))
            .then(a[0].y.total_cmp(&b[0].y))
    });
    let outer = rings.remove(0);
    let outer_area = signed_area(&outer);
    if outer_area.abs() < 1e-6 {
        return None;
    }
    let holes: Vec<Vec<Vec2>> = rings
        .into_iter()
        .filter(|r| signed_area(r).signum() != outer_area.signum())
        .collect();
    Some(FillResult {
        outer,
        holes,
        scale: grid.scale,
        closures: Vec::new(),
    })
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
