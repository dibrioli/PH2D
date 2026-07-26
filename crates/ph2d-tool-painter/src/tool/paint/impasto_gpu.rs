//! **Impasto** — handing the light pass to the GPU.
//!
//! The light ([`super::impasto_light`]) is the last thing standing between a sculpted canvas and the GPU
//! compositor: `gpu_eligible` bails the moment `impasto_visible()`, so a document with relief composites
//! its whole layer stack on the CPU. This module is the seam that lets the GPU shade instead.
//!
//! ## What crosses, and what deliberately does NOT
//!
//! The CPU pass is two things wearing one coat: the **plumbing** (which layers, in which order, folded
//! how — [`super::impasto_light::ReliefFields`]) and the **optics** (a normal, four lamps, a BRDF —
//! [`super::impasto_shade::Rig`]). Only the optics port. The plumbing runs here, once, and hands the GPU
//! three finished planes:
//!
//! | plane | what it already is |
//! |---|---|
//! | `relief` | depth-scaled, `Add`/`Level`-folded bottom-up, live stroke merged, **ceiling applied** |
//! | `cover`  | the `max` over layers, live stroke merged |
//! | `mat0/1` | the `over` fold from `Material::NEUTRAL`, quantised to the canvas's 7 bytes |
//!
//! That split is the whole risk budget of the port. A shader that re-derived the fold would be a second
//! answer to *"how do layers of paint stack"* — and two doors to one question diverge
//! ([[feedback_two_doors_to_the_same_question_diverge]]), silently, in the one place nobody can read: a
//! screenshot. So the fold has exactly one implementation, it is the one the CPU pass has always used,
//! and this module calls it rather than mirroring it. What the shader re-implements is bounded, pure,
//! and pinned by a parity gate against the very function it ports.
//!
//! ## Why the planes are canvas-sized
//!
//! The normal is a central difference, so the shader reads a texel's NEIGHBOURS — and
//! [`super::impasto_light::ReliefFields::height_at`] clamps to the **canvas**, not to the lit region, so
//! a dirty-rect update lights its border exactly as a full recompose would. Uploading only the region
//! would clamp at the region's edge instead, and every partial update would draw a seam along a rectangle
//! nobody could explain. Canvas-sized planes make the shader's `clamp` the canvas clamp, by construction.

use super::impasto_light::ReliefFields;
use crate::tool::PainterTool;
use rayon::prelude::*;

/// One resolved lamp, in the form the shader consumes: direction, half-vector, and `intensity × colour`
/// already multiplied together.
///
/// Resolved on the CPU on purpose. Building `dir` from the artist's whole-degree azimuth/elevation goes
/// through [`ph2d_painter_brush::texture::rotate_by_degrees`] — the shared 1°-step rotor that keeps the
/// pass transcendental-free (HR-5) and that the brush's Jitter Rotate is built from. A shader computing
/// `sin`/`cos` itself would be a second rotor, disagreeing in the last bits with the one the rest of the
/// app turns by.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImpastoLamp {
    /// Unit light direction; `z > 0` points out of the canvas.
    pub dir: [f32; 3],
    /// Half-vector between the light and the orthographic view direction `(0, 0, 1)`.
    pub half: [f32; 3],
    /// `intensity × colour` — weight and hue never appear apart.
    pub tint: [f32; 3],
}

/// The composed relief, materialised canvas-wide for the GPU light pass.
///
/// Built fresh on each GPU-preview frame rather than cached behind a version key, and that is a
/// deliberate v1 choice: a version would have to track every input the fold reads — the height, cover
/// and material planes of every layer, each layer's `impasto_depth` and `impasto_composite`, group
/// visibility, the live stroke, the brush's material — and **the failure mode of missing one is a stale
/// light nobody can see is stale**. The GPU preview only recomposites on `take_preview_dirty()`, so this
/// runs on the frames the CPU path would have spent compositing the entire stack anyway. If it ever
/// shows up in a profile, the fix is a version key, and it should be introduced with a gate that proves
/// it invalidates on each of those inputs.
pub struct ImpastoPlanes {
    pub width: u32,
    pub height: u32,
    /// **Where in the canvas these buffers go** — `(x, y, w, h)`. The planes are a WINDOW, not the
    /// canvas: [`PainterTool::impasto_gpu_planes_in`] folds only this rect, and the pass writes only
    /// this rect into its persistent plane textures.
    ///
    /// `width`/`height` above stay the CANVAS, because they are what the shader clamps its central
    /// difference to. A window that renamed itself the canvas would move the clamp to the window's edge
    /// and draw a seam along a rectangle nobody could explain — the reason the planes were canvas-sized
    /// in the first place. Persisting the textures keeps that clamp while letting the fold shrink.
    pub region: (u32, u32, u32, u32),
    /// Composed height, **post-ceiling** — literally what `height_at` returns.
    pub relief: Vec<f32>,
    /// Composed coverage (`R8Unorm`): the shader's `byte / 255` is the CPU's `f32::from(c) / 255.0`,
    /// exactly.
    pub cover: Vec<u8>,
    /// `[shine, roughness, metallic, wax]` — the first 4 of the canvas's 7 material bytes.
    pub mat0: Vec<u8>,
    /// `[wax_r, wax_g, wax_b, 255]` — the last 3, padded to a texel. Seven bytes do not fit a GPU
    /// format; four and four do, and the split falls exactly on the boundary between the scalar
    /// properties and the Wax filter's colour.
    pub mat1: Vec<u8>,
    /// The lit lamps. Empty is impossible — [`PainterTool::impasto_gpu_planes`] returns `None` instead,
    /// which is the same bail the CPU pass makes when every lamp is off (the canvas comes back unlit, to
    /// the byte, rather than dividing by a zero flat response).
    pub lamps: Vec<ImpastoLamp>,
    /// The specular table the shader indexes, row-major `rough_levels x lut_width`.
    ///
    /// Carried here rather than looked up by the caller, so the question *"which table does the shader
    /// read"* is answered by the crate that owns the material model. A shell reaching into
    /// `ph2d_painter_brush::material` for itself would be a second place that has to be right about the
    /// table's shape, and it would go on compiling after the shape changed.
    ///
    /// `'static` because the table is baked ONCE per process: it is a pure function of nothing but
    /// itself. Uploading these exact floats is what keeps `powf` — the one transcendental in the model —
    /// off the device entirely, so it cannot diverge between the two paths at all.
    pub spec_lut: &'static [f32],
    pub lut_width: u32,
    pub rough_levels: u32,
}

impl PainterTool {
    /// Materialise everything the GPU light pass needs, or `None` when the pass has nothing to do.
    ///
    /// The `None` arms are the CPU pass's own bails, in the same order and for the same reasons
    /// ([`PainterTool::apply_impasto_light`]): the pass is switched off or nothing carries relief; the
    /// canvas is empty; every lamp is off. Keeping them aligned is what makes "the GPU path and the CPU
    /// path agree about when there is no light" a property rather than a coincidence.
    #[must_use]
    pub fn impasto_gpu_planes(&self) -> Option<ImpastoPlanes> {
        let (w, h) = self.source_size;
        self.impasto_gpu_planes_in((0, 0, w, h))
    }

    /// The same fold, over a WINDOW — the door every caller actually goes through.
    ///
    /// # Why this exists
    ///
    /// The full-canvas fold was measured at **202 ms per frame at 4096²** (`measure_the_impasto_fold`),
    /// and the GPU preview runs it on every dirty frame, which during a stroke means every pointer move.
    /// Dissected (`measure_what_the_fold_is_made_of`): 0,15 ms of allocation and **180 ms of per-texel
    /// walk** — so the cost is the number of texels and nothing else, and the same walk over a 512² rect
    /// costs **2,82 ms at BOTH canvas sizes**. Folding the rect that changed instead of the canvas that
    /// did not is therefore a ~64× cut at 4K, with no change to what any texel says.
    ///
    /// ⚠️ **A window is not always available, and that is where the artist waits.** The pass refuses a
    /// partial upload until its plane textures have held the whole painting once
    /// ([`ph2d_render::ImpastoLightPass::planes_seeded`]), so the **first lit frame folds the canvas** no
    /// matter how small the stroke was. Measured in the running app with `PH2D_PAINT_PERF`, that frame
    /// was **232,7 ms at 4096², 100% of it inside `preview`** — the "delay do primeiro traço" Enio
    /// reported across three rounds, of which this fold was **201,5 ms**.
    ///
    /// So the walk is **parallel by rows** now (`measure_the_fold_the_product_runs`, the same door the
    /// product calls):
    ///
    /// | canvas | serial | parallel |
    /// |---|---|---|
    /// | 2048² | 45,29 ms | **4,53** |
    /// | 4096² | **201,53 ms** | **14,55** |
    ///
    /// The window path gains the same way (2,85 → 0,38 ms), which matters because it is the steady state
    /// of every stroke frame after the first.
    ///
    /// # What makes a partial fold SOUND
    ///
    /// The pass's plane textures persist, so texels outside `region` keep what the last upload put
    /// there. That is correct exactly when the composed relief outside `region` did not change — and that
    /// is not a hope, it is the invariant the CPU lane's partial recompose already runs on:
    /// `invalidate_composite` drops `dirty_rect` to `None` for every structural or metadata edit
    /// (opacity, blend, visibility, reorder, add, select, `impasto_depth`), so the rect the GPU lane
    /// stashes ([`PainterTool::preview_gpu_region`]) is `Some` **only** when the change was confined to
    /// it. A caller with no confined rect asks for the whole canvas and gets the old behaviour, to the
    /// byte.
    ///
    /// The window is clamped to the canvas rather than refused: a rect that pokes over the edge is a
    /// caller being generous about what changed, which is safe, where refusing would drop a frame's light.
    #[must_use]
    pub fn impasto_gpu_planes_in(&self, region: (u32, u32, u32, u32)) -> Option<ImpastoPlanes> {
        if !self.impasto_visible() {
            return None;
        }
        let fields = self.impasto_fields()?;
        let lamps = super::impasto_shade::Rig::new(&self.paint.impasto_rig)?.export_lamps();
        let (w, h) = self.source_size;
        if (w as usize) * (h as usize) == 0 {
            return None;
        }
        let (rx, ry, rw, rh) = region;
        let rx = rx.min(w);
        let ry = ry.min(h);
        let rw = rw.min(w - rx);
        let rh = rh.min(h - ry);
        let n = (rw as usize) * (rh as usize);
        if n == 0 {
            return None;
        }
        let mut relief = vec![0f32; n];
        let mut cover = vec![0u8; n];
        let mut mat0 = vec![0u8; n * 4];
        let mut mat1 = vec![0u8; n * 4];
        // The samplers are the light's OWN, asked in the same order as before, so a texel inside the
        // window carries the identical bytes a full fold would have given it. That is what makes
        // "partial" a statement about how MUCH is folded and never about WHAT it says.
        //
        // **By ROWS, in parallel** (ADR-0109's exception, the same shape `sculpt_offset` and
        // `watercolor_field` already run on): every texel here is a pure function of `(x, y)` and the
        // frozen `fields`, so rows are disjoint and the result is **byte-identical by construction** —
        // what changes is which thread evaluates which row, never what any row says.
        //
        // ⚠️ The cure is aimed at the FULL fold, which is the one the artist waits on. Measured at 4096²
        // with 32 cores: the walk is the entire cost (0,1 ms of allocation against 183 ms of walk), and
        // it is the biggest single number a Painter frame can produce.
        let rww = rw as usize;
        relief
            .par_chunks_mut(rww)
            .zip(cover.par_chunks_mut(rww))
            .zip(mat0.par_chunks_mut(rww * 4))
            .zip(mat1.par_chunks_mut(rww * 4))
            .enumerate()
            .for_each(|(row, (((rrow, crow), m0row), m1row))| {
                let y = i64::from(ry) + row as i64;
                for i in 0..rww {
                    let x = i64::from(rx) + i as i64;
                    rrow[i] = fields.height_at(x, y);
                    // Quantised the way the canvas already stores coverage, so the shader's unorm decode
                    // lands on the CPU's `f32::from(u8) / 255.0` and not merely near it.
                    crow[i] = (fields.cover_at(x, y) * 255.0 + 0.5) as u8;
                    let m = fields.material_at(x, y);
                    m0row[i * 4..i * 4 + 4].copy_from_slice(&[m[0], m[1], m[2], m[3]]);
                    m1row[i * 4..i * 4 + 4].copy_from_slice(&[m[4], m[5], m[6], 255]);
                }
            });
        let lut = ph2d_painter_brush::material::SpecLut::get();
        Some(ImpastoPlanes {
            width: w,
            height: h,
            region: (rx, ry, rw, rh),
            relief,
            cover,
            mat0,
            mat1,
            lamps,
            spec_lut: lut.table(),
            lut_width: ph2d_painter_brush::material::SPEC_LUT as u32,
            rough_levels: ph2d_painter_brush::material::ROUGH_LEVELS as u32,
        })
    }
}

impl ReliefFields<'_> {
    /// The coverage the light weights by, as the byte the GPU plane carries. Split out so the
    /// materialiser above and the round-trip gate below quantise through one expression.
    #[cfg(test)]
    pub(super) fn cover_byte(&self, x: i64, y: i64) -> u8 {
        (self.cover_at(x, y) * 255.0 + 0.5) as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    use ph2d_painter_brush::{BrushSpec, Falloff};

    const SIZE: u32 = 48;

    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    /// A canvas with one sculpted stroke on it — the fixture every gate here needs, because the
    /// interesting arithmetic only exists where there IS paint.
    fn sculpted() -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        let b = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.3],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.5,
            impasto_smoothing: 0.0,
            impasto_body: 1.0,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([14.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Up));
        t
    }

    /// The planes ARE the fold, texel for texel. Not "close to" — the materialiser must call the light's
    /// own sampler, so a drift here means somebody re-derived the fold and the GPU is about to shade a
    /// different painting than the CPU does.
    #[test]
    fn the_planes_are_the_light_s_own_fold_texel_for_texel() {
        let t = sculpted();
        let p = t
            .impasto_gpu_planes()
            .expect("a sculpted canvas has planes");
        let f = t.impasto_fields().expect("…and fields to fold");
        let n = (SIZE * SIZE) as usize;
        assert_eq!(p.relief.len(), n, "one relief texel per canvas pixel");
        let mut touched = 0usize;
        for y in 0..i64::from(SIZE) {
            for x in 0..i64::from(SIZE) {
                let i = (y as usize) * (SIZE as usize) + (x as usize);
                assert_eq!(
                    p.relief[i],
                    f.height_at(x, y),
                    "relief must be height_at, to the float, at ({x}, {y})"
                );
                assert_eq!(p.cover[i], f.cover_byte(x, y), "cover at ({x}, {y})");
                let m = f.material_at(x, y);
                assert_eq!(&p.mat0[i * 4..i * 4 + 4], &[m[0], m[1], m[2], m[3]]);
                assert_eq!(&p.mat1[i * 4..i * 4 + 3], &[m[4], m[5], m[6]]);
                if p.relief[i] != 0.0 {
                    touched += 1;
                }
            }
        }
        // The fixture has to CONTAIN the phenomenon: a canvas of zeros would pass every line above
        // while proving nothing about a fold ([[feedback_a_fixture_only_proves_what_it_contains]]).
        assert!(
            touched > 200,
            "the fixture must actually carry relief (only {touched} texels do)"
        );
    }

    /// **A window says the same thing the canvas said, inside it.** The whole safety of the partial fold
    /// is that "partial" is a claim about how MUCH is folded and never about WHAT it says: the pass keeps
    /// the texels outside the window from the previous upload, so a window that disagreed with the full
    /// fold would paint a rectangle of a different painting into the middle of this one.
    ///
    /// Compared against the FULL door's own output rather than against `height_at` again — the full door
    /// is what shipped and what every existing gate pins, so it is the oracle a regression has to break.
    ///
    /// ⚠️ **And that is also this gate's blind spot, measured:** door-against-itself cannot see an error
    /// that shifts BOTH sides the same way. Mutating the row-to-`y` mapping of the parallel fold by one
    /// row leaves this gate GREEN — the window and the canvas move together, so they still agree — and
    /// only [`the_planes_are_the_light_s_own_fold_texel_for_texel`], which asks the SAMPLERS, goes red.
    /// The two gates are not redundant: one pins *the window equals the canvas*, the other pins *the
    /// canvas equals the fold*, and a uniform indexing bug satisfies the first.
    #[test]
    fn a_window_folds_exactly_what_the_whole_canvas_folded_there() {
        let t = sculpted();
        let whole = t
            .impasto_gpu_planes()
            .expect("a sculpted canvas has planes");
        // A window that CONTAINS the stroke — a rect over bare paper would agree trivially, since two
        // zero-filled buffers match whatever the fold does ([[feedback_a_fixture_only_proves_what_it_contains]]).
        let win = (10u32, 16u32, 28u32, 16u32);
        let part = t
            .impasto_gpu_planes_in(win)
            .expect("…and so does a window of it");
        assert_eq!(part.region, win, "the window reports where it goes");
        assert_eq!(
            (part.width, part.height),
            (whole.width, whole.height),
            "the CANVAS dims are unchanged — they are what the shader clamps to"
        );
        let (rx, ry, rw, rh) = win;
        let mut carried = 0usize;
        for j in 0..rh as usize {
            for i in 0..rw as usize {
                let p = j * (rw as usize) + i;
                let c = (ry as usize + j) * (SIZE as usize) + (rx as usize + i);
                assert_eq!(part.relief[p], whole.relief[c], "relief at ({i}, {j})");
                assert_eq!(part.cover[p], whole.cover[c], "cover at ({i}, {j})");
                assert_eq!(&part.mat0[p * 4..p * 4 + 4], &whole.mat0[c * 4..c * 4 + 4]);
                assert_eq!(&part.mat1[p * 4..p * 4 + 4], &whole.mat1[c * 4..c * 4 + 4]);
                if part.relief[p] != 0.0 {
                    carried += 1;
                }
            }
        }
        assert!(
            carried > 100,
            "the WINDOW must contain relief, or this compares two empty buffers (only {carried} texels do)"
        );
    }

    /// **The fold is bounded by the WINDOW, not by the canvas** — the property whose absence cost 202 ms
    /// a frame at 4096², and the one no gate in this module could see.
    ///
    /// A RATIO, not a wall clock: the same window folded on two canvases must cost the same, because the
    /// work is per texel and the window has the same texels either way. A ratio is also immune to machine
    /// drift and to the profile the suite happens to build in, which a millisecond bar is not.
    ///
    /// It earns its keep against exactly one failure mode the SHAPE gates cannot see: a fold that
    /// *returns* a window while *walking* the canvas (compute everything, then slice). `part.region`
    /// and `relief.len()` would both still be right.
    ///
    /// **Mutation that must bleed:** point the shell's fold back at `impasto_gpu_planes()` (or make
    /// `impasto_gpu_planes_in` ignore its argument) and this quadruples with the canvas.
    ///
    /// ⚠️ **The window is 512², and the size is load-bearing.** It used to be 128², which cost ~0,18 ms
    /// when the fold walked serially and **0,044 ms** once it walked by rows — and a ratio between two
    /// numbers that small is a measurement of rayon's scheduler, not of this property. It failed exactly
    /// that way under the full suite (0,0839 against 0,2470, "2,95×") while the swept table showed the
    /// window cost flat across canvas size to within 10%. A gate whose oracle dissolves when the thing it
    /// watches gets faster is a gate that will be silenced rather than believed, so it moved up to a
    /// window that still costs ~0,39 ms — ten times the noise floor — and reads the MINIMUM of its
    /// samples, because a loaded machine can only ever make a sample slower.
    #[test]
    fn the_fold_costs_what_the_window_costs_not_what_the_canvas_costs() {
        /// Fastest wall clock of the same window folded on a canvas of `size`.
        fn window_ms(size: u32) -> f64 {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            let b = BrushSpec {
                radius_px: 24.0,
                hardness: 1.0,
                falloff: Falloff::Constant,
                color: [0.1, 0.2, 0.3],
                space_attenuation: false,
                impasto: true,
                impasto_depth: 0.5,
                impasto_smoothing: 0.0,
                impasto_body: 1.0,
                ..Default::default()
            };
            t.paint.brush = b;
            for slot in &mut t.paint.brush_by_mode {
                *slot = b;
            }
            let mid = (size / 2) as f32;
            t.on_canvas_pointer(cp([40.0, mid], PointerPhase::Down));
            t.on_canvas_pointer(cp([160.0, mid], PointerPhase::Move));
            t.on_canvas_pointer(cp([160.0, mid], PointerPhase::Up));
            // Contains the stroke (x from 16 to 184) — a window over bare paper agrees trivially,
            // because two cheap early-outs cost the same whatever the canvas is.
            let win = (0u32, (size / 2) - 256, 512, 512);
            let mut s = Vec::new();
            for _ in 0..9 {
                let t0 = std::time::Instant::now();
                let p = t.impasto_gpu_planes_in(win).expect("sculpted and lit");
                s.push(t0.elapsed().as_secs_f64() * 1e3);
                drop(p);
            }
            s.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            s[0]
        }
        let small = window_ms(1024);
        let large = window_ms(2048);
        // 4x the canvas, the same window. Anything that walks the plane shows up as ~4x here; the bar
        // sits well below that and well above the noise of two sub-millisecond samples.
        let ratio = large / small.max(1e-6);
        assert!(
            ratio < 2.0,
            "the fold must be bounded by the window: 1024²={small:.4} ms vs 2048²={large:.4} ms \
             ({ratio:.2}x — a canvas-bound fold quadruples)"
        );
    }

    /// **E o walk do fold é PARALELO — a única coisa que segura os 13,8×.**
    ///
    /// ⚠️ **Este gate existe porque nenhum outro pode pegar a regressão.** O irmão de RAZÃO acima não a
    /// vê: um fold serial também é limitado pela janela, então a razão dele continua ~1 e ele fica verde
    /// enquanto o primeiro frame com relevo volta de 14,55 para 201,5 ms. Os gates de FORMA também não —
    /// serial e paralelo produzem os mesmos bytes, que é precisamente a propriedade que torna a cura
    /// segura. Trocar `par_chunks_mut` por `chunks_mut` num refactor é **uma letra**, e a suíte inteira
    /// ficaria verde sobre a regressão que motivou a wave.
    ///
    /// ⚠️ **E é ARQUITETURAL de propósito, não por preguiça.** "Este laço roda em paralelo" é uma
    /// afirmação sobre a FORMA do código, não sobre um resultado observável: um gate de comportamento
    /// mediria wall-clock, e o `ci-test` compila em `opt-level=1`, então uma barra de milissegundos
    /// mediria o PERFIL do build e não o produto — o mesmo raciocínio que o ADR-0124 já registra. O
    /// número vive na sonda (`measure_the_fold_the_product_runs`); o que este gate guarda é que o
    /// mecanismo que o produziu continua lá.
    ///
    /// **Mutação que deve sangrar:** `par_chunks_mut` → `chunks_mut` no walk.
    #[test]
    fn the_fold_walks_in_parallel_because_the_rows_are_disjoint() {
        let src = include_str!("impasto_gpu.rs");
        // Controle positivo: a função tem de ser ENCONTRADA, senão o gate passa por não achar nada — a
        // falha que o arch-gate do Shape Flow pegou em si mesmo.
        let at = src
            .find("pub fn impasto_gpu_planes_in(")
            .expect("controle: a porta do fold tem de existir");
        let body = &src[at..];
        let end = body
            .find("\n    }\n")
            .expect("controle: a funcao tem de terminar");
        let body = &body[..end];
        // Os QUATRO planos, porque um `zip` serial no meio de três paralelos serializa o conjunto todo:
        // `par_chunks_mut(...).zip(cover.chunks_mut(...))` nem compila como `ParallelIterator`, mas a
        // forma que compila e regride é trocar TODOS, e contar é o que distingue "três de quatro".
        let par = body.matches("par_chunks_mut(").count();
        assert_eq!(
            par, 4,
            "o walk do fold tem de percorrer os QUATRO planos em paralelo (achei {par}) — as linhas sao \
             disjuntas (ADR-0109) e este e o mecanismo dos 201,5 -> 14,55 ms a 4096². Nenhum gate de \
             razao ou de bytes pega esta regressao: serial e paralelo dao os MESMOS bytes e os dois sao \
             limitados pela janela"
        );
    }

    /// **Trocar de sprite ESQUECE o retângulo sujo da pista GPU.**
    ///
    /// Achado da auditoria adversarial (2026-07-25), CONFIRMADO por medição a 96²: `preview_dirty_region`
    /// é o retângulo que o fold usa para decidir quanto re-dobrar, e ele sobrevivia a um rebind. Então o
    /// 1º dab de escultura no sprite NOVO dobrava só a pegada dele — 8% do quadro — e a luz sombreava os
    /// outros 92% a partir das texturas de plano do sprite ANTIGO: contaminação entre DOCUMENTOS.
    ///
    /// Os quatro campos irmãos (`dirty_rect`, `preview_upload_bbox`, `composited`, os 3 planos) já eram
    /// zerados aqui, sob um comentário que os chama de *"state whose shape is tied to a document that is
    /// no longer bound"*. Este é da mesma espécie e foi esquecido quando o fold lhe deu um SEGUNDO leitor.
    ///
    /// **Mutação que deve sangrar:** tirar `preview_dirty_region = None` do `set_source`.
    #[test]
    fn binding_a_new_sprite_forgets_the_old_ones_dirty_region() {
        let mut t = sculpted();
        // A pista GPU drena o flag SEM compositar, e é isso que move o rect para o stash que o fold lê.
        assert!(
            t.take_preview_dirty(),
            "precondição: o traço sujou o preview"
        );
        assert!(
            t.preview_gpu_region().is_some(),
            "precondição: a pista GPU tem um retângulo do sprite ANTIGO em mãos"
        );
        // O artista troca de sprite.
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        assert_eq!(
            t.preview_gpu_region(),
            None,
            "o retângulo do documento anterior não pode sobreviver ao rebind — o fold o leria como \
             'só isto mudou' e a luz sombraria o resto do quadro com o relevo do sprite antigo"
        );
    }

    /// The GPU seam bails exactly where the CPU pass does. If these ever disagree the shell would upload
    /// planes for a pass that will not run, or (far worse) skip the upload for one that will.
    #[test]
    fn the_planes_bail_wherever_the_cpu_pass_bails() {
        let mut t = sculpted();
        assert!(
            t.impasto_gpu_planes().is_some(),
            "precondition: lit + sculpted"
        );
        t.paint.impasto_show = false;
        assert!(
            t.impasto_gpu_planes().is_none(),
            "the pass is switched off — the CPU one returns immediately, so must this"
        );
        t.paint.impasto_show = true;
        for l in &mut t.paint.impasto_rig.lights {
            l.on = false;
        }
        assert!(
            t.impasto_gpu_planes().is_none(),
            "every lamp off — the CPU pass leaves the canvas unlit to the byte, it does not shade black"
        );
        let bare = PainterTool::default();
        assert!(
            bare.impasto_gpu_planes().is_none(),
            "no canvas, no planes (and no zero-sized upload)"
        );
    }
}
