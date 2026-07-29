//! **The Expression modal — state and routing** (plano 10 W1).
//!
//! The card that replaces the inline formula field with the three degrees of
//! discovery the plan asks for: PICK from a searchable gallery, TUNE on a sheet of
//! rows, and READ the formula the sheet produces.
//!
//! ⚠️ **It lives in the PANEL, not the shell — a correction the construction made
//! to plano 10 §5.6.** The plan named `shells/desktop` with the onion card as the
//! precedent. But this panel already owns the thing being replaced
//! (the inline formula field, which also floated at a click position), already owns the
//! `SetBindingExpr` channel (`state::push_intent` -> the shell's `drain_intents`),
//! and can depend on the leaf `ph2d-expr-recipes` directly — so the gallery needs
//! no snapshot plumbing at all. Putting it in the shell would have bought a new
//! chrome z-slot and a view-snapshot type to carry 55 labels across a crate
//! boundary, for nothing.
//!
//! ⚠️ **The model is the [`RecipeStack`]; the widgets are a view of it.** The
//! store holds committed numbers and text, and [`sync_from_store`] reads them back
//! into the stack ONCE per frame, before anything is painted or projected — so
//! `to_formula()` and every result readout describe the same instant. The onion
//! modal does the same thing one crate up; the difference is only who owns the
//! model.
//!
//! ⚠️ **A structural change RESEEDS the widgets.** Row widgets are keyed by row
//! INDEX (see `ids::expr_knob_id`), so removing row 1 slides row 2 into its
//! widgets — and without the reseed the new occupant would inherit the old
//! tenant's numbers. `reseed` makes the next paint clobber instead of
//! `register_if_absent`.

use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, WidgetEvent, WidgetStore,
};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_expr_recipes::{CATALOG, Family, KnobKind, KnobValue, RecipeStack, Row, by_id, fmt_num};
use ph2d_timeline::{AnimTarget, TimelineIntent};

use crate::ids;
use crate::state::{self, TimelinePanelState};

/// Which face the gallery is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GalleryPage {
    /// The nine family rows.
    Families,
    /// One family's recipes, plus the row that walks back out.
    Family(Family),
}

/// The open card.
#[derive(Clone, Debug)]
pub struct ExprModal {
    /// The binding this authors.
    pub target: u64,
    /// `"Ball · Position Y"` — what the title band says.
    pub title: String,
    /// Top-left of the card. ⚠️ `None` until the first paint CENTRES it in the
    /// viewport — the panel that opens the card does not know how big the window
    /// is, and the first smoke found the card pinned half off the bottom of the
    /// screen because it opened at the click instead.
    pub pos: Option<(f32, f32)>,
    /// Card position and pointer at the start of a title-band drag.
    pub drag: Option<(f32, f32, f32, f32)>,
    pub page: GalleryPage,
    pub stack: RecipeStack,
    /// The clock the result readouts evaluate at — the playhead when the card was
    /// opened. ⚠️ Frozen on purpose: a readout that moved while the artist drags a
    /// knob could not be attributed to the knob.
    pub time: f64,
    /// Next paint clobbers the row widgets instead of seeding them (see the
    /// module docs).
    pub reseed: bool,
    /// First paint has run: the title and the clock are seeded from the snapshot
    /// there, because the menu that OPENS the card does not have one.
    pub opened: bool,
    /// The property being driven — what the preview's puppet follows.
    pub prop: ph2d_timeline::PropKind,
    /// The preview's frame counter (see `expr_modal_preview`).
    pub preview_frame: u32,
}

/// Open the card for `target`. It places itself on the first paint (see
/// [`ExprModal::pos`]).
pub(crate) fn open(state: &mut TimelinePanelState, target: u64) {
    state.expr_modal = Some(ExprModal {
        target,
        title: String::new(),
        pos: None,
        drag: None,
        page: GalleryPage::Families,
        stack: RecipeStack::new(),
        time: 0.0,
        reseed: true,
        opened: false,
        prop: ph2d_timeline::PropKind::TranslationX,
        preview_frame: 0,
    });
}

/// Close without authoring anything.
///
/// ⚠️ Cancel and the close X are the SAME thing. A card you can dismiss two ways
/// that dismisses differently is a card you cannot learn.
pub(crate) fn cancel(state: &mut TimelinePanelState) {
    state.expr_modal = None;
    // The live preview dies WITH the card, at every door that closes one — the scene
    // must not keep running a formula nobody can see or stop.
    state::set_expr_live(None);
}

/// Commit: the sheet's formula becomes the property's expression.
pub(crate) fn commit(state: &mut TimelinePanelState) {
    let Some(m) = state.expr_modal.take() else {
        return;
    };
    state::set_expr_live(None);
    let src = m.stack.to_formula();
    let trimmed = src.trim();
    // An empty sheet projects to `"value"` — the identity. Authoring that would
    // pin a formula that says "leave it alone", which is what NO formula already
    // means, so it clears instead.
    let expr = (!trimmed.is_empty() && trimmed != ph2d_expr_recipes::SEED_VALUE)
        .then(|| trimmed.to_string());
    state::push_intent(TimelineIntent::SetBindingExpr {
        target: AnimTarget::new(m.target),
        expr,
    });
}

/// What the stack produces at and including row `i`, formatted for the readout.
///
/// ⚠️ Evaluated through `ph2d_expr::eval` — the PRODUCT's evaluator (plano 10 P3).
/// A second little interpreter for readouts is precisely the *seed ≠ sample*
/// family of bug this codebase has paid for repeatedly.
#[must_use]
pub(crate) fn row_result(stack: &RecipeStack, i: usize, time: f64, base: f32) -> String {
    // ⚠️ **A recusa é VISÍVEL.** Uma linha que espera um alvo é inerte no fold
    // (`Row::waiting_for`, a porta única), e sem esta linha a única pista disso seria
    // um número que não muda — o artista escolhe `Follow`, nada acontece, e não há
    // nada na tela dizendo por quê. Antes desta wave era pior: a linha inacabada
    // descia como `0` e TELEPORTAVA o objeto.
    //
    // ⚠️ A linha NÃO é dimmed: o knob vazio é exatamente o que ele precisa clicar, e
    // um controle apagado que ainda despacha mente. O readout fala, o resto fica vivo.
    if stack.rows.get(i).and_then(Row::waiting_for).is_some() {
        return ph2d_i18n::tr("panel.timeline.expr_waiting").to_string();
    }
    let partial = RecipeStack {
        rows: stack.rows.iter().take(i + 1).cloned().collect(),
    };
    let src = partial.to_formula();
    let Ok(e) = ph2d_expr_parse::parse(&src) else {
        return "?".to_string();
    };
    struct B(f64, f32);
    impl ph2d_expr::Bindings for B {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "value" => self.1,
                "time" => self.0 as f32,
                _ => 0.0,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    let v = ph2d_expr::eval(&e, &B(time, base));
    if v.is_finite() {
        // Two decimals: the readout is a number the artist READS while dragging,
        // not a value anything computes from.
        const HUNDREDTHS: f32 = 100.0; // LITERAL-PX-OK: aritmetica de arredondamento, nao medida
        fmt_num((v * HUNDREDTHS).round() / HUNDREDTHS)
    } else {
        "-".to_string()
    }
}

/// Read every row widget back into the stack. Called ONCE at the top of the
/// paint, so the formula bar and every readout describe the same instant.
pub(crate) fn sync_from_store(m: &mut ExprModal, store: &WidgetStore) {
    for (ri, row) in m.stack.rows.iter_mut().enumerate() {
        let Some(rec) = by_id(row.recipe) else {
            continue;
        };
        for (ki, k) in rec.knobs.iter().enumerate() {
            let id = ids::expr_knob_id(ri, ki);
            match (k.kind, store.get(id)) {
                // ⚠️ The COMMITTED value, never the edit buffer: mid-typing the
                // buffer is `"-"` or `"0."`, and reading it would blank the formula
                // bar between two keystrokes.
                (
                    KnobKind::Number | KnobKind::Literal,
                    Some(InteractiveState::NumberInput { value, .. }),
                ) => row.knobs[ki] = KnobValue::Num(*value as f32),
                (KnobKind::Link, Some(InteractiveState::TextInput { text, .. })) => {
                    row.knobs[ki] = KnobValue::Link(text.clone());
                }
                (KnobKind::Text, Some(InteractiveState::TextInput { text, .. })) => {
                    row.knobs[ki] = KnobValue::Text(text.clone());
                }
                _ => {}
            }
        }
    }
}

/// Everything the modal answers. `None` = not ours (the caller falls through).
pub(crate) fn route(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: &WidgetEvent,
) -> Option<EventOutcome> {
    // ⚠️ The menu row comes FIRST, before the "is a card open?" guard — it is the
    // one event that arrives while there is no card, and putting it after the
    // guard is exactly how the "Expression…" row goes dead under the mouse.
    if matches!(*ev, WidgetEvent::Click(id) if id == ids::CTX_MENU_TL_EXPR) {
        return Some(crate::event_track_menu::open_expr(state, host));
    }
    let m = state.expr_modal.as_mut()?;
    match *ev {
        WidgetEvent::Click(id) if id == ids::EXPR_MODAL_CLOSE || id == ids::EXPR_MODAL_CANCEL => {
            cancel(state);
            Some(EventOutcome::Consumed)
        }
        WidgetEvent::Click(id) if id == ids::EXPR_MODAL_APPLY => {
            sync_from_store(m, host.store());
            commit(state);
            Some(EventOutcome::Consumed)
        }
        WidgetEvent::Click(id) => {
            // A gallery card: a family opens it, a recipe adds a row, `..` walks
            // back out. Resolved by RECOMPUTING the id from the catalog rather
            // than by keeping a second table of ids to drift from the first.
            if id == ids::expr_gallery_id("..") {
                m.page = GalleryPage::Families;
                return Some(EventOutcome::Consumed);
            }
            for f in Family::ALL {
                if id == ids::expr_gallery_id(f.label()) {
                    m.page = GalleryPage::Family(f);
                    return Some(EventOutcome::Consumed);
                }
            }
            for r in CATALOG {
                if id == ids::expr_gallery_id(r.id) {
                    sync_from_store(m, host.store());
                    if let Some(row) = Row::new(r.id) {
                        m.stack.push(row);
                        // ⚠️ A pair recipe inserts BOTH halves: half a circle is
                        // not a feature. (Which property each half drives is W4's
                        // question; the rows exist from the first click.)
                        if let Some(other) = r.pair
                            && let Some(o) = Row::new(other)
                        {
                            m.stack.push(o);
                        }
                    }
                    m.reseed = true;
                    return Some(EventOutcome::Consumed);
                }
            }
            // A row's bypass eye or remove button.
            for ri in 0..m.stack.rows.len() {
                if id == ids::expr_bypass_id(ri) {
                    sync_from_store(m, host.store());
                    m.stack.rows[ri].bypass = !m.stack.rows[ri].bypass;
                    return Some(EventOutcome::Consumed);
                }
                if id == ids::expr_remove_id(ri) {
                    sync_from_store(m, host.store());
                    m.stack.rows.remove(ri);
                    m.reseed = true;
                    return Some(EventOutcome::Consumed);
                }
                // The combine chip cycles, guarded by the SAME `combines()` the paint asks.
                //
                // ⚠️ The guard is HYGIENE, not correctness, and the mutation proved it:
                // removing it leaves every gate green, because `RecipeStack::to_formula`
                // does not consult `row.combine` for a modifier at all — the field would
                // move and nothing would read it. The real guarantee is in the FOLD; this
                // keeps the state from carrying a value that means nothing, and keeps the
                // chip and the router asking one question.
                if id == ids::expr_combine_id(ri) && m.stack.rows[ri].combines() {
                    sync_from_store(m, host.store());
                    m.stack.rows[ri].combine = m.stack.rows[ri].combine.next();
                    return Some(EventOutcome::Consumed);
                }
            }
            None
        }
        // Slider drags and typing land in the store; `sync_from_store` reads them
        // back. Consumed here so they never leak to another handler.
        WidgetEvent::ValueChanged(id) | WidgetEvent::Submit(id) | WidgetEvent::Blur(id) => {
            let ours = id == ids::EXPR_MODAL_SEARCH
                || (0..m.stack.rows.len()).any(|ri| {
                    by_id(m.stack.rows[ri].recipe).is_some_and(|rec| {
                        (0..rec.knobs.len()).any(|ki| ids::expr_knob_id(ri, ki) == id)
                    })
                });
            ours.then_some(EventOutcome::Consumed)
        }
        _ => None,
    }
}

/// Move the card by its title band.
///
/// The delta is taken against the position captured at `Begin`, never
/// accumulated per Update — accumulating is how a drag drifts from the pointer
/// (the lesson the exposure drag on the Flip strip paid for). The paint clamps
/// the result, so a card dragged at the edge cannot be pushed out of reach.
pub(crate) fn apply_drag(state: &mut TimelinePanelState, g: TimelineGesture) {
    let Some(m) = state.expr_modal.as_mut() else {
        return;
    };
    match g.phase {
        GesturePhase::Begin => {
            let (x, y) = m.pos.unwrap_or((g.x, g.y));
            m.drag = Some((x, y, g.x, g.y));
        }
        GesturePhase::Update => {
            if let Some((x0, y0, px, py)) = m.drag {
                m.pos = Some((x0 + (g.x - px), y0 + (g.y - py)));
            }
        }
        _ => m.drag = None,
    }
}
