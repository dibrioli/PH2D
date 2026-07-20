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
//!    raio `trap_px` fecha os vãos), cada componente subdividido em CÉLULAS.
//! 2. **Um componente de UMA cor é PREENCHIDO** — o contorno da cor cola na linha de graça
//!    (o flood de papel não atravessa tinta). É daqui que vem o casamento com o line-art.
//! 3. **Um componente CONTESTADO (≥2 cores)** — um blob aberto sem linha entre as cores — é
//!    dividido por **Voronoi geodésico** entre os rabiscos: faixas parelhas, a fronteira no
//!    meio (não há tinta a que colar). Fechar um vão para colar na linha é o knob **Trap**.
//!
//! ⚠️ **O min-cut de fluxo (`flow.rs`) NÃO é o produto** — é a referência (oráculo `#[cfg(test)]`,
//! provada `BK ≡ Edmonds–Karp`). Ele foi medido e reprovado como solver do produto: o guloso
//! um-contra-todos espreme as cores do meio, e o min-cut de Potts *encolhe* uma cor de semente
//! fina (minimizar a energia de Potts É minimizar fronteira). Detalhe em `voronoi_contested_component`.
//!
//! ⚠️ **O custo restante é a PARTIÇÃO** — 4096² ≈ 1,5 s, EDT + BFS sobre 16 M pixels (a `§7.1`
//! já apontava para cá). A alavanca nomeada é a exceção `rayon`, decisão do Enio.

use ph2d_core::Vec2;
use ph2d_flip_fill::{
    BOUNDARY, FILLED, FillResult, Grid, RDP_EPSILON_PX, signed_area, simplify_ring, trace_contours,
};

#[cfg(test)]
mod flow;
mod segment;
use segment::{NO_REGION, segment};

// Mirram o `fill_at` (`09 §2.1` — MESMO raster, MESMO back-end).
const MARGIN_PX: usize = 20;
const MAX_SIDE: usize = 4096;
const AXIS_COVER_PASSES: usize = 3;
const MIN_SCALE: f32 = 1e-3;

/// LazyBrush smoothness (`09 §3`). Cutting **through ink is free** (`V_INK = 0`), so the cut
/// runs along the line at no cost and the labelled set is confined by the line — the region a
/// scribble sits in is exactly what it colours. Crossing a GAP costs `V_WHITE` per pixel, so
/// a colour leaks through it only when that is cheaper than the whole boundary (the "a gap
/// need not close" of LazyBrush). White-white is the maximum clarity.
const V_WHITE: i32 = 8;
const V_INK: i32 = 0;

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

/// Colore o line-art (`09 §3`). **A subdivisão em células é CONDICIONAL**, e essa é a espinha:
///
/// - Um **componente estanque com UMA cor** é PREENCHIDO inteiro — sem corte. O contorno da
///   cor já cola na linha de graça, porque o componente nasceu de um flood de papel que não
///   atravessa tinta (`segment` §4a). Preencher em vez de cortar mata a *degenerescência da
///   semente fina*: um corte binário sobre uma célula pequena prefere **cercar a semente**
///   (perímetro barato) a achar a fronteira real, e uma pincelada fina cairia nessa cilada.
/// - Um **componente contestado por ≥2 cores** — o caso do 3º smoke, quatro cores num blob
///   aberto sem uma linha entre elas — é subdividido em células e **cortado** (LazyBrush): sem
///   tinta a que colar, a fronteira cai no MEIO. O corte roda sobre as células DAQUELE
///   componente, então é barato (dezenas–centenas de nós) e só paga onde de fato há disputa.
///
/// ⚠️ **Nunca sobre a grade de pixels.** `§7.1` mediu 3,3 s a 4096² e **157 s** com dois
/// rabiscos se contradizendo sobre uma linha. Aqui o corte é local ao componente contestado.
fn solve(grid: &Grid, labels: &[(u16, Vec<usize>)], trap_px: f32) -> Vec<Option<usize>> {
    let n = grid.w * grid.h;
    let mut assign: Vec<Option<usize>> = vec![None; n];

    let seg = segment(grid, trap_px, V_WHITE, V_INK);
    if seg.count == 0 {
        return assign;
    }

    // O componente de cada célula (uma célula nunca cruza componente, por construção).
    let mut cell_comp: Vec<u32> = vec![NO_REGION; seg.count];
    for i in 0..n {
        let r = seg.region[i];
        if r != NO_REGION {
            cell_comp[r as usize] = seg.component[i];
        }
    }
    let comp_count = seg
        .component
        .iter()
        .filter(|&&c| c != NO_REGION)
        .fold(0u32, |m, &c| m.max(c + 1)) as usize;

    // As cores que semeiam cada componente, e a célula-semente de cada (cor, componente).
    // Primeira reivindicação de uma célula vence (determinístico: `labels` é ordem estável).
    let mut comp_labels: Vec<Vec<usize>> = vec![Vec::new(); comp_count];
    let mut claimed: Vec<Option<usize>> = vec![None; seg.count];
    for (k, (_, pixels)) in labels.iter().enumerate() {
        for &p in pixels {
            let r = seg.region[p];
            if r == NO_REGION || claimed[r as usize].is_some() {
                continue;
            }
            claimed[r as usize] = Some(k);
            let c = cell_comp[r as usize] as usize;
            if !comp_labels[c].contains(&k) {
                comp_labels[c].push(k);
            }
        }
    }

    let mut region_label: Vec<Option<usize>> = vec![None; seg.count];
    for (comp, ls) in comp_labels.iter().enumerate() {
        match ls.as_slice() {
            [] => {}
            // UMA cor: preenche o componente inteiro (a linha já é respeitada — §4a).
            &[only] => {
                for (cell, &cc) in cell_comp.iter().enumerate() {
                    if cc as usize == comp {
                        region_label[cell] = Some(only);
                    }
                }
            }
            // ≥2 cores: corta as células DESTE componente (LazyBrush, fronteira no meio).
            _ => voronoi_contested_component(&seg, &cell_comp, comp, &claimed, &mut region_label),
        }
    }

    if std::env::var("PH2D_COLORIZE_LOG").is_ok() {
        let mut px_of = vec![0usize; labels.len()];
        for &rl in &region_label {
            if let Some(k) = rl {
                // conta CÉLULAS por rótulo (proxy da área)
                px_of[k] += 1;
            }
        }
        for (comp, ls) in comp_labels.iter().enumerate() {
            if !ls.is_empty() {
                eprintln!("[colorize]   componente {comp}: cores {ls:?}");
            }
        }
        eprintln!("[colorize]   celulas por rotulo: {px_of:?}");
    }

    // De volta aos pixels. A tinta nunca recebe cor — a linha fica por cima (`09 §2`).
    for (i, a) in assign.iter_mut().enumerate() {
        if grid.flags[i] & BOUNDARY != 0 {
            continue;
        }
        let r = seg.region[i];
        if r != NO_REGION {
            *a = region_label[r as usize];
        }
    }
    assign
}

/// Divide UM componente contestado (≥2 cores) entre as cores, por **Voronoi geodésico**: cada
/// célula recebe o rótulo do rabisco mais próximo, na métrica das arestas do grafo (`V_pq`).
///
/// ⚠️ **Por que Voronoi, e não o min-cut do LazyBrush.** Dois atalhos foram MEDIDOS e
/// reprovados, e o próprio min-cut também:
/// - **guloso um-contra-todos** — *espreme as cores do meio* (`[856,128,128,856]` no blob de
///   4 cores: a externa reivindica primeiro até o vizinho e a do meio fica com uma tira);
/// - **min-cut de Potts** (α-expansion) — ⚠️ **minimizar a energia de Potts É minimizar o
///   comprimento total de fronteira**, e o mínimo de verdade *encolhe uma cor do meio com
///   semente fina* (o perímetro de um blobinho custa menos que duas cordas), dando
///   `[2131,128,2991,909]` — o oposto do que o artista, vendo quatro rabiscos parelhos,
///   espera. Só uma semente GORDA fixaria uma faixa gorda.
///
/// O Voronoi dá **faixas parelhas** (a expectativa), é **confinado** (a cor não vaza — só
/// atribuo as células DESTE componente) e é determinístico. O casamento com a LINHA de um
/// **line-art de verdade** não vem daqui: vem do preenchimento por-componente (uma cor numa
/// região delimitada cola na tinta de graça, §4a). Este caminho só decide um componente que
/// tem ≥2 cores E nenhuma linha as separando — em geral um blob aberto, onde a fronteira é
/// arbitrária e o meio geométrico é a resposta honesta. Fechar um vão para colar na linha é o
/// que o knob **Trap** faz (separa em componentes de uma cor → preenchidos → colados).
fn voronoi_contested_component(
    seg: &segment::Segmentation,
    cell_comp: &[u32],
    comp: usize,
    claimed: &[Option<usize>],
    region_label: &mut [Option<usize>],
) {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    // Reindexação local + adjacência interna ao componente.
    let mut local: Vec<u32> = vec![u32::MAX; seg.count];
    let mut cells: Vec<usize> = Vec::new();
    for (cell, &cc) in cell_comp.iter().enumerate() {
        if cc as usize == comp {
            local[cell] = cells.len() as u32;
            cells.push(cell);
        }
    }
    let m = cells.len();
    let mut adj: Vec<Vec<(u32, i64)>> = vec![Vec::new(); m];
    for &(a, b, w) in &seg.edges {
        let (la, lb) = (local[a as usize], local[b as usize]);
        if la != u32::MAX && lb != u32::MAX {
            adj[la as usize].push((lb, i64::from(w)));
            adj[lb as usize].push((la, i64::from(w)));
        }
    }

    // Multi-fonte Dijkstra: as sementes partem com distância 0 carregando o rótulo. O `done`
    // fixa o 1º (menor) que chega; empate resolve por rótulo então célula (HR-5, na ordem do
    // heap `(dist, label, cell)`).
    let mut done = vec![false; m];
    let mut label = vec![usize::MAX; m];
    let mut heap: BinaryHeap<Reverse<(i64, usize, u32)>> = BinaryHeap::new();
    for (li, &cell) in cells.iter().enumerate() {
        if let Some(k) = claimed[cell] {
            heap.push(Reverse((0, k, li as u32)));
        }
    }
    while let Some(Reverse((d, k, li))) = heap.pop() {
        let li = li as usize;
        if done[li] {
            continue;
        }
        done[li] = true;
        label[li] = k;
        for &(nb, w) in &adj[li] {
            if !done[nb as usize] {
                heap.push(Reverse((d + w, k, nb)));
            }
        }
    }

    for (li, &cell) in cells.iter().enumerate() {
        if label[li] != usize::MAX {
            region_label[cell] = Some(label[li]);
        }
    }
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
