#![forbid(unsafe_code)]
//! Codegen logic for `ph2d-tool-registry-init`: scan `crates/ph2d-tool-*` and
//! render the `register_all` body + the Cargo `[dependencies]` block.
//!
//! Single source of truth shared by the `ph2d-tool-sync` **binary** (which
//! rewrites the files) and the registry-init **staleness gate** (which
//! re-renders and asserts the checked-in files match). Adding a tool is then
//! dropping a crate + running the binary; the gate fails CI if someone forgot.
//!
//! This is the tool-family twin of `ph2d-node-sync` (ADR-0040 §2.3): the same
//! "drop a crate, zero central edit" mechanism the node fan-out uses. The one
//! shape difference is the registration call — a tool manifest `register`
//! returns `()`, a node `register` returns `Result`, so the rendered line has
//! no `?`.
//!
//! Pure std, line-oriented.

use std::path::Path;

/// Markers delimiting the generated `register_all` (manifests) region.
pub const RS_BEGIN: &str = "// <ph2d-tool-sync:begin>";
pub const RS_END: &str = "// <ph2d-tool-sync:end>";
/// Markers delimiting the generated `register_all_tools` (behavior /
/// `Box<dyn Tool>`) region in `src/lib.rs`.
pub const TOOLS_BEGIN: &str = "// <ph2d-tool-sync:tools:begin>";
pub const TOOLS_END: &str = "// <ph2d-tool-sync:tools:end>";
/// Markers delimiting the generated region in `Cargo.toml` (TOML comments).
pub const TOML_BEGIN: &str = "# <ph2d-tool-sync:begin>";
pub const TOML_END: &str = "# <ph2d-tool-sync:end>";
/// Markers delimiting the codegen'd canonical paint order of the
/// `image_tools` cluster (consumed by the
/// `image_tools_cluster_in_canonical_order` test). Sourced from each
/// tool's `docs/design/tools/<id>.toml` `[tool].order` field — the
/// designer-canonical source the registry already validates against.
pub const IMAGE_TOOLS_ORDER_BEGIN: &str = "// <ph2d-tool-sync:image-tools-order:begin>";
pub const IMAGE_TOOLS_ORDER_END: &str = "// <ph2d-tool-sync:image-tools-order:end>";
/// Markers delimiting the codegen'd `id -> icon_slug` match arms inside
/// `expected_icon_slug` (used by `every_design_icon_slug_points_at_a_real_svg`
/// and `every_registered_manifest_has_matching_design_toml`). Sourced from
/// each `docs/design/tools/<id>.toml` `[tool].icon_slug` field.
pub const ICON_SLUGS_BEGIN: &str = "// <ph2d-tool-sync:icon-slugs:begin>";
pub const ICON_SLUGS_END: &str = "// <ph2d-tool-sync:icon-slugs:end>";

/// Scan `crates_dir` for tool crates (`ph2d-tool-*`), excluding the registry
/// contract crate and the registry-init aggregator themselves. Returns kebab
/// crate names, sorted (the sort is the deterministic, merge-stable order the
/// `architecture_register_all_alphabetical` gate also enforces).
pub fn scan_tool_crates(crates_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crates_dir)
        .expect("crates dir readable")
        // A real crate: a directory with a Cargo.toml. Rejects stray dirs,
        // scratch folders, and bare symlinks that share the `ph2d-tool-` prefix
        // but would generate a `register` call to a crate that does not exist.
        .filter_map(Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            n.starts_with("ph2d-tool-")
                && n != "ph2d-tool-registry"
                && n != "ph2d-tool-registry-init"
                // Wave 10 / Etapa 1.B audit fix [3.1]: ph2d-tool-runtime
                // is shell-facing infra, not a tool. It bates the prefix
                // `ph2d-tool-*` (kept for namespace consistency with
                // ph2d-tool-registry / -registry-init) but exposes no
                // `pub fn register` nor `pub fn make` — without this
                // exclusion, `cargo_deps_in_sync_with_folder` would add
                // an inert `ph2d-tool-runtime` dep to
                // `ph2d-tool-registry-init/Cargo.toml` that nothing
                // actually uses.
                && n != "ph2d-tool-runtime"
        })
        .collect();
    names.sort();
    names
}

fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

/// Of `crates`, keep those whose `src/lib.rs` contains `needle`. Used to
/// split tool crates by capability: `"pub fn register"` → data-registry
/// (manifest/chrome) tools; `"pub fn make"` → behavior-registry
/// (modal/palette) tools. A crate may match both (e.g. bgremoval), one
/// (make_square = register only; brush = make only), driving which
/// generated list it lands in. Text scan, like the alphabetical gate.
pub fn filter_exposing(crates_dir: &Path, crates: &[String], needle: &str) -> Vec<String> {
    crates
        .iter()
        .filter(|c| {
            std::fs::read_to_string(crates_dir.join(c).join("src/lib.rs"))
                .map(|s| s.contains(needle))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

/// The generated `register_all` body lines (4-space indented to match the
/// function body; rustfmt leaves these untouched). Tool `register` returns
/// `()`, so no `?` — unlike the node twin.
pub fn render_register_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    {}::register(reg);", crate_ident(c)))
        .collect()
}

/// The generated `register_all_tools` body lines — one boxed `make()`
/// per behavior (modal/palette) tool crate, registered into the
/// `ToolRegistry`. Tool register returns `()`.
pub fn render_register_tools_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    reg.register({}::make());", crate_ident(c)))
        .collect()
}

/// The generated Cargo `[dependencies]` lines for the tool crates.
pub fn render_cargo_dep_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("{c} = {{ path = \"../{c}\" }}"))
        .collect()
}

/// Fields scraped from a `docs/design/tools/<id>.toml`'s `[tool]` table.
/// The design TOML is the designer-canonical source the registry already
/// validates against (`every_registered_manifest_has_matching_design_toml`);
/// we re-read it here to drive two registry-init tests that previously
/// duplicated this same data by hand — extending tool-sync to derive them
/// from the same TOML closes the fan-out friction documented in
/// `feedback_fanout_registry_init_friction.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignEntry {
    pub id: String,
    pub cluster: String,
    pub order: u32,
    pub icon_slug: String,
}

/// Parse one design TOML's `[tool]` table for the four fields we consume.
/// Pure-std line scan (matches the rest of this lib's "dependency-free
/// codegen" property). Returns `Err(missing_field_name)` if any required
/// field is absent — the caller surfaces that with the file path so a
/// malformed TOML never silently disappears from the codegen output
/// (the silent-drop variant of this parser would cause downstream
/// staleness gates to fail with a generic "lists differ" message and
/// no breadcrumb to the bad TOML).
pub fn parse_design_toml(text: &str) -> Result<DesignEntry, &'static str> {
    let mut in_tool = false;
    let mut id: Option<String> = None;
    let mut cluster: Option<String> = None;
    let mut order: Option<u32> = None;
    let mut icon_slug: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_tool = section.trim() == "tool";
            continue;
        }
        if !in_tool {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "id" => id = strip_quotes(value).map(str::to_string),
            "cluster" => cluster = strip_quotes(value).map(str::to_string),
            "icon_slug" => icon_slug = strip_quotes(value).map(str::to_string),
            "order" => order = strip_inline_comment(value).trim().parse::<u32>().ok(),
            _ => {}
        }
    }
    Ok(DesignEntry {
        id: id.ok_or("id")?,
        cluster: cluster.ok_or("cluster")?,
        order: order.ok_or("order")?,
        icon_slug: icon_slug.ok_or("icon_slug")?,
    })
}

/// Strip a trailing TOML inline comment (`key = value # comment`).
/// Quote-aware: the `#` must be OUTSIDE a quoted string, so an
/// `icon_slug = "#hash"` value is preserved verbatim. The previous
/// naive `split('#').next()` truncated such strings silently — gold-
/// standard audit catch (2026-05-24).
fn strip_inline_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    let mut in_quote: Option<u8> = None;
    for (i, &b) in bytes.iter().enumerate() {
        match (in_quote, b) {
            (None, b'"') => in_quote = Some(b'"'),
            (None, b'\'') => in_quote = Some(b'\''),
            (Some(q), c) if c == q => in_quote = None,
            (None, b'#') => return &value[..i],
            _ => {}
        }
    }
    value
}

fn strip_quotes(value: &str) -> Option<&str> {
    let value = strip_inline_comment(value).trim();
    value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
}

/// Scan `design_dir` for `*.toml` (skipping `README.md` & friends),
/// parse each, and return entries sorted alphabetically by `id`. The
/// alphabetical sort is deterministic + matches the merge-conflict-hygiene
/// convention the rest of the registry follows. Panics with the file
/// path if any TOML is malformed — the design TOMLs are checked-in
/// source-of-truth, not user input, so a parse failure is a hard error
/// that needs an explicit fix, not a silent drop from the codegen.
pub fn scan_design_tomls(design_dir: &Path) -> Vec<DesignEntry> {
    let mut entries: Vec<DesignEntry> = std::fs::read_dir(design_dir)
        .expect("design dir readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("toml"))
        .map(|e| {
            let path = e.path();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            parse_design_toml(&text).unwrap_or_else(|missing| {
                panic!(
                    "design TOML {} is missing required field `{missing}` in [tool] — \
                     ph2d-tool-sync cannot codegen without it",
                    path.display()
                )
            })
        })
        .collect();
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

/// Render the `image_tools` cluster's canonical paint order as Rust
/// string-literal lines (the body of the `expected: &[&str] = &[ ... ]`
/// array inside `image_tools_cluster_in_canonical_order`). Order is
/// `(design.order, id)` — exactly what `Registry::cluster` produces, so
/// the gate stays a pure parity check.
///
/// Indented 8 spaces to sit cleanly inside the array literal at function
/// scope depth 2 (the host file's `#[rustfmt::skip]` keeps rustfmt out).
pub fn render_image_tools_order_lines(entries: &[DesignEntry]) -> Vec<String> {
    let mut filtered: Vec<&DesignEntry> = entries
        .iter()
        .filter(|e| e.cluster == "image_tools")
        .collect();
    filtered.sort_by(|a, b| (a.order, &a.id).cmp(&(b.order, &b.id)));
    filtered
        .iter()
        .map(|e| format!("        \"{}\",", e.id))
        .collect()
}

/// Render the `id -> icon_slug` match arms (the body of the `match
/// manifest_id` block inside `expected_icon_slug`). Arms are sorted by
/// id alphabetically — the same merge-stable order the registry uses.
///
/// Indented 8 spaces to sit inside the `Some(match manifest_id { ... })`
/// block (the host file's `#[rustfmt::skip]` keeps rustfmt out). Sorts
/// internally so the function's output contract holds even if the
/// caller passes unordered entries (the production caller
/// `scan_design_tomls` already sorts; the defensive sort makes the
/// function's doc claim self-enforcing).
pub fn render_icon_slug_lines(entries: &[DesignEntry]) -> Vec<String> {
    let mut sorted: Vec<&DesignEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    sorted
        .iter()
        .map(|e| format!("        \"{}\" => \"{}\",", e.id, e.icon_slug))
        .collect()
}

/// Replace the lines strictly between the `begin` and `end` marker lines with
/// `body_lines`, preserving the marker lines themselves (and everything else).
/// Marker match is on the trimmed line, so indentation is irrelevant.
pub fn splice_lines(content: &str, begin: &str, end: &str, body_lines: &[String]) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let count = |marker: &str| lines.iter().filter(|l| l.trim() == marker.trim()).count();
    assert_eq!(
        count(begin),
        1,
        "begin marker must appear exactly once: {begin}"
    );
    assert_eq!(count(end), 1, "end marker must appear exactly once: {end}");
    let bi = lines
        .iter()
        .position(|l| l.trim() == begin.trim())
        .expect("begin marker present");
    let ei = lines
        .iter()
        .position(|l| l.trim() == end.trim())
        .expect("end marker present");
    assert!(bi < ei, "begin marker must precede end marker");

    let mut out: Vec<String> = Vec::new();
    out.extend(lines[..=bi].iter().map(|s| s.to_string()));
    out.extend(body_lines.iter().cloned());
    out.extend(lines[ei..].iter().map(|s| s.to_string()));
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idents_replace_hyphens() {
        assert_eq!(crate_ident("ph2d-tool-bgremoval"), "ph2d_tool_bgremoval");
    }

    #[test]
    fn register_lines_have_no_question_mark() {
        // Tool register returns (), unlike node register's Result.
        let lines = render_register_lines(&["ph2d-tool-padding".to_string()]);
        assert_eq!(lines, vec!["    ph2d_tool_padding::register(reg);"]);
    }

    #[test]
    fn splice_replaces_between_markers_only() {
        let content =
            "head\n    // <ph2d-tool-sync:begin>\n    OLD;\n    // <ph2d-tool-sync:end>\ntail\n";
        let out = splice_lines(content, RS_BEGIN, RS_END, &["    NEW;".to_string()]);
        assert_eq!(
            out,
            "head\n    // <ph2d-tool-sync:begin>\n    NEW;\n    // <ph2d-tool-sync:end>\ntail\n"
        );
    }

    #[test]
    fn splice_is_idempotent() {
        let content = "head\n# <ph2d-tool-sync:begin>\na = 1\n# <ph2d-tool-sync:end>\ntail\n";
        let once = splice_lines(content, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        let twice = splice_lines(&once, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        assert_eq!(once, twice);
    }

    #[test]
    fn parse_design_toml_extracts_tool_fields() {
        let text = r#"
# leading comment
[tool]
id          = "bgremoval"
cluster     = "image_tools"
zone        = "top_right"
order       = 60
a11y_role   = "Button"
icon_slug   = "bg-removal"
touches_sim = false

[label]
fluent_key  = "tool.bgremoval.label"
"#;
        let entry = parse_design_toml(text).expect("parses");
        assert_eq!(entry.id, "bgremoval");
        assert_eq!(entry.cluster, "image_tools");
        assert_eq!(entry.order, 60);
        assert_eq!(entry.icon_slug, "bg-removal");
    }

    #[test]
    fn parse_design_toml_ignores_other_sections() {
        // `id` outside [tool] must not leak into the result.
        let text = r#"
[label]
id = "wrong"

[tool]
id = "real"
cluster = "image_tools"
order = 1
icon_slug = "real-icon"
"#;
        let entry = parse_design_toml(text).expect("parses");
        assert_eq!(entry.id, "real");
    }

    #[test]
    fn parse_design_toml_preserves_hash_inside_quoted_string() {
        // Regression for the audit catch: the previous `split('#').next()`
        // truncated quoted strings containing `#`. Quote-aware comment
        // stripping must keep the literal value intact.
        let text = r#"
[tool]
id = "with_hash"
cluster = "image_tools"
order = 1
icon_slug = "color-#7f3f"
"#;
        let entry = parse_design_toml(text).expect("parses");
        assert_eq!(entry.icon_slug, "color-#7f3f");
    }

    #[test]
    fn parse_design_toml_strips_inline_comment_after_integer() {
        // `order = 60  # canonical paint position` must parse as 60.
        let text = r#"
[tool]
id = "ok"
cluster = "image_tools"
order = 60  # canonical paint position (Wave 2 PR 11.4)
icon_slug = "ok"
"#;
        let entry = parse_design_toml(text).expect("parses");
        assert_eq!(entry.order, 60);
    }

    #[test]
    fn parse_design_toml_names_missing_field() {
        // No silent None on malformed: the caller gets the field name
        // so `scan_design_tomls` can panic with the file path.
        let text = "[tool]\nid = \"foo\"\ncluster = \"image_tools\"\norder = 1\n";
        let err = parse_design_toml(text).expect_err("missing icon_slug");
        assert_eq!(err, "icon_slug");
    }

    #[test]
    fn render_image_tools_order_filters_and_sorts() {
        let entries = vec![
            DesignEntry {
                id: "upscale".into(),
                cluster: "image_tools".into(),
                order: 120,
                icon_slug: "upscale".into(),
            },
            DesignEntry {
                id: "grid_snap".into(),
                cluster: "topbar_settings".into(),
                order: 10,
                icon_slug: "grid-settings".into(),
            },
            DesignEntry {
                id: "trim_transparency".into(),
                cluster: "image_tools".into(),
                order: 40,
                icon_slug: "trim-transparency".into(),
            },
        ];
        let lines = render_image_tools_order_lines(&entries);
        assert_eq!(
            lines,
            vec![
                "        \"trim_transparency\",".to_string(),
                "        \"upscale\",".to_string(),
            ],
            "grid_snap (wrong cluster) excluded; image_tools sorted by order"
        );
    }

    #[test]
    fn render_icon_slug_emits_alphabetical_match_arms() {
        let entries = vec![
            DesignEntry {
                id: "upscale".into(),
                cluster: "image_tools".into(),
                order: 120,
                icon_slug: "upscale".into(),
            },
            DesignEntry {
                id: "bgremoval".into(),
                cluster: "image_tools".into(),
                order: 60,
                icon_slug: "bg-removal".into(),
            },
        ];
        let lines = render_icon_slug_lines(&entries);
        assert_eq!(
            lines,
            vec![
                "        \"bgremoval\" => \"bg-removal\",".to_string(),
                "        \"upscale\" => \"upscale\",".to_string(),
            ],
        );
    }
}
