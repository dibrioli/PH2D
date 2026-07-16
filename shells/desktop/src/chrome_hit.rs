//! **Whose pixel is this?** — the one door between the editor's CHROME and the artwork under it.
//!
//! Split from [`crate::forwarding`] for the shell's LOC cap, along the seam the two actually have:
//! `forwarding` moves input INTO the hero; this answers *"should the canvas see this press at all?"*.
//!
//! Enio, 2026-07-16: *"os painéis de botões lateral esquerdo e superiores permitem que o clique sobre
//! alguns botões pinte a pintura atrás de si."* Every arm that can consume a Primary Down before the
//! canvas sees it asks [`pointer_over_chrome`] first — and `tests/the_chrome_swallows_the_click_it_was_given.rs`
//! keeps it that way, per ARM (a file-level check stayed green while one of three guards was deleted).

use crate::AppGfx;

/// **Is this point the editor's CHROME, rather than the artwork?** — the one door, and the pure half of
/// it (the `hit_plan` pattern: the decision is a tested function, the dispatch just asks it).
///
/// `panel` is what [`InteractionState::panel_at`] says, `widget` what [`HitIndex::hit`] says. The
/// question has **two halves and asking one is a bug that has now shipped three times**:
///
/// * `panel_at` covers panel **bodies** — the Inspector, the Hierarchy, the docked Layers/Vector panels.
///   It answers about *rects panels publish*.
/// * `hit_index` covers every interactive widget the chrome **painted, wherever it painted it**. The left
///   rail's buttons and the top bars are **not inside any panel rect**, so `panel_at` alone answers "not
///   chrome" over them — and the click falls straight through to the canvas behind. That is Enio's report
///   of 2026-07-16 (*"o clique sobre alguns botões pinta a pintura atrás de si"*), and it is the same bug
///   the C&F button was patched for one-off on 2026-07-03 (*"caiu em `painter_canvas_down` e a shape tool
///   largou um ponto no canvas atrás do botão"*) — patched **for that one id**, by hard-coding it.
///
/// ## The gizmo is NOT chrome, and that is the whole subtlety
///
/// A blanket *"the hit index claims this pixel ⇒ the UI owns it"* would be wrong and worse than the bug:
/// the gizmo is drawn **on the artwork**, around the very sprite being painted, and its handles sit on the
/// sprite's own corners and edges. Refusing there would carve dead zones out of the picture exactly where
/// the artist paints most. The gizmo answers for itself
/// ([`ph2d_editor::gizmo::is_gizmo_id`]) — the module that owns the ids owns the classification.
/// `on_artwork` is the caller's answer to *"is this id drawn ON the picture rather than beside it?"* —
/// see [`pointer_over_chrome`], which knows where to ask.
#[must_use]
pub fn chrome_claims(
    panel: Option<ph2d_editor::NodeId>,
    widget: Option<ph2d_editor::NodeId>,
    on_artwork: impl Fn(ph2d_editor::NodeId) -> bool,
) -> bool {
    if panel.is_some() {
        return true;
    }
    widget.is_some_and(|id| !on_artwork(id))
}

/// [`chrome_claims`], asked of the live hero screen at `(x, y)`.
///
/// Returns `false` with no hero (the demo's fixture mode paints no chrome, so the whole window is canvas).
///
/// ## Who is "on the artwork"
///
/// The gizmo, and **all** of it — which is two questions, not one, and getting that wrong cost Enio a
/// working brush (2026-07-16: *"o app para de pintar e o brush move a sprite"*). The gizmo registers its
/// hit rects under **two** id schemes:
///
/// * the **canonical** ids (`GIZMO_BBOX_INTERIOR`, the handles, the pivot) — what `is_gizmo_id` knows, and
///   what the PRIMARY selection paints (`keyed_handle_id(PrimaryIndividual, id) == id`);
/// * **keyed** ids — `canonical ^ hash(entity bits)` — for every EXTRA selection and for the global gizmo
///   (`register_keyed_handle`). Those are unforgeable by construction and `is_gizmo_id` cannot recognise
///   them; the only thing that knows them is the map the gizmo filled in while painting,
///   `hero.gizmo.gizmo_hit_map` — which is exactly what the pick cascade consults for the same reason.
///
/// So the door asks BOTH. Asking only `is_gizmo_id` calls every extra/global handle "chrome", the Painter
/// refuses the press, it falls through to the pick, and the artist's brush silently turns into a move
/// gesture — with the tool still saying Paint.
#[must_use]
pub fn pointer_over_chrome(gfx: Option<&AppGfx>, x: f32, y: f32) -> bool {
    let Some(hero) = gfx.and_then(|g| g.hero_screen.as_ref()) else {
        return false;
    };
    let hit = hero.hit_index.hit(x, y);
    let claimed = chrome_claims(hero.store.panel_at(x, y), hit, |id| {
        ph2d_editor::gizmo::is_gizmo_id(id) || hero.gizmo.gizmo_hit_map.contains_key(&id)
    });
    if claimed && std::env::var("PH2D_CHROME_DIAG").is_ok() {
        eprintln!(
            "[chrome] refusing the canvas at ({x:.0},{y:.0}): panel={:?} widget={:?} \
             (gizmo_id={} keyed={})",
            hero.store.panel_at(x, y),
            hit,
            hit.is_some_and(ph2d_editor::gizmo::is_gizmo_id),
            hit.is_some_and(|id| hero.gizmo.gizmo_hit_map.contains_key(&id)),
        );
    }
    claimed
}

#[cfg(test)]
mod chrome_claims_tests {
    use super::chrome_claims;
    use ph2d_editor::NodeId;
    use ph2d_editor::gizmo::ids as gz;

    /// The canonical half of "is this drawn on the artwork" — what the live door asks, minus the keyed
    /// gizmo map (which needs a painted hero). The keyed half has its own gate below.
    fn on_artwork(id: NodeId) -> bool {
        ph2d_editor::gizmo::is_gizmo_id(id)
    }

    /// A left-rail button: no panel rect (the rail is not a panel), but the hit index claims it.
    /// This is Enio's report — `panel_at` alone answers "not the UI" here and the brush paints behind
    /// the button.
    #[test]
    fn a_rail_button_is_chrome_even_though_it_is_in_no_panel() {
        let rail_button = NodeId(ph2d_editor::ids::PAINTER_RAIL_FILL.0);
        assert!(
            chrome_claims(None, Some(rail_button), on_artwork),
            "a button that belongs to no panel is still the editor's UI — the left rail and the top \
             bars publish no panel rect, and asking `panel_at` alone is what let the click through to \
             the artwork behind them"
        );
    }

    /// A panel body: the background between a panel's widgets. No widget under the cursor, but the
    /// press is still the panel's.
    #[test]
    fn a_panel_body_is_chrome_with_no_widget_under_the_cursor() {
        assert!(chrome_claims(Some(NodeId(1)), None, on_artwork));
    }

    /// **The gizmo is NOT chrome** — and this is the assertion that keeps the fix from being worse than
    /// the bug. The gizmo is drawn ON the artwork, around the very sprite being painted; its handles sit
    /// on the sprite's own corners and edges. A blanket "the hit index claims this pixel ⇒ the UI owns
    /// it" would carve dead zones out of the picture exactly where the artist paints most.
    ///
    /// The pivot dot rides along: `is_gizmo_handle_id` reports `false` for it (it begins no drag), which
    /// is why the door asks `is_gizmo_id` instead.
    #[test]
    fn the_gizmo_is_not_chrome_so_the_brush_can_paint_over_its_handles() {
        for (name, id) in [
            ("bbox interior", gz::GIZMO_BBOX_INTERIOR),
            ("corner handle", gz::GIZMO_HANDLE_TL),
            ("edge handle", gz::GIZMO_HANDLE_R),
            ("rotate region", gz::GIZMO_ROTATE_BR),
            ("pivot dot", gz::GIZMO_PIVOT),
        ] {
            assert!(
                !chrome_claims(None, Some(id), on_artwork),
                "the gizmo's {name} was called chrome, so the brush would refuse to paint there — a dead \
                 zone on the sprite's own edge, which is worse than the bug this door exists to fix"
            );
        }
    }

    /// **The gizmo's KEYED handles are on the artwork too** — the half that `is_gizmo_id` cannot see, and
    /// the one that cost Enio a working brush (2026-07-16: *"o app para de pintar e o brush move a
    /// sprite"*).
    ///
    /// Every EXTRA selection and the global gizmo register their hit rects under `canonical ^ hash(bits)`
    /// (`register_keyed_handle`) — unforgeable ids that no static table can classify. The only thing that
    /// knows them is `hero.gizmo.gizmo_hit_map`, filled in by the same paint pass, which is exactly what
    /// the pick cascade consults. A door that asks only the canonical table calls them chrome, the Painter
    /// refuses, the press falls to the pick, and the brush turns into a move gesture with the tool still
    /// saying Paint.
    ///
    /// **Mutation that must bleed:** drop the `gizmo_hit_map` half of the classifier in
    /// [`super::pointer_over_chrome`].
    #[test]
    fn a_keyed_gizmo_handle_is_on_the_artwork_even_though_no_table_knows_its_id() {
        // What the gizmo actually registers for an extra selection: the canonical id, hashed by the
        // entity's bits. Nothing static can recognise it — only the map.
        let keyed = NodeId(gz::GIZMO_BBOX_INTERIOR.0 ^ 589u64.wrapping_mul(0x_9E37_79B9_7F4A_7C15));
        assert!(
            !ph2d_editor::gizmo::is_gizmo_id(keyed),
            "fixture: this id IS in the canonical table, so it cannot show the gap it exists to show"
        );
        // The live door's classifier: the canonical table OR the map the gizmo filled in.
        let map: std::collections::BTreeMap<NodeId, ()> = [(keyed, ())].into_iter().collect();
        let live = |id: NodeId| ph2d_editor::gizmo::is_gizmo_id(id) || map.contains_key(&id);
        assert!(
            !chrome_claims(None, Some(keyed), live),
            "an extra sprite's gizmo handle was called chrome, so the Painter refuses the press and the \
             brush silently becomes a move gesture — Enio's 2026-07-16 regression, exactly"
        );
        // …and the canonical-only classifier is what got it wrong. This is the regression, pinned.
        assert!(
            chrome_claims(None, Some(keyed), on_artwork),
            "fixture: the canonical-only classifier now recognises the keyed id, so this gate no longer \
             describes the bug it was written for"
        );
    }

    /// Bare canvas: nothing claims it.
    #[test]
    fn bare_canvas_is_not_chrome() {
        assert!(!chrome_claims(None, None, on_artwork));
    }

    /// A panel body wins even when the widget under the cursor is a gizmo id — the gizmo is never drawn
    /// inside a panel, so this is a nonsense state, and the honest answer to a nonsense state is the
    /// conservative one (the press is the panel's).
    #[test]
    fn a_panel_wins_over_a_gizmo_id() {
        assert!(chrome_claims(
            Some(NodeId(1)),
            Some(gz::GIZMO_HANDLE_TL),
            on_artwork
        ));
    }
}
