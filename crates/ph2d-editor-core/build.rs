//! build.rs — gera `OUT_DIR/icons_generated.rs` a partir de
//! `docs/design/icons/*.svg`.
//!
//! Wave 2 PR 11.2. SVGs em `docs/design/icons/` são source-of-truth
//! para os 89 glyphs do editor. Pré-Wave-2, cada glyph era portado MANUALMENTE
//! para `IconCmd` literals em `src/icons.rs::cmds()` (715 LOC de match arms).
//! Este script extrai os elementos SVG (`<path>`, `<polyline>`, `<line>`,
//! `<rect>`, `<circle>`) e emite tabelas Rust automaticamente.
//!
//! ## Parser
//!
//! SVG ad-hoc — zero build-deps. Auditoria pré-PR confirmou que os 89
//! SVGs usam apenas 5 element types (sem `<g>`, sem `transform`, sem
//! styling complexo). Strings de atributos extraídas via busca de
//! substring; valores numéricos parseados por `str::parse`.
//!
//! ## Output (em `OUT_DIR/icons_generated.rs`)
//!
//! ```rust
//! pub const ADD_CMDS: &[crate::icons::IconCmd] = &[
//!     crate::icons::IconCmd::Path("M12 5v14M5 12h14"),
//! ];
//! pub const ASSET_CMDS: &[crate::icons::IconCmd] = &[ ... ];
//! // ... 89 consts
//!
//! pub fn lookup_cmds(slug: &str) -> Option<&'static [crate::icons::IconCmd]> {
//!     match slug {
//!         "add" => Some(ADD_CMDS),
//!         "asset" => Some(ASSET_CMDS),
//!         // ... 89 arms
//!         _ => None,
//!     }
//! }
//! ```
//!
//! Consumer em `src/icons.rs`: `IconId::slug()` mapeia variant →
//! kebab-case slug; `cmds()` faz `lookup_cmds(self.slug())`.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let icons_dir = crate_root
        .parent() // crates/
        .unwrap()
        .parent() // workspace root
        .unwrap()
        .join("docs/design/icons");

    println!("cargo:rerun-if-changed={}", icons_dir.display());
    println!("cargo:rerun-if-changed=build.rs");

    let mut svgs: Vec<(String, String)> = Vec::new();
    for entry in
        fs::read_dir(&icons_dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", icons_dir.display()))
    {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("stem")
            .to_string();
        let content =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        svgs.push((slug, content));
    }
    svgs.sort_by(|a, b| a.0.cmp(&b.0));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("icons_generated.rs");
    fs::write(&out_path, emit_rust(&svgs))
        .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
}

/// One drawing command emitted as Rust source for an IconCmd literal.
enum Cmd {
    /// `IconCmd::Path("...")`
    Path(String),
    /// `IconCmd::Polyline("...")`
    Polyline(String),
    /// `IconCmd::Line(x1, y1, x2, y2)`
    Line(f32, f32, f32, f32),
    /// `IconCmd::Rect(x, y, w, h, rx)`
    Rect(f32, f32, f32, f32, f32),
    /// `IconCmd::Circle(cx, cy, r)`
    Circle(f32, f32, f32),
}

impl Cmd {
    fn to_rust(&self) -> String {
        match self {
            Self::Path(d) => format!("crate::icons::IconCmd::Path({:?})", d),
            Self::Polyline(p) => format!("crate::icons::IconCmd::Polyline({:?})", p),
            Self::Line(x1, y1, x2, y2) => format!(
                "crate::icons::IconCmd::Line({:.6}, {:.6}, {:.6}, {:.6})",
                x1, y1, x2, y2
            ),
            Self::Rect(x, y, w, h, rx) => format!(
                "crate::icons::IconCmd::Rect({:.6}, {:.6}, {:.6}, {:.6}, {:.6})",
                x, y, w, h, rx
            ),
            Self::Circle(cx, cy, r) => format!(
                "crate::icons::IconCmd::Circle({:.6}, {:.6}, {:.6})",
                cx, cy, r
            ),
        }
    }
}

/// Parse an SVG file body into a list of [`Cmd`]s in document order.
fn parse_svg(svg: &str) -> Vec<Cmd> {
    let mut cmds = Vec::new();
    // Walk character-by-character looking for `<path`, `<polyline`,
    // `<line`, `<rect`, `<circle` opens, then collect attributes until
    // matching `>`. SVGs in design/icons/ are flat (no nested groups),
    // and tags are self-closing or open-then-close immediately.
    let bytes = svg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        // Identify tag name.
        let tag_start = i + 1;
        let mut tag_end = tag_start;
        while tag_end < bytes.len() && !matches!(bytes[tag_end], b' ' | b'/' | b'>') {
            tag_end += 1;
        }
        let tag = &svg[tag_start..tag_end];
        let close_idx = svg[tag_end..]
            .find('>')
            .map(|n| tag_end + n)
            .unwrap_or(bytes.len());
        let attrs = &svg[tag_end..close_idx];

        match tag {
            "path" => {
                if let Some(d) = attr(attrs, "d") {
                    cmds.push(Cmd::Path(d));
                }
            }
            "polyline" => {
                if let Some(p) = attr(attrs, "points") {
                    cmds.push(Cmd::Polyline(p));
                }
            }
            "line" => {
                let x1 = attr_f32(attrs, "x1").unwrap_or(0.0);
                let y1 = attr_f32(attrs, "y1").unwrap_or(0.0);
                let x2 = attr_f32(attrs, "x2").unwrap_or(0.0);
                let y2 = attr_f32(attrs, "y2").unwrap_or(0.0);
                cmds.push(Cmd::Line(x1, y1, x2, y2));
            }
            "rect" => {
                let x = attr_f32(attrs, "x").unwrap_or(0.0);
                let y = attr_f32(attrs, "y").unwrap_or(0.0);
                let w = attr_f32(attrs, "width").unwrap_or(0.0);
                let h = attr_f32(attrs, "height").unwrap_or(0.0);
                let rx = attr_f32(attrs, "rx").unwrap_or(0.0);
                cmds.push(Cmd::Rect(x, y, w, h, rx));
            }
            "circle" => {
                let cx = attr_f32(attrs, "cx").unwrap_or(0.0);
                let cy = attr_f32(attrs, "cy").unwrap_or(0.0);
                let r = attr_f32(attrs, "r").unwrap_or(0.0);
                cmds.push(Cmd::Circle(cx, cy, r));
            }
            _ => {} // <svg>, </svg>, comments, etc.
        }
        i = close_idx + 1;
    }
    cmds
}

/// Extract `name="value"` from an attribute string. Returns None if
/// the attribute is missing.
fn attr(attrs: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = attrs.find(&needle)?;
    let after = &attrs[start + needle.len()..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn attr_f32(attrs: &str, name: &str) -> Option<f32> {
    attr(attrs, name).and_then(|v| v.parse().ok())
}

/// Convert `"my-slug-name"` → `"MY_SLUG_NAME"` for use as Rust const
/// identifier.
fn slug_to_const(slug: &str) -> String {
    slug.replace('-', "_").to_uppercase()
}

/// Emit Rust source for `icons_generated.rs`.
fn emit_rust(svgs: &[(String, String)]) -> String {
    let mut s = String::new();
    s.push_str(
        "// AUTO-GENERATED by build.rs from `docs/design/icons/*.svg`.\n\
         // DO NOT EDIT BY HAND — changes here are overwritten every build.\n\
         // Source-of-truth: `docs/design/icons/<slug>.svg` (Wave 2 PR 11.2).\n\
         \n",
    );

    // Per-icon const arrays.
    for (slug, content) in svgs {
        let cmds = parse_svg(content);
        let const_name = format!("{}_CMDS", slug_to_const(slug));
        s.push_str(&format!(
            "/// Drawing commands for the `{}` glyph. Generated from \
             `docs/design/icons/{}.svg`.\n",
            slug, slug
        ));
        s.push_str(&format!(
            "pub const {}: &[crate::icons::IconCmd] = &[\n",
            const_name
        ));
        for cmd in &cmds {
            s.push_str(&format!("    {},\n", cmd.to_rust()));
        }
        s.push_str("];\n\n");
    }

    // Aggregate array indexable by `IconId as usize` — assumes enum
    // variants in icons.rs are declared in the SAME alphabetical order
    // as svgs[] (which is sorted by slug). icons.rs ships a regression
    // test pinning that contract.
    s.push_str(
        "/// All icon command lists in alphabetical-slug order, indexable\n\
         /// by `IconId as usize`. Consumer: `IconId::cmds()`.\n\
         pub const ICON_CMDS_BY_ID: &[&[crate::icons::IconCmd]] = &[\n",
    );
    for (slug, _) in svgs {
        let const_name = format!("{}_CMDS", slug_to_const(slug));
        s.push_str(&format!("    {},  // {}\n", const_name, slug));
    }
    s.push_str("];\n\n");

    // Lookup function by slug string.
    s.push_str(
        "/// Lookup the drawing commands for an icon by its kebab-case\n\
         /// slug. Returns `None` if the slug isn't a known canonical icon.\n\
         pub fn lookup_cmds(slug: &str) -> Option<&'static [crate::icons::IconCmd]> {\n\
         \x20   match slug {\n",
    );
    for (slug, _) in svgs {
        let const_name = format!("{}_CMDS", slug_to_const(slug));
        s.push_str(&format!("        {:?} => Some({}),\n", slug, const_name));
    }
    s.push_str(
        "        _ => None,\n\
         \x20   }\n\
         }\n\
         \n\
         /// All canonical icon slugs in alphabetical order. Used by\n\
         /// integration tests + future chrome derivation. Order matches\n\
         /// `ICON_CMDS_BY_ID` — both are indexable by `IconId as usize`.\n\
         pub const ALL_ICON_SLUGS: &[&str] = &[\n",
    );
    for (slug, _) in svgs {
        s.push_str(&format!("    {:?},\n", slug));
    }
    s.push_str("];\n");

    s
}
