use super::*;

/// Snapshot of the selected sprite's editor-facing fields. Host
/// rebuilds this each frame from `gizmo_selection` + SimWorld;
/// inspector renders read-only display + a Reimport button.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorSpriteInfo {
    /// Entity bits (= same shape `gizmo_selection` carries).
    pub entity_bits: u64,
    /// Display label — entity's `Name` component, or
    /// `Entity_{hex_bits}` when nameless.
    pub name: String,
    /// World-space size in meters at the current Transform scale.
    pub world_size: [f32; 2],
    /// Which storage strategy backs the sprite (Atlas / Hand-packed
    /// / Individual). Surfaced as a read-only display for now;
    /// switching strategies is M14.5 follow-up.
    pub source_kind: InspectorSpriteSource,
    /// Source-image dimensions (pixels). `None` for procedural /
    /// generated sprites that don't trace back to an `AssetId`.
    pub source_pixels: Option<(u32, u32)>,
    /// `true` when Reimport is meaningful — the entity's source
    /// resolves to an `AssetId` we can re-decode at the new px/m.
    pub can_reimport: bool,
    /// Logical horizontal flip (mirrors sampled U; survives reparenting).
    /// Editable via the Render Source / Sprite Sheet Flip toggles (W2).
    pub flip_x: bool,
    /// Logical vertical flip (mirrors sampled V).
    pub flip_y: bool,
    /// Final opacity multiplier `[0, 1]` (Color & Tint section). Renders
    /// today via `RenderInstance.opacity`.
    pub opacity: f32,
    /// Silhouette mode — texel RGB ignored, tint RGB fills. Renders today
    /// via `flip_uv` bit 2.
    pub tint_fill: bool,
    /// Sprite-sheet columns (`>= 1`). Renders today: the extract slices
    /// the atlas rect into an hframes×vframes grid.
    pub hframes: u32,
    /// Sprite-sheet rows (`>= 1`).
    pub vframes: u32,
    /// Active sheet frame index (`< hframes*vframes`).
    pub frame: u32,
    /// Inherited modulate (`Sprite::tint`, cascades to children).
    /// Linear RGBA `[0, 1]`. Edited via the Color & Tint section's
    /// Tint swatch (W2.T2.7); renders today via `RenderInstance.tint`.
    pub tint: [f32; 4],
    /// Local modulate (`Sprite::self_tint`, does NOT cascade). Linear
    /// RGBA `[0, 1]`. Edited via the Self Tint swatch; multiplies
    /// `tint` for this sprite only (Godot `self_modulate` semantics).
    pub self_tint: [f32; 4],
    /// Per-corner tint `[TL, TR, BL, BR]` — a 4-stop bilinear gradient.
    /// Each entry linear RGBA `[0, 1]`; default WHITE (no gradient).
    /// Edited via the Per-corner 2×2 swatch grid (W2.T2.7); renders via
    /// the shader's `@location(9..12)` per-corner attributes.
    pub per_corner_tint: [[f32; 4]; 4],
    /// Region sampling on/off (`Sprite::region_enabled`). When `true`,
    /// the sprite samples only `region_rect` of its source. Edited in the
    /// Render Source section (W2.T2.4); renders via the extract sub-UV.
    pub region_enabled: bool,
    /// Region sub-rect `[x, y, w, h]` in SOURCE pixels
    /// (`Sprite::region_rect`). Only meaningful when `region_enabled`.
    pub region_rect: [f32; 4],
    /// Region filter clip (`Sprite::region_filter_clip`) — clamps the
    /// sampler to `region_rect` (half-texel inset) to stop atlas bleed.
    pub region_filter_clip: bool,
    /// Origin mode (`Sprite::centered`): `true` (default) = quad center;
    /// `false` = texture top-left + `offset`. Edited in the Sprite Sheet
    /// section (W2.T2.6); renders via `Sprite::resolve_anchor`.
    pub centered: bool,
    /// Intrinsic image offset in pixels (`Sprite::offset`), applied after
    /// `centered`. Renders via `Sprite::resolve_anchor` (px → local m).
    pub offset: [f32; 2],
    /// Number of sprites in the active selection (primary + extras).
    /// `1` for a single selection; `> 1` enables BulkSelect (T2.0): edits
    /// apply to all, and diverging fields show as "Mixed" via [`mixed`].
    ///
    /// [`mixed`]: InspectorSpriteInfo::mixed
    pub selected_count: usize,
    /// Per-field "values diverge across the selection" flags (BulkSelect).
    /// All `false` for a single selection. A `true` flag makes the field
    /// render its Mixed affordance (checkbox → Indeterminate, NumberInput
    /// → blank) so editing it doesn't silently stomp the diverging values.
    pub mixed: InspectorSpriteMixed,
}

/// BulkSelect (T2.0): which editable `Sprite` fields diverge across a
/// multi-selection. Computed by the host each frame (compare every
/// selected sprite against the primary). Default = nothing mixed (the
/// single-selection case). The Inspector reads these to show "Mixed"
/// affordances instead of a misleading single value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorSpriteMixed {
    pub flip_x: bool,
    pub flip_y: bool,
    pub tint_fill: bool,
    pub centered: bool,
    pub region_enabled: bool,
    pub region_filter_clip: bool,
    pub opacity: bool,
    pub hframes: bool,
    pub vframes: bool,
    pub frame: bool,
    pub offset_x: bool,
    pub offset_y: bool,
    pub region_x: bool,
    pub region_y: bool,
    pub region_w: bool,
    pub region_h: bool,
    pub tint: bool,
    pub self_tint: bool,
    /// Any of the 4 per-corner tints diverge.
    pub per_corner: bool,
}

/// A single editable `Sprite` field, dispatched Inspector → shell as
/// [`EditorAction::InspectorSpriteEdit`]. The shell reads the entity's
/// current `Sprite`, applies the one field (clamping where the schema
/// requires), and commits the whole struct via
/// `EditorCommand::SetComponent` — the same write path the Transform
/// commit uses. Payloads are primitives so this enum stays free of any
/// `ph2d-render` dependency (editor-core must not depend on the renderer).
///
/// Variants are added as each W2 section lands; only the ones a wired
/// section emits are produced today (Flip in W2.T2.4/T2.5). The full set
/// is declared up front so the action contract is stable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SpriteFieldEdit {
    /// Logical horizontal flip.
    FlipX(bool),
    /// Logical vertical flip.
    FlipY(bool),
    /// `false` = top-left origin + `offset` applies; `true` = centered.
    Centered(bool),
    /// Intrinsic-pixel offset (whole vector) applied after `centered`.
    Offset([f32; 2]),
    /// Intrinsic-pixel offset X only — leaves Y untouched. The Inspector
    /// emits this (not [`Offset`]) so editing one axis on a multi-selection
    /// can't stomp a diverging Y (BulkSelect, audit D-1).
    ///
    /// [`Offset`]: SpriteFieldEdit::Offset
    OffsetX(f32),
    /// Intrinsic-pixel offset Y only — leaves X untouched. See [`OffsetX`].
    ///
    /// [`OffsetX`]: SpriteFieldEdit::OffsetX
    OffsetY(f32),
    /// Sprite-sheet columns (`>= 1`; clamped at the commit boundary).
    Hframes(u32),
    /// Sprite-sheet rows (`>= 1`).
    Vframes(u32),
    /// Active frame index (`< hframes * vframes`; clamped).
    Frame(u32),
    /// Region (sub-rect) sampling on/off.
    RegionEnabled(bool),
    /// Region rect `[x, y, w, h]` in source pixels (whole vector).
    RegionRect([f32; 4]),
    /// Region rect X only — leaves Y/W/H untouched (BulkSelect, audit D-1).
    RegionX(f32),
    /// Region rect Y only.
    RegionY(f32),
    /// Region rect W only (`>= 0`; clamped at commit).
    RegionW(f32),
    /// Region rect H only (`>= 0`; clamped at commit).
    RegionH(f32),
    /// Clamp the sampler to the region (atlas-bleed guard).
    RegionFilterClip(bool),
    /// Inherited modulate (cascades to children).
    Tint([f32; 4]),
    /// Local modulate (does NOT cascade).
    SelfTint([f32; 4]),
    /// Silhouette mode — texel RGB ignored, tint RGB fills.
    TintFill(bool),
    /// Final opacity multiplier (`[0, 1]`; clamped).
    Opacity(f32),
    /// Per-corner bilinear tint `[TL, TR, BL, BR]`.
    PerCornerTint([[f32; 4]; 4]),
}

/// Snapshot of the selected entity's W3 ordering/sorting components
/// published to the Inspector §7 (Ordering / Sorting). Every field is
/// *optional* (the components are presence-overrides, spec §02): `None`
/// / `false` markers mean "component absent → pipeline default". Raw
/// primitives keep editor-core loose-coupled from `ph2d-ecs`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorOrderingInfo {
    pub entity_bits: u64,
    /// `ZIndexOverride` — `None` = absent (DFS counter). `Some(v)` =
    /// forced Z (spec §3.7: "Z Index: —" vs explicit).
    pub z_index: Option<i32>,
    /// `ZAsRelative.0` — only meaningful when `z_index.is_some()`.
    pub z_as_relative: bool,
    /// `ShowBehindParent` marker present.
    pub show_behind_parent: bool,
    /// `SortingLayer.0.0` (LayerId index); default-layer index when absent.
    pub sorting_layer: u8,
    /// `OrderInLayer.0`.
    pub order_in_layer: i32,
    /// `YSort.enabled` (false when the component is absent).
    pub y_sort_enabled: bool,
    /// `YSort.sort_point` as a tag: 0 Center · 1 Pivot · 2 Custom.
    pub y_sort_point: u8,
    /// `YSort.axis` (only meaningful when `y_sort_point == 2`).
    pub y_sort_axis: [f32; 2],
    /// `SortingGroup` present.
    pub sorting_group: bool,
    /// `SortingGroup.sort_at_root` (only meaningful when `sorting_group`).
    pub sort_at_root: bool,
    /// `TopLevel` marker present.
    pub top_level: bool,
    pub selected_count: usize,
    pub mixed: InspectorOrderingMixed,
}

/// BulkSelect (T2.0) divergence flags for the §7 ordering fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorOrderingMixed {
    pub z_index: bool,
    pub z_as_relative: bool,
    pub show_behind_parent: bool,
    pub sorting_layer: bool,
    pub order_in_layer: bool,
    pub y_sort_enabled: bool,
    pub y_sort_point: bool,
    pub y_sort_axis: bool,
    pub sorting_group: bool,
    pub sort_at_root: bool,
    pub top_level: bool,
}

/// A single editable §7 ordering field, dispatched Inspector → shell as
/// [`EditorAction::InspectorOrderingEdit`]. Unlike [`SpriteFieldEdit`]
/// (which mutates the always-present `Sprite`), each variant maps to an
/// *optional* ECS component: the shell reads the component-or-default,
/// applies the edit, and commits via `EditorCommand::SetComponent`
/// (insert/update) or `EditorCommand::RemoveComponent` (detach). The
/// full set is declared up front so the action contract is stable; only
/// wired controls emit today (spec §3.7).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OrderingFieldEdit {
    /// `Some(v)` attaches/updates `ZIndexOverride(v)` (clamped to
    /// ±i32::MAX/2 at commit); `None` detaches it (back to DFS).
    ZIndex(Option<i32>),
    /// `ZAsRelative(b)` (attaches the component if absent).
    ZAsRelative(bool),
    /// Toggle the `ShowBehindParent` marker (insert / remove).
    ShowBehindParent(bool),
    /// `SortingLayer(LayerId(idx))`.
    SortingLayer(u8),
    /// `OrderInLayer(v)`.
    OrderInLayer(i32),
    /// `YSort.enabled` (read-modify-write the YSort component).
    YSortEnabled(bool),
    /// `YSort.sort_point` tag: 0 Center · 1 Pivot · 2 Custom.
    YSortPoint(u8),
    /// `YSort.axis`.
    YSortAxis([f32; 2]),
    /// Toggle `SortingGroup` presence (insert default / remove).
    SortingGroup(bool),
    /// `SortingGroup.sort_at_root` (attaches the component if absent).
    SortAtRoot(bool),
    /// Toggle the `TopLevel` marker (insert / remove).
    TopLevel(bool),
}

/// Snapshot of the selected entity's W3 §9 sampling components
/// (`TextureFilter`/`TextureRepeat`). Tags are the
/// `ph2d_ecs::FilterMode`/`RepeatMode` discriminants; `0` = `Inherit`
/// (component absent or explicitly Inherit). Raw primitives keep
/// editor-core loose-coupled from `ph2d-ecs`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorSamplingInfo {
    pub entity_bits: u64,
    pub filter_tag: u8,
    pub repeat_tag: u8,
    /// `UvTransform.scale` (W3 tiling; `[1,1]` = no tiling).
    pub uv_scale: [f32; 2],
    /// `UvTransform.offset` (W3 scroll; `[0,0]` = none).
    pub uv_offset: [f32; 2],
    pub selected_count: usize,
    pub mixed: InspectorSamplingMixed,
}

/// BulkSelect divergence flags for the §9 sampling fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorSamplingMixed {
    pub filter: bool,
    pub repeat: bool,
    pub uv_scale: bool,
    pub uv_offset: bool,
}

/// A single editable §9 sampling field, dispatched as
/// [`EditorAction::InspectorSamplingEdit`]. Filter/Repeat map to the
/// optional `TextureFilter`/`TextureRepeat` (tag `0` = detach); UV
/// scale/offset map to the optional `UvTransform`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SamplingFieldEdit {
    /// `FilterMode` tag (`0 Inherit … 6 LinearAniso`).
    Filter(u8),
    /// `RepeatMode` tag (`0 Inherit · 1 Disabled · 2 Enabled · 3 Mirror`).
    Repeat(u8),
    /// `UvTransform.scale.x` (W3 tiling). Read-modify-write the component.
    UvScaleX(f32),
    UvScaleY(f32),
    /// `UvTransform.offset.x` (W3 scroll).
    UvOffsetX(f32),
    UvOffsetY(f32),
}

/// §10 Material & Blend-section snapshot. Mirrors the optional ECS
/// `BlendMode` component (absent = `Mix`); editor-core stays loose-
/// coupled from `ph2d-ecs` (carries the resolved `blend_tag` only).
/// Material/InstanceShaderParams are a future runtime — the section
/// shows a placeholder for them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InspectorBlendInfo {
    pub entity_bits: u64,
    /// `BlendMode::tag()` (`0 Mix · 1 Add · 2 Subtract · 3 Multiply ·
    /// 4 Screen · 5 PremultAlpha`). `0` = component absent (default).
    pub blend_tag: u8,
    pub selected_count: usize,
    pub mixed: InspectorBlendMixed,
}

/// BulkSelect divergence flags for the §10 blend fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorBlendMixed {
    pub blend: bool,
}

/// A single editable §10 blend field, dispatched as
/// [`EditorAction::InspectorBlendEdit`]. Maps to the optional
/// `BlendMode` component (tag `0` = `Mix` = detach the component).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendFieldEdit {
    /// `BlendMode` tag (`0..5`); `0` (Mix) detaches the component.
    Blend(u8),
}

/// §11 Physics Body-section snapshot (ADR-0131 D8). Mirrors the optional
/// `RigidBody` + `Collider` pair from `ph2d-physics-ecs`; editor-core stays
/// loose-coupled from both `ph2d-ecs` and the physics crate, so the shell
/// maps tags ↔ enums at the boundary — the same discipline as §10's
/// `blend_tag`.
///
/// **`has_body` is the whole reason this is `Some` for a plain sprite.** The
/// other sections describe something that exists; this one also has to offer
/// the thing that does not yet, or a body could never be authored at all —
/// physics would be reachable only from a smoke scene.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorPhysicsInfo {
    pub entity_bits: u64,
    /// Does this entity carry `RigidBody` + `Collider` right now?
    pub has_body: bool,
    /// `0` Dynamic · `1` Static.
    pub kind_tag: u8,
    /// `0` Ball · `1` Box.
    pub shape_tag: u8,
    /// Ball radius, meters (meaningless when `shape_tag == 1`).
    pub radius: f32,
    /// Box HALF-extents, meters (meaningless when `shape_tag == 0`).
    pub half_x: f32,
    pub half_y: f32,
    pub density: f32,
    pub restitution: f32,
    pub friction: f32,
    /// Which collision layer this body is on (`0..MAX_LAYERS`).
    ///
    /// The per-body half of collision layers; the other half — *which layers
    /// collide with which* — is a WORLD rule and lives in the Physics panel.
    /// Splitting it this way is the whole point: the rule is authored once,
    /// and a body only says where it belongs.
    pub layer: u8,
    /// How many seconds a Bake would cover, resolved by the shell (W4): the
    /// armed loop if there is one, else the document's extent, else the
    /// measured default.
    ///
    /// Shown ON the button, because a button whose effect depends on an
    /// invisible number is a button you have to experiment with. The shell
    /// resolves it once and both halves read the same answer — the painter to
    /// label it, the bake to honour it.
    pub bake_seconds: f32,
    /// Is the current selection exactly **two** bodies? Then §11 offers the
    /// Join button (W3).
    ///
    /// The precondition is answered by the shell, which is the half that owns
    /// the selection — the panel sees one entity at a time and could not work
    /// it out. It is a snapshot field rather than a check inside the painter
    /// for the usual reason: the painter decides whether to OFFER the button
    /// and the event handler decides whether to HONOUR the click, and both
    /// have to read the same fact.
    pub can_join: bool,
    /// Is this collider a **sensor** (trigger, W7)? Passes through, reports overlaps; the overlay lights it up. `false` is solid.
    pub is_sensor: bool,
    /// Which pose channels the Bake writes: `0` All · `1` Position · `2` Rotation (a global bake option the shell owns).
    pub bake_channels_tag: u8,
}

/// A single editable §11 physics field, dispatched as
/// [`EditorAction::InspectorPhysicsEdit`].
///
/// [`PhysicsFieldEdit::Add`] carries no geometry on purpose: the default
/// collider is derived from the sprite's own bounds **by the shell**, which
/// is the half that knows how big the art is. A collider that starts as the
/// sprite's box is the one shape that can never disagree with what is drawn.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PhysicsFieldEdit {
    /// Attach `RigidBody{Dynamic}` + a `Collider` boxed to the sprite.
    Add,
    /// Detach both components — the entity goes back to being plain art.
    Remove,
    /// `BodyKind` tag: `0` Dynamic · `1` Static · `2` Kinematic.
    Kind(u8),
    /// `ColliderShape` tag: `0` Ball · `1` Box. Switching preserves the
    /// footprint (a box becomes the ball that fits it, and back).
    Shape(u8),
    Radius(f32),
    HalfX(f32),
    HalfY(f32),
    Density(f32),
    Restitution(f32),
    Friction(f32),
    /// Move this body to a collision layer (`0..MAX_LAYERS`).
    Layer(u8),
    /// The "Solid | Sensor" toggle (W7): make this collider a sensor or solid.
    Sensor(bool),
    /// Which pose channels the Bake writes: `0` All · `1` Position · `2` Rotation.
    BakeChannels(u8),
    /// Create a joint between the two selected bodies (W3). Carries no
    /// operands: the shell owns the selection, and a second copy of "which
    /// two" would be a second answer to a question only one half can answer.
    Join,
    /// Bake the selection's simulated motion into timeline curves (W4).
    ///
    /// Carries no range for the same reason `Join` carries no bodies: the
    /// shell owns the clock, and the panel only shows the number it is told
    /// ([`InspectorPhysicsInfo::bake_seconds`]). And like `Join`, it must NOT
    /// fan out over a multi-selection — one bake covers every selected body in
    /// one run of the simulation, where a fan-out would re-simulate the whole
    /// scene once per body and file a separate undo step for each.
    Bake,
}

/// §12 Physics Joint snapshot (W3) — the selected **joint object**.
///
/// A joint is an entity, so this is the section that describes it: what kind
/// of constraint it is, which two bodies it names, and the parameters that
/// kind actually uses. Not `Copy`, unlike its siblings, because it carries the
/// two bodies' NAMES — the joint stores name hashes, and a hash is not
/// something to show a person.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorJointInfo {
    pub entity_bits: u64,
    /// `0` Pin · `1` Spring · `2` Rope.
    pub kind_tag: u8,
    /// The bodies, resolved for display. Empty means the name no longer
    /// matches any body in the scene — deleted or renamed.
    pub body_a_name: String,
    pub body_b_name: String,
    /// Are BOTH bodies present right now? The section says so out loud: a
    /// joint whose body was renamed is dormant, not broken, and silently
    /// showing its parameters as if it were live would be a lie.
    pub bound: bool,
    pub limits_enabled: bool,
    /// **Degrees** at this boundary; the component stores radians, exactly as
    /// `rotation_rad` does.
    pub limit_min_deg: f32,
    pub limit_max_deg: f32,
    pub motor_enabled: bool,
    /// Degrees per second.
    pub motor_speed_deg: f32,
    pub motor_max_force: f32,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_length: f32,
}

/// A single editable §12 joint field, dispatched as
/// [`EditorAction::InspectorJointEdit`](crate::action_bus::EditorAction).
///
/// Angles arrive in **degrees** and the shell converts, so the panel never
/// holds a radian and the component never holds a degree.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum JointFieldEdit {
    /// `JointKind` tag: `0` Pin · `1` Spring · `2` Rope.
    Kind(u8),
    LimitsEnabled(bool),
    LimitMinDeg(f32),
    LimitMaxDeg(f32),
    MotorEnabled(bool),
    MotorSpeedDeg(f32),
    MotorMaxForce(f32),
    RestLength(f32),
    Stiffness(f32),
    Damping(f32),
    MaxLength(f32),
    /// Delete the joint object.
    Remove,
}

/// W3 §8 Visibility-section snapshot (the collapsible section body, NOT
/// the always-on "Visible" toggle — that stays [`InspectorVisibilityInfo`]).
/// Mirrors the optional ECS components `VisibilityLayer` / `ClipChildren`
/// / `MaskInteraction` / `OnScreenEnabler`; editor-core stays loose-coupled
/// from `ph2d-ecs` so the shell maps tags ↔ enums at the boundary.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorVisibilitySectionInfo {
    pub entity_bits: u64,
    /// `VisibilityLayer` bitmask; absent component → `u32::MAX` (all 32
    /// layers, the canonical "visible to every camera" default).
    pub layer_mask: u32,
    /// `ClipChildren.mode` tag (`0 Disabled · 1 ClipOnly · 2 ClipAndDraw`).
    pub clip_mode: u8,
    /// `MaskInteraction.mode` tag (`0 None · 1 VisibleInside · 2 VisibleOutside`).
    pub mask_mode: u8,
    /// `MaskInteraction.alpha_cutoff` (`[0,1]`; shown when mask != None).
    pub alpha_cutoff: f32,
    /// `Mask2D` present? (this sprite is a mask SOURCE).
    pub mask_source: bool,
    /// `OnScreenEnabler` present?
    pub on_screen: bool,
    /// `OnScreenEnabler.rect` `[x, y, w, h]` world meters (shown when on).
    pub rect: [f32; 4],
    pub selected_count: usize,
    pub mixed: InspectorVisibilityMixed,
}

/// BulkSelect divergence flags for the §8 visibility-section fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorVisibilityMixed {
    pub layer_mask: bool,
    pub clip_mode: bool,
    pub mask_mode: bool,
    pub alpha_cutoff: bool,
    pub mask_source: bool,
    pub on_screen: bool,
    pub rect: bool,
}

/// A single editable §8 visibility-section field, dispatched as
/// [`EditorAction::InspectorVisibilitySectionEdit`]. Each maps to an
/// optional ECS component (presence = override): a `0`/`Disabled`/`None`
/// tag or an all-layers/false value detaches it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VisibilityFieldEdit {
    /// Toggle `VisibilityLayer` bit `n` (`0..32`) on/off.
    LayerBit(u8, bool),
    /// `ClipChildren.mode` tag (`0` detaches → no clip).
    ClipMode(u8),
    /// `MaskInteraction.mode` tag (`0` detaches → ignores masks).
    MaskMode(u8),
    /// `MaskInteraction.alpha_cutoff` (read-modify-write, keeps mode).
    AlphaCutoff(f32),
    /// `Mask2D` present? — make this sprite a mask source (`false` detaches).
    MaskSource(bool),
    /// `OnScreenEnabler` present? (`false` detaches.)
    OnScreen(bool),
    /// `OnScreenEnabler.rect` components (read-modify-write).
    RectX(f32),
    RectY(f32),
    RectW(f32),
    RectH(f32),
}

/// Mirror of `ph2d_render::SpriteSource` that doesn't depend on
/// the renderer crate. Stays small (1 enum tag + opt u32) so the
/// `Inspector*` struct is cheap to clone per frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InspectorSpriteSource {
    Atlas {
        key: u32,
    },
    Individual {
        texture_id: u32,
    },
    HandPacked,
    /// A tier-cooked KTX2 texture (KTX2 Fase 2, W2.T2). A read-only
    /// display marker — the inspector shows "Cooked texture" but offers
    /// no authoring radio (cooked sources come from the asset pipeline,
    /// not the inspector). The runtime source carries the tier-agnostic
    /// `LogicalTextureId`; the mirror drops it (display only).
    CookedTexture,
}

/// Snapshot of the selected entity's local `Transform` published to
/// the Inspector. Mirrors the canonical [`ph2d_ecs::Transform`]
/// fields as raw arrays so the editor crate stays loose-coupled to
/// glam types and the snapshot stays cheap to clone.
///
/// Lifecycle: host writes this when the selection changes (or a
/// gizmo-driven external mutation lands); inspector seeds its
/// NumberInput buffers from it; commits flow back through
/// [`HeroScreen::pending_transform_edit`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorTransformInfo {
    /// Entity bits — same shape `gizmo_selection` carries.
    pub entity_bits: u64,
    /// Local-space translation (meters). 2D-only by design — see
    /// SKILL §3 "Não é engine 3D" and ADR-0025.
    pub translation: [f32; 2],
    /// Local-space rotation in radians. The inspector renders this
    /// as degrees for UX parity with Unity/Godot/Blender; conversion
    /// happens at the paint/commit boundary via
    /// `f32::to_degrees`/`to_radians` (HR-5 bit-deterministic).
    pub rotation_rad: f32,
    /// Local-space scale (unitless). Identity = `[1.0, 1.0]`.
    pub scale: [f32; 2],
    /// Local-space skew `[skew_x, skew_y]` in radians (ADR-0025
    /// amendment-1). The inspector renders these as degrees like
    /// `rotation_rad`; conversion happens at the paint/commit boundary.
    /// Identity = `[0.0, 0.0]`. Authoring values are clamped to
    /// `Transform::SKEW_LIMIT` at the ECS-commit boundary.
    pub skew_rad: [f32; 2],
}

/// M14.D: snapshot of the selected entity's `Visibility` state.
///
/// The Inspector renders this as a single checkbox above the
/// Transform section, mirroring the eye toggle in the Hierarchy
/// panel (M14.6 A). Both surfaces drive the same underlying
/// `ph2d_ecs::Visibility { hidden: bool }` component via
/// `EditorCommand::SetComponent`.
///
/// `visible == true` ↔ no `Visibility` component OR
/// `Visibility { hidden: false }`. Absence-equals-visible is the
/// canonical invariant ([`ph2d_ecs::visibility`]); on commit, the
/// host always writes an explicit `Visibility { hidden: ... }` so
/// the round-trip is unambiguous.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InspectorVisibilityInfo {
    pub entity_bits: u64,
    pub visible: bool,
}

/// M14.E: snapshot of the selected entity's `Name` component.
///
/// Mirrors the M14.A / M14.D snapshot pattern. The Inspector renders
/// this as the editable name field at the very top of the panel body
/// (above the Visibility row + Transform section). Loose-coupled
/// (the editor crate doesn't depend on `ph2d-ecs::Name`); the shell
/// converts to/from `Name(String)` at the boundary.
///
/// `name` is the human-readable label. The host falls back to
/// `format!("Entity_{hex_bits}")` when the entity has no `Name`
/// component yet, so the field is always non-empty for a selected
/// entity (matches the existing `InspectorSpriteInfo::name` shape).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorNameInfo {
    pub entity_bits: u64,
    pub name: String,
}

/// M14.C: which sprite render-source strategy the user requested via
/// the Render Source segmented switcher in the Inspector. The host
/// translates this into the actual renderer + ECS mutations.
///
/// The variants intentionally drop the inner `key` / `texture_id`
/// fields of [`InspectorSpriteSource`] — the user is asking for a
/// *kind* change; the host picks (or allocates) the new identifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RequestedSpriteStrategy {
    Atlas,
    Individual,
    HandPacked,
}

/// Which framing action the VIEW button (TOOL_HOME) + F/Home key
/// should run when the user triggers a reframe.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ViewFocusKind {
    /// Focus on the currently selected sprite. Falls back to (0,0)
    /// when no selection. Doesn't change zoom.
    Selected,
    /// Focus the active camera. No camera-object exists yet, so the
    /// host pans to (0,0). Doesn't change zoom.
    Camera,
    /// Frame all sprites in the scene. Walks PresentWorld for every
    /// `(GlobalTransform, RenderInstance)` and adjusts both center +
    /// height_world so they all fit (with a 10% padding margin).
    /// Empty scene falls back to (0,0) + default zoom.
    All,
}

/// M14.6B host-side reparent intent. Mirrors the
/// `WidgetEvent::HierReparent` payload one-to-one. `new_parent =
/// None` is a root-level drop; `before = None` means "append at end
/// of siblings" (or, when `new_parent` is also `None`, "end of root
/// list").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HierReparentIntent {
    pub dragged: NodeId,
    pub new_parent: Option<NodeId>,
    pub before: Option<NodeId>,
    /// M14.7 polish: when set, the host inserts `dragged` AFTER this
    /// target sibling. Mirrors the `WidgetEvent::HierReparent.after`
    /// field. Mutually exclusive with `before` — only one resolution
    /// fires per drop.
    pub after: Option<NodeId>,
}
