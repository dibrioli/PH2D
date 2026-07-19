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
//! ## O modelo (`09 §3`)
//!
//! Um **Potts multiway cut** sobre a grade de pixels: `V_pq` (suavidade) = a clareza do papel
//! entre `p` e `q` (barato cortar dentro da tinta, caro no branco) ⇒ a fronteira é *atraída*
//! para o meio da linha, e **um vão não precisa fechar** (passar por ele custa a largura dele
//! em branco). `D_p` (dados) = o rabisco. O multiway é resolvido **guloso um-contra-todos**
//! (uma sequência de cortes BINÁRIOS — `flow.rs`), que é 9–18× o α-expansion com ΔE ≤ 0,04%.
//!
//! ⚠️ **O corte NÃO roda na grade de pixels** (`§8`, `segment.rs`). Ele rodava, e a `§7.1`
//! mediu o preço: **3,3 s a 4096²**, e **157 s** quando dois rabiscos se contradiziam sobre a
//! mesma linha — um clique que trava por minutos. Hoje a arte é primeiro particionada em
//! regiões estanques (trapped-ball) e o corte roda sobre esse grafo, de centenas de nós.
//! Medido: o penhasco virou **586 ms**, indistinguível do caso limpo (539 ms).
//!
//! ⚠️ **O que sobra caro é a PARTIÇÃO, não o corte** — 4096² ainda custa ~2,6 s, e é EDT +
//! BFS sobre 16 M pixels (a `§7.1` já apontava para cá). A alavanca nomeada é a exceção
//! `rayon`, que é decisão do Enio.

use ph2d_core::Vec2;
use ph2d_flip_fill::{
    BOUNDARY, FILLED, FillResult, Grid, RDP_EPSILON_PX, signed_area, simplify_ring, trace_contours,
};

mod flow;
mod segment;
use flow::Flow;
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

/// The guloso multiway (`09 §3`), run over the **trapped-ball region graph** (`§8`): for each
/// label, one binary cut (its regions vs the union of the others), first-claim wins. Returns
/// the label INDEX per pixel, or `None` (ink/unassigned).
///
/// ⚠️ **The cut does not run on pixels.** `§7.1` measured the pixel instance at 3,3 s on a
/// 4096² grid, and at **157 s** when two scribbles contradict each other across one line. The
/// region graph has hundreds of nodes instead of millions, and `segment` guarantees a cut on
/// it weighs exactly what the corresponding pixel cut would — so this is the same answer,
/// found in the time a click is allowed to take.
fn solve(grid: &Grid, labels: &[(u16, Vec<usize>)], trap_px: f32) -> Vec<Option<usize>> {
    let n = grid.w * grid.h;
    let mut assign: Vec<Option<usize>> = vec![None; n];

    let seg = segment_that_separates(grid, labels, trap_px);
    if seg.count == 0 {
        return assign;
    }
    let k_weight = seg.seed_weight();

    // Os rabiscos, traduzidos de pixels para REGIÕES. Primeira reivindicação vence — e é
    // determinístico porque `labels` já está em ordem estável. Um rabisco que atravessa a
    // linha reivindica os dois lados: é ele quem gerava o penhasco de 157 s, e agora é só
    // uma região a mais na lista de sementes.
    let mut seeds_of: Vec<Vec<u32>> = vec![Vec::new(); labels.len()];
    let mut claimed: Vec<Option<usize>> = vec![None; seg.count];
    for (k, (_, pixels)) in labels.iter().enumerate() {
        for &p in pixels {
            let r = seg.region[p];
            if r == NO_REGION {
                continue;
            }
            if claimed[r as usize].is_none() {
                claimed[r as usize] = Some(k);
                seeds_of[k].push(r);
            }
        }
    }

    // O rótulo de cada REGIÃO; depois ele é espalhado para os pixels dela.
    let mut region_label: Vec<Option<usize>> = vec![None; seg.count];

    if labels.len() == 1 {
        // **Uma cor colore exatamente as regiões que ela toca — e não atravessa costura.**
        //
        // Sem um segundo rabisco não há contra quem cortar, e o mínimo do LazyBrush fica
        // degenerado (custa 0 pôr TUDO do lado da fonte). Espalhar por qualquer aresta de
        // papel foi MEDIDO e é o vazamento clássico: uma linha com um furo de um pixel vira
        // uma costura de peso baixo, e a cor toma a grade inteira (94% dela, no gate). Uma
        // costura existe precisamente porque a bola NÃO passou ali — tratá-la como caminho
        // aberto desfaz a única promessa da trapped-ball.
        for &r in &seeds_of[0] {
            region_label[r as usize] = Some(0);
        }
    } else {
        for (k, seeds) in seeds_of.iter().enumerate() {
            if seeds.is_empty() {
                continue;
            }
            let mut f = Flow::build(seg.count, seg.edges.iter().copied());
            for &r in seeds {
                f.set_tlink(r as usize, k_weight, 0);
            }
            for (j, other) in seeds_of.iter().enumerate() {
                if j == k {
                    continue;
                }
                for &r in other {
                    f.set_tlink(r as usize, 0, k_weight);
                }
            }
            f.max_flow();
            for (r, &on) in f.source_side().iter().enumerate() {
                if on && region_label[r].is_none() {
                    region_label[r] = Some(k);
                }
            }
        }
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

/// **O raio da bola não é um palpite — é o menor que mantém os rabiscos do artista
/// separados.**
///
/// A pré-segmentação só serve ao corte se a partição tiver uma costura por onde ele possa
/// passar. Escolher esse raio por constante foi tentado e MEDIDO como o modelo errado: o que
/// ele precisa vencer é metade do VÃO, e vãos não escalam com nada que a arte ofereça (nem
/// com a bbox — a fração que servia ao smoke quebrava as fixtures — nem com a espessura da
/// linha: no smoke o traço mede 0,26 e o vão 1,2).
///
/// Mas a informação existe e é do próprio artista: **se dois rótulos caem na MESMA região, a
/// partição é grossa demais**, porque nenhuma escolha do corte poderá separá-los. Então a
/// bola cresce até isso deixar de acontecer. O `trap` do painel entra como PISO (o artista
/// dizendo *"vãos até aqui estão fechados"*), nunca como teto.
///
/// Termina em `MAX_STEPS` dobras, e a saída é sempre uma partição válida: quando nem a maior
/// bola separa dois rótulos, eles estão de fato na mesma área fechada e a primeira
/// reivindicação vence — que é a resposta honesta, não um laço infinito.
fn segment_that_separates(
    grid: &Grid,
    labels: &[(u16, Vec<usize>)],
    trap_px: f32,
) -> segment::Segmentation {
    /// Dobras de raio antes de desistir. 8 cobrem 256x do raio inicial — muito além de
    /// qualquer vão que caiba numa grade de 4096.
    const MAX_STEPS: usize = 8;

    // O passo inicial: o que o artista pediu, ou um pixel. Zero não costura nada.
    let mut r = trap_px.max(1.0);
    let mut best = segment(grid, r, V_WHITE, V_INK);
    let mut passes = 1usize;
    for _ in 0..MAX_STEPS {
        if !two_labels_share_a_region(&best, labels) {
            break;
        }
        r *= 2.0;
        let next = segment(grid, r, V_WHITE, V_INK);
        passes += 1;
        // A bola maior que a arte cai no fallback (papel inteiro) e volta a fundir tudo —
        // aí a anterior é a melhor resposta que existe.
        if next.count < best.count {
            break;
        }
        best = next;
    }
    if std::env::var("PH2D_COLORIZE_LOG").is_ok() {
        eprintln!(
            "[colorize] segment passes={passes} r={r:.1}px regions={}",
            best.count
        );
    }
    best
}

/// Existe alguma região reivindicada por DOIS rótulos diferentes?
fn two_labels_share_a_region(seg: &segment::Segmentation, labels: &[(u16, Vec<usize>)]) -> bool {
    let mut owner: Vec<Option<usize>> = vec![None; seg.count];
    for (k, (_, pixels)) in labels.iter().enumerate() {
        for &p in pixels {
            let r = seg.region[p];
            if r == NO_REGION {
                continue;
            }
            match owner[r as usize] {
                None => owner[r as usize] = Some(k),
                Some(j) if j != k => return true,
                Some(_) => {}
            }
        }
    }
    false
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
