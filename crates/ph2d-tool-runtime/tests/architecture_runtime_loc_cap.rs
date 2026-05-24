//! Arch-gate: `ph2d-tool-runtime` is a deliberately small helper crate.
//! HARD CAP: 500 LOC across `src/**`. Enforced from day 1 to prevent
//! this crate becoming the next god-crate successor of
//! `shells/desktop/src/render_loop/mod.rs` — the very problem Wave 10
//! Etapa 1.B was created to retire.
//!
//! Why cap from day 1 (Wave 10 v4 adversarial finding RA2):
//! "tool-runtime virar god-crate successor de render_loop/mod.rs — toda
//! lógica shell-específica (atlas repointing, recenter math, GPU max-dim
//! cap, undo snapshot) tem hora de migrar. Sem disciplina de **NÃO**
//! mover essas pro runtime, o crate cresce até ser o novo escape-hatch
//! HR-18." The cap is the discipline mechanism.
//!
//! How to raise: this is a Coordenador-A decision + DIRETRIZ note +
//! justification. Default answer: NO — push the new logic back into
//! the bridge that needs it (each tool's bridge stays free to be as
//! complex as the tool demands; the runtime stays generic-only).

use std::fs;
use std::path::PathBuf;

fn count_loc_recursive(dir: &PathBuf) -> u32 {
    let mut total: u32 = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += count_loc_recursive(&path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(content) = fs::read_to_string(&path)
            {
                // All lines including blanks and doc-comments count.
                // We're capping the FILE size budget, not "real code".
                // Doc-comments are part of the crate's surface and
                // contribute to read load just like impl lines do.
                total += content.lines().count() as u32;
            }
        }
    }
    total
}

#[test]
fn ph2d_tool_runtime_loc_under_cap() {
    const CAP: u32 = 500;
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let loc = count_loc_recursive(&src_dir);
    assert!(
        loc <= CAP,
        "ph2d-tool-runtime src/** has {loc} LOC; cap is {CAP}.\n\
         \n\
         This crate is HELPERS ONLY for the per-tool bridges. If you need\n\
         more room, the logic likely belongs in the bridge that wants it,\n\
         not here. (Wave 10 v4 adversarial finding RA2: 'tool-runtime\n\
         virar god-crate successor' — the cap is the discipline mechanism.)\n\
         \n\
         Raising the cap is a Coordenador-A decision with explicit\n\
         justification. Default answer: NO. Push the new logic back into\n\
         the bridge that needs it."
    );
}
