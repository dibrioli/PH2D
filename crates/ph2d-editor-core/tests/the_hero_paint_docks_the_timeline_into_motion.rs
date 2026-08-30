//! **Arch-gate: the hero's paint must actually CALL the dock** (W4.T4).
//!
//! `HeroLayout::dock_timeline_into_motion` is a pure function with four gates of its own — and I
//! deleted its *call site* and every one of them stayed green. Of course they did: they test the
//! function, and the function was still correct. What died was the feature.
//!
//! That is [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] — the surviving mutation
//! did not mean the gates were loose, it meant a gate was MISSING: nothing said *"and somebody has
//! to call this"*. A paint gate would be the honest answer, but the hero screen is not a `Panel`
//! and has no headless seam; so this reads the product's source, exactly like the z-projection's
//! frame-order gate does, and asserts the three things that can silently break:
//!
//! 1. the call exists;
//! 2. it is guarded by BOTH visibility flags (docking with the timeline hidden would carve a band
//!    for a panel nobody can see, and the graph would just be short);
//! 3. the guard uses the shared CONSTS, not typed-again string literals — a `panel_visibility` miss
//!    reads as `false`, so a typo does not error, it just quietly never docks.

use std::path::Path;

/// ⛔⛔ **O ficheiro que responde a esta pergunta MUDA, e o gate tem de o PROCURAR.**
///
/// Ele nomeava `src/screens/hero/paint.rs`. Em 2026-08-30 o tecto de LOC obrigou a cortar dali o
/// bloco da geometria para o `frame_layout.rs` — *pure code motion*, produto intacto — e este gate
/// reprovou a dizer *«ninguém chama a docagem»*. A acusação era falsa e a cura de a calar seria
/// mudar o nome do ficheiro, que é a mesma dívida um ano depois.
///
/// ⚠️ **A propriedade é «alguém no hero chama isto, dentro da guarda», não «este ficheiro chama».**
/// ⇒ varre-se o directório e procura-se o ficheiro que contém a chamada; zero ficheiros é a falha.
fn hero_sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/screens/hero");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("o directório do hero existe") {
        let path = entry.expect("entrada legível").path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(src) = std::fs::read_to_string(&path)
        {
            out.push((path.display().to_string(), src));
        }
    }
    out
}

#[test]
fn the_hero_paint_docks_the_timeline_into_motion() {
    let files = hero_sources();
    let (name, src) = files
        .iter()
        .find(|(_, s)| s.contains("dock_timeline_into_motion()"))
        .expect(
            "nobody in screens/hero calls the dock: the timeline would go on painting over the \
             node graph, and every layout gate would stay green about it",
        );

    assert!(
        src.contains("PANEL_MOTION_GRAPH") && src.contains("PANEL_TIMELINE"),
        "{name}: the dock must be guarded by BOTH panel flags, and by the shared consts - a \
         re-typed string key that misses reads as `false`, so the feature just never happens"
    );
    // The guard and the call are one thought: the call must sit inside the `if`.
    let guard = src.find("PANEL_MOTION_GRAPH").expect("checked above");
    let call = src
        .find("dock_timeline_into_motion()")
        .expect("checked above");
    assert!(
        guard < call && call - guard < 200,
        "{name}: the call must be INSIDE the visibility guard, not merely near it"
    );
}
