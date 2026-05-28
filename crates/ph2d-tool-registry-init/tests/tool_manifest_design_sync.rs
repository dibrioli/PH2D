//! Wave 2 PR 11.5 contract test. Validates that every
//! `docs/design/tools/<slug>.toml` matches the registered
//! [`ToolManifest`] field-by-field. The TOML is the **designer's
//! canonical source** (Claude Design / Enio edits it); this test
//! catches Rust manifests that drift from the design contract.
//!
//! Comparison fields (per the Wave 2 PR 11.5 schema):
//! - `[tool]` id, cluster, zone, order, a11y_role, icon_slug,
//!   touches_sim
//! - `[label]` fluent_key (the `*_inline` fallbacks are
//!   documentation-only and deferred until Fluent activation per
//!   HR-15)
//! - `[memory_budget]` vram_mb, ram_mb, heap_script_mb
//!
//! When the test fails it prints a precise field diff so the author
//! knows exactly which value to align.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{Registry, ToolManifest, Zone};
use toml::Value;

/// Build a fresh Registry with every manifest declared in
/// `ph2d_tool_registry_init::register_all`. Re-implemented inline
/// instead of calling `register_all` directly so this test depends on
/// the same surface a peripheral agent would touch when adding a new
/// tool.
fn build_full_registry() -> Registry {
    let mut reg = Registry::default();
    ph2d_tool_registry_init::register_all(&mut reg);
    reg.build().expect("registry must build");
    reg
}

/// Workspace-root-relative path to `docs/design/tools/`. Resolved
/// from `CARGO_MANIFEST_DIR` so the test runs from any cwd.
fn tools_design_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .join("docs/design/tools")
}

/// Parse one design TOML into a flat string-keyed map of normalized
/// values. Returns `None` for files that don't exist; the test loop
/// asserts on `None` separately so a missing TOML produces a clear
/// error message naming the slug.
fn load_design(slug: &str) -> Option<BTreeMap<String, String>> {
    let path = tools_design_dir().join(format!("{slug}.toml"));
    let text = std::fs::read_to_string(&path).ok()?;
    let v: Value = toml::from_str(&text).expect("design TOML must parse");
    let mut out = BTreeMap::new();
    let tool = v.get("tool").expect("missing [tool] table");
    out.insert("id".into(), tool["id"].as_str().unwrap().into());
    out.insert("cluster".into(), tool["cluster"].as_str().unwrap().into());
    out.insert("zone".into(), tool["zone"].as_str().unwrap().into());
    out.insert(
        "order".into(),
        tool["order"].as_integer().unwrap().to_string(),
    );
    out.insert(
        "a11y_role".into(),
        tool["a11y_role"].as_str().unwrap().into(),
    );
    out.insert(
        "icon_slug".into(),
        tool["icon_slug"].as_str().unwrap().into(),
    );
    out.insert(
        "touches_sim".into(),
        tool["touches_sim"].as_bool().unwrap().to_string(),
    );
    let label = v.get("label").expect("missing [label] table");
    out.insert(
        "fluent_key".into(),
        label["fluent_key"].as_str().unwrap().into(),
    );
    let mb = v
        .get("memory_budget")
        .expect("missing [memory_budget] table");
    out.insert(
        "vram_mb".into(),
        mb["vram_mb"].as_integer().unwrap().to_string(),
    );
    out.insert(
        "ram_mb".into(),
        mb["ram_mb"].as_integer().unwrap().to_string(),
    );
    out.insert(
        "heap_script_mb".into(),
        mb["heap_script_mb"].as_integer().unwrap().to_string(),
    );
    Some(out)
}

/// Stringify a [`ToolManifest`] into the same schema so the comparison
/// is symmetric and produces an easy-to-read diff.
fn manifest_to_design_map(m: &ToolManifest) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert("id".into(), m.id.into());
    out.insert("cluster".into(), m.cluster.into());
    out.insert("zone".into(), zone_to_design(m.zone).into());
    out.insert("order".into(), m.order.to_string());
    out.insert("a11y_role".into(), role_to_design(m.a11y_role).into());
    // icon_slug isn't on the manifest yet — it's a design-side hint
    // the painter doesn't consume (manifest carries `icon_fn`, not a
    // slug). Cross-check is best-effort: TOML must reference a slug,
    // but we don't try to derive it from the icon_fn pointer.
    out.insert(
        "icon_slug".into(),
        expected_icon_slug(m.id).unwrap_or("<unknown>").into(),
    );
    out.insert("touches_sim".into(), m.touches_sim.to_string());
    out.insert("fluent_key".into(), m.label_key.into());
    out.insert("vram_mb".into(), m.memory_budget.vram_mb.to_string());
    out.insert("ram_mb".into(), m.memory_budget.ram_mb.to_string());
    out.insert(
        "heap_script_mb".into(),
        m.memory_budget.heap_script_mb.to_string(),
    );
    let _ = MemoryBudget::new(0, 0, 0); // keep the import live
    out
}

fn zone_to_design(z: Zone) -> &'static str {
    match z {
        Zone::TopLeft => "top_left",
        Zone::TopRight => "top_right",
        Zone::Sidebar => "sidebar",
        Zone::Center => "center",
    }
}

fn role_to_design(r: Role) -> &'static str {
    // Role is wide (`accesskit::Role` re-export). We only normalize
    // the cases the design schema actually emits; anything outside
    // returns `"<unsupported>"` so the diff is loud.
    if r == Role::Button {
        "Button"
    } else if r == Role::Switch {
        "Switch"
    } else if r == Role::CheckBox {
        "CheckBox"
    } else if r == Role::MenuItem {
        "MenuItem"
    } else {
        "<unsupported>"
    }
}

/// Map manifest id → expected `icon_slug` value in the matching TOML.
/// **Codegen'd by `ph2d-tool-sync`** from `docs/design/tools/*.toml`
/// — adding a new tool with a design TOML is enough; the match arms
/// regenerate. The `#[rustfmt::skip]` keeps rustfmt from reformatting
/// around the codegen markers.
#[rustfmt::skip]
fn expected_icon_slug(manifest_id: &str) -> Option<&'static str> {
    Some(match manifest_id {
        // <ph2d-tool-sync:icon-slugs:begin>
        "bgremoval" => "bg-removal",
        "color_equalization" => "color-equalization",
        "equalize_sizes" => "equalize-sizes",
        "grid_snap" => "grid-settings",
        "make_square" => "make-square",
        "padding" => "padding",
        "painter" => "painter",
        "rasterize" => "rasterize",
        "real_size" => "real-size",
        "trim_transparency" => "trim-transparency",
        "upscale" => "upscale",
        "vector_pen" => "vector-pen",
        // <ph2d-tool-sync:icon-slugs:end>
        _ => return None,
    })
}

/// Every registered manifest must have a matching `docs/design/tools/<slug>.toml`
/// and every field in that TOML must equal the manifest's field.
#[test]
fn every_registered_manifest_has_matching_design_toml() {
    let reg = build_full_registry();
    let mut missing = Vec::new();
    let mut mismatched = Vec::new();
    for m in reg.manifests() {
        let Some(design) = load_design(m.id) else {
            missing.push(m.id);
            continue;
        };
        let actual = manifest_to_design_map(m);
        for (key, design_value) in &design {
            let actual_value = actual.get(key).expect("schema parity");
            if actual_value != design_value {
                mismatched.push((
                    m.id,
                    key.clone(),
                    design_value.clone(),
                    actual_value.clone(),
                ));
            }
        }
    }
    if !missing.is_empty() || !mismatched.is_empty() {
        let mut msg = String::new();
        if !missing.is_empty() {
            msg.push_str("Manifests with no `docs/design/tools/<slug>.toml`:\n");
            for slug in missing {
                msg.push_str(&format!("  - {slug}\n"));
            }
        }
        if !mismatched.is_empty() {
            msg.push_str("Field mismatches between design TOML and registered manifest:\n");
            for (slug, key, design_value, actual_value) in mismatched {
                msg.push_str(&format!(
                    "  - {slug}.{key}: design={design_value:?} manifest={actual_value:?}\n"
                ));
            }
        }
        panic!("{msg}");
    }
}

/// Every `docs/design/tools/*.toml` must reference a manifest the
/// registry knows about. Catches the inverse drift: a designer ships
/// a TOML for a tool that's been removed or renamed.
#[test]
fn every_design_toml_has_matching_manifest() {
    let reg = build_full_registry();
    let known: std::collections::BTreeSet<&str> = reg.manifests().iter().map(|m| m.id).collect();

    let dir = tools_design_dir();
    let mut orphaned = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read docs/design/tools/") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).expect("stem");
        if !known.contains(stem) {
            orphaned.push(stem.to_string());
        }
    }
    assert!(
        orphaned.is_empty(),
        "Design TOMLs with no registered manifest (remove the file or \
         add the tool crate): {orphaned:?}"
    );
}

/// Every `icon_slug` declared in a design TOML must point at a real
/// SVG in `docs/design/icons/`. Without this, a typo in the design
/// source would render an invisible chrome pill.
#[test]
fn every_design_icon_slug_points_at_a_real_svg() {
    let icons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/design/icons");
    let reg = build_full_registry();
    let mut missing_icons = Vec::new();
    for m in reg.manifests() {
        let Some(design) = load_design(m.id) else {
            continue; // covered by the missing_design test
        };
        let icon_slug = design.get("icon_slug").expect("icon_slug field");
        let svg_path = icons_dir.join(format!("{icon_slug}.svg"));
        if !svg_path.exists() {
            missing_icons.push((m.id, icon_slug.clone()));
        }
    }
    assert!(
        missing_icons.is_empty(),
        "Design TOMLs reference SVGs that don't exist in docs/design/icons/: {missing_icons:?}"
    );
}
