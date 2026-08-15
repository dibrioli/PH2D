//! Seamless **Tiling** (wrap-around) painting — split from `paint.rs` for the workspace LOC cap.
//! When Tiling is on for an axis, a dab whose footprint crosses that sprite edge is replicated with a
//! copy shifted by ∓span, so a stroke crossing the border also paints the part that wraps onto the
//! opposite edge → the painted texture is seamless when the sprite is repeated as a tile.

use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

/// Replicate each dab across the wrapped sprite edges for the enabled `tiling` axes. The originals
/// are always included; an edge-crossing dab additionally gets its wrapped copy (up to 4 per dab at a
/// corner). Returns the expanded list. Called only when at least one axis tiles.
#[must_use]
pub(super) fn tiled_dabs(dabs: &[Dab], source_size: (u32, u32), tiling: [bool; 2]) -> Vec<Dab> {
    tiled_dabs_grouped(dabs, source_size, tiling).0
}

/// [`tiled_dabs`] + the **group map**: `groups[i]` is the index of the ORIGINAL dab that output `i` was
/// replicated from (copies of one dab are emitted consecutively).
///
/// **Why this exists (sweep 2026-07-12):** a dab's wrapped copies are not new dabs — they are the SAME
/// dab seen from the other side of the seam. Anything drawn PER DAB (Shape/Grain Random-Angle, Random
/// Offset, Randomize Color) must therefore be drawn ONCE and shared by the copies. The paint routes used
/// to iterate the already-wrapped list and pull from `tex_rng` per entry, so the two sides of the seam got
/// DIFFERENT random frames and the tile stopped matching — the one thing Tiling exists to guarantee.
/// Smear/Blur/Clone always got this right (they compute the frame once per dab and wrap inside).
/// Feed this map to [`DabRng`], which replays the stream per group.
#[must_use]
pub(super) fn tiled_dabs_grouped(
    dabs: &[Dab],
    source_size: (u32, u32),
    tiling: [bool; 2],
) -> (Vec<Dab>, Vec<u32>) {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let mut out = Vec::with_capacity(dabs.len() * 2);
    let mut groups = Vec::with_capacity(dabs.len() * 2);
    for (gi, d) in dabs.iter().enumerate() {
        let mut xs = [0.0f32; 3];
        let nx = axis_offsets(d.center[0], d.radius_px, fw, tiling[0], &mut xs);
        let mut ys = [0.0f32; 3];
        let ny = axis_offsets(d.center[1], d.radius_px, fh, tiling[1], &mut ys);
        for &dx in &xs[..nx] {
            for &dy in &ys[..ny] {
                out.push(Dab {
                    center: [d.center[0] + dx, d.center[1] + dy],
                    ..*d
                });
                groups.push(gi as u32);
            }
        }
    }
    (out, groups)
}

/// The per-dab texture RNG, replayed so that a dab's wrapped Tiling copies share ONE random frame.
///
/// Call [`Self::enter`] once per entry of the dab list, in order, BEFORE any draw for that entry. The
/// first copy of a group draws normally; every further copy re-runs the stream from the group's start, so
/// it makes the IDENTICAL draws. Because all copies of a dab draw the same amount, the stream ends up
/// advanced exactly once per ORIGINAL dab — which is also what makes the un-tiled path **byte-identical**
/// (with no group map every entry is its own group, so `enter` never rewinds anything).
pub(super) struct DabRng {
    rng: u64,
    group_start: u64,
    cur: u32,
}

impl DabRng {
    pub(super) fn new(rng: u64) -> Self {
        Self {
            rng,
            group_start: rng,
            cur: u32::MAX,
        }
    }

    /// Position the stream for list entry `i`, given the group map (empty ⇒ every entry its own group).
    pub(super) fn enter(&mut self, groups: &[u32], i: usize) -> &mut u64 {
        let g = groups.get(i).copied().unwrap_or(i as u32);
        if g == self.cur {
            self.rng = self.group_start; // a wrapped copy: replay its original's draws
        } else {
            self.cur = g;
            self.group_start = self.rng; // a new dab: this is where its draws begin
        }
        &mut self.rng
    }

    /// The stream state to write back (advanced once per original dab).
    pub(super) fn finish(self) -> u64 {
        self.rng
    }
}

/// Fill `out` with the wrap offsets (`[dx, dy]`, always including `[0, 0]`) for a dab at
/// `center`/`radius` under the enabled `tiling` axes — the same edge-crossing logic as [`tiled_dabs`],
/// written into a stack buffer (no alloc) so the **Smear** route can apply one offset to BOTH its lift
/// (`from`) and stamp (`to`) positions (a wrapped copy keeps the same displacement). Returns the count
/// (`1` when interior; up to `nx·ny ≤ 9` for a dab wider than the sprite that crosses both edges).
pub(super) fn tiled_offsets_into(
    center: [f32; 2],
    radius: f32,
    source_size: (u32, u32),
    tiling: [bool; 2],
    out: &mut [[f32; 2]; 9],
) -> usize {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let mut xs = [0.0f32; 3];
    let nx = axis_offsets(center[0], radius, fw, tiling[0], &mut xs);
    let mut ys = [0.0f32; 3];
    let ny = axis_offsets(center[1], radius, fh, tiling[1], &mut ys);
    let mut n = 0;
    for &dx in &xs[..nx] {
        for &dy in &ys[..ny] {
            out[n] = [dx, dy];
            n += 1;
        }
    }
    n
}

/// Replica **laços fechados** (o `Style: Solid`) através das bordas envolvidas — a irmã de
/// [`tiled_dabs`], sobre a MESMA [`axis_offsets`].
///
/// ⚠️ **A régua de um laço é a CAIXA dele, e é isso que a torna a mesma pergunta:** um dab cruza a
/// borda quando `centro ± raio` a passa, e um laço quando a sua caixa a passa — então basta entregar
/// o centro e o meio-lado da caixa à função que já responde. Uma segunda regra de *"o que cruza a
/// costura"* divergiria da dos dabs no dia em que alguém afinasse uma delas, e a tinta do traço
/// apareceria do outro lado sem a mancha (ou o contrário).
///
/// ⚠️ **O SENTIDO de cada laço é preservado** (a cópia é uma translação, que não mexe no sinal da
/// área) — ver [`ph2d_painter_brush::symmetry::symmetric_loops`], onde a reflexão exige o oposto.
#[must_use]
pub(super) fn tiled_loops(
    loops: &[Vec<[f32; 2]>],
    source_size: (u32, u32),
    tiling: [bool; 2],
) -> Vec<Vec<[f32; 2]>> {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let mut out = Vec::with_capacity(loops.len() * 2);
    for lp in loops {
        let Some((c, r)) = bbox_centre_half(lp) else {
            continue; // laço vazio: não cerca área, não cruza costura nenhuma
        };
        let mut xs = [0.0f32; 3];
        let nx = axis_offsets(c[0], r[0], fw, tiling[0], &mut xs);
        let mut ys = [0.0f32; 3];
        let ny = axis_offsets(c[1], r[1], fh, tiling[1], &mut ys);
        for &dx in &xs[..nx] {
            for &dy in &ys[..ny] {
                out.push(lp.iter().map(|p| [p[0] + dx, p[1] + dy]).collect());
            }
        }
    }
    out
}

/// Replica os **FIOS** (Sketchy / Wire / as travessas da fita) através das bordas envolvidas.
///
/// ⚠️ **O meio-lado leva a espessura do fio**, senão um fio que corre RENTE à costura (caixa de
/// altura zero, mas tinta de `width_px` de largura) não seria replicado e a tile ficaria com meia
/// linha — o mesmo motivo pelo qual a régua de um dab é `centro ± raio` e não o centro sozinho.
#[must_use]
pub(super) fn tiled_threads(
    threads: &[[f32; 4]],
    width_px: f32,
    source_size: (u32, u32),
    tiling: [bool; 2],
) -> Vec<[f32; 4]> {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let half = 0.5 * width_px.max(0.0);
    let mut out = Vec::with_capacity(threads.len() * 2);
    for t in threads {
        let c = [0.5 * (t[0] + t[2]), 0.5 * (t[1] + t[3])];
        let r = [
            0.5 * (t[0] - t[2]).abs() + half,
            0.5 * (t[1] - t[3]).abs() + half,
        ];
        let mut xs = [0.0f32; 3];
        let nx = axis_offsets(c[0], r[0], fw, tiling[0], &mut xs);
        let mut ys = [0.0f32; 3];
        let ny = axis_offsets(c[1], r[1], fh, tiling[1], &mut ys);
        for &dx in &xs[..nx] {
            for &dy in &ys[..ny] {
                out.push([t[0] + dx, t[1] + dy, t[2] + dx, t[3] + dy]);
            }
        }
    }
    out
}

/// O centro e o meio-lado da caixa de um laço — o par que [`axis_offsets`] chama de `centre`/`radius`.
fn bbox_centre_half(lp: &[[f32; 2]]) -> Option<([f32; 2], [f32; 2])> {
    let first = *lp.first()?;
    let mut bb = [first[0], first[1], first[0], first[1]];
    for p in lp {
        bb[0] = bb[0].min(p[0]);
        bb[1] = bb[1].min(p[1]);
        bb[2] = bb[2].max(p[0]);
        bb[3] = bb[3].max(p[1]);
    }
    Some((
        [0.5 * (bb[0] + bb[2]), 0.5 * (bb[1] + bb[3])],
        [0.5 * (bb[2] - bb[0]), 0.5 * (bb[3] - bb[1])],
    ))
}

/// Wrap offsets for one axis of a tiled dab: always `0`, plus `∓span` when the dab's footprint
/// (`centre ± radius`) crosses an enabled edge — so an edge dab also paints the wrapped copy. Writes
/// up to 3 offsets into `out` and returns the count (`1` when tiling is off or the dab is interior).
fn axis_offsets(centre: f32, radius: f32, span: f32, tiling: bool, out: &mut [f32; 3]) -> usize {
    out[0] = 0.0;
    let mut n = 1;
    if tiling {
        if centre + radius > span {
            out[n] = -span; // crosses the far edge → wrap to the near edge
            n += 1;
        }
        if centre - radius < 0.0 {
            out[n] = span; // crosses the near edge → wrap to the far edge
            n += 1;
        }
    }
    n
}

impl PainterTool {
    /// Toggle seamless **Tiling** (wrap-around painting) for `axis` (`0` = X, `1` = Y). Plain paint
    /// state — no undo / pixel touch (it only affects future stamps).
    pub fn toggle_brush_tiling(&mut self, axis: usize) {
        if axis < 2 {
            self.paint.tiling[axis] = !self.paint.tiling[axis];
        }
    }

    /// The current per-axis Tiling flags `[x, y]` — the panel reads this to show the toggle state.
    #[must_use]
    pub fn brush_tiling(&self) -> [bool; 2] {
        self.paint.tiling
    }

    /// Toggle **Repeat Image** — the on-canvas 3×3 tile preview (the shell draws the sprite repeated
    /// in all 8 neighbour directions). The shell reads [`Self::repeat_image`] each frame — and the
    /// toggle also flips WHICH producer owns the preview slot (the GPU refuses while the tile
    /// preview is on, `gpu_eligible`), so the composite is invalidated to make the incoming producer
    /// publish immediately. Without it the handoff waited for the next stroke: the painting VANISHED
    /// on the toggle and the tiles only appeared after the first dab (Enio 2026-07-20).
    pub fn toggle_repeat_image(&mut self) {
        self.paint.repeat_image = !self.paint.repeat_image;
        self.invalidate_composite();
    }

    /// Whether the Repeat-Image tile preview is on.
    #[must_use]
    pub fn repeat_image(&self) -> bool {
        self.paint.repeat_image
    }

    /// Reset the **Tiling** section to defaults (no wrap-around tiling, no Repeat-Image preview).
    pub fn reset_brush_tiling(&mut self) {
        self.paint.tiling = [false, false];
        self.paint.repeat_image = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um quadrado 100..140 numa tela de 128: ele passa da borda direita.
    fn crossing_loop() -> Vec<Vec<[f32; 2]>> {
        vec![vec![
            [100.0, 40.0],
            [140.0, 40.0],
            [140.0, 80.0],
            [100.0, 80.0],
        ]]
    }

    /// **DESLIGADO É A ENTRADA VERBATIM** — a rede que mantém o mundo sem Tiling byte-idêntico.
    #[test]
    fn loops_and_threads_are_untouched_with_tiling_off() {
        let lp = crossing_loop();
        assert_eq!(tiled_loops(&lp, (128, 128), [false, false]), lp);
        let th = [[10.0f32, 10.0, 120.0, 10.0]];
        assert_eq!(tiled_threads(&th, 2.0, (128, 128), [false, false]), th);
    }

    /// **UM LAÇO QUE CRUZA A COSTURA GANHA A CÓPIA DO OUTRO LADO** — e ela é uma TRANSLAÇÃO exata,
    /// que é o que mantém o sentido (e portanto o `nonzero`) intacto.
    #[test]
    fn a_loop_crossing_the_seam_is_replicated_by_the_span() {
        let out = tiled_loops(&crossing_loop(), (128, 128), [true, false]);
        assert_eq!(out.len(), 2, "o laço da borda tem de ganhar UMA cópia");
        for (a, b) in out[0].iter().zip(out[1].iter()) {
            assert!(
                (b[0] - (a[0] - 128.0)).abs() < 1e-3 && (b[1] - a[1]).abs() < 1e-3,
                "a cópia não é a translação de −128 em x: {a:?} -> {b:?}"
            );
        }
    }

    /// **A ESPESSURA DO FIO ENTRA NA RÉGUA** — um fio RENTE à costura (caixa de altura zero) só é
    /// replicado porque a tinta dele tem largura. Sem o meio-lado, a tile fica com meia linha.
    ///
    /// **Mutação que sangra:** tirar o `+ half` do meio-lado (o fio deixa de ser replicado).
    #[test]
    fn a_thread_grazing_the_seam_is_replicated_because_its_ink_has_width() {
        // Um fio horizontal a 1 px da borda de baixo (y = 127) de uma tela 128, com 6 px de largura.
        let th = [[10.0f32, 127.0, 60.0, 127.0]];
        let out = tiled_threads(&th, 6.0, (128, 128), [false, true]);
        assert_eq!(out.len(), 2, "o fio rente à costura tem de ganhar a cópia");
        assert!(
            (out[1][1] - (127.0 - 128.0)).abs() < 1e-3,
            "a cópia do fio não desceu um span: {:?}",
            out[1]
        );
    }
}
