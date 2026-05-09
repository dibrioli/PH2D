//! M6 visual gate as an integration test (the gate calls for "100
//! PNG → load → display"; we cover the load + reload + rename
//! invariants here, leaving the actual display to the desktop shell
//! when the asset pipeline is wired into the renderer in a follow-up).
//!
//! Generates 100 procedural 4×4 PNG files in a `tempfile` directory, runs
//! them through `AssetDb::load_png_path`, then exercises the watcher
//! by mutating + renaming files.

use ph2d_asset::{Asset, AssetDb, AssetWatcher, ReloadEvent};
use std::path::PathBuf;
use std::time::Duration;

const N: u32 = 100;

fn write_solid_png(path: &std::path::Path, r: u8, g: u8, b: u8) {
    let mut img = image::RgbaImage::new(4, 4);
    for px in img.pixels_mut() {
        *px = image::Rgba([r, g, b, 255]);
    }
    image::DynamicImage::ImageRgba8(img)
        .save_with_format(path, image::ImageFormat::Png)
        .expect("write png");
}

fn unique_color(i: u32) -> (u8, u8, u8) {
    // Spread distinct colors across the 100 fixtures so no two share
    // bytes (and therefore no two share an AssetId).
    let r = ((i * 53) % 251) as u8;
    let g = ((i * 97) % 241) as u8;
    let b = ((i * 31) % 233) as u8;
    (r, g, b)
}

#[test]
fn loads_one_hundred_pngs_with_distinct_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let db = AssetDb::new();

    let mut paths = Vec::with_capacity(N as usize);
    for i in 0..N {
        let p = tmp.path().join(format!("sprite_{i:03}.png"));
        let (r, g, b) = unique_color(i);
        write_solid_png(&p, r, g, b);
        paths.push(p);
    }

    let mut ids = Vec::with_capacity(N as usize);
    for p in &paths {
        ids.push(db.load_png_path(p).expect("load"));
    }

    assert_eq!(db.len_assets(), N as usize);
    assert_eq!(db.len_paths(), N as usize);

    // Distinct content → distinct ids.
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), N as usize, "ids must be unique");

    // Every loaded asset is the expected 4×4 RGBA8.
    for (i, id) in ids.iter().enumerate() {
        let asset = db.get(id).expect("present");
        let Asset::ImageRgba8 {
            width,
            height,
            pixels,
        } = &*asset
        else {
            panic!("M6 only ships ImageRgba8");
        };
        assert_eq!((*width, *height), (4, 4));
        assert_eq!(pixels.len(), 4 * 4 * 4);
        let (r, g, b) = unique_color(i as u32);
        assert_eq!(&pixels[..4], &[r, g, b, 255]);
    }
}

#[test]
fn rename_does_not_invalidate_id_handle() {
    let tmp = tempfile::tempdir().unwrap();
    let db = AssetDb::new();

    let original = tmp.path().join("original.png");
    write_solid_png(&original, 200, 100, 50);
    let id = db.load_png_path(&original).expect("load");

    // Rename on disk. This is the "renaming não invalida handles"
    // invariant: code holding `id` must continue to resolve.
    let renamed = tmp.path().join("renamed.png");
    std::fs::rename(&original, &renamed).expect("rename");

    // No watcher in this test → db isn't notified, but the contract
    // for direct id handles is unchanged: `get(id)` still works.
    let asset = db.get(&id).expect("id still resolves after disk rename");
    let Asset::ImageRgba8 { pixels, .. } = &*asset else {
        panic!("M6 only ships ImageRgba8");
    };
    assert_eq!(&pixels[..4], &[200, 100, 50, 255]);
}

#[test]
fn watcher_emits_changed_event_on_modification() {
    let tmp = tempfile::tempdir().unwrap();
    let db = AssetDb::new();

    let path = tmp.path().join("evolving.png");
    write_solid_png(&path, 1, 1, 1);
    let initial_id = db.load_png_path(&path).expect("load");

    let watcher = AssetWatcher::watch_dir(tmp.path()).expect("watch");

    // Some platforms (macOS FSEvents) coalesce events that fire
    // within a few hundred ms of the watch starting. Let the watcher
    // settle before the mutation.
    std::thread::sleep(Duration::from_millis(150));

    write_solid_png(&path, 250, 200, 100);

    // Drain events for up to 5 s. Some editor/save patterns produce
    // multiple events per write — collect them all.
    let mut events = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        let batch = watcher.drain_blocking(Duration::from_millis(250));
        events.extend(batch);
        // Continue until we see a Changed event for our file
        // (matched by canonical filename, since macOS reports
        // /private/var/folders/... while tmp.path() is /var/folders/...).
        if events.iter().any(|e| match e {
            ReloadEvent::Changed { path: p, .. } => p.file_name() == path.file_name(),
            _ => false,
        }) {
            break;
        }
    }
    assert!(!events.is_empty(), "watcher must report the change");

    let mut new_id_seen: Option<ph2d_asset::AssetId> = None;
    for ev in &events {
        if let ReloadEvent::Changed { path: p, id, .. } = ev
            && p.file_name() == path.file_name()
        {
            new_id_seen = Some(*id);
        }
    }
    let new_id = new_id_seen.expect("Changed event for our file");
    assert_ne!(new_id, initial_id, "content changed → id changed");

    db.apply_pending(events);

    // Old handle still valid (HR-6).
    assert!(db.get(&initial_id).is_some());
    // New handle resolves to the new pixels.
    let new_asset = db.get(&new_id).expect("new asset present");
    let Asset::ImageRgba8 { pixels, .. } = &*new_asset else {
        panic!("M6 only ships ImageRgba8");
    };
    assert_eq!(&pixels[..4], &[250, 200, 100, 255]);
}

#[cfg(unix)]
#[test]
fn watcher_drops_symlink_escapes() {
    // Two directories: one watched, one outside. Inside the watched
    // dir, a symlink points at the outside dir's PNG. The watcher
    // MUST NOT emit a Changed event for the symlink — that would let
    // a malicious link drain arbitrary file contents into AssetDb.
    use std::os::unix::fs::symlink;

    let watched = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();

    // Real PNG outside the watched dir.
    let outside_png = outside.path().join("secret.png");
    write_solid_png(&outside_png, 200, 0, 0);

    // Plant a symlink inside the watched dir pointing at it.
    let escape_link = watched.path().join("escape.png");
    symlink(&outside_png, &escape_link).expect("symlink");

    // Also plant a legitimate PNG inside the watched dir as a
    // positive control — proves the watcher itself is firing events.
    let legit = watched.path().join("legit.png");
    write_solid_png(&legit, 0, 200, 0);

    let watcher = AssetWatcher::watch_dir(watched.path()).expect("watch");
    std::thread::sleep(Duration::from_millis(150));

    // Touch both files: rewrite the legit one + chmod the symlink to
    // trigger a Modify event on it.
    write_solid_png(&legit, 0, 250, 0);
    // Re-touch the symlink target via the symlink path — this will
    // generate a Modify event for `escape.png` (the link itself), but
    // canonicalization should resolve to outside the root and reject.
    write_solid_png(&escape_link, 250, 0, 0);

    let mut events = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        events.extend(watcher.drain_blocking(Duration::from_millis(250)));
        if events
            .iter()
            .any(|e| matches!(e, ReloadEvent::Changed { .. }))
        {
            break;
        }
    }

    let mut saw_legit = false;
    for ev in &events {
        if let ReloadEvent::Changed { path, .. } = ev {
            // Compare by file_name to dodge /var → /private/var on macOS.
            assert_ne!(
                path.file_name().and_then(|s| s.to_str()),
                Some("escape.png"),
                "watcher leaked symlink-escape file: {ev:?}"
            );
            if path.file_name().and_then(|s| s.to_str()) == Some("legit.png") {
                saw_legit = true;
            }
        }
    }
    assert!(
        saw_legit,
        "positive control: legit.png Changed event missing"
    );
}

#[test]
fn apply_pending_is_idempotent_on_empty_batch() {
    let db = AssetDb::new();
    db.apply_pending(Vec::<ReloadEvent>::new());
    assert_eq!(db.len_assets(), 0);
    assert_eq!(db.len_paths(), 0);
}

#[test]
fn png_bomb_oversized_dimensions_rejected() {
    // Craft a 16k × 16k PNG header — over the 8k MAX_DIMENSION limit
    // baked into ph2d-asset's loader. Real allocation would be 16384 ×
    // 16384 × 4 = 1 GiB; with limits the decoder must refuse before
    // touching that memory.
    let mut img = image::RgbaImage::new(16_384, 16_384);
    // Don't actually fill 1 GiB of test memory — `image::RgbaImage::new`
    // already does. Skip the test if allocation would fail (unlikely on
    // dev machines but graceful in resource-constrained CI runners).
    if img.as_raw().len() < (16_384 * 16_384 * 4) as usize {
        eprintln!("skip: cannot allocate 1 GiB for synthetic bomb");
        return;
    }
    for px in img.pixels_mut() {
        *px = image::Rgba([0, 0, 0, 255]);
    }
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("encode");

    let db = AssetDb::new();
    let result = db.insert_png_bytes(&buf);
    assert!(
        result.is_err(),
        "16k × 16k PNG must be rejected by image::Limits"
    );
}

#[test]
fn case_insensitive_png_extension_in_load() {
    // The watcher uses is_png_extension() to filter events; verify
    // load_png_path covers a .PNG file too. (load_png_path doesn't
    // filter by extension itself — that's the caller's job — but a
    // common Windows pattern is uppercase .PNG.)
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("upper.PNG");
    let mut img = image::RgbaImage::new(2, 2);
    for px in img.pixels_mut() {
        *px = image::Rgba([10, 20, 30, 255]);
    }
    image::DynamicImage::ImageRgba8(img)
        .save_with_format(&path, image::ImageFormat::Png)
        .unwrap();

    let db = AssetDb::new();
    let id = db.load_png_path(&path).expect("load .PNG");
    assert!(db.get(&id).is_some());
}

#[test]
fn collect_paths_is_canonical_order() {
    let tmp = tempfile::tempdir().unwrap();
    let db = AssetDb::new();
    let p_b = tmp.path().join("b.png");
    let p_a = tmp.path().join("a.png");
    let p_c = tmp.path().join("c.png");
    for (p, k) in [(&p_b, 1u8), (&p_a, 2), (&p_c, 3)] {
        write_solid_png(p, k, k, k);
        db.load_png_path(p).unwrap();
    }
    let paths = db.tracked_paths();
    let names: Vec<String> = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec!["a.png".to_string(), "b.png".into(), "c.png".into()]
    );
    let _ = (p_a, p_b, p_c, PathBuf::new()); // suppress unused warning under cold compiles
}
