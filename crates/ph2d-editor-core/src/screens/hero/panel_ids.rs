//! A identidade dos painéis do Hero — o mapa de visibilidade default e a
//! canonicalização das chaves. Irmão de `hero.rs` cortado na integração de
//! 2026-08-23: as linhas `Sprite` e `Vector` apendaram campos ao mesmo arquivo
//! e a união passou o teto de 700 LOC (706) — o corte segue a fronteira que já
//! existia por dentro: *quem é um painel* ≠ *o que o ecrã faz com ele*.

/// Build the default per-panel visibility map for a fresh
/// `HeroScreen`. Inspector + Hierarchy visible by default; floating
/// panels (Widget Gallery, Grid Snap) hidden.
pub(super) fn default_panel_visibility() -> std::collections::BTreeMap<&'static str, bool> {
    let mut map = std::collections::BTreeMap::new();
    map.insert("inspector", true);
    map.insert("hierarchy", true);
    map.insert("widget_gallery", false);
    map.insert("grid_snap", false);
    map.insert("timeline", false);
    map
}

/// **The two panel ids the Motion dock is a conversation between** (W4.T4).
///
/// The shell's Motion bridge WRITES these keys into `panel_visibility`; the hero's paint READS
/// them to decide whether to carve the timeline's band out of the graph. A string written on one
/// side and typed again on the other is two doors to the same question, and two doors diverge
/// ([[feedback_two_doors_to_the_same_question_diverge]]) — silently, because a missing key just
/// reads as `false` and the feature simply never happens.
pub const PANEL_MOTION_GRAPH: &str = "motion_graph";
pub const PANEL_TIMELINE: &str = "timeline";

/// Canonical `&'static str` for known panel ids — keeps the
/// visibility HashMap keys stable across calls without leaking.
pub(super) fn canonical_panel_id(id: &str) -> Option<&'static str> {
    match id {
        "inspector" => Some("inspector"),
        "hierarchy" => Some("hierarchy"),
        "widget_gallery" => Some("widget_gallery"),
        "grid_snap" => Some("grid_snap"),
        PANEL_TIMELINE => Some(PANEL_TIMELINE),
        // O painel da cena 3D (ADR-0150 W12). A ponte do shell o abre no frame
        // em que a escultura nasce, então a chave é escrita a cada sessão de
        // smoke — vazá-la pelo `Box::leak` seria uma alocação por processo, o
        // que é barato e mesmo assim errado quando o nome é conhecido.
        "sculpt3d" => Some("sculpt3d"),
        // O painel de MODELAGEM 3D (ADR-0161 W4) — o irmão do de cima. Sem esta entrada o
        // `set_panel_visible` cai no `Box::leak`, o que funciona e mesmo assim é errado
        // quando o nome é conhecido.
        "model3d" => Some("model3d"),
        _ => None,
    }
}
