//! W0 stub for the `smoke_fixture_renderable` gate
//! (Sprite_projeto §15.8.2). The spec line:
//!
//! > **Gate `smoke_fixture_renderable`** (W0.T0.X cria): cada
//! > fixture carrega no `./play.command` sem panic; goldens existem
//! > em `smoke_goldens/`. Sem isso, smoke é teatral.
//!
//! In W0 the actual scene fixtures don't exist yet (W2-W5 schema
//! features required to author them); the gate is reduced to a
//! **directory-presence contract** — both
//! `assets/smoke_fixtures/sprite_inspector_v2/` and
//! `docs/Sprite_projeto/smoke_goldens/` exist and carry a README
//! enumerating the wave-by-wave fixture roadmap.
//!
//! As each wave lands its `.scene` fixture + PNG goldens, the gate
//! grows to actually load the scenes through `./play.command`
//! headless and bit-compare against the goldens.

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-render; pop twice to reach
    // the workspace root deterministically.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate is two levels below workspace root")
        .to_path_buf()
}

#[test]
fn smoke_fixture_dir_canonical_path_exists() {
    let dir = workspace_root().join("assets/smoke_fixtures/sprite_inspector_v2");
    assert!(
        dir.is_dir(),
        "spec §15.8.2 canonical smoke fixture dir missing: {dir:?}"
    );
    let readme = dir.join("README.md");
    assert!(
        readme.is_file(),
        "smoke fixture dir README enumerating the wave roadmap missing: {readme:?}"
    );
}

#[test]
fn smoke_goldens_dir_canonical_path_exists() {
    let dir = workspace_root().join("docs/Sprite_projeto/smoke_goldens");
    assert!(
        dir.is_dir(),
        "spec §15.8.2 canonical goldens dir missing: {dir:?}"
    );
    let readme = dir.join("README.md");
    assert!(
        readme.is_file(),
        "goldens dir README enumerating the per-wave PNG list missing: {readme:?}"
    );
}

#[test]
#[ignore = "W2.T2.X smoke fixture: smoke_w2_color_tint.scene + 5 PNG goldens. Un-ignore + replace body when the W2 wave lands its scene + per_corner_tint + self_tint + tint_fill + opacity features."]
fn w2_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!("W2.T2.X populates this gate");
}

#[test]
#[ignore = "W3.T3.X smoke fixture: smoke_w3_sorting.scene + 5 PNG goldens. Depends on SortingLayer / ZIndexOverride / YSort / SortingGroup / ShowBehindParent landing in W3."]
fn w3_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!("W3.T3.X populates this gate");
}

#[test]
#[ignore = "W4.T4.X smoke fixture: smoke_w4_material_animation.scene + 3 PNG goldens. Depends on Material + UseParentMaterial + InstanceShaderParams + SpriteAnimator."]
fn w4_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!("W4.T4.X populates this gate");
}

#[test]
#[ignore = "W5.T5.X smoke fixture: smoke_w5_named_anchors.scene + 4 PNG goldens. Depends on NamedAnchorList (socket / slice / 9slice) + per-frame override."]
fn w5_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!("W5.T5.X populates this gate");
}
