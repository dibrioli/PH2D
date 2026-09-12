//! Gates for the Flip bake (doc 86 §2/§6 A3). FILHO of `motion_flip_bake` via
//! `#[path]` (so `use super::*` reaches the private `content_key`/`bake_one`), split
//! off to keep the parent under the shell LOC cap.

use super::*;
use ph2d_flip::{Fill, FlipStroke, Hold, KeyKind, Point, Rgba};

/// A two-layer named Flip object, mirroring the smoke: a filled rectangle (BG) + a
/// square (FG) of half-size `fg_half`. The parameter lets the edit case build a
/// genuinely different geometry.
fn two_layer_object(fg_half: f32) -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Object");
    let obj = doc.object_mut(oid).expect("object");
    obj.fps = 12.0;
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("bg drawing")
            .strokes
            .push(filled_rect(
                [-0.6, -0.4],
                [0.6, 0.4],
                Rgba::new(0.2, 0.5, 0.9, 1.0),
            ));
    }
    let fg = obj.add_layer("FG");
    if let Some(d) = obj.insert_frame(fg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("fg drawing")
            .strokes
            .push(filled_rect(
                [-fg_half, -fg_half],
                [fg_half, fg_half],
                Rgba::new(0.95, 0.7, 0.2, 1.0),
            ));
    }
    (doc, oid)
}

fn filled_rect(a: [f32; 2], b: [f32; 2], color: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for corner in [
        Vec2::new(a[0], a[1]),
        Vec2::new(b[0], a[1]),
        Vec2::new(b[0], b[1]),
        Vec2::new(a[0], b[1]),
    ] {
        s.push_point(Point {
            pos: corner,
            width: 0.04,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color,
        opacity: 1.0,
    });
    s
}

fn key_of(doc: &FlipDoc, oid: FlipObjectId, model: Xform) -> u64 {
    let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
    content_key(obj, &model, obj.frame_at(&Playhead::default()))
}

/// The design decision the cache stands on (doc 86 §2, A3): the tile is the object's
/// composed drawing, bbox-normalized. So a MOVE (object translation only) is a cache
/// HIT — the linear transform, the layer poses and the geometry are unchanged, and
/// the tile does not carry the translation. A ROTATE (linear changes) or an EDIT
/// (geometry changes) re-bakes. A key that folded translation in would re-bake on
/// every drag (A2's rule, in the Flip axis).
#[test]
fn moving_the_object_does_not_rebake_but_rotating_and_editing_do() {
    let (doc, oid) = two_layer_object(0.2);

    let base = key_of(&doc, oid, Xform::IDENTITY);
    // A move: translation only (Xform e,f) — the linear part is unchanged.
    let moved = key_of(&doc, oid, Xform([1.0, 0.0, 0.0, 1.0, 3.7, -2.1]));
    assert_eq!(base, moved, "a move is a cache hit — no re-bake");
    // A 90° rotation: the linear part changes.
    let rotated = key_of(&doc, oid, Xform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]));
    assert_ne!(base, rotated, "a rotate re-bakes");

    // MEASURED — the LIVE cache's whole point: a STATIC hold (this object has
    // drawings only at frame 0, held forever) must yield ONE content key across a
    // sweep of frames, so it bakes ONCE and does not re-rasterize every frame of
    // playback. A key that hashed the raw frame would re-bake 25 times here.
    let mut ph = Playhead::new(1.0 / 60.0);
    let mut distinct = std::collections::BTreeSet::new();
    for f in 0..25 {
        ph.seek_frame(f, 12.0);
        let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
        distinct.insert(content_key(obj, &Xform::IDENTITY, obj.frame_at(&ph)));
    }
    assert_eq!(
        distinct.len(),
        1,
        "a static hold bakes ONCE, not per-frame (25 frames -> 1 key)"
    );

    // An edit: a LARGER FG square (different geometry) -> a different content key.
    let (edited, eid) = two_layer_object(0.35);
    let key_e = key_of(&edited, eid, Xform::IDENTITY);
    assert_ne!(base, key_e, "editing the geometry re-bakes");
}

/// **gate-5-flip (doc 86 §6):** the baked tile carries the object's COMPOSED
/// silhouette — an APPEARANCE oracle (coverage + BOTH layers' colours present),
/// not bytes (the bake is GPU). It drives the real Flip raster + compositor +
/// readback on a device, so it also proves the GPU path doesn't panic.
///
/// `#[ignore]`: needs a GPU adapter (none on CI); run on a dev machine with
/// `--ignored`.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn a_baked_flip_object_carries_the_composed_two_layer_silhouette() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let (doc, oid) = two_layer_object(0.2);
    let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
    let mut bake = FlipObjectBake::default();

    // ⚠️ Warm-up bake, DISCARDED: a fresh process pays cold GPU init + the first
    // pipeline/kernel COMPILE (the walk kernel, the compositor). In the running app
    // the device + pipelines are already warm (the frame pass compiled them), so the
    // representative bake cost is the SECOND one — the first would let the cold path
    // define the product (§0).
    let warm = bake
        .bake_one(
            obj,
            &Xform::IDENTITY,
            obj.frame_at(&Playhead::default()),
            &gpu,
            &mut renderer,
        )
        .expect("the warm-up bake produced a tile");
    renderer.individual_mut().release(warm.0);

    let t0 = std::time::Instant::now();
    let (tid, wsize, _thumb) = bake
        .bake_one(
            obj,
            &Xform::IDENTITY,
            obj.frame_at(&Playhead::default()),
            &gpu,
            &mut renderer,
        )
        .expect("the bake produced a tile");
    let dt = t0.elapsed();

    let (w, h, rgba) = renderer
        .readback_individual(tid)
        .expect("readback the tile");
    let (mut opaque, mut blue, mut orange) = (0usize, 0usize, 0usize);
    for px in rgba.as_chunks::<4>().0.iter() {
        let (r, g, b, a) = (i32::from(px[0]), i32::from(px[1]), i32::from(px[2]), px[3]);
        if a > 128 {
            opaque += 1;
            if b > r + 20 && b > g {
                blue += 1; // BG
            }
            if r > b + 20 && r > g - 20 {
                orange += 1; // FG
            }
        }
    }
    let coverage = opaque as f64 / (w * h).max(1) as f64;
    eprintln!(
        "[flip-bake gate-5] tile {w}x{h} px (world {:.2}x{:.2}) | coverage {:.1}% | \
         BG-blue {blue} px + FG-orange {orange} px | bake {:.2} ms",
        wsize[0],
        wsize[1],
        coverage * 100.0,
        dt.as_secs_f64() * 1000.0
    );
    assert!(
        coverage > 0.5,
        "the composed object fills its bbox (coverage {coverage:.2})"
    );
    assert!(blue > 0, "the BG (blue) layer is present in the tile");
    assert!(
        orange > 0,
        "the FG (orange) layer is present — the two layers COMPOSED"
    );

    renderer.individual_mut().release(tid);
}

/// **The Flip twin of `motion_object_bake::select_present`** (doc 86 §9.6): a Flip object
/// is tiled iff it is NAMED or sits inside a named group — and nothing else, so unnamed
/// canvas art never wastes a tile. The three rows are the whole decision table; two
/// mutations each break a distinct row (drop the group check ⇒ the group child vanishes;
/// drop the `continue` ⇒ loose art is tiled).
#[test]
fn select_present_bakes_named_and_group_flip_objects_but_not_loose_art() {
    use ph2d_ecs::{ChildOf, GroupedChildren};
    let mut sim = SimWorld::new();
    let named = sim.world_mut().spawn((Name::new("Named"),)).id();
    let group = sim
        .world_mut()
        .spawn((Name::new("Group"), GroupedChildren))
        .id();
    let child = sim.world_mut().spawn((ChildOf(group),)).id(); // UNNAMED group child
    let loose = sim.world_mut().spawn(()).id(); // unnamed, no group

    // The map is FlipObjectId -> entity bits (what `flip_entities::sync` builds).
    let mut map = FlipEntityMap::new();
    map.insert(FlipObjectId(10), named.to_bits());
    map.insert(FlipObjectId(20), child.to_bits());
    map.insert(FlipObjectId(30), loose.to_bits());

    let present = select_present(sim.world(), &map);

    assert_eq!(
        present.get(&FlipObjectId(10)),
        Some(&Some("Named".to_string())),
        "a named Flip object is tiled, carrying its name"
    );
    assert_eq!(
        present.get(&FlipObjectId(20)),
        Some(&None),
        "an UNNAMED group child is tiled by its id, with no name"
    );
    assert!(
        !present.contains_key(&FlipObjectId(30)),
        "unnamed canvas Flip art no group references is NOT tiled"
    );
}

/// **SONDA — o que o bake faz quadro a quadro com um objeto ANIMADO** (o repro do
/// report *"=7 piscando o tempo todo"*, 2026-08-13).
///
/// Todos os modos anteriores do smoke montam um Flip de UM desenho, cuja chave de
/// conteúdo nunca muda ⇒ ele assa uma vez e nunca mais. A cena `=7` é a **primeira com
/// um Flip animado**, logo a primeira a percorrer o caminho de RE-BAKE (despejo +
/// aquisição) — e é ali que um piscar tem de ser procurado, não no offset.
///
/// Roda com `cargo test -p ph2d-host-desktop --bins probe_animated_flip_bake_churn --
/// --ignored --nocapture`.
#[test]
#[ignore = "sonda de medicao; requires a GPU adapter"]
fn probe_animated_flip_bake_churn() {
    use ph2d_ecs::{Name, SimWorld};
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to measure");
        return;
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );

    // O MESMO objeto que a cena `=7` monta: 12 fps, quatro desenhos em 0/3/6/9.
    let mut flip = FlipDoc::default();
    crate::motion_object_smoke::times::spawn_flip_walk_named(&mut flip, "Walk");
    let oid = flip.objects().first().expect("o objeto").id;

    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((Name::new("Walk"),)).id();
    let mut map = FlipEntityMap::new();
    map.insert(oid, e.to_bits());

    let mut bake = FlipObjectBake::default();
    let shifts = [0.0_f32, 0.25];
    let mut ph = Playhead::default();

    eprintln!("frame |  t   | asked | cache | tiles() | tid(agora) | tid(+0,25)");
    for f in 0..40 {
        ph.seek(f64::from(f) / 60.0);
        bake.bake(&flip, &map, &ph, &shifts, &gpu, &mut renderer, &sim);
        let tiles: Vec<_> = bake
            .tiles()
            .map(|(n, t)| (n.to_string(), t.texture_id))
            .collect();
        let now = tiles.first().map_or(u32::MAX, |(_, t)| *t);
        let shifted = bake
            .tile_named_shifted("Walk", 0.25)
            .map_or(u32::MAX, |t| t.texture_id);
        eprintln!(
            "{f:5} | {:.3} | {:5} | {:5} | {:7} | {:10} | {:10}",
            ph.time(),
            bake.asked_len_for_test(),
            bake.cache_len_for_test(),
            tiles.len(),
            now,
            shifted
        );
    }
}

/// **O despejo nunca alcança o que o quadro anterior PUBLICOU** — a lei que fecha o
/// report *"o flip do grid à esquerda pisca uma vez por quadro"* (2026-08-14).
///
/// O quadro do app tem UM de profundidade: `publish_objects` entrega os `texture_id`
/// deixados pelo bake ANTERIOR, o cook constrói as instâncias com eles, e só então o
/// bake deste quadro corre. Uma tile despejada aqui é uma tile que o desenho deste
/// quadro ainda referencia — `bind_group` devolve `None` e o lote é PULADO.
///
/// A fixture contém as TRÊS espécies, e é o terceiro que impede *"nunca despejar
/// nada"* de passar: pedida agora · publicada no quadro anterior e não mais pedida (a
/// transição, que tem de SOBREVIVER) · velha nos dois (que tem de MORRER).
#[test]
fn the_eviction_never_takes_a_tile_the_previous_frame_published() {
    let oid = FlipObjectId(7);
    let (now, leaving, stale) = ((oid, 100u64), (oid, 200u64), (oid, 300u64));

    let mut cache: BTreeMap<(FlipObjectId, u64), FlipBaked> = BTreeMap::new();
    for (k, tid) in [(now, 1u32), (leaving, 2), (stale, 3)] {
        cache.insert(
            k,
            FlipBaked {
                name: Some("Walk".to_string()),
                texture_id: tid,
                size: [1.0, 1.0],
                thumb: ph2d_panel_motion_graph::PreviewThumb {
                    rgba: std::sync::Arc::new(Vec::new()),
                    w: 0,
                    h: 0,
                },
            },
        );
    }
    let wanted: BTreeMap<(FlipObjectId, u64), Frame> = [(now, 0)].into_iter().collect();
    // O quadro ANTERIOR resolveu `now` e `leaving` — logo ESTE quadro publicou os dois.
    let published: std::collections::BTreeSet<_> = [now, leaving].into_iter().collect();

    let gone = evictable(&cache, &wanted, &published);

    assert!(
        !gone.contains(&leaving),
        "a tile que este quadro PUBLICOU sobrevive ao bake do mesmo quadro"
    );
    assert!(
        !gone.contains(&now),
        "a tile pedida agora nunca e despejada"
    );
    assert_eq!(
        gone,
        vec![stale],
        "e a carencia e de UM quadro so: o que saiu ha dois quadros MORRE"
    );
}

/// **SONDA — a tile que o quadro PUBLICOU sobrevive ao bake do MESMO quadro?** (o
/// repro do report *"o flip do grid à esquerda pisca uma vez por quadro"*, 2026-08-14).
///
/// O quadro do app corre nesta ordem: `publish_objects` (mod.rs:5969, que entrega os
/// `texture_id` deixados pelo bake do quadro ANTERIOR) → o cook, que constrói as
/// instâncias com esses ids → `bake_flip_objects` (7526, que despeja o que ninguém
/// pede mais) → o desenho. Se o bake despeja um id que o cook deste quadro já
/// referenciou, o `bind_group` devolve `None`, o lote é PULADO, e o objeto some por
/// exactamente um quadro.
///
/// A sonda reproduz essa ordem: lê o que seria publicado, roda o bake, e pergunta se
/// o id publicado ainda vive.
#[test]
#[ignore = "sonda de medicao; requires a GPU adapter"]
fn probe_the_published_tile_survives_the_bake() {
    use ph2d_ecs::{Name, SimWorld};
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to measure");
        return;
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );

    let mut flip = FlipDoc::default();
    crate::motion_object_smoke::times::spawn_flip_walk_named(&mut flip, "Walk");
    let oid = flip.objects().first().expect("o objeto").id;
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((Name::new("Walk"),)).id();
    let mut map = FlipEntityMap::new();
    map.insert(oid, e.to_bits());

    let mut bake = FlipObjectBake::default();
    let shifts = [0.0_f32, 0.25];
    let mut ph = Playhead::default();

    let (mut dead_now, mut dead_shifted) = (0usize, 0usize);
    eprintln!("frame |  t    | publicou(agora, +0,25) | vivo depois do bake");
    for f in 0..40 {
        // 1) PUBLISH — o que este quadro entrega ao cook (estado do bake anterior).
        let pub_now = bake.tile_named_shifted("Walk", 0.0).map(|t| t.texture_id);
        let pub_shift = bake.tile_named_shifted("Walk", 0.25).map(|t| t.texture_id);
        // 2) BAKE deste quadro.
        ph.seek(f64::from(f) / 60.0);
        bake.bake(&flip, &map, &ph, &shifts, &gpu, &mut renderer, &sim);
        // 3) o id publicado ainda existe na loja de texturas?
        let alive = |id: Option<u32>| id.is_none_or(|i| renderer.individual().dims(i).is_some());
        let (a_now, a_shift) = (alive(pub_now), alive(pub_shift));
        if pub_now.is_some() && !a_now {
            dead_now += 1;
        }
        if pub_shift.is_some() && !a_shift {
            dead_shifted += 1;
        }
        eprintln!(
            "{f:5} | {:.3} | {:>6?} {:>6?}         | agora {} | +0,25 {}",
            ph.time(),
            pub_now,
            pub_shift,
            if a_now { "vivo" } else { "MORTO" },
            if a_shift { "vivo" } else { "MORTO" },
        );
    }
    eprintln!(
        "\nquadros em que a tile publicada MORREU no mesmo quadro: agora {dead_now} | +0,25 {dead_shifted}"
    );
}

/// **Um desenho SEGURADO tem a mesma chave em quadros diferentes** — a propriedade
/// que a cache inteira monta em cima, e a que a 1ª versão da wave do `time_offset`
/// violou ao chavear pelo QUADRO.
///
/// O report do smoke foi *"piscando o tempo todo"*: com a chave por quadro, os
/// quadros 1 e 2 de um hold eram entradas diferentes ⇒ despejo + re-bake **doze vezes
/// por segundo** com a imagem idêntica (medido: os `texture_id` subiam sem parar).
/// *Um quadro é o que se PEDE; uma imagem é o que se ASSA* — e é a imagem que
/// endereça a tile.
#[test]
fn a_held_drawing_hashes_the_same_at_every_frame_it_is_held() {
    let (doc, oid) = two_layer_object(0.2);
    let obj = doc.objects().iter().find(|o| o.id == oid).expect("object");
    // O objeto tem UMA chave (quadro 0, hold implícito) ⇒ todo quadro mostra a mesma
    // figura, e todo quadro tem de dar a mesma chave.
    let k0 = content_key(obj, &Xform::IDENTITY, 0);
    for f in [1, 2, 5, 40, 999] {
        assert_eq!(
            content_key(obj, &Xform::IDENTITY, f),
            k0,
            "o quadro {f} segura o mesmo desenho e tem de ser a MESMA tile"
        );
    }
}

/// O CONTROLE do gate acima: desenhos diferentes dão chaves diferentes. Sem ele, um
/// `content_key` que devolvesse uma constante passaria — e a cache colapsaria toda a
/// animação numa tile só, que é o defeito oposto e igualmente invisível num gate
/// que só afirma igualdade.
#[test]
fn different_drawings_hash_differently() {
    let mut flip = FlipDoc::default();
    crate::motion_object_smoke::times::spawn_flip_walk_named(&mut flip, "Walk");
    let obj = flip.objects().first().expect("o objeto");
    let keys: std::collections::BTreeSet<u64> = [0, 3, 6, 9]
        .into_iter()
        .map(|f| content_key(obj, &Xform::IDENTITY, f))
        .collect();
    assert_eq!(
        keys.len(),
        4,
        "quatro desenhos distintos tem de dar quatro chaves distintas"
    );
}
