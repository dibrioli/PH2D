//! **A ROTA DO DAB** — como um dab do Painter entra na água (filho de
//! [`super`], LOC cap).
//!
//! O corte é por responsabilidade: o pai guarda *o que o tool FAZ com uma
//! sessão* (o ciclo do traço, o tick, o guard de identidade), o `session.rs`
//! guarda *o que uma sessão É*, e aqui fica **a tradução de um dab do sistema
//! de pincel para o motor de fluido** — a única porta onde as duas linguagens
//! se encontram, e por isso a única que precisa saber que [`super::grid_map`]
//! existe.
//!
//! Três conversões vivem nesta porta, e as três TÊM de concordar: a POSIÇÃO e o
//! RAIO viram células (`px_to_cell` / `px_len_to_cell`), a SILHUETA é avaliada
//! de volta em pixels de canvas (`cell_subsample_px`, com o AA por cobertura) e
//! o PAPEL é semeado na lei de canvas. Se qualquer uma discordar, o carimbo sai
//! deslocado de meia célula e **nenhum número o denuncia** — só a tinta no lugar
//! errado.

use super::*;

impl PainterTool {
    /// The Wet Paint dab route ([`Self::stamp_dabs_inner`] arm). The dab list
    /// is already mirrored (Symmetry) and replicated (Tiling) — the engine
    /// sees exactly what the colour routes would have seen.
    pub(in crate::tool::paint) fn stamp_dabs_wetpaint(&mut self, dabs: &[Dab], brush: &BrushSpec) {
        let (w, h) = self.source_size;
        // Two doors in, one belt (doc 21): a LIVE incremental gesture (the
        // W1/W2 path — dabs are the artist's hand, once) or the commit
        // door's one-shot `deposit_pass` replay. Everything else is refused
        // — authoring batches are un-owned and can no longer even reach
        // here (routing sends them flat); this gate is the belt against a
        // routing regression, and it must stay a WALL, not a debug_assert.
        let live = self.paint.wetpaint.live_gesture && brush.stroke_method.is_incremental();
        if dabs.is_empty() || w == 0 || h == 0 || !(live || self.paint.wetpaint.deposit_pass) {
            return;
        }
        self.wetpaint_guard();
        // The wet ERASER (W2.6) erases the FLUID and only the fluid: with no
        // live session there is nothing wet, and the routing predicate
        // (`wet_owns_the_dabs`) already sent those dabs to the normal
        // eraser — this early-out only covers the guard killing the session
        // between the route decision and here (the batch then does nothing).
        let erasing = self.paint.eraser;
        if self.paint.wetpaint.session.is_none() {
            if erasing {
                return;
            }
            self.ensure_wet_session();
        }
        // Take the session out so the prep below can borrow `self.paint`'s
        // Shape state alongside it (disjoint in fact, not in the borrow
        // checker's eyes); restored before the composite.
        let mut taken = self.paint.wetpaint.session.take().expect("ensured above");
        let sess = &mut taken;
        // A PORTA: o motor pode estar com o worker desde o fim do último tick.
        sess.bring_home();
        // A razão da grade é a da SESSÃO, nunca a autorada: é ela que diz de
        // que tamanho o motor VIVO é. As duas concordam sempre (trocar a
        // razão encerra a sessão), e ler a que existe é o que mantém a
        // conversão honesta mesmo se essa porta ganhar um caminho novo.
        let ratio = sess.ratio;
        // ── The artist's PAPER drives the engine's tooth (W2.7) ────────────
        // RECONCILED per batch, not seeded once: paper is substrate under
        // live water, but the SLOT is authored state and the session spans
        // strokes — a key of everything the seed reads decides. Key moved =
        // re-seed; slot disarmed = the engine re-bakes its own preset
        // (`rebake_paper`); unchanged = one small struct compare. The law is
        // the painter's own canvas-fixed paper sampling (the NEUTRAL door
        // `sample_tiled_rot_wrapped`, the same law the watercolor substrate
        // reads — one paper, never a second system). Known v1 gap: the
        // bitmap Size seam-snap under sprite Tiling (`snap_slot_size`) is
        // not applied — procedurals wrap exactly; a bitmap paper may seam.
        let period = [
            if self.paint.tiling[0] { w as f32 } else { 0.0 },
            if self.paint.tiling[1] { h as f32 } else { 0.0 },
        ];
        let want = brush.paper.is_active().then_some(PaperKey {
            tex: brush.paper,
            image_version: self.paint.paper_image_version,
            period,
        });
        if sess.paper_key != want {
            if want.is_some() {
                let paper_tex = brush.paper;
                let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
                let rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
                // ⚠️ O `seed_paper_with` fala em CÉLULAS (ele passa `cell − 1`),
                // e o papel é uma lei de CANVAS — sob razão > 1 amostrá-lo na
                // coordenada de célula encolheria o dente do papel pelo mesmo
                // fator, e o papel deixaria de casar com o do resto do app.
                sess.engine.seed_paper_with(&mut |cx, cy| {
                    let px = grid_map::cell_center_texel(cx as i32 + 1, ratio);
                    let py = grid_map::cell_center_texel(cy as i32 + 1, ratio);
                    f64::from(ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                        &paper_tex,
                        px,
                        py,
                        paper_img.as_ref(),
                        rot,
                        period,
                    ))
                });
            } else {
                sess.engine.rebake_paper();
            }
            sess.paper_key = want;
        }
        // ── The authored facts — same reconcile law as the paper. ──────────
        sess.reconcile_facts(self.wet_facts());
        // The engine-side TOOL (doc 22): gives the lane doors their
        // `TrailMode` (begin picks Blend by it) and the sim its pause law
        // (`sim_should_run` keeps running under a Blow stroke, the model's
        // one exception). The ERASER always pauses — the engine erases with
        // its own tool untouched, exactly as before.
        let wet_tool = self.paint.wetpaint.tool;
        sess.engine.tool = if erasing {
            ph2d_wet_paint::painter::Tool::Paint
        } else {
            wet_tool.engine()
        };
        // ── The dab's SILHOUETTE — the painter's, not the engine's ─────────
        // `silhouette_at` is the single source of the dab's shape (falloff ×
        // Shape image/procedural × flatten/rotate footprint); the engine's
        // internal falloff/footprint step aside per dab via the shaped door,
        // and the bristle texture stays as the fluid's default grain (W2.3b).
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        // The artist's GRAIN replaces the engine's bristle texture (W2.4).
        // The bristle IS the fluid's default grain, so an armed Grain slot
        // takes its place outright — two textures multiplying would
        // double-darken, and everywhere else in the app the Grain is THE
        // texture. The per-pixel law is `dab::grain_at`, the same single
        // door the colour route and the impasto height kernel call.
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let grain_active = brush.texture.is_active();
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        let canvas_wh = [w as f32, h as f32];
        // The engine speaks sRGB **0..255** (its boot colour is `[50, 140, 210]` and the render
        // writes plane values straight through `clamp_u8`); the dab stores sRGB 0..1. Passing the
        // normalized value painted BLACK — Enio's W1 smoke, "a cor está preta".
        let ink = |c: [f32; 3]| {
            [
                f64::from(c[0]) * 255.0,
                f64::from(c[1]) * 255.0,
                f64::from(c[2]) * 255.0,
            ]
        };
        if !sess.stroke_open {
            sess.stroke_open = true;
            sess.lanes.clear();
            // The eraser and the DIRECT wet tools (Wet/Dry/Blow/Smear) are
            // LANE-LESS (per-dab grid ops, no trail) — open one direct
            // stroke so the sim gets its stroke gate as usual (paused under
            // the gesture; the Blow exception rides `engine.tool`).
            if (erasing || !wet_tool.uses_lanes())
                && let Some(d0) = dabs.first()
            {
                sess.engine.begin_direct_stroke(
                    0,
                    grid_map::px_to_cell(f64::from(d0.center[0]), ratio),
                    grid_map::px_to_cell(f64::from(d0.center[1]), ratio),
                );
            }
        }
        let strength = brush.strength.clamp(1e-3, 1.0);
        for (didx, d) in dabs.iter().enumerate() {
            let [x, y] = d.center;
            // LANE matching, geometric: the dab belongs to the lane whose
            // last position is within its own radius (consecutive dabs of one
            // copy are a spacing apart; other copies are far). A dab with no
            // lane in reach BEGINS one — a symmetry copy at stroke start, or
            // a Tiling wrap born mid-stroke at the sprite edge. Near a radial
            // centre the copies converge and may swap lanes; there their
            // positions coincide, so a swap deposits the same paint.
            // The ERASER skips the lanes entirely — every copy just erases
            // where it lands. The direct wet tools keep the geometric lane
            // MATCHING (it is what carries each symmetry/tiling copy's own
            // previous dab centre — the smear/blow displacement source) but
            // never touch the engine's lane trails.
            let mut lane_born = false;
            let li = if erasing {
                0
            } else {
                let thr = d.radius_px.max(4.0);
                let mut best = thr * thr;
                let mut lane = None;
                for (i, l) in sess.lanes.iter().enumerate() {
                    let (ddx, ddy) = (x - l.pos[0], y - l.pos[1]);
                    let d2 = ddx * ddx + ddy * ddy;
                    if d2 <= best {
                        best = d2;
                        lane = Some(i);
                    }
                }
                match lane {
                    Some(i) => {
                        if wet_tool.uses_lanes() {
                            let chord = grid_map::px_len_to_cell(f64::from(best.sqrt()), ratio);
                            sess.engine.direct_segment(i, chord);
                        }
                        i
                    }
                    None => {
                        let i = sess.lanes.len();
                        lane_born = true;
                        if wet_tool.uses_lanes() {
                            // The DAB's colour, not the brush's: Randomize
                            // is already resolved per dab by the stroke
                            // engine (W2.2).
                            sess.engine.color = ink(d.color);
                            sess.engine.begin_direct_stroke(
                                i,
                                grid_map::px_to_cell(f64::from(x), ratio),
                                grid_map::px_to_cell(f64::from(y), ratio),
                            );
                        }
                        sess.lanes.push(Lane {
                            pos: d.center,
                            ink: d.color,
                        });
                        i
                    }
                }
            };
            // The lane's PREVIOUS centre (before this dab updates it) — the
            // direct tools' displacement source; a born lane has none, and
            // the eraser has no lanes at all.
            let lane_prev: Option<[f32; 2]> = (!erasing && !lane_born).then(|| sess.lanes[li].pos);
            // Per-dab fresh ink (Randomize): reload the lane's trail — a
            // brush dipped in new paint (see `Trail::set_base_color`).
            if !erasing && wet_tool == WetTool::Paint && d.color != sess.lanes[li].ink {
                sess.engine.set_stroke_color(li, ink(d.color));
                sess.lanes[li].ink = d.color;
            }
            let b = ((d.coverage / strength).clamp(0.0, 1.0) as f64) * 10.0;
            // Per-dab silhouette closure — the impasto walk's exact recipe
            // (spec at the dab's radius → rotor → footprint → Shape basis in
            // the stroke frame), evaluated per engine cell (cell − 1 = px).
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            let rotor = spec.dab_rotor(d);
            let fp = spec.dab_footprint(rotor);
            let dab_index = didx;
            let tex_rng = dab_rng.enter(&groups, dab_index);
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &spec.shape,
                    &mut *tex_rng,
                    canvas_wh,
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let shape_input = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            // Grain basis AFTER the Shape basis, from the same RNG stream —
            // the colour route's resolve order (Shape before Grain), so the
            // random draws land where every other route puts them.
            let grain_basis = grain_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(&spec.texture, &mut *tex_rng, canvas_wh, fp)
            });
            let inv_r = 1.0 / d.radius_px.max(0.01);
            // ⚠️ A silhueta é avaliada em espaço de CANVAS, sempre — só
            // AMOSTRADA por célula. É por isso que a razão da grade não muda a
            // forma do pincel, apenas quão fino o fluido a resolve: o `t` sai
            // da distância em pixels ao centro do dab, e o Shape/Grain leem o
            // texel de canvas onde o sub-ponto caiu.
            //
            // ⚠️ **E ela é amostrada por COBERTURA, não no centro** — o AA do
            // depósito (`grid_map::cell_subsamples`): com a razão > 1 uma
            // decisão de um ponto por célula quantizava a borda do pincel em
            // degraus de `ratio` px, e nenhuma interpolação da SAÍDA recupera
            // isso. Em razão 1 são `n = 1` sub-pontos e o corpo abaixo é o de
            // sempre, ao bit.
            let (aa_n, aa_step) = grid_map::cell_subsamples(ratio);
            let aa_inv = 1.0 / f32::from(u16::from(aa_n) * u16::from(aa_n));
            let mut sil = |cx: i32, cy: i32| -> f64 {
                let mut acc = 0.0f32;
                for j in 0..aa_n {
                    let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step);
                    for i in 0..aa_n {
                        let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step);
                        let t =
                            fp.falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r);
                        acc += ph2d_painter_brush::dab::silhouette_at(
                            &spec,
                            shape_input,
                            t,
                            sx.floor() as i64,
                            sy.floor() as i64,
                            d.center,
                            d.radius_px,
                        );
                    }
                }
                f64::from(acc * aa_inv)
            };
            // Per-dab grain closure (armed slot only): replaces the bristle
            // sample inside the shaped stamp, cell − 1 = px like `sil`.
            let spec_ref = &spec;
            let gimg = grain_image.as_ref();
            let mut grain = grain_basis.as_ref().map(|gb| {
                move |cx: i32, cy: i32| -> f64 {
                    let px = grid_map::cell_center_texel(cx, ratio);
                    let py = grid_map::cell_center_texel(cy, ratio);
                    f64::from(ph2d_painter_brush::dab::grain_at(
                        spec_ref,
                        gb,
                        gimg,
                        px,
                        py,
                        d.center,
                        d.radius_px,
                    ))
                }
            });
            let grain_arg = grain.as_mut().map(|g| g as &mut dyn FnMut(i32, i32) -> f64);
            // O ponto e o raio, em CÉLULAS — as duas coisas que o motor mede
            // na sua própria grade. O `d.radius_px` continua sendo o raio da
            // silhueta (em pixels), e é assim que o pincel não muda de tamanho
            // quando a razão muda: o mesmo disco, resolvido mais grosso.
            let cell_x = grid_map::px_to_cell(f64::from(x), ratio);
            let cell_y = grid_map::px_to_cell(f64::from(y), ratio);
            let cell_r = grid_map::px_len_to_cell(f64::from(d.radius_px), ratio);
            if erasing {
                // W2.6: the same §9 mapping, routed to the engine's ERASER —
                // silhouette and grain shape the erase exactly as they shape
                // the deposit.
                sess.engine.dispatch_pressure_dab_erase(
                    cell_x,
                    cell_y,
                    b,
                    f64::from(d.dir[0]),
                    f64::from(d.dir[1]),
                    cell_r,
                    Some(&mut sil),
                    grain_arg,
                );
                continue;
            }
            match wet_tool {
                WetTool::Paint => sess.engine.dispatch_pressure_dab_lane(
                    li,
                    cell_x,
                    cell_y,
                    b,
                    f64::from(d.dir[0]),
                    f64::from(d.dir[1]),
                    cell_r,
                    Some(&mut sil),
                    grain_arg,
                ),
                // Blend keeps the engine's own fixed-hardness stamp (the
                // model's tools ignore the brush silhouette on purpose).
                WetTool::Blend => sess.engine.dispatch_pressure_dab_lane_blend(
                    li,
                    cell_x,
                    cell_y,
                    b,
                    f64::from(d.dir[0]),
                    f64::from(d.dir[1]),
                    cell_r,
                ),
                // Direct tools: per-dab grid ops; the displacement source is
                // THIS lane's previous centre (host-tracked per copy).
                WetTool::Wet | WetTool::Dry | WetTool::Blow | WetTool::Smear => {
                    sess.engine.dispatch_pressure_dab_tool(
                        wet_tool.engine(),
                        cell_x,
                        cell_y,
                        b,
                        f64::from(d.dir[0]),
                        f64::from(d.dir[1]),
                        cell_r,
                        lane_prev.map(|p| {
                            [
                                grid_map::px_to_cell(f64::from(p[0]), ratio),
                                grid_map::px_to_cell(f64::from(p[1]), ratio),
                            ]
                        }),
                    );
                }
            }
            sess.lanes[li].pos = d.center;
        }
        self.paint.wetpaint.session = Some(taken);
        self.wetpaint_composite();
    }
}
