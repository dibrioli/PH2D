//! Vector tool ⟷ shell bridge (ADR-0108 cutover).
//!
//! Per-frame jobs (mirror of the Padding / Painter panel bridges):
//!
//! 1. **Panel visibility** — show the docked `ph2d-panel-vector` Style panel
//!    (right dock) iff the `vector` tool is active; hide the real Inspector
//!    (edge-triggered) so they don't both claim the slot.
//! 2. **Picker read-back** — a Down on the Stroke / Fill swatch opened the
//!    shared OKLCH picker (generic `is_picker_swatch` dispatch). While the
//!    picker targets one of our swatches, feed the live picked colour into the
//!    tool via [`VectorTool::set_stroke_rgba`] / [`VectorTool::set_fill_rgba`].
//! 3. **Style sync** — copy the tool's stroke / fill / width into the Pen so
//!    newly drawn paths honour the Style.
//! 4. **Recolour selected** — when a colour changed (`take_apply_to_selected`),
//!    recolour the selected path. ONE undo step per gesture: a picker drag
//!    commits on close; a discrete pick (Fill "None") commits the same frame.
//! 5. **Publish** — sync the swatches' `widget_color` to the live colour (seeds
//!    the picker on open) + publish the Style snapshot the panel paints.
//!
//! The concrete-tool downcast lives HERE (allowlisted:
//! `architecture_no_downcast_to_concrete_tool_in_shell`), so the central render
//! loop stays downcast-free — mirror of `painter_bridge`.

use ph2d_editor::{HeroScreen, ToolId, ToolRegistry};
use ph2d_tool_vector::VectorDrawConfig;
use ph2d_vec_edit::{History, PenStyle, PenTool, ShapeTool};
use ph2d_vec_render::GradHandle;
use ph2d_vec_scene::{LineCap, LineJoin, Paint, Rgba8, StrokeSpec, VecScene};
use std::cell::RefCell;

fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
}

/// The colour the selected gradient handle addresses on `fill` — a multi-point
/// point's colour, or the ramp stop at the START/END end the linear/radial handle
/// sits on. Drives the Fill swatch, the picker seed, and the recolour. `None` if the
/// handle doesn't match the fill kind (e.g. a stale selection after a kind switch).
fn selected_grad_color(fill: &Paint, handle: GradHandle) -> Option<Rgba8> {
    match (fill, handle) {
        (Paint::MultiPoint { points }, GradHandle::Point(i)) => points.get(i).map(|gp| gp.color),
        (Paint::Linear { stops, .. }, GradHandle::LinearStart)
        | (Paint::Radial { stops, .. }, GradHandle::RadialCenter) => stops.first().map(|s| s.color),
        (Paint::Linear { stops, .. }, GradHandle::LinearEnd)
        | (Paint::Radial { stops, .. }, GradHandle::RadialEdge) => stops.last().map(|s| s.color),
        (Paint::Linear { stops, .. }, GradHandle::Stop(i))
        | (Paint::Radial { stops, .. }, GradHandle::Stop(i)) => stops.get(i).map(|s| s.color),
        _ => None,
    }
}

/// Recolour the slot the selected gradient handle addresses to `c`; returns whether
/// it changed. Mirror of [`selected_grad_color`] on the mutable fill.
fn set_selected_grad_color(fill: &mut Paint, handle: GradHandle, c: Rgba8) -> bool {
    let slot = match (fill, handle) {
        (Paint::MultiPoint { points }, GradHandle::Point(i)) => {
            points.get_mut(i).map(|gp| &mut gp.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::LinearStart)
        | (Paint::Radial { stops, .. }, GradHandle::RadialCenter) => {
            stops.first_mut().map(|s| &mut s.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::LinearEnd)
        | (Paint::Radial { stops, .. }, GradHandle::RadialEdge) => {
            stops.last_mut().map(|s| &mut s.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::Stop(i))
        | (Paint::Radial { stops, .. }, GradHandle::Stop(i)) => {
            stops.get_mut(i).map(|s| &mut s.color)
        }
        _ => None,
    };
    match slot {
        Some(slot) if *slot != c => {
            *slot = c;
            true
        }
        _ => false,
    }
}

/// Push `alpha` (0..255) onto an Opacity slider's stored value — unless the user
/// is dragging it — so a colour-picker alpha change reflects on the panel and the
/// drag baseline stays correct. The linked chip's display is driven from the
/// slider track in `paint`, so it follows without a separate push.
fn sync_opacity_slider(
    store: &mut ph2d_editor::interaction::WidgetStore,
    id: ph2d_editor::NodeId,
    alpha: u8,
) {
    use ph2d_editor::InteractiveState;
    use ph2d_editor::widget::SliderState;
    if let Some(InteractiveState::Slider { state, value, .. }) = store.get_mut(id)
        && !matches!(*state, SliderState::Dragging)
    {
        *value = f32::from(alpha) / 255.0;
    }
}

thread_local! {
    /// Pre-image of the scene captured at the START of a recolour gesture (the
    /// first frame the colour actually changes the selected path). Committed to
    /// `History` as ONE undo step when the gesture ends (the picker closes /
    /// the discrete pick's frame finishes). `None` between gestures.
    static RECOLOR_PRE: RefCell<Option<VecScene>> = const { RefCell::new(None) };
}

/// Per-frame Vector-tool plumbing. Safe to call every frame; a no-op when the
/// Vector tool is absent from the registry.
/// Returns the tool's current [`VectorDrawConfig`] so the shell can mirror it
/// into `App` (the input dispatch reads it to route canvas gestures + size the
/// shapes without a downcast). Defaults when the Vector tool is absent.
#[allow(clippy::too_many_arguments)] // per-frame bridge inputs, each distinct
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    scene: &mut VecScene,
    pen: &mut PenTool,
    shape: &mut ShapeTool,
    history: &mut History,
    // World units per screen pixel (from the camera) — converts the tool's px
    // stroke width into the path's world-space width when restyling.
    px_to_world: f64,
    // Selected gradient handle (a multi-point point OR a linear/radial endpoint) —
    // the Fill swatch shows its colour and the picker recolours THAT slot instead of
    // replacing the whole fill with a solid colour.
    grad_handle: Option<GradHandle>,
) -> VectorDrawConfig {
    let vector_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("vector"));

    // ── 1. Panel visibility (mirror of the Padding dock takeover) ─────────
    hero.panel_visibility.insert("vector", vector_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(vector_active, Ordering::Relaxed);
        if was != vector_active {
            hero.panel_visibility.insert("inspector", !vector_active);
        }
    }

    // The tool persists in the registry whether or not it is active, so its
    // Style survives tool switches (mirror of the painter bridge).
    let Some(tool) = tools.tool_by_id_mut(&ToolId::new("vector")).and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_vector::VectorTool>()
    }) else {
        #[cfg(feature = "panel-vector")]
        ph2d_panel_vector::set_current_vector_style(None);
        return VectorDrawConfig::default();
    };

    // ── 2. Picker read-back: which swatch is the picker targeting? ────────
    let target = hero.store.picker_target();
    let stroke_open = target == Some(ph2d_editor::ids::VECTOR_STROKE_SWATCH);
    let fill_open = target == Some(ph2d_editor::ids::VECTOR_FILL_SWATCH);
    if (stroke_open || fill_open)
        && let Some((value, _, _, _)) = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
    {
        // The picker owns RGB **and alpha**: its alpha flows into the tool's
        // stroke/fill alpha, and the bridge pushes that back onto the Opacity
        // slider each frame (below) so the picker's alpha and the panel's
        // Opacity stay in sync (Enio 2026-07-07).
        let picked = value.rgba;
        if stroke_open {
            tool.set_stroke_rgba(picked);
        } else {
            tool.set_fill_rgba(picked);
        }
    }

    let stroke = tool.stroke_rgba();
    let fill = tool.fill_rgba();
    let cap = line_cap(tool.cap());
    let join = line_join(tool.join());
    // Dash + gap are MULTIPLES of the stroke width (width-aware) — the render
    // scales them by the path's own width, so no px→world conversion here.
    // `dash = 0` ⇒ solid; otherwise `(dash, gap)` sizes the dash and the space.
    let dash = (tool.dash() > 0.0).then_some((tool.dash(), tool.gap()));

    // ── 3. New paths honour the tool's Style (pen + shape share it). ──────
    let style = PenStyle {
        stroke: rgba(stroke),
        stroke_w_px: tool.stroke_width_px(),
        fill: rgba(fill),
        cap,
        join,
        dash,
    };
    pen.set_style(style);
    shape.set_style(style);

    // ── 4. Restyle the selected path — colour + width (undoable, one step per
    //    gesture). A width-slider DRAG is a gesture like a picker drag, so scope
    //    its undo the same way (one step per drag). Width follows the tool ONLY
    //    while the slider is dragged, so a plain colour pick never resizes it.
    let width_dragging = matches!(
        hero.store.slider(ph2d_editor::ids::VECTOR_WIDTH),
        Some((ph2d_editor::widget::SliderState::Dragging, _))
    );
    let session = stroke_open || fill_open || width_dragging;
    // The selected gradient handle, kept only if it still addresses a colour on the
    // current fill (a stale handle after a kind switch resolves to `None` and falls
    // through to the solid-fill path). The picker recolours THIS slot.
    let active_handle = grad_handle.filter(|&h| {
        pen.selected()
            .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
            .and_then(|p| p.fill.as_ref())
            .and_then(|f| selected_grad_color(f, h))
            .is_some()
    });
    if tool.take_apply_to_selected()
        && let Some(sel) = pen.selected()
    {
        let new_stroke = rgba(stroke);
        let new_fill = if fill[3] == 0 { None } else { Some(rgba(fill)) };
        let new_w = tool.stroke_width_px() * px_to_world;
        let will_change = scene.paths().iter().find(|p| p.id == sel).is_some_and(|p| {
            let stroke_differs = p.stroke.is_some_and(|s| {
                s.color != new_stroke
                    || s.cap != cap
                    || s.join != join
                    || s.dash != dash
                    || (width_dragging && (s.width - new_w).abs() > f64::EPSILON)
            });
            let fill_differs = if let Some(h) = active_handle {
                // Recolour the selected gradient slot (point / ramp stop).
                p.fill.as_ref().and_then(|f| selected_grad_color(f, h)) != Some(rgba(fill))
            } else {
                // No gradient handle selected → a fill pick only replaces a Solid /
                // None fill; it must NEVER clobber a gradient (use Fill Type → Solid
                // for that). Guards the linear/radial→solid regression (Enio 2026-07-08).
                p.closed
                    && matches!(p.fill, None | Some(Paint::Solid(_)))
                    && p.fill.as_ref().map(Paint::primary_color) != new_fill
            };
            stroke_differs || fill_differs
        });
        if will_change {
            RECOLOR_PRE.with(|c| {
                if c.borrow().is_none() {
                    *c.borrow_mut() = Some(scene.clone());
                }
            });
            if let Some(path) = scene.path_mut(sel) {
                if let Some(old) = path.stroke {
                    // Keep the path's width unless the Width slider is being dragged
                    // (mirror of the pre-existing width behaviour); apply the rest.
                    let width = if width_dragging { new_w } else { old.width };
                    path.stroke = Some(StrokeSpec {
                        color: new_stroke,
                        width,
                        cap,
                        join,
                        dash,
                    });
                }
                if let Some(h) = active_handle {
                    // Recolour the selected gradient slot (never converts to solid).
                    if let Some(paint) = path.fill.as_mut() {
                        set_selected_grad_color(paint, h, rgba(fill));
                    }
                } else if path.closed && matches!(path.fill, None | Some(Paint::Solid(_))) {
                    // Otherwise a fill pick sets a SOLID fill — but only over a solid /
                    // empty fill, never over a gradient.
                    path.fill = new_fill.map(Paint::solid);
                }
            }
        }
    }
    // Commit the gesture's undo when it ends (no picker / width-drag session):
    // a discrete pick (None) commits immediately; a drag commits on release.
    if !session {
        RECOLOR_PRE.with(|c| {
            if let Some(pre) = c.borrow_mut().take() {
                history.push_undo(pre);
            }
        });
    }

    // ── 5. Sync swatch colours (seeds the picker on open) + Opacity sliders
    //    (so a picker alpha shows on the panel) + publish. ──────────────────
    hero.store
        .set_widget_color(ph2d_editor::ids::VECTOR_STROKE_SWATCH, stroke);
    // The Fill swatch shows the selected gradient point's colour (so the picker
    // opens seeded on it) when a MultiPoint point is selected, else the tool fill.
    let fill_swatch_col = active_handle
        .and_then(|h| {
            pen.selected()
                .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
                .and_then(|p| p.fill.as_ref())
                .and_then(|f| selected_grad_color(f, h))
                .map(|c| [c.r, c.g, c.b, c.a])
        })
        .unwrap_or(fill);
    hero.store
        .set_widget_color(ph2d_editor::ids::VECTOR_FILL_SWATCH, fill_swatch_col);
    // Push the tool's alpha onto the Opacity sliders (unless being dragged) so
    // an alpha set in the colour picker reflects on the panel, and vice-versa.
    sync_opacity_slider(
        &mut hero.store,
        ph2d_editor::ids::VECTOR_STROKE_OPACITY,
        stroke[3],
    );
    sync_opacity_slider(
        &mut hero.store,
        ph2d_editor::ids::VECTOR_FILL_OPACITY,
        fill[3],
    );
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_vector_style(if vector_active {
        Some(tool.ui_snapshot())
    } else {
        None
    });
    // Publish the selected vertex's type so the panel shows the Vertex section
    // (Corner/Smooth/Symmetric) + highlights the active one. `None` hides it.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_selected_vertex_type(if vector_active {
        pen.selected_vertex_kind(scene).map(vertex_type_of)
    } else {
        None
    });

    // Publish the selected path's anchor bbox `[x, y, w, h]` (world) so the panel
    // shows + seeds the numeric Transform fields. `None` hides the section.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_transform(if vector_active {
        pen.selected()
            .and_then(|sel| scene.path_bbox(sel))
            .map(|(lo, hi)| [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]])
    } else {
        None
    });

    // Publish the selected path's closed flag so the panel labels the toggle
    // "Close Path" / "Open Path" correctly.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_path_closed(if vector_active {
        pen.selected()
            .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
            .map(|p| p.closed)
    } else {
        None
    });

    // Publish the selected path's fill kind (+ linear angle) so the Fill-type
    // selector reflects + drives it.
    #[cfg(feature = "panel-vector")]
    {
        use ph2d_panel_vector::FillKind;
        use ph2d_vec_scene::Paint;
        let (kind, angle) = if vector_active {
            match pen
                .selected()
                .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
                .and_then(|p| p.fill.as_ref())
            {
                Some(Paint::Solid(_)) => (Some(FillKind::Solid), None),
                Some(Paint::Linear { start, end, .. }) => {
                    // Angle of the ramp direction, normalized to [0, 360).
                    let mut deg = (end[1] - start[1]).atan2(end[0] - start[0]).to_degrees();
                    if deg < 0.0 {
                        deg += 360.0;
                    }
                    (Some(FillKind::Linear), Some(deg))
                }
                Some(Paint::Radial { .. }) => (Some(FillKind::Radial), None),
                Some(Paint::MultiPoint { .. }) => (Some(FillKind::MultiPoint), None),
                None => (None, None),
            }
        } else {
            (None, None)
        };
        ph2d_panel_vector::set_current_fill(kind, angle);
        // Publish the selected multi-point point's influence + jitter (drive the sliders).
        let sel_point = active_handle.and_then(GradHandle::point).and_then(|i| {
            pen.selected()
                .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
                .and_then(|p| match &p.fill {
                    Some(Paint::MultiPoint { points }) => points.get(i).copied(),
                    _ => None,
                })
        });
        ph2d_panel_vector::set_current_grad_influence(sel_point.map(|gp| gp.influence));
        ph2d_panel_vector::set_current_grad_jitter(sel_point.map(|gp| gp.jitter));
    }

    // Calibrate the Transform fields' drag scrub to the camera: `px_to_world`
    // value-units per cursor pixel ⇒ dragging a chip N px moves the shape N px
    // on screen at any zoom (unbounded — no clamp). Live each frame so zoom in/out
    // keeps the 1:1 feel.
    if vector_active {
        for id in [
            ph2d_editor::ids::VECTOR_TRANSFORM_X,
            ph2d_editor::ids::VECTOR_TRANSFORM_Y,
            ph2d_editor::ids::VECTOR_TRANSFORM_W,
            ph2d_editor::ids::VECTOR_TRANSFORM_H,
        ] {
            hero.store.set_number_drag_rate(id, px_to_world);
        }
        // The Angle (R) field is in DEGREES, not world units — a fixed, gentle
        // scrub (a full drag across the screen ≈ a couple turns), zoom-independent.
        const ROT_DRAG_DEG_PER_PX: f64 = 0.5;
        hero.store
            .set_number_drag_rate(ph2d_editor::ids::VECTOR_TRANSFORM_R, ROT_DRAG_DEG_PER_PX);
    }

    // Mirror the tool's mode + shape params so the input dispatch can route
    // canvas gestures (pen vs shape) + size the shapes without a downcast.
    tool.draw_config()
}

/// Map the UI-facing `StrokeCap`/`StrokeJoin` to the geometry enums.
fn line_cap(c: ph2d_tool_vector::StrokeCap) -> LineCap {
    use ph2d_tool_vector::StrokeCap;
    match c {
        StrokeCap::Butt => LineCap::Butt,
        StrokeCap::Round => LineCap::Round,
        StrokeCap::Square => LineCap::Square,
    }
}
fn line_join(j: ph2d_tool_vector::StrokeJoin) -> LineJoin {
    use ph2d_tool_vector::StrokeJoin;
    match j {
        StrokeJoin::Miter => LineJoin::Miter,
        StrokeJoin::Round => LineJoin::Round,
        StrokeJoin::Bevel => LineJoin::Bevel,
    }
}

/// Map the geometry `VertexKind` to the panel's UI-facing `VertexType`.
fn vertex_type_of(k: ph2d_vec_scene::VertexKind) -> ph2d_tool_vector::VertexType {
    use ph2d_tool_vector::VertexType;
    use ph2d_vec_scene::VertexKind;
    match k {
        VertexKind::Corner => VertexType::Corner,
        VertexKind::Smooth => VertexType::Smooth,
        VertexKind::Symmetric => VertexType::Symmetric,
    }
}
