//! Arch gate: `readback_individual` may be called from exactly ONE shell
//! site — `hero_intents/texture_edit.rs`. Every image tool must obtain
//! individual-texture pixels through that chokepoint, which returns a
//! `SpriteImage` carrying its `AlphaMode` and is the only place a tool
//! result writes `Sprite.premultiplied` (derived from that mode).
//!
//! This prevents the bug class where a tool (Trim / Make-Square) read back
//! a premultiplied BG-Removal result, silently dropped the `premultiplied`
//! flag, and re-introduced the edge fringe. A future Image Tool that
//! reaches for `readback_individual` directly fails this test in CI.

use std::fs;
use std::path::Path;

#[test]
fn readback_individual_only_in_texture_edit_chokepoint() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    visit(&src, &mut |path: &Path, contents: &str| {
        // The chokepoint module is the one sanctioned caller.
        if path.ends_with("hero_intents/texture_edit.rs") {
            return;
        }
        for (i, line) in contents.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip doc / line comments — only flag actual call sites.
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains(".readback_individual(") {
                offenders.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "`readback_individual` must only be called from \
         hero_intents/texture_edit.rs (the alpha-mode chokepoint that \
         carries `AlphaMode` + sets `Sprite.premultiplied`). New image \
         tools must route through `read_sprite_source` / \
         `commit_edited_texture`. Offending call sites:\n{}",
        offenders.join("\n"),
    );
}

fn visit(dir: &Path, f: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(contents) = fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
