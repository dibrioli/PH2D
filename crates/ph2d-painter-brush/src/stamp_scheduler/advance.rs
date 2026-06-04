//! `impl StampScheduler` (the advance/scheduling logic), split out of the
//! former `stamp_scheduler.rs` god-object (pure mechanical move).

use super::*;

impl StampScheduler {
    /// Construct com pool reservado para [`MAX_STAMPS_PER_DISPATCH`]
    /// (384 KB). One-shot alloc no construtor; HR-3 garantido por
    /// [`Self::advance`] daí em diante (`push` em Vec com headroom não
    /// re-aloca; [`Vec::clear`] preserva capacity).
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: Vec::with_capacity(MAX_STAMPS_PER_DISPATCH),
            last_point: None,
            residual_dist: 0.0,
            stroke_seed: 0,
            stamp_index: 0,
            stroke_rotation_base: None,
            last_follow_angle: None,
        }
    }

    /// Inicia um novo stroke. Reseta o estado interno mantendo a capacity do
    /// pool. `seed` deve ser derivado de inputs determinísticos pelo caller
    /// (tempo do pointer-down + entity bits + brush hash).
    ///
    /// **Audit T1.6 T-4:** `begin_stroke` is a HARD RESET — it clears
    /// state even if a prior stroke was open (e.g. a click-no-drag whose
    /// `end_stroke` got dropped). The `debug_assert!` fires in dev/test
    /// builds when the prior stroke wasn't ended, so bridge bugs
    /// (missed pointer-up event) surface immediately. Release builds
    /// skip the check; the reset is correct in either case.
    pub fn begin_stroke(&mut self, seed: u64) {
        debug_assert!(
            !self.is_in_stroke(),
            "begin_stroke called on non-ended stroke (prior end_stroke \
             or break_segment missing — bridge bug or dropped pointer-up); \
             state will be reset, but the previous stroke's stamps are \
             discarded silently in release"
        );
        self.pool.clear();
        self.last_point = None;
        self.residual_dist = 0.0;
        self.stroke_seed = seed;
        self.stamp_index = 0;
        self.stroke_rotation_base = None;
        self.last_follow_angle = None;
    }

    /// Finaliza o stroke atual. Limpa estado de continuação mas mantém
    /// `stroke_seed` para auditoria. Idempotente.
    pub fn end_stroke(&mut self) {
        self.pool.clear();
        self.last_point = None;
        self.residual_dist = 0.0;
        self.stroke_rotation_base = None;
        self.last_follow_angle = None;
    }

    /// "Brush lifted" — interrompe o segmento atual SEM encerrar o stroke
    /// (mantém `stroke_seed`, `stamp_index` counter, AND
    /// `stroke_rotation_base`). O próximo `advance` trata o sample como
    /// NOVO ponto inicial, igual ao primeiro stamp do stroke — sem
    /// interpolar uma linha reta entre o último ponto antes do "lift" e o
    /// novo ponto após o "drop".
    ///
    /// Caso de uso: cursor sai do footprint do sprite mid-drag e re-entra
    /// noutro local. Sem `break_segment`, `advance` interpola stamps ao
    /// longo do gap (smear visual). Audit T1.5 round 3 R3-LE-1.
    pub fn break_segment(&mut self) {
        self.pool.clear();
        self.last_point = None;
        // `residual_dist` deve voltar a 0 também — o gap não consome
        // espaçamento; a próxima chamada começa fresca como se fosse a
        // primeira do stroke.
        self.residual_dist = 0.0;
        // `stroke_rotation_base` SURVIVES break_segment — same stroke
        // (pointer never lifted in the user-model sense; cursor merely
        // crossed the sprite footprint boundary). Re-entry produces
        // visually-continuous rotation pattern. Spec
        // `docs/Painter_projeto/01_brush_engine.md` §1.3.4
        // (`shape_randomized`) cross-references this behavior
        // (audit T1.6 Q-5).
    }

    /// Verdadeiro se um stroke já tem PELO MENOS UM stamp emitido (i.e. já
    /// houve uma chamada `advance` que produziu um stamp). Após `begin_stroke`
    /// mas antes do primeiro `advance` retorna `false` — "stroke armado mas
    /// ainda vazio". Após `end_stroke` retorna `false`.
    ///
    /// Usado pelo bridge pra decidir se um pointer-up encerra um stroke
    /// válido (que vira commit) ou se foi click vazio (no-op).
    #[must_use]
    pub fn is_in_stroke(&self) -> bool {
        self.last_point.is_some()
    }

    /// Avança o stroke até `sample`, emitindo todos os stamps que cabem no
    /// segmento `last_point → sample.position` conforme `brush.stroke_path`
    /// e `size_px` (diameter efetivo do stamp em pixels — caller computa a
    /// partir de pressure curve etc.; T-input integrado em T1.6).
    ///
    /// Retorna um slice estável até a próxima chamada (`Vec::clear` na
    /// próxima invocação invalida o slice — caller deve consumir antes).
    ///
    /// `color_oklab` é a cor STRAIGHT-alpha (L, a, b, α) que vai dentro de
    /// cada [`Stamp`]; T1.6 aplica Color Dynamics stamp-level jitter
    /// (hue/saturation/lightness/darkness) sobre essa cor antes de gravar
    /// cada stamp.
    ///
    /// ## T1.6 multi-stamp emission
    ///
    /// Each spacing step along the segment expands into
    /// `brush.shape.shape_count` stamps (1..=16, with `shape_count_jitter`
    /// perturbation) stacked at the same world position. Each stamp in the
    /// group has a distinct `rotation_rad` (composed from `shape_randomized`
    /// stroke base + `shape_rotation_follow` direction + `shape_scatter`
    /// per-stamp jitter) and a distinct color (per-stamp Color Dynamics
    /// jitter). The pool cap [`MAX_STAMPS_PER_DISPATCH`] is respected — if
    /// the cap fires mid-group, the partial group is committed and the
    /// `advance` returns early.
    ///
    /// ## Estabilidade do slice
    ///
    /// O caller é responsável por consumir o slice ANTES de chamar `advance`
    /// novamente — a próxima invocação faz `pool.clear()` e regrava do zero.
    /// Caller que precisa persistir os stamps copia o slice (Stamp é `Copy`).
    pub fn advance(
        &mut self,
        brush: &Brush,
        sample: PointerSample,
        size_px: f32,
        color_oklab: [f32; 4],
    ) -> &[Stamp] {
        self.pool.clear();

        // Saneamento de entrada — pontos com componente não-finito viram no-op.
        // Defesa antes do GPU pipeline (que também filtra) para evitar
        // poluir o ring buffer com lixo.
        if !sample.position[0].is_finite()
            || !sample.position[1].is_finite()
            || !sample.pressure.is_finite()
        {
            return &self.pool;
        }

        // Diameter efetivo clampado ao limite ABI do Stamp. Caller passou
        // tamanho derivado de slider+pressure; aqui só impomos o teto.
        let diameter = size_px.clamp(1.0, MAX_STAMP_SIZE_PX as f32);
        let spacing_frac = brush.stroke_path.spacing.clamp(0.01, 1.0);
        // `spacing_px` = `spacing_frac * diameter`. Lower bound 1.0 evita
        // divisão por zero em strokes de stamp tamanho mínimo + spacing < 1
        // (audit-edge: spacing 0.01 * diameter 1.0 = 0.01 → infinite loop
        // without lower bound).
        let spacing_px = (spacing_frac * diameter).max(1.0);

        // **R4-LG-5 fix:** hoist brush-param clamps ABOVE the while-loop.
        // `brush: &Brush` is read-only for the duration of this advance;
        // these values are constant per stroke — re-evaluating them per
        // stamp wastes ~7 cycles each iteration (LLVM can't prove
        // invariance through the &Brush reference).
        let jitter_amp = brush.stroke_path.spacing_jitter.clamp(0.0, 1.0);
        let lat_amp = brush.stroke_path.jitter_lateral.clamp(0.0, 1.0);

        // Lazy-init `stroke_rotation_base` on the first advance under a
        // brush with `shape_randomized=true`. det_random with axis 0xCC
        // (distinct from spacing 0xA1, lateral 0xB2, scatter 0xCD,
        // color* 0xC1..0xC4). Once set, survives `break_segment`.
        if brush.shape.shape_randomized && self.stroke_rotation_base.is_none() {
            let r = self.det_random(0, 0xCC) * core::f32::consts::TAU;
            self.stroke_rotation_base = Some(r);
        }

        match self.last_point {
            None => {
                // Primeiro pointer do stroke: deposita o group de stamps na
                // posição (sem stroke direction ainda → rotation_follow
                // contribui 0 nesse step). Single-pointer click = single
                // group emit.
                self.push_stamp_group(brush, sample, diameter, color_oklab, [1.0, 0.0]);
                self.last_point = Some(sample.position);
                self.residual_dist = 0.0;
                return &self.pool;
            }
            Some(last) => {
                let delta = [sample.position[0] - last[0], sample.position[1] - last[1]];
                let segment_len = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
                // **Audit T1.6 T-12:** finite-overflow guard. The is_finite
                // filter at advance entry catches NaN/Inf positions, but
                // `[f32::MAX, 0]` to `[0, 0]` produces `delta = [-MAX, 0]`
                // (finite), then `(-MAX)² = +Inf` overflow → `sqrt(Inf) =
                // Inf` → `inv_len = 1/Inf = 0` → `ux/uy = -MAX * 0 = NaN`
                // (0 × Inf is NaN per IEEE). Without this guard, the
                // pool fills with 4096 NaN-position stamps that the GPU/
                // CPU paths reject downstream — wasted work + corrupted
                // residual_dist on the next advance.
                if !segment_len.is_finite() {
                    return &self.pool;
                }
                if segment_len < f32::EPSILON {
                    // Pointer ficou parado (jitter de driver / sample
                    // duplicado). Não emite stamp novo nem move last_point.
                    return &self.pool;
                }
                // Unit vector + perpendicular para jitter_lateral.
                let inv_len = 1.0 / segment_len;
                let ux = delta[0] * inv_len;
                let uy = delta[1] * inv_len;
                let perp = [-uy, ux];
                let stroke_dir = [ux, uy];

                // Walk: `cursor` é a distância acumulada ao longo do segmento
                // a partir de `last`. Começa em `(spacing_px - residual_dist)`:
                // a contribuição residual do segmento anterior + a nova
                // distância forma o primeiro spacing inteiro neste segmento.
                //
                // Audit T1.5 round 1 A-H3: usar `rem_euclid` no residual
                // final garante invariante `residual ∈ [0, spacing_px)`
                // independente de quantos segmentos curtos foram
                // descartados em sequência.
                let cursor_initial = spacing_px - self.residual_dist;
                let mut cursor = cursor_initial;
                while cursor <= segment_len && self.pool.len() < MAX_STAMPS_PER_DISPATCH {
                    // Spacing jitter — variação aleatória multiplicativa do
                    // intervalo até o próximo stamp. `spacing_jitter` em
                    // [0, 1]; jitter em [-J, +J] frações do spacing.
                    //
                    // **Audit T1.6 P-4:** axis 0xA1 binds to `stamp_index`
                    // — same channel as color jitter. This means the
                    // spacing-jitter sequence ADVANCES once per emitted
                    // stamp, not once per spacing step. Consequence: two
                    // strokes that differ only in `shape_count` produce
                    // different spacing-jitter sequences (group of 4
                    // burns 4 PRNG steps before the next spacing-jitter
                    // draw). Documented behavior, not bug — `shape_count`
                    // is part of the brush identity and intentionally
                    // produces a distinct stroke fingerprint.
                    let j_offset = if jitter_amp > 0.0 {
                        let j = self.det_random(self.stamp_index, 0xA1) * 2.0 - 1.0;
                        j * jitter_amp * spacing_px
                    } else {
                        0.0
                    };
                    let t_along = (cursor + j_offset).clamp(0.0, segment_len);

                    // Jitter lateral — deslocamento perpendicular ao stroke
                    // direction. `jitter_lateral` em [0, 1]; offset em
                    // [-L, +L] frações do diameter.
                    let lat_offset = if lat_amp > 0.0 {
                        let l = self.det_random(self.stamp_index, 0xB2) * 2.0 - 1.0;
                        l * lat_amp * diameter
                    } else {
                        0.0
                    };

                    let pos = [
                        last[0] + ux * t_along + perp[0] * lat_offset,
                        last[1] + uy * t_along + perp[1] * lat_offset,
                    ];
                    let interp_sample = PointerSample {
                        position: pos,
                        pressure: sample.pressure,
                        tilt: sample.tilt,
                    };
                    self.push_stamp_group(brush, interp_sample, diameter, color_oklab, stroke_dir);

                    cursor += spacing_px;
                }
                // Residual = posição efetiva mod spacing_px. Trata
                // uniformemente os 3 cenários:
                // 1. Stamps emitted → `consumed = last_cursor_that_fired
                //    = cursor - spacing_px` cai em `[0, segment_len)`;
                //    residual = `segment_len - consumed` cai em
                //    `(0, spacing_px]`, e o `rem_euclid` normaliza
                //    para `[0, spacing_px)`.
                // 2. Zero stamps (segment curto) → `consumed = -
                //    residual_dist_prev`, então
                //    `segment_len - consumed = segment_len +
                //    residual_dist_prev`, que pode ser > spacing_px;
                //    `rem_euclid` traz de volta para `[0, spacing_px)`.
                // 3. Stamp count cap atingido → cursor parou antes de
                //    consumir todo segmento; mesma fórmula vale.
                let consumed = cursor - spacing_px;
                let raw_residual = segment_len - consumed;
                self.residual_dist = raw_residual.rem_euclid(spacing_px);
                self.last_point = Some(sample.position);
            }
        }
        &self.pool
    }

    /// Emit a group of `shape_count` stamps at the same world position,
    /// each with its own rotation + color jitter. The group's effective
    /// count is `brush.shape.shape_count + count_jitter`, clamped to
    /// `[1, MAX_SHAPE_COUNT]`. Stops early if the pool cap fires
    /// mid-group (audit T1.6 — partial group is fine; the next advance
    /// continues from the new `stamp_index`).
    ///
    /// `stroke_dir` is a unit vector along the segment direction, used
    /// when `shape_rotation_follow=true`. For the very first stamp of a
    /// stroke (no direction yet), pass `[1.0, 0.0]` — that's the canonical
    /// "no direction" sentinel matching `atan2(0, 1) = 0`.
    pub(crate) fn push_stamp_group(
        &mut self,
        brush: &Brush,
        sample: PointerSample,
        diameter: f32,
        color_oklab: [f32; 4],
        stroke_dir: [f32; 2],
    ) {
        // Hoist per-group brush params.
        let shape_layer = match &brush.shape.shape_source {
            crate::shape::ShapeSource::Builtin { atlas_layer, .. } => *atlas_layer,
            crate::shape::ShapeSource::Imported { atlas_layer, .. } => *atlas_layer,
        };
        // **Audit T1.6 Z-4:** hoist the radial-symmetry check above the
        // per-stamp loop. For radial shapes (default `round_hard`, also
        // `round_soft`), `rotated_footprint_scale` short-circuits to
        // 1.0 regardless of rotation_rad — no need to call it per stamp.
        // Saves ~7 cycles × `effective_count` for the radial-default
        // brushes (most common case).
        let is_radial = crate::library::shape_is_radial_symmetric(shape_layer);
        let scatter_rad = brush.shape.shape_scatter.clamp(0.0, 360.0).to_radians();
        // **Audit T1.6 R7 K1-10 — continuous-angle unwrap.** `atan2`
        // returns values in `(-π, π]`; a stroke that crosses the ±π
        // discontinuity (any path crossing the negative x-axis, or a
        // U-turn near the +x-axis) jumps by ±2π between consecutive
        // pointer samples. Add or subtract whole turns so the new
        // angle lands within `[prev - π, prev + π]`. Invisible for
        // radial shapes (rotation is a no-op); for `oval_hard` and
        // future asymmetric shapes it kills the 180°/360° "snap".
        // Reset across `begin_stroke`/`end_stroke`; preserved across
        // `break_segment` (continuous user-intent stroke).
        //
        // **Audit T1.6 R8 Q1-1 — boundary edge case.** At `diff = π`
        // exactly, `round(π / TAU) = round(0.5) = 1.0` (Rust
        // `f32::round` is round-half-away-from-zero), so unwrap moves
        // the result to `prev - π` (left bound). The range is
        // `[prev - π, prev + π)` (left-inclusive). For continuity
        // analysis this is irrelevant — `prev ± π` are antipodal on
        // the circle, and either choice produces the same visible
        // orientation up to sign convention.
        //
        // **Audit T1.6 R8 M1-1 — NaN guard.** Defensive: if `raw` or
        // `unwrapped` is non-finite (theoretically impossible because
        // `atan2` on a finite stroke_dir is bounded, and the advance
        // entry filter rejects non-finite positions), DON'T poison
        // `last_follow_angle` with NaN — that would propagate NaN
        // through every subsequent stamp of the stroke. Fall back to
        // the raw value (still NaN-safe-by-skip downstream because the
        // per-stamp `rotation_rad.is_finite()` guard inside
        // `push_one_stamp` drops the bad stamp), and keep
        // `last_follow_angle` at its prior finite state so the next
        // valid sample's unwrap reference isn't permanently corrupted.
        let follow_angle = if brush.shape.shape_rotation_follow {
            let raw = stroke_dir[1].atan2(stroke_dir[0]);
            let unwrapped = match self.last_follow_angle {
                None => raw,
                Some(prev) => {
                    let diff = raw - prev;
                    let turns = (diff / std::f32::consts::TAU).round();
                    raw - turns * std::f32::consts::TAU
                }
            };
            if unwrapped.is_finite() {
                self.last_follow_angle = Some(unwrapped);
            }
            unwrapped
        } else {
            0.0
        };
        let base_rotation = self.stroke_rotation_base.unwrap_or(0.0);
        // **Audit T1.6 Z-8:** hoist a single bool for "any color jitter
        // active". For default brushes (all jitter = 0), we skip the
        // 4× clamp + 4× comparison inside `apply_stamp_color_jitter`.
        let cd_has_any_jitter = brush.color_dynamics.stamp_lightness_jitter > 0.0
            || brush.color_dynamics.stamp_darkness_jitter > 0.0
            || brush.color_dynamics.stamp_hue_jitter > 0.0
            || brush.color_dynamics.stamp_saturation_jitter > 0.0;
        // **Audit T1.6 A1-Z8-SKIP-INTENT-EROSION:** the
        // `stamp_secondary_color` + `stamp_secondary_amount` reads
        // (P-1 intentional-field-access signal) used to live inside
        // `apply_stamp_color_jitter`. After Z-8 hoisting, calling that
        // function is skipped for jitter-free brushes (the common
        // case), erasing the read for grep-based "who reads these
        // fields" searches. Pull the reads up to per-group scope so
        // they ALWAYS execute once regardless of cd_has_any_jitter —
        // 2-cycle cost, eliminates the false "field is forgotten"
        // signal a future T-color-full implementer would see.
        let _ = brush.color_dynamics.stamp_secondary_color;
        let _ = brush.color_dynamics.stamp_secondary_amount;

        // **Audit T1.6 Q-11:** flag bits are per-GROUP (constant across
        // all N stamps in this `push_stamp_group` invocation). The
        // current spec scope (`shape_flip_x`, `shape_flip_y`) is
        // brush-level. A future per-stamp randomized-flip feature would
        // require moving this computation INSIDE the stamp loop with a
        // fresh axis_tag (e.g. 0xCF / 0xD0) on `self.stamp_index`.
        let mut flags = 0u32;
        if brush.shape.shape_flip_x {
            flags |= FLAG_SHAPE_FLIP_X;
        }
        if brush.shape.shape_flip_y {
            flags |= FLAG_SHAPE_FLIP_Y;
        }

        // `shape_count` group size, with `shape_count_jitter` perturbation.
        // Jitter axis 0xCE (distinct from scatter 0xCD). Jitter in [0,1]
        // additive: `n_effective = round(n + jitter[-1,+1] * n_jitter * n)`.
        //
        // **Audit T1.6 Q-1 — boundary-clamp bias:** at `base_count` near
        // either edge of `[1, MAX_SHAPE_COUNT]`, the saturating clamp
        // skews the distribution. For `base = MAX` + `jitter = 1.0`,
        // perturbed values in `[1, MAX]` are uniform but the upper-half
        // `(MAX, 2·MAX]` ALL collapse to MAX — so `P(effective = MAX)`
        // ≈ 0.5 (mass-biased toward base). Symmetric bias at `base = 1`.
        // Documented behavior: the jitter expresses a "shrink-or-grow
        // intent" with hard-cap saturation, not an unbiased perturbation
        // distribution. Spec §1.3.4 should cross-reference.
        let base_count = brush.shape.shape_count.clamp(1, MAX_SHAPE_COUNT);
        let count_jitter_amp = brush.shape.shape_count_jitter.clamp(0.0, 1.0);
        let effective_count = if count_jitter_amp > 0.0 {
            let j = self.det_random(self.stamp_index, 0xCE) * 2.0 - 1.0;
            let perturbed =
                (base_count as f32 + j * count_jitter_amp * base_count as f32).round() as i32;
            perturbed.clamp(1, MAX_SHAPE_COUNT as i32) as u32
        } else {
            base_count
        };

        for _ in 0..effective_count {
            if self.pool.len() >= MAX_STAMPS_PER_DISPATCH {
                return;
            }
            // Per-stamp rotation: stroke base + follow angle + scatter
            // jitter (independent per stamp via stamp_index advance).
            let scatter_offset = if scatter_rad > 0.0 {
                (self.det_random(self.stamp_index, 0xCD) * 2.0 - 1.0) * scatter_rad
            } else {
                0.0
            };
            let rotation_rad = base_rotation + follow_angle + scatter_offset;

            // **Audit T1.6 Q-8 — NaN guard.** All three components above
            // are constructed from finite inputs (atan2 of unit vector is
            // finite, det_random returns `[0, 1)`, base_rotation is from
            // `det_random * TAU`). A NaN here would propagate to the
            // GPU's `cos/sin` (impl-defined on Vulkan/Metal/D3D12) and
            // break HR-5 cross-OS. Debug-asserting here pins the
            // invariant; if a future input path can produce NaN, the
            // assert fires loudly instead of corrupting the canvas.
            debug_assert!(
                rotation_rad.is_finite(),
                "rotation_rad must be finite (NaN propagates to GPU cos/sin and breaks HR-5)"
            );

            // Per-stamp color jitter (OKLab L offset + (a,b) rotate/scale).
            // Distinct axis tags per channel (0xC1..0xC4) so a brush with
            // only `stamp_hue_jitter` set doesn't disturb the lightness
            // axis's PRNG stream. Z-8: skip the entire call when no
            // channel is active (saves 4 clamps + 4 comparisons per stamp).
            let color = if cd_has_any_jitter {
                self.apply_stamp_color_jitter(color_oklab, &brush.color_dynamics, self.stamp_index)
            } else {
                color_oklab
            };

            // Footprint enlargement: non-radial shape with rotation needs
            // an enlarged bounding box so the rotated shape isn't clipped.
            //
            // **Audit T1.6 Z-4 + Q-6:** for radial shapes, short-circuit
            // to scale=1.0 without the cos/sin call (hoisted `is_radial`
            // above). For non-radial, the tight bound `|cos θ| + |sin θ|`
            // peaks at √2 at 45°. When `diameter` is already at
            // `MAX_STAMP_SIZE_PX = 2048`, the multiplied result clips
            // back to 2048 → rotated square's corners GET CLIPPED.
            // Effective practical upper limit for non-radial rotated
            // brushes is `MAX / √2 ≈ 1448 px`. UI-side soft-cap is W2+
            // follow-up.
            let footprint_scale = if is_radial {
                1.0
            } else {
                rotation_rad.cos().abs() + rotation_rad.sin().abs()
            };
            let size_px = (diameter * footprint_scale).min(MAX_STAMP_SIZE_PX as f32);

            self.push_one_stamp(
                brush,
                sample,
                size_px,
                rotation_rad,
                color,
                shape_layer,
                flags,
            );
        }
    }

    /// Single-stamp write into the pool. Owns the `Stamp::zeroed()` init,
    /// the field population, and the `stamp_index` advance. Hot path —
    /// stays small + branch-free where possible. All parameters are
    /// primitives or `Copy` refs; the `too_many_arguments` allow avoids
    /// a wrapper struct that the optimizer would have to unwrap anyway
    /// (HR-3 zero-overhead invariant).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn push_one_stamp(
        &mut self,
        brush: &Brush,
        sample: PointerSample,
        size_px: f32,
        rotation_rad: f32,
        color_oklab: [f32; 4],
        shape_layer: u32,
        flags: u32,
    ) {
        // **Audit T1.6 U-10:** defense-in-depth — size_px must be
        // positive finite at the push boundary. `advance` clamps
        // `diameter` to `[1.0, MAX_STAMP_SIZE_PX]`, then `push_stamp_
        // group` multiplies by `rotated_footprint_scale` (NaN-safe — NaN
        // rotation_rad falls through to NaN scale → NaN size_px). The
        // existing `debug_assert!(rotation_rad.is_finite())` (Q-8) plus
        // this guard catch the entire non-finite stamp surface before
        // it hits the Stamp ABI. Release builds skip (zero cost); the
        // shader / CPU `apply_one_stamp` are still defense-in-depth at
        // the dispatch boundary.
        debug_assert!(
            size_px.is_finite() && size_px > 0.0,
            "push_one_stamp size_px must be positive finite; got {size_px}"
        );
        let len_before = self.pool.len();
        let mut s = Stamp::zeroed();
        s.position_world = sample.position;
        s.size_px = size_px;
        s.rotation_rad = rotation_rad;
        s.pressure = sample.pressure.clamp(0.0, 1.0);
        s.tilt = sample.tilt.clamp(0.0, std::f32::consts::FRAC_PI_2);
        s.azimuth = 0.0; // T-input (ADR-0050)
        s.barrel_roll = 0.0; // T-input (ADR-0050)
        s.color_oklab = color_oklab; // STRAIGHT alpha (shader premultiplies)
        s.opacity = 1.0; // T1.7 — taper opacity + stroke-level opacity
        s.flow = brush.rendering.flow.clamp(0.0, 1.0);
        s.wet_amount = 0.0; // T-wet-mix W7+
        s.shape_layer = shape_layer;
        s.grain_layer = u32::MAX; // sem grain ainda — T-grain W5+
        s.grain_offset_uv = [0.0, 0.0];
        s.grain_scale = 1.0;
        s.flags = flags;
        s.rendering_mode = brush.rendering.rendering_mode as u32;
        s.pigment_mode = brush.rendering.pigment_mode as u32;
        self.pool.push(s);
        debug_assert_eq!(
            self.pool.len(),
            len_before + 1,
            "push_one_stamp must add exactly one stamp before incrementing stamp_index (HR-5)"
        );
        self.stamp_index = self.stamp_index.wrapping_add(1);
    }

    /// Apply per-stamp Color Dynamics jitter to `color_oklab`. Returns the
    /// perturbed `[L, a, b, alpha]`. Spec §1.3.8 stamp-level slots:
    /// - `stamp_lightness_jitter`: bidirectional L offset.
    /// - `stamp_darkness_jitter`: monotonic-down L offset (composes after
    ///   lightness so the user can have BOTH set without one zeroing the
    ///   other — darkness pulls down from wherever lightness landed).
    /// - `stamp_hue_jitter`: rotates `(a, b)` by an angle proportional to
    ///   jitter (full 360° at jitter=1.0).
    /// - `stamp_saturation_jitter`: multiplies `(a, b)` by `(1 ± jitter)`.
    ///
    /// **HR-5 determinism:** each channel uses a distinct `axis_tag`
    /// (`0xC1..0xC4`) so toggling one jitter slot doesn't shift the PRNG
    /// stream of the other slots. Alpha is preserved (color dynamics
    /// don't touch alpha — `flow` and `opacity` are the alpha modulators).
    pub(crate) fn apply_stamp_color_jitter(
        &self,
        color_oklab: [f32; 4],
        cd: &crate::color_dynamics::ColorDynamicsParams,
        stamp_index: u64,
    ) -> [f32; 4] {
        // **Audit T1.6 P-1 + R-4:** `stamp_secondary_*` slots are
        // reserved for T-color-full (ADR-0051) and have no effect in
        // T1.6. An earlier T1.6 draft used `debug_assert!` here to flag
        // the silent-no-op, but that would panic in debug builds when a
        // Procreate-imported brush (§1.9.2) legitimately has the field
        // set — UX-hostile. The field docs in `color_dynamics.rs` carry
        // the deferral warning in rustdoc; future T-color-full ship
        // wires the read without touching this assert. Explicit `_ = …`
        // reads keep the field-access intentional (so a future implementer
        // sees the field is "known reserved", not "forgotten").
        let _ = cd.stamp_secondary_color;
        let _ = cd.stamp_secondary_amount;

        // **Audit T1.6 T-1 + T-2 + U-5:** symmetric finite-guard with
        // the position/pressure filter at advance entry. NaN/Inf color
        // would propagate through `l.clamp/.cos/.sin` and reach the
        // Stamp.color_oklab field, then poison the shader's
        // `oklab_to_linear_srgb` and CPU `apply_one_stamp` premul math.
        // Return input unchanged on non-finite (downstream's
        // `apply_one_stamp` is_finite filter then drops the stamp). The
        // `debug_assert!` fires loudly in dev/test builds so the
        // CALLER (PainterTool) gets immediate feedback on a bad
        // PainterParams.primary_color.
        if !color_oklab.iter().all(|c| c.is_finite()) {
            debug_assert!(
                false,
                "apply_stamp_color_jitter received non-finite color_oklab \
                 {:?} (caller should filter at queue_pointer — audit T1.6 T-1)",
                color_oklab
            );
            // Audit T1.6 A1-T1: previous draft returned `color_oklab`
            // unchanged on non-finite — but that propagates NaN/Inf
            // through 6 layers (push_one_stamp → Stamp.color_oklab →
            // GPU oklab_to_linear_srgb → clamp(NaN)=NaN → render
            // mode → final per-pixel NaN guard catches it). Cleaner:
            // return alpha=0 stamp, which the downstream
            // `combined_alpha < 1/255` short-circuit drops at the
            // first pixel — no NaN propagation.
            return [0.0, 0.0, 0.0, 0.0];
        }

        let [mut l, mut a, mut b, alpha] = color_oklab;

        // **Composition order rationale (audit T1.6 P-8):** the spec
        // §1.3.8 defines `stamp_lightness_jitter` (bidirectional) and
        // `stamp_darkness_jitter` (monotonic-down) as independent
        // channels. T1.6 applies them in order: LIGHTNESS first, then
        // DARKNESS. When both > 0, the mean output L shifts down
        // (darkness pulls L further toward 0 after lightness has placed
        // it somewhere). This is the documented semantics — "darkness
        // applies on top of lightness" — and pins the test
        // `darkness_jitter_monotonic_down`. Spec §1.3.8 cross-references
        // this composition.

        // 1. Lightness offset (bidirectional).
        //
        // **Audit T1.6 R7 K1-4 — clamp-tail mass at extremes.** With
        // `base_L = 0.5` and `l_jit = 1.0`, the unclamped distribution
        // `L + j` is uniform on `[-0.5, +1.5]`; the `clamp(0, 1)` folds
        // the tails into the endpoints, producing
        //   P(L=0) = 25 %, P(L=1) = 25 %, P(L ∈ (0,1)) = 50 %.
        // The user sees a SPLOTCHY stroke with high-frequency
        // black/white speckles, NOT a "smooth gentle tonal variation".
        // **Documented behavior, not a bug** — this is the same
        // "saturating perturbation" semantic as `shape_count_jitter`
        // (Q-1, lines 513-521 above). The slider expresses a "vary
        // brightness intensely" intent with hard endpoint capture, not
        // an unbiased Gaussian-like perturbation. Spec §1.3.8 cross-
        // references this. UI guidance (W2 sidebar): label the upper
        // half of the slider "(may clamp at black/white)" or soft-cap
        // at 0.5 for the default brushes. Alternative distribution
        // shapes (triangular, beta) are a W6+ artistic-knob feature,
        // not a T1.6 invariant.
        let l_jit = cd.stamp_lightness_jitter.clamp(0.0, 1.0);
        if l_jit > 0.0 {
            let j = self.det_random(stamp_index, 0xC1) * 2.0 - 1.0;
            l = (l + j * l_jit).clamp(0.0, 1.0);
        }

        // 2. Darkness offset (monotonic down).
        let d_jit = cd.stamp_darkness_jitter.clamp(0.0, 1.0);
        if d_jit > 0.0 {
            let j = self.det_random(stamp_index, 0xC2); // [0, 1)
            l = (l - j * d_jit).clamp(0.0, 1.0);
        }

        // 3. Hue rotation of (a, b).
        // **Audit T1.6 P-2:** angle magnitude is `j * h_jit * π` (NOT
        // `* TAU`). With `j ∈ [-1, +1]` and `h_jit ∈ [0, 1]`, the angle
        // range at `h_jit = 1.0` is `[-π, +π]` — which already covers
        // ALL hues uniformly (the full 360° hue circle). Using `* TAU`
        // would make the slider's top half a dead zone (PRNG advances
        // but visual output saturates — full-rotation wraps to
        // identity). Procreate convention matches: slider max = ±180°
        // = full hue coverage. **Zero-chroma input** (`a = b = 0`)
        // survives all rotations as `(0, 0)` — gated by
        // `hue_jitter_preserves_zero_chroma`.
        let h_jit = cd.stamp_hue_jitter.clamp(0.0, 1.0);
        if h_jit > 0.0 {
            let j = self.det_random(stamp_index, 0xC3) * 2.0 - 1.0;
            let angle = j * h_jit * core::f32::consts::PI;
            let cos_h = angle.cos();
            let sin_h = angle.sin();
            let a_new = a * cos_h - b * sin_h;
            let b_new = a * sin_h + b * cos_h;
            a = a_new;
            b = b_new;
        }

        // 4. Saturation scale of (a, b). `.max(0.0)` avoids sign flip when
        // jitter pushes the multiplier negative.
        //
        // **Audit T1.6 R7 K1-5 — half-axis suppression at s_jit=1.0.**
        // With `s_jit = 1.0`, `scale = 1 + j` ∈ `[0, 2]` (after `.max(0)`
        // for `j = -1` corner). Half the stamps have `j < 0` →
        // `scale < 1` → reduced chroma; the other half have `j > 0` →
        // `scale > 1` → boosted chroma. The user's perceived stroke
        // looks WASHED OUT relative to the primary because: (a) the
        // boosted-chroma stamps may exit gamut and clamp, (b) the
        // human visual system weights low-chroma noise heavier than
        // high-chroma noise (Weber-Fechner). **Documented behavior**
        // matching the Lightness case (K1-4) — the slider is a "vary
        // intensely" knob with hard zero capture, not a centered
        // Gaussian. Spec §1.3.8 cross-references. UI guidance: soft-
        // cap default brushes at 0.6; W6+ may offer alternative
        // distribution shapes as an artistic-knob extension.
        let s_jit = cd.stamp_saturation_jitter.clamp(0.0, 1.0);
        if s_jit > 0.0 {
            let j = self.det_random(stamp_index, 0xC4) * 2.0 - 1.0;
            let scale = (1.0 + j * s_jit).max(0.0);
            a *= scale;
            b *= scale;
        }

        [l, a, b, alpha]
    }

    /// Method wrapper kept for ergonomics inside the scheduler — delegates
    /// to the free function [`det_random`]. **Audit T1.6 Z-5:** the
    /// free-function form is the canonical entry point (no `&self` borrow
    /// to confuse the optimizer's aliasing analysis on the hot path).
    #[inline]
    pub(crate) fn det_random(&self, stamp_index: u64, axis_tag: u64) -> f32 {
        det_random(self.stroke_seed, stamp_index, axis_tag)
    }
}
