//! Tests for [`super`] (the vector→tile bake) — moved to a sibling via `#[path]`
//! to keep `motion_object_bake.rs` under the shell LOC cap. It stays a FILE
//! child, so `use super::*` reaches the module's private `bake_camera`/`bake_rgba`.

use super::*;

fn star() -> VecPath {
    let mut p = ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.45);
    p.fill = Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
        255, 170, 40, 255,
    )));
    p
}

#[test]
fn moving_the_shape_does_not_rebake_but_rotating_and_editing_do() {
    // The design decision the cache stands on (doc 86 §2): the tile is the
    // shape's DRAWING at a fixed DPI, bbox-normalized. So a MOVE (translation
    // only) must be a cache hit — the local `VecPath` and the LINEAR part of
    // the transform are unchanged; only the translation moved, and the tile
    // does not carry it. A ROTATE (linear changes) or an EDIT (path changes)
    // re-bakes. A key that folded translation in would re-bake on every drag.
    let base = BakeKey {
        path: star(),
        linear: [1.0, 0.0, 0.0, 1.0],
        dpi_q: 256,
    };
    // A move never touches the local path or the linear coeffs.
    let moved = BakeKey {
        path: star(),
        linear: [1.0, 0.0, 0.0, 1.0],
        dpi_q: 256,
    };
    assert_eq!(base, moved, "a move is a cache hit — no re-bake");
    // A rotate changes the linear part.
    let rotated = BakeKey {
        path: star(),
        linear: [0.0, 1.0, -1.0, 0.0],
        dpi_q: 256,
    };
    assert_ne!(base, rotated, "a rotate re-bakes");
    // An edit changes the authored path.
    let edited = BakeKey {
        path: ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 6, 0.45),
        linear: [1.0, 0.0, 0.0, 1.0],
        dpi_q: 256,
    };
    assert_ne!(base, edited, "editing the shape re-bakes");
}

/// **The A5 thumbnail is bounded and keeps aspect** (doc 86 A5). A wide opaque tile
/// downsamples so its LONG side is `THUMB_MAX`, the 3:1 aspect survives, the bytes are
/// tightly packed, and an opaque colour comes out unchanged. FALSIFIED by an unbounded
/// or stretched thumbnail.
#[test]
fn the_thumbnail_is_bounded_and_keeps_aspect() {
    let (w, h) = (600u32, 200u32);
    let rgba = vec![255u8; (w * h * 4) as usize]; // opaque white
    let t = thumbnail(&rgba, w, h);
    assert_eq!(t.w.max(t.h), THUMB_MAX, "long side capped at THUMB_MAX");
    assert!(
        (t.w as f32 / t.h as f32 - 3.0).abs() < 0.05,
        "the 3:1 aspect is preserved"
    );
    assert_eq!(
        t.rgba.len(),
        (t.w * t.h * 4) as usize,
        "tightly packed RGBA8"
    );
    assert!(
        t.rgba.chunks(4).all(|p| p == [255, 255, 255, 255]),
        "an opaque solid colour survives the downsample"
    );
}

/// **A tile under the cap is never upscaled** (doc 86 A5) — the thumbnail of a small
/// shape is the tile itself, not a blurry blow-up. FALSIFIED by scaling toward THUMB_MAX.
#[test]
fn a_small_tile_is_never_upscaled() {
    let (w, h) = (10u32, 8u32);
    let rgba = vec![128u8; (w * h * 4) as usize];
    let t = thumbnail(&rgba, w, h);
    assert_eq!(
        (t.w, t.h),
        (w, h),
        "under the cap the thumbnail is the tile"
    );
}

/// **The downsample does not bleed a dark halo into a transparent edge** (doc 86 A5).
/// A row of alternating opaque-RED / fully-transparent pixels merges pairwise: the
/// PREMULTIPLIED average keeps the surviving colour pure red (`Σc·a/Σa = 255`); a naive
/// STRAIGHT average would pull it toward black (`(255+0)/2 = 127`), the premul trap the
/// overlay lesson names. FALSIFIED by averaging straight RGBA.
#[test]
fn the_downsample_does_not_bleed_a_halo_into_a_transparent_edge() {
    let (w, h) = (THUMB_MAX * 2, 1u32); // 2:1 downsample merges pixel pairs
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for x in (0..w).step_by(2) {
        let i = (x * 4) as usize;
        rgba[i..i + 4].copy_from_slice(&[255, 0, 0, 255]); // opaque red; odd stays transparent
    }
    let t = thumbnail(&rgba, w, h);
    assert_eq!(t.w, THUMB_MAX, "downsampled 2:1");
    for p in t.rgba.chunks(4) {
        assert_eq!(
            &p[0..3],
            &[255, 0, 0],
            "colour stays pure red — a straight average would darken it to 127"
        );
        assert!(
            (p[3] as i32 - 127).abs() <= 1,
            "alpha is the coverage average of the merged pair"
        );
    }
}

#[test]
fn select_present_bakes_named_and_group_children_but_not_loose_art() {
    // doc 86 §9.6: the bake tiles a vector drawing iff it is NAMED (the picker path)
    // OR sits inside a named group (so the group stamp has its child's tile) — and
    // NOTHING else, so unnamed canvas art never wastes a tile (§0 VRAM). The three
    // rows are the whole decision table; two mutations each break a distinct row.
    use ph2d_ecs::{ChildOf, GroupedChildren};
    let mut sim = SimWorld::new();
    let named = sim.world_mut().spawn((Name::new("Named"),)).id();
    let group = sim
        .world_mut()
        .spawn((Name::new("Group"), GroupedChildren))
        .id();
    let child = sim.world_mut().spawn((ChildOf(group),)).id(); // UNNAMED group child
    let loose = sim.world_mut().spawn(()).id(); // unnamed, no group

    // The map is VecPathId -> entity bits (the same thing `sync` builds).
    let mut map = VecEntityMap::new();
    map.insert(10, named.to_bits());
    map.insert(20, child.to_bits());
    map.insert(30, loose.to_bits());

    let present = select_present(sim.world(), &map);

    assert_eq!(
        present.get(&10),
        Some(&Some("Named".to_string())),
        "a named drawing is tiled, carrying its name"
    );
    // ⚠️ Mutation `if name.is_none()` (drop the group check) SKIPS this — the exact
    // doc-86 item-3 bug (an unnamed group child gets no tile).
    assert_eq!(
        present.get(&20),
        Some(&None),
        "an UNNAMED group child is tiled by its id, with no name"
    );
    // ⚠️ Mutation dropping the `continue` (bake-all) makes this present — a wasted tile.
    assert!(
        !present.contains_key(&30),
        "unnamed canvas art no group references is NOT tiled"
    );
}

#[test]
fn select_present_skips_stale_bits() {
    // A map value whose entity was despawned must not be baked — its tile is evicted,
    // not resurrected. ⚠️ This pins the END-TO-END invariant, not the `get_entity`
    // guard specifically: a despawned entity also has no `Name`, so it falls into the
    // unnamed-AND-no-group skip even without the guard — dropping the guard does NOT
    // falsify this. The guard is robustness (it mirrors `vec_entities::sync`'s own
    // `get_entity(..).is_err()`); it earns its keep the day a Name-independent tiling
    // path appears, which is exactly what this gate would then catch.
    let mut sim = SimWorld::new();
    let live = sim.world_mut().spawn((Name::new("Live"),)).id();
    let dead = sim.world_mut().spawn((Name::new("Dead"),)).id();
    sim.world_mut().despawn(dead);
    let mut map = VecEntityMap::new();
    map.insert(1, live.to_bits());
    map.insert(2, dead.to_bits());
    let present = select_present(sim.world(), &map);
    assert!(present.contains_key(&1), "the live drawing is tiled");
    assert!(
        !present.contains_key(&2),
        "the despawned drawing is skipped"
    );
}

/// **The bake camera flips Y** (the smoke: *"a estrela no grid fica de cabeça para
/// baixo"*). The sprite renderer displays texture row 0 at screen-TOP (world-up) and
/// Vello renders Y-DOWN, so the bake must map world-Y-up to a SMALLER device Y (row 0)
/// — the SAME `scale_non_uniform(k, -k)` the live camera
/// (`Camera2d::world_to_screen_affine`) and the Flip bake apply. RED-FIRST: the pre-fix
/// bare `Affine::scale(BAKE_DPI)` (positive Y) inverts it, and this gate fails. The Y
/// scale is the affine's `d` coefficient (index 3): it must be strictly negative.
#[test]
fn the_bake_camera_flips_y_so_a_tile_is_upright() {
    let [_, _, _, d, _, _] = bake_camera().as_coeffs();
    assert!(
        d < 0.0,
        "bake-camera Y scale (coeff d) must be negative so world-up bakes to row 0 / \
             screen-top; got d={d} — a positive Y bakes the tile upside down"
    );
}

/// **The baked tile is UPRIGHT, end to end** (the smoke report, on the device). An
/// apex-UP filled triangle bakes to a tile whose TOP rows carry the narrow apex (few
/// opaque texels) and whose BOTTOM rows carry the wide base (many) — the top-down row
/// order the sprite renderer displays point-up. Drives the REAL [`bake_rgba`] on a GPU,
/// so it also proves no OTHER step re-flips. With the pre-fix `Affine::scale(BAKE_DPI)`
/// the tile inverts (base at top) → `top >= bottom` → RED. Needs an adapter (RTX);
/// `#[ignore]`, run with `-- --ignored`.
#[test]
#[ignore = "needs a GPU adapter (RTX); run with --ignored"]
fn the_baked_tile_is_upright() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping the_baked_tile_is_upright");
        return;
    };
    // Apex at world-y +0.5 (up), wide base at world-y -0.5 (down). Filled, so the
    // opaque mass is asymmetric top↔bottom: a narrow apex vs a full-width base.
    let mut scene = VecScene::new();
    let mut tri = VecPath {
        verts: vec![
            ph2d_vec_scene::VecVertex::corner([0.0, 0.5]),
            ph2d_vec_scene::VecVertex::corner([-0.5, -0.5]),
            ph2d_vec_scene::VecVertex::corner([0.5, -0.5]),
        ],
        closed: true,
        ..VecPath::default()
    };
    tri.fill = Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
        255, 170, 40, 255,
    )));
    let id = scene.push_path(tri);
    let xforms = VecXforms::default(); // path absent ⇒ identity pose
    let live: LiveGeometry = LiveGeometry::new();
    let mut scratch: Option<VelloPass> = None;
    let (rgba, w, h, _size) = bake_rgba(
        &mut scratch,
        &scene,
        &xforms,
        &live,
        id,
        &gpu,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )
    .expect("bake produced a tile");

    // Opaque coverage (alpha > 128) in the top vs the bottom band (~14% each).
    let band = (h / 7).max(1);
    let opaque = |y0: u32, y1: u32| -> u64 {
        let mut n = 0u64;
        for y in y0..y1 {
            for x in 0..w {
                if rgba[((y * w + x) * 4 + 3) as usize] > 128 {
                    n += 1;
                }
            }
        }
        n
    };
    let top = opaque(0, band);
    let bottom = opaque(h - band, h);
    assert!(
        top < bottom,
        "UPRIGHT tile: the top rows (apex, narrow) must have FEWER opaque texels than \
             the bottom rows (base, wide); got top={top} bottom={bottom} (w={w} h={h}) — \
             top >= bottom means the tile baked upside down"
    );
}

/// ⭐⭐⭐ **UM GRUPO ASSA COMO UM SÓ LADRILHO** (Enio, 2026-08-30: *"assim o grupo poderia ser usado
/// como pattern"*), na GPU de verdade.
///
/// Duas formas afastadas assam num ladrilho cuja caixa é a **união** das duas, e **as duas
/// desenham** nele. As duas metades importam e falham por razões diferentes:
/// - sem a união da caixa, o ladrilho tem a largura de UMA forma e o grupo sai cortado;
/// - sem o laço de desenho, a caixa é larga e o segundo membro **não aparece** — um ladrilho com
///   metade do grupo e um vazio do tamanho da outra metade.
///
/// ⚠️ A régua é a TINTA em cada extremo, e não a largura sozinha: uma caixa larga com um membro só
/// passaria numa medida de largura.
#[test]
#[ignore = "needs a GPU adapter; run with --ignored"]
fn a_group_bakes_into_one_tile_with_both_members_in_it() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping a_group_bakes_into_one_tile_with_both_members_in_it");
        return;
    };
    let quadrado = |x: f64, cor: ph2d_vec_scene::Rgba8| VecPath {
        verts: [[x, -0.5], [x + 1.0, -0.5], [x + 1.0, 0.5], [x, 0.5]]
            .map(ph2d_vec_scene::VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(ph2d_vec_scene::Paint::solid(cor)),
        ..VecPath::default()
    };
    let mut scene = VecScene::new();
    let a = scene.push_path(quadrado(0.0, ph2d_vec_scene::Rgba8::new(255, 0, 0, 255)));
    // ⚠️ Um VÃO entre os dois: sem ele a união é indistinguível de um rectângulo esticado.
    let b = scene.push_path(quadrado(3.0, ph2d_vec_scene::Rgba8::new(0, 0, 255, 255)));
    let xforms = VecXforms::default();
    let live: LiveGeometry = LiveGeometry::new();
    let mut scratch: Option<VelloPass> = None;

    let so_um = bake_rgba(
        &mut scratch,
        &scene,
        &xforms,
        &live,
        a,
        &gpu,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )
    .expect("um membro assa");
    let (rgba, w, h, _) = super::bake_rgba_many(
        &mut scratch,
        &scene,
        &xforms,
        &live,
        &[a, b],
        &gpu,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )
    .expect("o grupo assa");

    assert!(
        w > so_um.1 * 2,
        "o ladrilho do grupo tem {w} px e o de um membro so' tem {} - a caixa nao e' a UNIAO, e o \
         grupo sai cortado",
        so_um.1
    );
    // A tinta nos dois extremos: a fatia esquerda e a direita, a meia altura.
    let opaco_em = |fx: f32| -> bool {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let x = ((w as f32 - 1.0) * fx) as u32;
        let y = h / 2;
        let o = ((y * w + x) * 4) as usize;
        rgba.get(o + 3).is_some_and(|a| *a > 128)
    };
    assert!(opaco_em(0.02), "o membro da ESQUERDA nao esta' no ladrilho");
    assert!(
        opaco_em(0.98),
        "o membro da DIREITA nao esta' no ladrilho - a caixa cresceu e o laco de desenho nao \
         desenhou o segundo: o grupo sai com metade e um vazio do tamanho da outra"
    );
}
