//! **O que uma sessão de pintura SEGURA** — irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é o mesmo que este subsistema já vinha fazendo, um degrau adiante: o `state_default`
//! levou o CONSTRUTOR ("com que valores ela nasce") e deixou a struct para trás; agora a struct
//! vai junto, e o `paint.rs` fica sendo o que ele de fato é — o **manifesto** do subsistema: a
//! lista de módulos, os re-exports e as suítes de gate. Um manifesto tem de morar na raiz do
//! módulo; o estado não.
//!
//! Os campos são `pub(super)` e não privados porque quem os lê são os ~40 módulos IRMÃOS
//! (`brush_settings`, `sculpt`, `wetpaint`, …), que são DESCENDENTES de `paint` — `pub(super)`
//! daqui alcança exatamente eles e nada mais. Nenhum símbolo novo sai da crate.

use super::*;

pub(crate) struct PaintState {
    /// The active brush.
    pub(super) brush: BrushSpec,
    /// Pen pressure → size/coverage mapping.
    pub(super) dynamics: Dynamics,
    /// The stroke in progress between pointer-down and pointer-up (`None` when idle).
    pub(super) stroke: Option<Stroke>,
    /// The **hover heading** (unit vector; `[0, 0]` = none yet) + last hover position — the brush
    /// ring aims before pen-down ([`PainterTool::on_canvas_hover`]); a live stroke's heading outranks.
    pub(super) hover_heading: [f32; 2],
    pub(super) hover_pos: Option<[f32; 2]>,
    /// Reused dab buffer so a hot pointer stream allocates nothing per sample.
    pub(super) dabs: Vec<Dab>,
    /// Per-stroke jitter seed; bumped each stroke so jitter is reproducible yet varies.
    pub(super) seed: u64,
    /// Splitmix64 for the texture's per-dab Random rotation/offset — reset per stroke (seed-decorrelated), advanced once per textured dab (HR-5).
    pub(super) tex_rng: u64,
    /// Model snapshot at pointer-down (before the first dab) — committed to undo at pointer-up so the whole stroke undoes as one unit.
    pub(super) stroke_undo: Option<crate::undo::ModelSnapshot>,
    /// Eraser mode: overrides the brush blend with Erase Alpha at stamp time (`brush.blend` preserved for when it's off).
    pub(super) eraser: bool,
    /// Which operation the pointer performs (Brush=Paint / Smear); driven by the left-rail tool selection.
    pub(super) paint_mode: PaintMode,
    /// Per-mode saved brush settings — the "independent tools" model (the default): each
    /// [`PaintMode`] keeps its OWN [`BrushSpec`] (indexed by [`PaintMode::slot`]), swapped into
    /// `brush` on a mode change so one tool's panel never bleeds into another. Ignored while
    /// `link_shared_settings` is on; the active mode's slot is stale while active (edits go to
    /// `brush`, written back on the next switch). See [`tool_link`].
    pub(super) brush_by_mode: [BrushSpec; PAINT_MODE_COUNT],
    /// "Sync with other tools": when `true`, every paint tool SHARES the live `brush` (a mode change no
    /// longer swaps slots), so a change in one panel shows in all. Default `false` = each tool independent.
    pub(super) link_shared_settings: bool,
    /// **Line** stroke method: live dx/dy + corner-angle CAD overlay while drawing. Default `true`;
    /// a per-Line display pref (the "Dimensions" checkbox).
    pub(super) line_show_dimensions: bool,
    /// **Mask** sub-brush (Mask mode): `0` Paint (conceal/black) · `1` Erase (reveal/white) · `2` Blur · `3` Smear. [`mask`].
    pub(super) mask_brush: u8,
    /// **Mask** overlay tint index (`0` gray + 4 fluorescent) — tints the composite where a mask conceals. [`mask`].
    pub(super) mask_overlay_color: u8,
    /// **Eyedropper** armed: the next canvas Down samples the composited pixel into the brush colour, then disarms. [`eyedropper`].
    pub(super) eyedropper_armed: bool,
    /// Whether the LIVE brush's falloff was installed by a tool-default ARM (never by the artist) —
    /// the provenance that lets a verb switch re-arm its own default without ever overriding a
    /// deliberate choice (`arm_tool_falloff_defaults`). Cleared by the artist's falloff setter;
    /// re-presumed on a brush-slot switch from the factory value.
    pub(super) falloff_armed: bool,
    /// **Mask** brush transient scratch (tool-side mask for `mask_scratch_target`, white=reveal; NOT a stack layer). [`mask`].
    pub(super) mask_scratch_rgba: Arc<Vec<u8>>,
    pub(super) mask_scratch_target: Option<crate::layers::LayerId>,
    /// **Selection** mask (ADR-0103): a document-wide single-channel coverage buffer (`w*h` bytes,
    /// `0` = outside / `255` = inside; Feather softens the edge). Gates every paint op to the selected
    /// region ([`selection`]) and is undo-integrated via the `ModelSnapshot` exactly like `mask_scratch`.
    pub(super) selection_mask: Arc<Vec<u8>>,
    /// `true` while a selection is live (has coverage). `false` = no selection, painting unrestricted.
    pub(super) selection_active: bool,
    /// **Selection** sub-mode: `0` Automatic · `1` Freehand · `2` Rectangle · `3` Ellipse. Driven by the
    /// Selection panel (Wave 3); consumed by the on-canvas router ([`selection`]).
    pub(super) selection_mode: u8,
    /// **Selection** boolean operator for the next gesture: `0` New (replace) · `1` Add · `2` Remove.
    pub(super) selection_bool_op: u8,
    /// **Selection** Automatic threshold (`0..1`) — colour tolerance for the flood-select mode.
    pub(super) selection_threshold: f32,
    /// The in-progress selection gesture (marquee rubber-band / lasso path / Automatic seed); `None` when
    /// idle. The overlay (Wave 4) draws from this; the mask is rasterized on pen-up. [`selection`].
    pub(super) selection_drag: Option<selection_input::SelectionDrag>,
    /// The selection mask at the START of the current gesture — Add/Remove combine against this base. [`selection`].
    pub(super) selection_base: Arc<Vec<u8>>,
    /// The CRISP selection mask (pre-Feather), the accumulator the Feather slider re-derives from — so
    /// dragging Feather never compounds a blur-of-a-blur (mirrors the Shape Offset accumulator). [`selection`].
    pub(super) selection_crisp: Arc<Vec<u8>>,
    /// **Free-selection stabilizer** (`0..1`) — the lazy-mouse smoothing applied to the Freehand lasso path
    /// (its own knob, independent of the brush stabilizer). [`selection_input`].
    pub(super) selection_stabilizer: f32,
    /// **Feather** amount (`0..1` → edge-softening radius); the effective `selection_mask` is a blur of
    /// `selection_crisp` at this radius. [`selection`].
    pub(super) selection_feather: f32,
    /// **Show Selection Gizmos** mode: when `true`, EVERY editable selection shape shows its own isolated
    /// gizmo at once (ellipse / polygon / freehand), each manipulable — WITHOUT touching the stroke shape
    /// editors (ADR-0103 Am.2 v2). A transient UI mode. [`selection_gizmo`].
    pub(super) selection_edit_mode: bool,
    /// **Selection overlay opacity** (`0..1`) — how strongly the deselected-area hatching reads. A view
    /// preference (not undoable); scales the hatch alpha in [`selection`]. Default `0.2` (Enio 2026-07-02).
    pub(super) selection_overlay_opacity: f32,
    /// **Selection shape list** (ADR-0103 Am.2) — the parametric source of truth (Ellipse / Polygon /
    /// Freehand / Raster + a boolean op each). The `selection_mask` is a DERIVED cache: rasterize + composite
    /// this list. A gizmo drag mutates one entry's params in place and recomposites. [`selection_shapes`].
    pub(super) selection_shapes: Vec<selection_shapes::SelectionEntry>,
    /// **Per-shape rasterization cache** (perf): `(shape, coverage)` parallel to `selection_shapes`
    /// at the last recompose — an UNCHANGED shape reuses its cached coverage (`Arc` clone), so a
    /// gizmo drag over N boolean shapes re-rasterizes only the ONE that moved (O(A) vs O(N·A) per
    /// frame). Self-validating by value, no manual invalidation.
    pub(super) selection_raster_cache: Vec<(selection_shapes::SelectionShape, Arc<Vec<u8>>)>,
    /// The isolated gizmo grab currently dragged (shape idx + handle + pristine geometry for drift-free
    /// whole-shape transforms); `None` when idle. [`selection_gizmo`].
    pub(super) selection_grab: Option<selection_gizmo::SelectionGrab>,
    /// In-memory **Copy** buffer of selected pixels (source bbox + coverage-premultiplied RGBA), consumed by
    /// **Paste**. `None` until a Copy. [`selection_actions`].
    pub(super) selection_clipboard: Option<selection_actions::SelectionClip>,
    /// **Selection Offset** state (ADR-0103 Am.3) — grow/shrink + concentric protected/paint rings. See
    /// [`selection_offset`]: `_norm` = slider (`0.5` = no offset); `_active` = ring mode (post-Apply & Keep);
    /// `_rings` = frozen cumulative band offsets (px, PAINT iff even index); `_source` = the pre-offset crisp
    /// the offset reads from; `_sdf` = its lazily-cached signed distance field (`<0` inside).
    pub(super) selection_offset_norm: f32,
    pub(super) selection_offset_active: bool,
    pub(super) selection_offset_rings: Vec<f32>,
    pub(super) selection_offset_source: Arc<Vec<u8>>,
    /// Corner-true contours of the offset source (outer + holes, refit + grow-calibrated) — the sharp
    /// offset's derived cache; rebuilt lazily off `selection_offset_source`. [`selection_offset_geom`].
    pub(super) selection_offset_curves: Vec<selection_offset_geom::OffsetContour>,
    /// Per-level effective masks (ring boundaries + the live level) — derived cache keyed by exact level.
    pub(super) selection_offset_level_cache: Vec<(f32, Arc<Vec<u8>>)>,
    /// **Ring stack**: `true` once the offset rings were materialised into the editable Freehand curves in
    /// `selection_shapes` (Edit Gizmos on an offset selection, Enio 2026-07-04). While set, the mask is a
    /// BAND-PARITY composite of those nested curves (paint iff enclosed by `≡ n (mod 2)` of them, `n>0`),
    /// so editing any ring curve reshapes its intercalated band. Cleared by Clear / a new selection gesture.
    pub(super) selection_ring_stack: bool,
    /// **Composite Brush**: run Brush + Smear + Blur together (a Brush-tool upgrade, panel checkbox). See [`composite`].
    pub(super) composite_enabled: bool,
    /// The composite layer stack in display order (index 0 = layer 1 = top; run bottom→top per dab). [`composite`].
    pub(super) composite: [CompositeLayer; 3],
    /// **Clone** sampled source anchor (image px), set by the "Set Source" pick mode; `None` until sampled. [`clone`].
    pub(super) clone_source: Option<[f32; 2]>,
    /// **Clone** established source→dest offset (px) = `clone_source − stroke_start`; `None` until a stroke begins. [`clone`].
    pub(super) clone_offset: Option<[f32; 2]>,
    /// **Clone** Aligned mode: keep the offset fixed across strokes (on); re-anchor to the source each stroke (off).
    pub(super) clone_aligned: bool,
    /// **Clone** "Set Source" pick mode armed — the next canvas Down sets [`Self::clone_source`] instead of painting.
    pub(super) clone_sample_armed: bool,
    /// Previous dab centre during a **Smear** stroke (the source each dab lifts from); `None` at stroke start. [`stamp_route`].
    pub(super) last_smear_pos: Option<[f32; 2]>,
    /// Reused per-dab scratch for the Smear's map composition (no allocation in a hot stroke).
    pub(super) smear_scratch: ph2d_painter_brush::smear_field::SmearScratch,
    /// **Tiling** `[x, y]`: seamless wrap-around painting — a dab near an edge also stamps the wrapped part on the opposite edge. Off by default.
    pub(super) tiling: [bool; 2],
    /// **Repeat Image**: the shell draws the sprite in the 8 neighbour directions (3×3); with Tiling on, those tiles are paintable (the shell wraps the pointer back).
    pub(super) repeat_image: bool,
    /// **Symmetry** on-canvas pick mode armed (draw mirror line / pick radial centre); `None` = paint normally. [`symmetry`].
    pub(super) symmetry_pick: Option<symmetry::SymmetryPick>,
    /// First endpoint captured while drawing the custom symmetry line; `None` when not mid-draw.
    pub(super) symmetry_line_start: Option<[f32; 2]>,
    /// Whether the symmetry centre auto-tracks the canvas centre (X/Y mirror + radial default); a user-drawn line / picked centre clears it. See [`PainterTool::resolve_symmetry_geometry`].
    pub(super) symmetry_auto_center: bool,
    /// Cleared per frame; set by `paint_extend` on a move. A parked frame lets `paint_tick` settle the stabilizer.
    pub(super) moved_this_frame: bool,
    /// Restore record for the in-progress Drag Dot's single moving dab; `None` for every other method.
    pub(super) drag_preview: Option<DragPreview>,
    /// **Um gesto de shape está em VOO** — entre o Down e o Up de um arrasto no canvas.
    ///
    /// Enquanto isto vale, o re-carimbo da figura roda um RASCUNHO: o meio caro (Impasto, Aquarela)
    /// é desarmado e a figura sai plana. No Up ele cai e o editor re-carimba pela última vez, agora
    /// com o meio de verdade — *o caro renderiza em REPOUSO*. Medido a 4096² com figura de 400 px:
    /// um move de Impasto custa **101,17 ms** com o meio ligado e **4,38** sem
    /// (`measure_what_a_flat_preview_would_cost`), porque o carimbo é 99,3% do evento e o shape
    /// editor re-carimba a figura INTEIRA a cada quadro do arrasto.
    ///
    /// ⚠️ **Transiente, como o `line_constrain`:** não entra no `ModelSnapshot` nem no arquivo — é
    /// fato sobre a mão do artista, não sobre o documento.
    pub(super) shape_draft: bool,
    /// Quantos re-carimbos de figura já rodaram — o sinal de *"o editor re-carimbou por conta"*.
    ///
    /// ⚠️ Ele existe porque a alternativa é **enumerar** quais ramos de Up de quais editores refazem
    /// a figura, e essa lista apodrece no dia em que um ramo ganha um early-return: o modo de falha
    /// seria a figura ficando PLANA para sempre, em silêncio. Com o contador o Up pergunta o FATO
    /// (*alguém re-carimbou?*) e força o carimbo final se ninguém o fez.
    pub(super) restamp_seq: u32,
    /// The press point of the in-progress stroke — the pivot the Line's Alt-constrain snaps around (45°).
    pub(super) line_anchor: Option<[f32; 2]>,
    /// Alt held this event — constrains the Line to 45° increments (Blender `constrain_line`); set by the shell each pointer event.
    pub(super) line_constrain: bool,
    /// Shift held this event — a Stencil corner scale becomes UNIFORM (aspect-locked, like the Sprite gizmo); set by the shell each pointer event.
    pub(super) scale_uniform: bool,
    /// Shift held this event — the polyline **Line** editor snaps each new segment to 15° increments from the previous point; set by the shell each pointer event. See [`line`].
    pub(super) line_snap: bool,
    /// The editor GRID-snapped image position for the current pointer, forwarded by the shell each event
    /// (world→grid via `GridSnapState::snap_world`, mapped back to image px) — `None` when grid snap is off.
    /// Drawing-tool point placement/drag uses it as the base position; gizmo / corner-handle drags ignore
    /// it (so grid snap can't corrupt a parameter drag). See [`line_snap`].
    pub(super) grid_snap_pos: Option<[f32; 2]>,
    /// Last NON-shape method — the rail's Brush button restores it. See [`PainterTool::restore_non_shape_stroke_method`].
    pub(super) last_non_shape_method: StrokeMethod,
    /// In-progress Curve session (the on-canvas point editor); `None` when idle. [`curve`].
    pub(super) curve: Option<curve::CurveEditor>,
    /// In-progress Ellipse session (the on-canvas ellipse editor); `None` when idle. [`circle`].
    pub(super) ellipse: Option<ellipse::EllipseEditor>,
    /// In-progress Line session (the on-canvas polyline editor); `None` when idle. [`line`].
    pub(super) line: Option<line::LineEditor>,
    /// In-progress Polygon session (the on-canvas regular-N-gon editor); `None` when idle. [`polygon`].
    pub(super) polygon: Option<polygon::PolygonEditor>,
    /// **Multi-shape** PARKED stroke shapes (source of truth; pixels are a derived recompose) — every editable shape but the live editor. Empty = single-shape. [`stroke_multi`] (Enio 2026-07-04).
    pub(super) parked_shapes: Vec<stroke_multi::StrokeShape>,
    /// Operation of the ACTIVE shape (its `+`/`−`/`o` gizmo glyph); a new shape adopts [`stroke_op_mode`]. [`stroke_multi`].
    pub(super) active_op: stroke_multi::StrokeOp,
    /// Current panel Operation mode a NEW shape is created with (stroke analogue of `selection_bool_op`; "New"→Overlay).
    pub(super) stroke_op_mode: stroke_multi::StrokeOp,
    /// Pending op-cycle **tap** (Down pos on the active shape's centre square): Up without a drag cycles the op; a drag past the slop clears it + moves the shape. [`stroke_multi`].
    pub(super) op_tap: Option<[f32; 2]>,
    /// Seamless-Tiling **edit-in-tile** offset (Enio 2026-07-11): a shape's overlay is drawn in the
    /// wrapped neighbour tiles, so a grab there must edit the ORIGINAL. Fixed at the grab Down (the
    /// tile offset landing the pointer on the shape's bbox, `route_shape_pointer_multi`), subtracted
    /// from every pointer of the gesture — a CONTINUOUS drag (no per-sample seam jump), works beyond
    /// the sprite. `[0, 0]` = no wrap (off-tiling / drawing / empty-space click).
    pub(super) shape_edit_wrap: [f32; 2],
    /// Pending SELECTION op-cycle tap — Down on a shape's centre-move square arms `Some((shape, pos))`; Up without a drag past the slop cycles THAT shape's Add↔Remove op; a drag clears it + moves the shape. Mirrors [`op_tap`] but selection toggles only Add/Remove. [`selection_gizmo`].
    pub(super) selection_op_tap: Option<(usize, [f32; 2])>,
    /// Control-handle grab radius (image px) for the shape editors — shell forwards a footprint-scaled value.
    pub(super) shape_grab_tol_px: f32,
    /// **Offset** slider track (`0..1`, `0.5` = none) — perpendicular path offset for the shape editors.
    pub(super) shape_offset_norm: f32,
    /// **Accumulated** offset (px) from prior Apply & Keep; EFFECTIVE = base + slider (a single offset of the pristine base).
    pub(super) shape_offset_base_px: f32,
    /// **Trim** (Offset card): cut the offset spine's self-intersections — drawing-only (see [`curve_offset`]).
    pub(super) offset_trim: bool,
    /// In-progress Stencil overlay drag (move/resize/rotate the texture rect); `None` when idle.
    pub(super) stencil_grab: Option<stencil::StencilGrab>,
    /// Seconds left on the transient in-gizmo Stencil texture preview (decayed each `paint_tick`).
    pub(super) stencil_preview_s: f32,
    /// Imported brush-**Grain** luminance (heavy → not in the `Copy` spec); borrowed as an `ImageMask`.
    pub(super) texture_image: Option<brush_settings::BrushTextureImage>,
    /// Watercolor **Paper** slot luminance (a tagged layer used as the substrate; `paper.kind == Image`).
    /// Heavy, so out of the `Copy` spec; borrowed as an `ImageMask` by the render-path ([`watercolor_render`]).
    /// (The **Granulation** map is the Grain slot, so it reuses [`Self::texture_image`].)
    pub(super) paper_image: Option<brush_settings::BrushTextureImage>,
    /// Bumped whenever [`Self::paper_image`] changes, so the shell re-publishes it for the Paper preview.
    pub(super) paper_image_version: u64,
    /// Set when the user picks the Image kind; the shell polls it to open a file picker.
    pub(super) texture_image_pending: bool,
    /// Bumped whenever [`texture_image`] changes, so the stamp cache re-renders the Image mask.
    pub(super) texture_image_version: u64,
    /// Imported brush-**Shape** luminance (silhouette tip; borrowed as `ImageMask`). `None` ⇒ silhouette = falloff.
    pub(super) shape_image: Option<brush_settings::BrushTextureImage>,
    /// Set when the user picks the Image source in the Shape dropdown; the shell polls it to open a picker.
    pub(super) shape_image_pending: bool,
    /// Bumped whenever [`shape_image`] changes, so the stamp cache re-renders the Shape mask.
    pub(super) shape_image_version: u64,
    /// Multi-layer Shape (z-ordered luminance layers) + per-layer-colour mode/colours; OFF ⇒ flattened into [`shape_image`]; see [`crate::tool::paint::shape_layers`].
    pub(super) shape_layers: shape_layers::ShapeLayers,
    /// Cached brush stamp (falloff × View texture) + its key; re-rendered on appearance/mask-size change, scale-blitted per dab. See [`crate::tool::paint::stamp_cache`].
    pub(super) stamp_cache: Option<(ph2d_painter_brush::StampMask, stamp_cache::StampKey)>,
    /// Cached per-layer coloured stamps (bottom→top) + key, blitted in cross-stroke z-order; `stamp_color_cache`.
    pub(super) color_stamp_cache: Option<(
        Vec<ph2d_painter_brush::ColorStampMask>,
        stamp_color_cache::ColorStampKey,
    )>,
    /// Cached Grain+Ramp coloured stamp + key (the cacheable grain-ramp colour path); `stamp_color_cache`.
    pub(super) ramp_color_stamp_cache: Option<(
        ph2d_painter_brush::ColorStampMask,
        stamp_color_cache::RampColorStampKey,
    )>,
    /// Lazily-filled canvas-space texture cache for Tiled / Stencil mappings (computed once per canvas pixel per stroke). See [`crate::tool::paint::stamp_cache`].
    pub(super) canvas_tex_cache: Option<stamp_cache::CanvasTexCache>,
    /// The brush Grain + Shape **Color Ramps** + Shape **tone** LUT (engine model + baking: [`ramp_lut`]).
    pub(super) texture_ramp: ph2d_color::ColorRamp,
    pub(super) texture_ramp_enabled: bool,
    pub(super) texture_ramp_bw: bool,
    pub(super) texture_ramp_lut: Vec<[f32; 4]>,
    pub(super) texture_ramp_dirty: bool,
    /// Bumped when `ensure_ramp_lut` re-bakes the owner LUT — the colour-ramp **stamp** cache keys on it.
    pub(super) ramp_lut_version: u64,
    pub(super) texture_ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode,
    pub(super) shape_color_ramp: ph2d_color::ColorRamp,
    pub(super) shape_color_ramp_enabled: bool,
    pub(super) shape_color_ramp_bw: bool,
    pub(super) shape_color_ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode,
    pub(super) shape_ramp_lut: Vec<f32>,
    pub(super) shape_ramp_dirty: bool,
    pub(super) shape_ramp_version: u64,
    pub(super) ramp_lut_owner: ramp_lut::RampLutOwner,
    /// **Accumulate OFF** per-stroke coverage mask (1 byte/px), cleared on down; caps a stroke at Strength.
    pub(super) stroke_mask: Vec<u8>,
    /// **Impasto** — every per-stroke plane the relief lives in, and the window they are indexed
    /// against. See [`relief_state::ReliefState`].
    pub(super) relief: relief_state::ReliefState,
    /// **Watercolor render-path** per-stroke coverage (1 byte/px, `w*h`): the union footprint of the
    /// stroke's dabs (max-blended discs = wet_edges `stampCoverage`), the silhouette the optical composite
    /// reconstructs the wash from ([`super::watercolor_render`]). Empty unless the Watercolor section is
    /// active; sized lazily by the first dab, cleared on down.
    pub(super) stroke_coverage: Vec<u8>,
    /// **Watercolor render-path** per-stroke deposited colour (RGBA, `w*h*4` = wet_edges `colC`): each
    /// dab's colour splatted source-over (recent dab wins), so the composite pigment can vary along the
    /// stroke (RYB pickup when Pigment is on). Empty / cleared with [`Self::stroke_coverage`].
    pub(super) stroke_color: Vec<u8>,
    /// **Watercolor render-path** frozen base — the pre-stroke `canvas_rgba` (shared `Arc`, so holding it
    /// is O(1); the first composite `make_mut` forks the live buffer, leaving this pristine). The optical
    /// composite reads the "paper + prior paint" from here every frame instead of over-painting in place,
    /// so the wash never accumulates per-dab structure. `Some` only for the duration of a watercolor stroke.
    pub(super) watercolor_base: Option<Arc<Vec<u8>>>,
    /// **Watercolor render-path** frozen GROUND — the real backdrop under the active layer: the
    /// composite of the layers BELOW it, over the document [`Self::paper_color`] where nothing is
    /// painted (RGBA8, opaque by construction). The optics read the Beer–Lambert base, the rewet
    /// presence reference and the lift target from HERE, never from a global paper constant — a
    /// virtual cream baked into the wash was the "puxa pro bege" bug (Enio 2026-07-06). Frozen with
    /// [`Self::watercolor_base`]; `None` outside a watercolor stroke.
    pub(super) wet_backdrop: Option<Arc<Vec<u8>>>,
    /// Document **paper colour** (straight sRGB `0..1`) — the ground the watercolor optics see where
    /// the backdrop is fully transparent. Default WHITE (a plain canvas); panel swatch
    /// `PAINTER_WATERCOLOR_PAPER_COLOR_THUMB` edits it via the shared picker (Rebelle: canvas colour
    /// is a user-pickable document property). Tool-global (not persisted in the document yet).
    pub(super) paper_color: [f32; 3],
    /// **Impasto — Show** (canvas-level, like the paper colour / drying time): whether the relief is
    /// LIT. Off ⇒ the light pass does not run and the composite is byte-identical to a build with no
    /// Impasto at all. Default on — someone who sculpts wants to see it, and with no relief anywhere
    /// the pass costs a single `is_empty()`.
    pub(super) impasto_show: bool,
    /// The canvas's **light rig** — up to `MAX_LIGHTS` lamps, each with its own angle / elevation /
    /// intensity / colour, plus which one the card is editing. Light 0 is the key and starts on; the
    /// rest start off, so a canvas nobody has opened the rig on is byte-identical to the one-lamp build.
    /// See [`impasto_rig`].
    pub(super) impasto_rig: impasto_rig::LightRig,
    // (`impasto_shine` used to live HERE, canvas-global, while its own doc-comment called it "a
    // property of the PAINT". It is the paint's, and paint is per-pixel — so it moved to `BrushSpec`
    // and is baked into the canvas with the stroke, like Depth and Body. Enio, 2026-07-13.)
    /// **Adjust Last Stroke** — whether moving a slider re-derives the stroke already on the canvas.
    /// ON (the default, and how the section has always behaved): the artist lays a stroke and then
    /// dials it in *while looking at it* — every knob in the Body and Material cards re-derives the last
    /// stroke live, because the stroke stored its INGREDIENTS rather than its result.
    ///
    /// OFF: the sliders speak only to the strokes still to come. The stroke on the canvas is FINISHED —
    /// which is what an artist wants the moment they are happy with it and start dialling the brush in
    /// for the next one (Enio, 2026-07-13). It is a property of the EDITING SESSION, not of the paint,
    /// so it lives here and is never baked into a pixel.
    pub(super) impasto_live_edit: bool,
    /// **Watercolor render-path** per-stroke water DWELL (1 byte/px, `w*h`): how long the held brush
    /// soaked each pixel ([`PainterTool::grow_wet_soak`], tick heartbeat). The rewet reads it `0..1`:
    /// more soak ⇒ the dissolve reaches FARTHER (blur-scale lerp) and the lift digs DEEPER — "quanto
    /// mais a água fica, mais dissolve", without physics. Lazily sized; persists through the WET
    /// SESSION (cleared on a fresh one).
    pub(super) wet_soak: Vec<u8>,
    /// Current soak disc = the last dab's `(centre, radius)` — where the tick heartbeat pours dwell
    /// while the pointer is parked. `None` = stroke start.
    pub(super) wet_soak_pos: Option<([f32; 2], f32)>,
    /// Whether THIS stroke poured any soak yet — gates the composite's 2×-blur (far) fields, so a
    /// stroke with no dwell pays exactly the plain 4-blur rewet cost.
    pub(super) wet_soak_active: bool,
    /// Manual Shape stamp (Automatic OFF): the per-stroke **tip-density** buffer (`w*h`, `0..255`;
    /// sized lazily with the coverage). A TEXTURED tip must not HOLE the wash — in real watercolor
    /// the water fills the tip's outer silhouette while the texture modulates the PIGMENT deposited —
    /// so the coverage splat stores the saturated wetness ENVELOPE and this buffer carries the tip's
    /// texture (max-blend, "one pass"), which the composite multiplies into the interior fill term
    /// (`cw·fill·dens`): typical watercolor body + rim at the OUTER boundary, tip texture as pigment
    /// variation within. Empty / untouched ⇒ density 1 (byte-identical). Doc 13 #1 round 3.
    pub(super) stroke_density: Vec<u8>,
    /// Wet Mix (MIX-1): the per-stroke **pigment-reserve** map (`w*h`, `0..255`; lazily sized, only
    /// while the mixer is on). Charge depletion must fade the PIGMENT, never the WATER — scaling the
    /// coverage instead leaves `inner` short of 1.0 interior-wide, the edge term floods the centre
    /// and the wash reads as a flat opaque slab (Enio smoke 2026-07-08, "matou a borda em qualquer
    /// valor < 0.93"). Carries each dab's fresh+carry reserve (max-blend: re-inking a faded trail
    /// restores it); the composite multiplies it into the whole BRUSH density term (fill + edge)
    /// AFTER the rim derives from intact coverage — head keeps the watercolor anatomy, the tail
    /// fades rim + body toward plain water. Empty ⇒ factor 1 (byte-identical default).
    pub(super) stroke_deplete: Vec<u8>,
    /// EDGE-1 (doc 12): canvas-wide MOISTURE map (`w*h`) surviving pen-up — dries on the heartbeat
    /// (~8.5 s, DiVerdi/Adobe; Curtis wet-area mask); the bake pours the HARDENED coverage
    /// (max-blend). While wet, watercolor strokes CONTINUE one **wet session**
    /// ([`PainterTool::wet_session_continues`]) — the buffers accumulate the UNION over the session
    /// base: one wash, one rim. Empty = dry (tick drops it + the session).
    pub(super) canvas_wet: Vec<u8>,
    /// Live bounding rect of the wet area (the decay/pour window) — `None` = dry, zero idle cost.
    ///
    /// ⚠️ **Ele ENCOLHE.** A secagem é edges-to-centre por desenho, então a área molhada recua; o rect
    /// é re-derivado do conjunto NÃO-ZERO a cada passe de decaimento (2026-08-02), e não é mais a união
    /// histórica que só crescia. A troca é **byte-idêntica na tinta**: a lei de fronteira do decaimento
    /// já conta *fora do rect* como seco, e fora da bbox do não-zero o valor **é** zero — o vizinho lido
    /// dá o mesmo número pelas duas rotas. O que muda é o CUSTO, e ele é pago por dois consumidores
    /// (o decaimento e o véu de umidade do shell).
    pub(super) canvas_wet_rect: Option<(usize, usize, usize, usize)>,
    /// Scratch de UMA LINHA para o decaimento (a linha de cima, `up`) — persistente para não alocar por
    /// quadro. Ver [`PainterTool::dry_canvas_wet`]: a janela deslizante substituiu o snapshot do rect
    /// inteiro, e este buffer é tudo o que sobrou dele.
    pub(super) canvas_wet_snapshot: Vec<u8>,
    /// Fractional drying carry between whole-byte decay steps (heartbeat dt accumulator).
    pub(super) canvas_wet_carry: f32,
    /// EDGE-1 (doc 13 #11): paper drying RATE in wetness-bytes/second — CANVAS-level (not per-brush,
    /// so it never varies by paint mode). The Wetness card's Drying-Time slider drives it
    /// (`set_dry_time_s`, seconds → `255/seconds`); default `CANVAS_WET_DRY_DEFAULT` (~10 s).
    pub(super) dry_rate_per_s: f32,
    /// #12a (doc 14): the on-canvas wetness PREVIEW strength — the max veil alpha the shell paints over
    /// the wet region (`0` = no preview). CANVAS-level display setting (not per-brush); the Wetness card's
    /// slider drives it (`set_wet_preview_intensity`), the shell reads [`Self::wet_preview_intensity`].
    pub(super) wet_preview_intensity: f32,
    /// #3 (doc 14, Enio 2026-07-11): the **Wet the layer** forced rewet. `wet_canvas_now` (the Wet button)
    /// re-opens a wet session over the current canvas and sets this so strokes made now LIFT/blend the
    /// EXISTING paint even with the brush's own Rewet at `0` (Rebelle "Wet the layer"). `0` = no forcing
    /// (byte-identical). Cleared by [`Self::dry_session_now`] (the session's teardown / drying deadline).
    pub(super) wet_session_wetness: f32,
    /// #3 (doc 13): a SHAPE-editor watercolor preview is live (Curve/Line/Circle/Polygon/Free Hand with
    /// Watercolor on). Shapes have no `paint_begin`, so the wash ground (backdrop / substrate — expensive,
    /// static within the session) is frozen ONCE lazily on the first `stamp_drag_preview_watercolor` and
    /// this flag guards the rebuild; torn down (false) at the shape commit / cancel. `false` = no shape
    /// wash session ⇒ the freehand path is untouched.
    pub(super) wet_shape_active: bool,
    /// EDGE-1 per-stroke style (doc 13 topo): session param table + per-pixel owner map — an
    /// older wash keeps ITS look on the union re-bake ([`watercolor_field::WetSessionStyles`]).
    pub(super) wet_styles: watercolor_field::WetSessionStyles,
    /// EDGE-2 backrun: the CARRIED-water pool (`w*h`, session-scoped) — Dilution pours it per dab
    /// regardless of pigment; the composite lifts/blooms against it (serrated ring = backrun
    /// edge). Separate from the session dwell soak (`wet_soak`). Empty = inert.
    pub(super) stroke_water: Vec<u8>,
    /// EDGE-1 wet session: the optical base frozen at the SESSION start (first stroke of the wet
    /// window) — every bake of the session re-composites the UNION buffers over this, never over
    /// its own previous bake (which would double-count). Per-stroke `watercolor_base` (refrozen
    /// each pen-down, so it INCLUDES the union baked so far) keeps serving the mixer pickup and
    /// the rewet fields.
    pub(super) wet_session_base: Option<Arc<Vec<u8>>>,
    /// EDGE-1 wet session guard: the exact `canvas_rgba` Arc OUR last session bake produced. Any
    /// foreign mutation (undo, layer switch, fill, resize, other tools) swaps the canvas Arc, so a
    /// failed `Arc::ptr_eq` at pen-down ends the session — no per-site invalidation hooks needed.
    pub(super) wet_session_canvas: Option<Arc<Vec<u8>>>,
    /// **Live-editable wash** (Enio 2026-07-11): the LAST committed wash stays re-renderable while the
    /// paper is still wet (until the next stroke or Dry), so changing a Grain/Paper texture param
    /// (Size/Angle/Offset/kind/…) re-renders the whole wash — central AND every Tiling copy — instead of
    /// only affecting the NEXT stroke. This is the pre-wash BASE + the frozen GROUND our last bake
    /// composited over; `apply_watercolor` reconstructs the wash from them over [`Self::wet_editable_region`]
    /// with the CURRENT brush texture. `None` ⇒ no editable wash (byte-identical: nothing re-renders).
    pub(super) wet_editable_base: Option<Arc<Vec<u8>>>,
    /// The frozen GROUND ([`Self::wet_backdrop`]) of the editable wash — kept past `close_stroke` (which
    /// drops the live one) so the re-render's Beer–Lambert base matches the committed look.
    pub(super) wet_editable_backdrop: Option<Arc<Vec<u8>>>,
    /// The committed wash's footprint (already full-axis on a tiled axis, from `dab_batch_region`), so the
    /// live re-render touches exactly the wash + its Tiling copies, not the whole canvas.
    pub(super) wet_editable_region: Option<Region>,
    /// The **substrate signature** the editable wash was last rendered with — the paint tick
    /// re-renders when the live brush differs, then refreshes this. `None` ⇒ inert. It used to be
    /// just `(Grain, Paper)` `TextureSettings`, which left the rest of the substrate OUT (sweep
    /// 2026-07-12): **Paper Depth**/**Granulation** live on `BrushSpec`, and swapping the Paper/Grain
    /// IMAGE under `kind: Image` changes only the pixel version — so dragging Paper *Size*
    /// re-rendered the wet pool while Paper *Depth*, right beside it, did nothing.
    pub(super) wet_editable_tex: Option<wet_editable::WetEditableSig>,
    /// Manual Shape stamp (Automatic OFF): the tip image's luminance NORMALISER (`1 / max_lum`;
    /// `1.0` when no image / all-black). The watercolor coverage is WETNESS GEOMETRY — a max-blend
    /// union that must SATURATE in the wash core (`cw → 1` body, `inner → 1` rim confinement) — not
    /// the plain brush's tonal per-dab alpha, so a raw grey tip starved the optics: pale centre, no
    /// rim (Enio 2026-07-07). Scaling by this keeps the tip's RELATIVE texture with a saturating
    /// core. Computed once per stroke at pen-down (`freeze_watercolor_ground`).
    pub(super) wet_shape_norm: f32,
    /// **Watercolor substrate cache** (perf, byte-identical): the paper-tooth `paper_h` per canvas
    /// pixel (`f32`, `w*h`; `NaN` = not computed). The paper is CANVAS-ANCHORED, yet the composite
    /// recomputed it (~28 integer-hashes) every frame — this memoises it: filled on first touch,
    /// reused by later frames AND the pen-up bake. **Reset to all-`NaN` at pen-down**
    /// ([`PainterTool::freeze_watercolor_ground`]) so a stroke never reads a previous stroke's
    /// settings (the paper cannot change mid-stroke ⇒ no in-stroke invalidation to get wrong).
    /// Empty outside a watercolor stroke. Pure memoisation keyed by canvas index ⇒ byte-identical.
    pub(super) wet_substrate: Vec<f32>,
    /// **Watercolor mixer** (Wet Mix — `wet_charge`/`wet_pull`/`wet_dilution`, `docs/Painter/07` §4)
    /// per-stroke state: the picked-up colour reservoir (unpremultiplied rgb + a presence-weighted
    /// confidence `w`) and its `recentness` (the Pull-gated resample clock). The brush deposits
    /// `lerp(brush, reservoir, (1−charge)·w)` — it picks up the frozen surface it crosses and (with
    /// Pull) drags it downstream. Reset on pen-down; inert unless `wet_charge < 1` (default → skipped,
    /// byte-identical). See [`super::watercolor_mixer`].
    pub(super) wet_mix: watercolor_mixer::WetMix,
    /// The previous dab centre of the Smudge TRUE-SMEAR chain (`None` = stroke start / no smear yet).
    /// With `wet_smudge > 0` each dab DRAGS the frozen base's paint from here to its own centre
    /// (`smear_dab` on the forked [`Self::watercolor_base`]) before the wash composites over it — the
    /// physical "borrar" that moves already-painted paint (Enio 2026-07-06), not just a colour tint.
    pub(super) wet_smear_pos: Option<[f32; 2]>,
    /// **Watercolor render-path** per-frame dirty rect — the union footprint of the dabs accumulated
    /// since the last optical composite (wet_edges `fMin..fMax`/`resetFrame`). The live
    /// [`Self::apply_watercolor`] recomposites ONLY this (padded by the influence radius), so the
    /// per-frame cost tracks the new dabs, not the whole stroke. Consumed (reset) by each composite.
    pub(super) wet_frame_dirty: Option<Region>,
    /// **Watercolor render-path** cumulative dirty rect — the union footprint of EVERY dab this stroke
    /// (wet_edges `cMin..cMax`), tracked incrementally so the pen-up bake never scans the canvas for
    /// its bbox. [`Self::clear_wet_coverage`] folds it into the frame dirty (the cleared shape must be
    /// recomposited — the moving-preview union) before dropping it.
    pub(super) wet_cum_dirty: Option<Region>,
    /// **Watercolor render-path** THIS-STROKE dirty rect — reset every `paint_begin` (even inside a wet
    /// session, unlike [`Self::wet_cum_dirty`] which accumulates the whole session's union). Only the
    /// current stroke's OWN footprint re-wets the moisture map at the bake ([`Self::pour_canvas_wet`]),
    /// so a second stroke never resets the drying clock of the earlier washes (doc 14 #4, Enio 2026-07-11).
    pub(super) wet_stroke_dirty: Option<Region>,
    /// **Inpaint** defect mask (1 byte/px, `>= 128` ⇒ heal). Accumulated as the user brushes in Inpaint
    /// mode; on pen-up [`super::inpaint`] reconstructs the marked region and clears it. Sized `w*h`.
    pub(super) inpaint_mask: Vec<u8>,

    // ── Fill (Bucket) — Procreate ColorDrop state ([`super::fill`]). ──
    /// ColorDrop threshold (`0..1`) → per-channel colour tolerance; adjusted live by the post-drop drag.
    pub(super) fill_threshold: f32,
    /// Image-space seed of the current drop (`None` when idle).
    pub(super) fill_seed: Option<[f32; 2]>,
    /// Pre-fill layer pixels, so every threshold change re-fills from the ORIGINAL region (not the
    /// already-filled result).
    pub(super) fill_snapshot: Vec<u8>,
    /// The previous refill's filled bbox — so a SHRINKING fill dirties the vacated pixels too (the
    /// union of the old + new rects), not just the smaller new region (else the overflow ghosts).
    pub(super) fill_last_rect: Option<Region>,
    /// The mode id to RESTORE after a momentary **ColorDrop** (C&F drag) finalizes (`None` = deliberate Fill).
    /// Set by the shell's C&F drag, consumed by `fill_commit` / `fill_cancel` ([`fill`], Enio 2026-07-03).
    pub(super) fill_return_mode: Option<String>,
    /// **Inpaint** Patch Size (`0..1` track → patch radius `2..=6`); the reconstruction's patch footprint.
    pub(super) inpaint_patch_norm: f32,
    /// **Inpaint** Quality (`0..1` track → EM iterations `3..=12`); more iterations = better fit, slower.
    pub(super) inpaint_quality_norm: f32,
    /// **Inpaint** Search (`0..1` track → context-margin multiplier `0.5..3.0`); how much surrounding
    /// context PatchMatch samples from around the hole.
    pub(super) inpaint_search_norm: f32,
    /// Per-stroke per-layer-colour accumulation (recomposite); see [`stamp_color_cache`].
    pub(super) per_layer_stroke: stamp_color_cache::PerLayerStroke,
    /// For the dab list currently being stamped: which ORIGINAL dab each entry was replicated from
    /// ([`tiling::tiled_dabs_grouped`]). Empty ⇒ no Tiling ⇒ every entry is its own dab. The routes feed
    /// it to [`tiling::DabRng`] so a dab's wrapped copies SHARE its random frame — they are the same dab
    /// seen from both sides of the seam, and a per-copy draw made the tile stop matching itself.
    pub(super) dab_groups: Vec<u32>,
    /// Cached coloured Shape **preview** (premul RGBA), re-baked only on appearance change; [`stamp_color_cache`].
    pub(super) shape_color_preview: stamp_color_cache::ShapeColorPreview,
    /// **Deform** (Liquify) settings + session state — sub-mode, brush knobs, Freeze, and the pre-deform
    /// buffer Reconstruct/Amount read from. Mode-exclusive; see [`warp`] (Deform Wave 1).
    pub(super) deform: warp::DeformState,
    /// The frozen baselines + cumulative displacement shared by Deform and Smear (`warp::session`).
    /// Mode-exclusive: at most one tool has a live session, and leaving that mode ends it.
    pub(super) warp: warp::session::WarpSession,
    /// **Sculpt** settings + per-stroke session — the sub-mode, the kernel Radius, and the frozen relief
    /// plus the accumulated intensity the re-render reads. Unlike Deform this is NOT mode-exclusive: the
    /// sculpt rides the same dab list the colour does, so the brush's own knobs are its knobs. See
    /// [`sculpt`].
    pub(super) sculpt: sculpt::SculptState,
    /// **Wet Paint** session (fluid engine + frozen base) — display-state, not document-state; the
    /// canvas-identity guard, the undo stance and the composite live in [`wetpaint`] (ADR-0134).
    pub(super) wetpaint: wetpaint::WetPaintState,
}
