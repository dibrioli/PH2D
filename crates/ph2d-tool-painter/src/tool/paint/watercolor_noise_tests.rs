//! Gates e medições do ruído canvas-anchored da aquarela — irmão de `watercolor_noise.rs`,
//! separado dele pelo teto de LOC do workspace (HR-18). Fica como `mod tests` FILHO daquele módulo
//! (`#[path]`), então `use super::*` continua alcançando os itens privados que ele testa.

use super::*;

/// **A fatoração dos dois eixos do warp é BYTE-EXATA** — e o oráculo é a formulação ANTIGA.
///
/// [`warp_offset`] deixou de chamar [`warp_axis`] duas vezes e passou a compartilhar a aritmética de
/// grade entre os dois eixos ([`value_noise_pair`]). O `warp_axis` **continua no arquivo, agora
/// `#[cfg(test)]`**, e é verbatim o que o produto fazia antes — por isso é o oráculo certo: não é um
/// espelho que eu escrevi para o teste, é a REFERÊNCIA CONGELADA do código que shipava. ⚠️ O `cfg` não
/// é higiene de warning: um `pub(super)` sem chamador de produção é uma segunda resposta a *"qual é o
/// deslocamento do warp aqui?"*, esperando alguém chamá-la e divergir.
///
/// ⚠️ **`assert_eq!` em `f32`, de propósito.** Uma tolerância aqui aceitaria uma *aproximação*, e a
/// afirmação que se quer é mais forte: mesmas operações, mesma ordem, mesmos bits. Se algum dia isto
/// precisar de épsilon, a mudança deixou de ser fatoração e virou outra coisa — e é isso que o gate
/// tem de contar.
///
/// **Mutação que deve sangrar:** cruzar os seeds dos dois eixos (`SEED_WARP_Y_A, SEED_WARP_X_A`) — o
/// erro realista desta refatoração, já que a fatoração passou a receber os DOIS seeds numa chamada só.
///
/// ⚠️ **E uma mutação minha que NÃO sangrou, registrada porque a lição é sobre `f32`:** eu havia escrito
/// que trocar a ordem dos termos (`bx * 0.35 + ax * 0.65`) sangraria *"porque em `f32` não é igual"*.
/// **É igual:** a adição IEEE-754 é COMUTATIVA (`a + b == b + a` exatamente); o que falha é a
/// ASSOCIATIVIDADE. Aquela mutação era um no-op e não podia sangrar — o defeito estava nela, não no
/// gate ([[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]], aqui
/// indiciando a própria mutação).
#[test]
fn the_two_axis_warp_factoring_is_byte_exact() {
    // Posições espalhadas por várias células das DUAS oitavas (22 e 8 px), incluindo fronteiras
    // exatas de célula, onde `floor` e `smooth01` são mais frágeis — uma fixture só no meio da
    // célula não conteria o fenômeno.
    let mut checked = 0usize;
    for tile in [NoiseTile::NONE, NoiseTile::new((512, 512), [true, true])] {
        for &x in &[0.0f32, 0.5, 7.999, 8.0, 21.999, 22.0, 63.25, 512.0, -13.75] {
            for &y in &[0.0f32, 1.25, 8.0, 22.0, 44.5, 300.125, -7.5] {
                let want = (
                    warp_axis(x, y, SEED_WARP_X_A, SEED_WARP_X_B, tile),
                    warp_axis(x, y, SEED_WARP_Y_A, SEED_WARP_Y_B, tile),
                );
                let got = warp_offset(x, y, tile);
                assert_eq!(
                    got, want,
                    "o warp fatorado tem de ser BIT-idêntico ao antigo em ({x}, {y})"
                );
                checked += 1;
            }
        }
    }
    // Controle positivo: o laço tem de ter rodado, senão o gate passa por não comparar nada.
    assert!(checked >= 100, "controle: so {checked} posicoes comparadas");
}

/// **Quanto a fatoração vale, medida ONDE o sinal existe.**
///
/// ⚠️ Na sonda de produto (`measure_what_a_watercolor_move_is_made_of`) o ganho ficou em 0,12–0,17 ms
/// sobre um piso de ruído calibrado em ±0,13 — ou seja **dentro do ruído, e portanto não é resultado**.
/// Isto mede a função em si, com milhões de chamadas, onde a razão é limpa. Duas grandezas diferentes:
/// lá *quanto do move o artista recupera*, aqui *quanto a peça melhorou*.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_two_axis_warp_factoring() {
    const N: usize = 4_000_000;
    let tile = NoiseTile::NONE;
    // Um sink que o otimizador não pode dobrar, e posições que ANDAM (uma constante deixaria o
    // compilador içar a chamada inteira para fora do laço e mediria um laço vazio).
    let run = |f: &dyn Fn(f32, f32) -> (f32, f32)| -> f64 {
        let mut best = f64::MAX;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let mut acc = 0.0f32;
            for i in 0..N {
                #[allow(clippy::cast_precision_loss)]
                let x = (i % 977) as f32 * 0.37;
                #[allow(clippy::cast_precision_loss)]
                let y = (i % 641) as f32 * 0.53;
                let (dx, dy) = f(x, y);
                acc += dx + dy;
            }
            assert!(
                acc.is_finite(),
                "o sink impede o laço de ser otimizado fora"
            );
            best = best.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        best
    };
    let old = run(&|x, y| {
        (
            warp_axis(x, y, SEED_WARP_X_A, SEED_WARP_X_B, tile),
            warp_axis(x, y, SEED_WARP_Y_A, SEED_WARP_Y_B, tile),
        )
    });
    let new = run(&|x, y| warp_offset(x, y, tile));
    println!(
        "\n[warp] {N} avaliacoes · dois eixos SEPARADOS {old:.2} ms · FATORADO {new:.2} ms · \
         {:.2}x\n",
        old / new.max(1e-9)
    );
}

/// Value noise is deterministic, in `[0, 1]`, and varies across cells (not a constant field).
#[test]
fn value_noise_is_deterministic_and_bounded() {
    let a = value_noise_tiled(12.3, 45.6, 5.0, SEED_GRAIN, NoiseTile::NONE);
    let b = value_noise_tiled(12.3, 45.6, 5.0, SEED_GRAIN, NoiseTile::NONE);
    assert_eq!(a, b, "same input ⇒ same value (deterministic)");
    assert!((0.0..=1.0).contains(&a), "in range");
    let c = value_noise_tiled(112.3, 245.6, 5.0, SEED_GRAIN, NoiseTile::NONE);
    assert!(
        (a - c).abs() > 1e-4,
        "distant cells differ (it actually varies)"
    );
}

/// **Tiling (doc 13 #2): the canvas-anchored noise is SEAMLESS across the sprite period.** A tiled
/// axis wraps the lattice at a whole number of cells spanning the period, so `noise(x) == noise(x +
/// period)` for every cell size the wash uses (warp 22/8, paper 5/2.5, jag) — the RaggedEdge lines up
/// at the seam. `NoiseTile::NONE` (Tiling off) stays NON-periodic, guarding the byte-identical path.
#[test]
fn tiled_noise_is_seamless_across_the_sprite_period() {
    let (pw, ph) = (64.0f32, 48.0f32);
    let tile = NoiseTile::new((pw as usize, ph as usize), [true, true]);
    for &cell in &[22.0f32, 8.0, 5.0, 2.5] {
        for k in 0..53 {
            let x = k as f32 * 1.7;
            let y = k as f32 * 1.1;
            let vx = value_noise_tiled(x, y, cell, SEED_GRAIN, tile);
            let vx2 = value_noise_tiled(x + pw, y, cell, SEED_GRAIN, tile);
            assert!(
                (vx - vx2).abs() < 1e-5,
                "X seam discontinuous (cell={cell}, k={k}): {vx} vs {vx2}"
            );
            let vy = value_noise_tiled(x, y + ph, cell, SEED_GRAIN, tile);
            assert!(
                (vx - vy).abs() < 1e-5,
                "Y seam discontinuous (cell={cell}, k={k}): {vx} vs {vy}"
            );
        }
    }
    // warp_axis (the RaggedEdge boundary) wraps too — the visible bug in the smoke.
    for k in 0..64 {
        let y = k as f32 * 0.9;
        let w = warp_axis(3.0, y, SEED_WARP_X_A, SEED_WARP_X_B, tile);
        let w2 = warp_axis(3.0 + pw, y, SEED_WARP_X_A, SEED_WARP_X_B, tile);
        assert!((w - w2).abs() < 1e-5, "warp seam discontinuous (k={k})");
    }
    // NONE must NOT be periodic (the historical non-tiled noise — no accidental tiling).
    let none = NoiseTile::NONE;
    let differs = (0..64).any(|k| {
        let y = k as f32;
        (value_noise_tiled(1.0, y, 8.0, SEED_GRAIN, none)
            - value_noise_tiled(1.0 + pw, y, 8.0, SEED_GRAIN, none))
        .abs()
            > 1e-4
    });
    assert!(
        differs,
        "NoiseTile::NONE must stay non-periodic (byte-identical path)"
    );
}

/// **#2b: a slot IMAGE tiles seamlessly under Tiling.** Snapping Size to a whole number of tiles across
/// the sprite makes the `fract`-wrapped image repeat exactly at the seam; the RAW size seams (the
/// control that proves the snap is what fixes it). Off-tiling + procedural kinds ⇒ unchanged.
#[test]
fn slot_image_tiles_seamlessly_under_tiling() {
    use ph2d_painter_brush::texture::{ImageMask, angle_basis, sample_tiled_rot};
    let (pw, ph) = (100i64, 60i64);
    let tile = NoiseTile::new((pw as usize, ph as usize), [true, true]);
    let lum: Vec<u8> = (0..64).map(|i| ((i * 37) % 256) as u8).collect(); // non-uniform 8×8
    let mask = ImageMask {
        lum: &lum,
        width: 8,
        height: 8,
    };
    let raw = TextureSettings {
        kind: TextureKind::Image,
        size: [1.37, 0.83],
        ..Default::default()
    };
    let snapped = snap_slot_size(raw, tile);
    let rot = angle_basis(0);
    let tx = pw as f32 * snapped.size[0] / TEX_TILE_BASE_PX;
    let ty = ph as f32 * snapped.size[1] / TEX_TILE_BASE_PX;
    assert!(
        (tx - tx.round()).abs() < 1e-4 && (ty - ty.round()).abs() < 1e-4,
        "snap must yield whole tiles across the sprite ({tx}, {ty})"
    );
    for y in [3i64, 19, 41] {
        let a = sample_tiled_rot(&snapped, 0, y, Some(&mask), rot);
        let b = sample_tiled_rot(&snapped, pw, y, Some(&mask), rot);
        assert!(
            (a - b).abs() < 1e-4,
            "X seam not seamless at y={y}: {a} vs {b}"
        );
    }
    for x in [5i64, 27, 63] {
        let a = sample_tiled_rot(&snapped, x, 0, Some(&mask), rot);
        let b = sample_tiled_rot(&snapped, x, ph, Some(&mask), rot);
        assert!((a - b).abs() < 1e-4, "Y seam not seamless at x={x}");
    }
    // Control: the RAW (unsnapped) size seams somewhere across the sprite.
    let seams = (0..ph).any(|y| {
        (sample_tiled_rot(&raw, 0, y, Some(&mask), rot)
            - sample_tiled_rot(&raw, pw, y, Some(&mask), rot))
        .abs()
            > 1e-4
    });
    assert!(
        seams,
        "control: an unsnapped image should seam across the sprite"
    );
    // Off-tiling ⇒ unchanged (byte-identical), for every kind. A NON-tileable kind (turbulence-based
    // Marble/Magic/Wood; irrational Triangles/Hexagons) stays unchanged even under tiling.
    assert_eq!(snap_slot_size(raw, NoiseTile::NONE).size, raw.size);
    let non_tileable = TextureSettings {
        kind: TextureKind::Marble,
        size: [1.37, 0.83],
        ..Default::default()
    };
    assert_eq!(snap_slot_size(non_tileable, tile).size, non_tileable.size);
}

/// **#2c: a LATTICE procedural (Noise) tiles seamlessly under Tiling.** Snapping Size to a whole
/// number of cells across the sprite + wrapping the value-noise hash at that period makes the field
/// periodic, so `noise(x) == noise(x + sprite)`. The RAW (unsnapped, unwrapped) sample seams — the
/// control that proves the snap+wrap is what fixes it. Off-tiling ⇒ unchanged (byte-identical).
#[test]
fn slot_lattice_tiles_seamlessly_under_tiling() {
    use ph2d_painter_brush::texture::{angle_basis, sample_tiled_rot, sample_tiled_rot_wrapped};
    // Dimensions + Size chosen so the snap lands on period ≥ 2 cells (a period-1 wrap collapses the
    // lattice to a CONSTANT field — trivially "seamless" but no test of the wrap). `2.6/3.1` snap to
    // 2 cells across `200×140` px, so the field genuinely varies across the seam.
    let (pw, ph) = (200i64, 140i64);
    let tile = NoiseTile::new((pw as usize, ph as usize), [true, true]);
    let rot = angle_basis(0);
    // Warp on (knob 2 = params[4]) + multi-octave (knob 0 = params[2]) so the warp/fbm paths run too.
    let mut params = [0.5f32; ph2d_painter_brush::MAX_TEX_PARAMS];
    params[2] = 0.6; // Detail → multi-octave
    params[4] = 0.4; // Warp → domain distortion
    for kind in [
        TextureKind::Noise,
        TextureKind::Clouds,
        TextureKind::Grain,
        TextureKind::Voronoi,
        TextureKind::Musgrave,
    ] {
        let raw = TextureSettings {
            kind,
            size: [2.6, 3.1],
            params,
            ..Default::default()
        };
        let snapped = snap_slot_size(raw, tile);
        let per = tile.slot_period();
        // The field must actually VARY (guards against a degenerate constant collapsing the test).
        let (s0, s1) = (
            sample_tiled_rot_wrapped(&snapped, 13, 29, None, rot, per),
            sample_tiled_rot_wrapped(&snapped, 91, 67, None, rot, per),
        );
        assert!(
            (s0 - s1).abs() > 1e-3,
            "{kind:?} wrapped field is degenerate/constant"
        );
        for y in [3i64, 19, 41, 111] {
            let a = sample_tiled_rot_wrapped(&snapped, 0, y, None, rot, per);
            let b = sample_tiled_rot_wrapped(&snapped, pw, y, None, rot, per);
            assert!((a - b).abs() < 1e-4, "{kind:?} X seam at y={y}: {a} vs {b}");
        }
        for x in [5i64, 27, 63, 177] {
            let a = sample_tiled_rot_wrapped(&snapped, x, 0, None, rot, per);
            let b = sample_tiled_rot_wrapped(&snapped, x, ph, None, rot, per);
            assert!((a - b).abs() < 1e-4, "{kind:?} Y seam at x={x}: {a} vs {b}");
        }
        // Control: the plain tiled sample (no snap, no wrap) seams somewhere across the sprite.
        let seams = (0..ph).any(|y| {
            (sample_tiled_rot(&raw, 0, y, None, rot) - sample_tiled_rot(&raw, pw, y, None, rot))
                .abs()
                > 1e-4
        });
        assert!(
            seams,
            "control: unsnapped {kind:?} should seam across the sprite"
        );
    }
}

/// **Fase 2: every ANALYTIC pattern tiles seamlessly under Tiling.** The pure-periodic patterns are
/// already exactly periodic, so snapping Size to a whole number of their per-axis period
/// (`analytic_tile_period`) lands the seam on a period boundary — no sampler change. The HASH-JITTERED
/// ones (Dots/Scales) additionally need the cell-jitter hash wrapped (`sample_tiled_rot_wrapped` passes
/// the period, gated by `analytic_needs_hash_wrap`). Proves `sample(0,y) == sample(pw,y)` and
/// `sample(x,0) == sample(x,ph)` for all tileable kinds; the RAW (unsnapped, unwrapped) size seams (the
/// control). Ignored axes (Stripes/Gradient constant on v) are seamless at any size (seam holds trivially).
#[test]
fn slot_analytic_pattern_tiles_seamlessly_under_tiling() {
    // Import the variants by name (NOT a glob — a `TextureKind::None` glob would shadow `Option::None`
    // in the `sample_tiled_rot(.., None, ..)` calls).
    use ph2d_painter_brush::TextureKind::{
        Bricks, Checker, Chevron, Crosshatch, Diamonds, Dots, Gradient, Grid, Hexagons, Magic,
        Marble, Scales, Stripes, Triangles, Waves, Weave, Wood,
    };
    use ph2d_painter_brush::texture::{
        analytic_needs_hash_wrap, analytic_tile_period, angle_basis, sample_tiled_rot,
        sample_tiled_rot_wrapped,
    };
    let (pw, ph) = (200i64, 140i64);
    let tile = NoiseTile::new((pw as usize, ph as usize), [true, true]);
    let rot = angle_basis(0);
    let per = tile.slot_period(); // the sprite period the hash-wrap kinds (Dots/Scales) need
    // Every kind `analytic_tile_period` accepts — the pure-periodic ones (snap only) AND Dots/Scales
    // (snap + cell-hash wrap). Excluded on purpose: turbulence (Marble/Magic/Wood), irrational period
    // (Triangles/Hexagons) — guarded below.
    let kinds = [
        Checker, Diamonds, Stripes, Grid, Crosshatch, Waves, Chevron, Weave, Bricks, Gradient,
        Dots, Scales,
    ];
    for kind in kinds {
        assert!(
            analytic_tile_period(kind, [0.5; ph2d_painter_brush::MAX_TEX_PARAMS]).is_some(),
            "{kind:?} must be analytic-tileable"
        );
        let raw = TextureSettings {
            kind,
            size: [2.6, 3.1],
            ..Default::default()
        };
        let snapped = snap_slot_size(raw, tile);
        // Sample the seamless form: the wrap is a no-op for the pure kinds (their gate is off, so
        // `per` is ignored) and lands the hash wrap for Dots/Scales.
        let s = |x: i64, y: i64| sample_tiled_rot_wrapped(&snapped, x, y, None, rot, per);
        // The field must actually VARY across the sprite (guards a degenerate constant). Scan a 2-D
        // grid — a 1-D line can miss a sparse pattern's features (e.g. gaps between Dots rows).
        let v0 = s(3, 5);
        let varies = (0..pw)
            .step_by(11)
            .any(|x| (0..ph).step_by(13).any(|y| (s(x, y) - v0).abs() > 1e-3));
        assert!(varies, "{kind:?} snapped field is degenerate/constant");
        for y in [3i64, 19, 41, 111] {
            let (a, b) = (s(0, y), s(pw, y));
            assert!((a - b).abs() < 2e-3, "{kind:?} X seam at y={y}: {a} vs {b}");
        }
        for x in [5i64, 27, 63, 177] {
            let (a, b) = (s(x, 0), s(x, ph));
            assert!((a - b).abs() < 2e-3, "{kind:?} Y seam at x={x}: {a} vs {b}");
        }
        // Control: the RAW (unsnapped, unwrapped) size seams somewhere across the sprite (proves the
        // snap+wrap fixes it). Checked on both axes — a 1D kind (Stripes/Gradient) seams on its live axis.
        let x_seams = (0..ph).any(|y| {
            (sample_tiled_rot(&raw, 0, y, None, rot) - sample_tiled_rot(&raw, pw, y, None, rot))
                .abs()
                > 2e-3
        });
        let y_seams = (0..pw).any(|x| {
            (sample_tiled_rot(&raw, x, 0, None, rot) - sample_tiled_rot(&raw, x, ph, None, rot))
                .abs()
                > 2e-3
        });
        assert!(
            x_seams || y_seams,
            "control: unsnapped {kind:?} should seam across the sprite"
        );
    }
    // Dots/Scales are hash-jittered (need the wrap); the pure kinds are NOT — refutable boundary.
    assert!(analytic_needs_hash_wrap(Dots) && analytic_needs_hash_wrap(Scales));
    assert!(!analytic_needs_hash_wrap(Checker) && !analytic_needs_hash_wrap(Grid));
    // The excluded kinds are NOT snap-tileable (turbulence / irrational period) — documents the
    // boundary refutably so a future edit can't silently (mis)snap one.
    for kind in [Marble, Magic, Wood, Triangles, Hexagons] {
        assert!(
            analytic_tile_period(kind, [0.5; ph2d_painter_brush::MAX_TEX_PARAMS]).is_none(),
            "{kind:?} must NOT be analytic-snap-tileable"
        );
    }
    // Off-tiling ⇒ unchanged (byte-identical) for an analytic kind.
    let raw = TextureSettings {
        kind: Stripes,
        size: [1.37, 0.83],
        ..Default::default()
    };
    assert_eq!(snap_slot_size(raw, NoiseTile::NONE).size, raw.size);
}

/// **Watercolor grain now matches the brush's ViewPlane scale (Enio 2026-07-11).** The wash samples the
/// Grain slot canvas-anchored (`px·size/256`), but the brush's default ViewPlane maps `size` per DAB
/// RADIUS, so the SAME grain rendered ~`256/radius`× COARSER in the wash — a fine Voronoi became giant
/// blobs. `grain_view_to_canvas_size` rescales the Size so the canvas sample reproduces the brush's
/// feature scale EXACTLY. Proof = value correspondence: the converted canvas sample equals the brush's
/// ViewPlane `sample` at the matching coord; the RAW (unconverted) canvas sample DIVERGES (the bug). It's
/// kind-INDEPENDENT (a mapping/scale gap) — shown here on Voronoi + Stripes + Grid (worst on Voronoi).
#[test]
fn watercolor_grain_matches_the_brush_viewplane_scale() {
    use ph2d_painter_brush::texture::{TexDabBasis, angle_basis, sample, sample_tiled_rot};
    let basis = TexDabBasis::identity();
    let rot = angle_basis(0);
    let radius = 40.0f32;
    for kind in [
        TextureKind::Voronoi,
        TextureKind::Stripes,
        TextureKind::Grid,
    ] {
        let gtex = TextureSettings {
            kind,
            mapping: TextureMapping::ViewPlane,
            size: [1.3, 1.3],
            ..Default::default()
        };
        let scaled = grain_view_to_canvas_size(gtex, radius);
        let mut raw_diverges = false;
        for px in (0..512).step_by(3) {
            // Brush ViewPlane (the reference the user compares against): a dab at the origin, radius R.
            let brush_v = sample(&gtex, &basis, px, 0, [0.0, 0.0], radius, None);
            // Watercolor canvas sample of the CONVERTED grain — reproduces the brush's tex coord exactly
            // (`px·(size·256/R)/256 = px·size/R`), so the value MATCHES the brush.
            let wash = sample_tiled_rot(&scaled, px, 0, None, rot);
            assert!(
                (brush_v - wash).abs() < 1e-4,
                "{kind:?} px={px}: brush {brush_v} vs wash {wash} (scale must match)"
            );
            // Control: the RAW (unconverted) canvas sample uses base-256 → diverges from the brush.
            let wash_raw = sample_tiled_rot(&gtex, px, 0, None, rot);
            if (brush_v - wash_raw).abs() > 0.1 {
                raw_diverges = true;
            }
        }
        assert!(
            raw_diverges,
            "{kind:?}: the RAW (unconverted) wash must diverge from the brush — the coarse bug"
        );
    }
    // A Tiled Grain is already canvas-anchored (matches the brush) ⇒ unchanged; inactive ⇒ unchanged.
    let tiled = TextureSettings {
        kind: TextureKind::Voronoi,
        mapping: TextureMapping::Tiled,
        size: [1.3, 1.3],
        ..Default::default()
    };
    assert_eq!(grain_view_to_canvas_size(tiled, radius).size, tiled.size);
    let off = TextureSettings::default(); // kind None
    assert_eq!(grain_view_to_canvas_size(off, radius).size, off.size);
}

/// The lattice wrap is a no-op off-tiling and under rotation: `sample_tiled_rot_wrapped` with a zero
/// period (or a rotated basis) is byte-identical to the plain `sample_tiled_rot` (byte-identity guard).
#[test]
fn lattice_wrap_is_byte_identical_off_tiling_and_rotated() {
    use ph2d_painter_brush::texture::{angle_basis, sample_tiled_rot, sample_tiled_rot_wrapped};
    let s = TextureSettings {
        kind: TextureKind::Noise,
        size: [1.37, 0.83],
        ..Default::default()
    };
    let per = [100.0f32, 60.0];
    for (rot, period) in [
        (angle_basis(0), [0.0f32, 0.0]), // no sprite period → no wrap
        (angle_basis(30), per),          // rotated → wrap gated off
    ] {
        for (x, y) in [(0i64, 0i64), (7, 13), (50, 31)] {
            let plain = sample_tiled_rot(&s, x, y, None, rot);
            let wrapped = sample_tiled_rot_wrapped(&s, x, y, None, rot, period);
            assert_eq!(
                plain.to_bits(),
                wrapped.to_bits(),
                "wrap must be byte-identical here"
            );
        }
    }
}

/// **#2b: a baked PAPER preset tiles seamlessly under Tiling.** The 256² paper tile repeats every 1
/// unit of `rel`, so snapping Size to a whole tile count across the sprite lands the seam on a tile
/// boundary. Control: the raw (unsnapped) Size seams. Off-tiling ⇒ unchanged.
#[test]
fn slot_paper_preset_tiles_seamlessly_under_tiling() {
    use ph2d_painter_brush::texture::{angle_basis, sample_tiled_rot};
    let (pw, ph) = (100i64, 60i64);
    let tile = NoiseTile::new((pw as usize, ph as usize), [true, true]);
    let raw = TextureSettings {
        kind: TextureKind::PaperCold,
        size: [0.9, 1.3],
        ..Default::default()
    };
    let snapped = snap_slot_size(raw, tile);
    let rot = angle_basis(0);
    for y in [3i64, 29, 51] {
        let a = sample_tiled_rot(&snapped, 0, y, None, rot);
        let b = sample_tiled_rot(&snapped, pw, y, None, rot);
        assert!((a - b).abs() < 1e-4, "paper X seam at y={y}: {a} vs {b}");
    }
    for x in [7i64, 41, 88] {
        let a = sample_tiled_rot(&snapped, x, 0, None, rot);
        let b = sample_tiled_rot(&snapped, x, ph, None, rot);
        assert!((a - b).abs() < 1e-4, "paper Y seam at x={x}");
    }
    let seams = (0..ph).any(|y| {
        (sample_tiled_rot(&raw, 0, y, None, rot) - sample_tiled_rot(&raw, pw, y, None, rot)).abs()
            > 1e-4
    });
    assert!(
        seams,
        "control: an unsnapped paper preset should seam across the sprite"
    );
    assert_eq!(snap_slot_size(raw, NoiseTile::NONE).size, raw.size);
}
