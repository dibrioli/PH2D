//! Sprite components.
//!
//! [`Sprite`] is a SimComponent (lives in SimWorld; canonical state).
//! [`RenderInstance`] is a PresentComponent (built each frame from
//! Sprite via the extract phase; uploaded to instance buffer).
//!
//! ## World position lives in `Transform`
//!
//! Since ADR-0025 (M14.1) the canonical world-space pose for a sprite
//! comes from [`ph2d_ecs::Transform`] + the hierarchical
//! [`ph2d_ecs::propagate_transforms`] pass — **not** from a separate
//! `WorldPos`/`Position` component. The extract closure reads the
//! freshly computed `GlobalTransform.translation()` and stamps it
//! into `RenderInstance.world_pos` so the renderer stays a pure
//! PresentWorld consumer.

use bevy_ecs::component::Component;
use ph2d_ecs::{PresentComponent, SimComponent};

/// Which texture source a [`Sprite`] reads its pixels from. M14.5
/// introduces the multi-strategy model documented in the post-spike
/// plan §M14.5:
///
/// - [`SpriteSource::Atlas`] — shared dynamic atlas (the M14.4d/4f
///   Skyline packer). Many sprites in one 4096² texture; 1 draw call
///   for every atlas-backed sprite per frame.
/// - [`SpriteSource::Individual`] — sprite owns its own
///   `wgpu::Texture` at native resolution. The renderer groups
///   contiguous same-texture instances into one draw call each
///   (Godot 4 `RenderingServer` pattern). Use when packing-or-stretch
///   trade-offs don't suit the content (large HD sprites, procedural
///   textures, or content that gets hot-reloaded independently).
///
/// `Hand-packed` (artist-authored atlas + JSON) is M14.5 B — separate
/// variant, separate PR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteSource {
    /// Index into the shared atlas. The atlas resolves to a UV
    /// rectangle via `TextureAtlas::region_uv` at extract time.
    Atlas { key: u32 },
    /// Renderer-assigned id for an individually-owned texture, handed
    /// out by [`crate::individual::IndividualTextureStore::acquire`].
    /// Stable for the lifetime of the texture in the store.
    Individual { texture_id: u32 },
}

impl SpriteSource {
    /// Convenience for the common "atlas key 0" case used in fixture
    /// tests and the demo's HSV tiles.
    pub const ATLAS_ZERO: Self = Self::Atlas { key: 0 };
}

/// Canonical sprite description in simulation state. World position
/// is read from the entity's [`ph2d_ecs::Transform`] during the
/// extract phase (ADR-0025).
///
/// `Serialize`/`Deserialize` derives let `Sprite` round-trip through
/// the `PrefabDoc` / `SceneDoc` postcard pipeline (M14.3). All fields
/// are POD-shaped (`SpriteSource` is `#[repr(Rust)]` enum with `u32`
/// payloads), so the wire format stays stable across rustc versions
/// as long as `SpriteSource`'s variant order doesn't change.
#[derive(Component, Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    /// Schema version (HR-14 mitigation). Bumped 3 → 4 in W1 when the
    /// 14 intrinsic-appearance fields below landed (Sprite Inspector v2,
    /// ADR-0069/0070). Redundant with the `SpriteVersioned` wrapper
    /// discriminant on the wire, but kept as an explicit field for
    /// schema honesty + the migrator's `version ∈ {3, 4}` invariant
    /// (anatomia §1.6) — accepted as Lens-C-M2 redundancy.
    ///
    /// `#[serde(default = "default_version_4")]` is documentary under
    /// postcard (positional format; the attribute never fires — see
    /// `sprite_versioned` module docs + ADR-0070-amendment-2): a V4
    /// blob always carries `version` positionally. It activates only
    /// under a hypothetical self-describing format swap.
    #[serde(default = "default_version_4")]
    pub version: u32,
    /// Where the pixels come from — shared atlas or individual texture.
    pub source: SpriteSource,
    /// Sprite size in world units (meters).
    pub size: [f32; 2],
    /// RGBA tint multiplied with the texel color in the fragment shader.
    ///
    /// v4 semantic refinement (ADR-0071): `tint` is the **inherited**
    /// modulate — it cascades to descendants (Godot `modulate`). For a
    /// non-inheriting per-sprite tint use [`Sprite::self_tint`].
    pub tint: [f32; 4],
    /// Offset, in INTRINSIC local meters (pre-`Transform::scale`,
    /// pre-rotation), from the entity's transform origin — the
    /// **pivot** (`GlobalTransform.translation`) — to the geometric
    /// **center of the texture quad**. `[0.0, 0.0]` (the default) means
    /// the quad is centered on the pivot, which is the historical
    /// strictly-centered behavior every sprite had before M14.x pivot
    /// support.
    ///
    /// A non-zero anchor lets the pivot sit somewhere other than the
    /// quad center: the TOOL_PIVOT tool writes it directly, and the
    /// Padding tool's "Keep" mode sets it so an asymmetric resize keeps
    /// the original content + pivot world-fixed while only the
    /// transparent borders grow. Since ADR-0070-amendment-4 extract
    /// stamps this raw into `RenderInstance.anchor` (LOCAL, no longer
    /// scale-folded); the shader adds it to the centered quad corner
    /// BEFORE applying the world basis, so the quad orbits the pivot
    /// (`world_pos`) under rotation/scale/skew, not the quad center.
    ///
    /// `#[serde(default)]` (NOT `skip`): the anchor IS persisted in the
    /// prefab/scene postcard format (a Keep-mode bake must survive
    /// save/load). Older cooked docs that predate the field deserialize
    /// to `[0.0, 0.0]` = centered, so reading stays backward-compatible;
    /// `VERSION` is bumped to 3 for schema honesty + cook-hash.
    #[serde(default)]
    pub anchor: [f32; 2],
    /// `true` → this sprite's texture pixels are stored PREMULTIPLIED,
    /// not straight alpha. Set only by the BG-Removal Apply path, which
    /// bakes a premultiplied Individual texture so the GPU's bilinear
    /// `textureSample` composites the anti-aliased edge band like the
    /// Vello preview (premultiply-before-sample) instead of letting a
    /// partial-alpha edge texel's straight RGB bleed in at full weight
    /// (the purple/dark fringe). Drives `RenderInstance.premultiplied`
    /// at extract time AND tells the Image-Tools readback paths to
    /// un-premultiply before re-running an algorithm that assumes
    /// straight alpha. Defaults to `false`.
    ///
    /// `#[serde(skip)]`: this is a RUNTIME rendering hint, never part of
    /// the persisted prefab/scene format. It is only ever `true` for a
    /// live Individual texture (itself runtime — `texture_id` isn't
    /// portably serializable), and always `false` for Atlas sprites, so
    /// it carries no meaning on disk. Skipping it keeps the postcard cook
    /// hash stable (no fixture churn) and deserializes as `false`.
    #[serde(skip)]
    pub premultiplied: bool,

    // ─── NEW in v4 (Sprite Inspector v2; ADR-0069..0074) ──────────────
    //
    // Every `#[serde(default = ...)]` below is DOCUMENTARY under postcard
    // (positional, non-self-describing): the attribute never fires because
    // a V4 blob carries every field positionally. The sole back-compat
    // path is the `SpriteVersioned` wrapper enum + `migrate_v3_to_v4`
    // (W1.T1.6), per ADR-0070-amendment-2. The attributes are kept as a
    // faithful mirror for a hypothetical self-describing format swap.
    /// Self tint — does NOT cascade to children (Godot `self_modulate`).
    /// Multiplies [`Sprite::tint`] for this sprite only. Default WHITE
    /// (identity, zero visual effect).
    #[serde(default = "default_white")]
    pub self_tint: [f32; 4],
    /// Per-corner tint — a 4-stop bilinear gradient with no custom
    /// shader (Phaser-style). Order `[TopLeft, TopRight, BottomLeft,
    /// BottomRight]`, each RGBA. Default all-WHITE = identity. 64 bytes.
    #[serde(default = "default_per_corner_white")]
    pub per_corner_tint: [[f32; 4]; 4],
    /// Tint fill (Phaser `setTintFill`): when `true`, the texel RGB is
    /// IGNORED and the tint color replaces it (colored silhouette /
    /// damage flash) while alpha is preserved. Default `false`.
    #[serde(default)]
    pub tint_fill: bool,
    /// Final opacity multiplier, orthogonal to `tint[3]`. `tint.a` is the
    /// color's alpha (blend channel); `opacity` is a separate visibility
    /// multiplier, independently animatable. Default `1.0`. Clamped to
    /// `[0.0, 1.0]` and rejected on NaN/Inf by the setter (anatomia §1.6).
    #[serde(default = "default_one")]
    pub opacity: f32,
    /// Logical horizontal flip (distinct from a negative `Transform`
    /// scale — survives reparenting and keeps the gizmo upright).
    #[serde(default)]
    pub flip_x: bool,
    /// Logical vertical flip. See [`Sprite::flip_x`].
    #[serde(default)]
    pub flip_y: bool,
    /// When `true` (default, legacy v3 behavior) the sprite origin is the
    /// quad center. When `false` the origin is top-left and [`offset`]
    /// applies. Default `true`.
    ///
    /// [`offset`]: Sprite::offset
    #[serde(default = "default_true")]
    pub centered: bool,
    /// Intrinsic image offset in pixels, applied AFTER `centered`. Lets
    /// the pivot sit at e.g. the character's feet without touching the
    /// `Transform`. Default `[0.0, 0.0]`.
    #[serde(default)]
    pub offset: [f32; 2],
    /// Inline sprite-sheet horizontal frame count. `hframes × vframes`
    /// divides the texture into a grid that [`frame`] indexes — no
    /// separate `SpriteFrames` asset needed. Default `1` (single frame);
    /// `>= 1` enforced by the setter.
    ///
    /// [`frame`]: Sprite::frame
    #[serde(default = "default_one_u32")]
    pub hframes: u32,
    /// Inline sprite-sheet vertical frame count. See [`Sprite::hframes`].
    #[serde(default = "default_one_u32")]
    pub vframes: u32,
    /// Active frame index into the `hframes × vframes` grid. Default `0`;
    /// kept `< hframes * vframes` by the setter.
    #[serde(default)]
    pub frame: u32,
    /// When `true`, the sprite samples the arbitrary sub-rect
    /// [`region_rect`] instead of the whole texture. Default `false`.
    ///
    /// [`region_rect`]: Sprite::region_rect
    #[serde(default)]
    pub region_enabled: bool,
    /// Sub-region rectangle `[x, y, w, h]` in texture pixels. Only read
    /// when [`region_enabled`] is `true`. `w`/`h` kept `>= 0`.
    ///
    /// [`region_enabled`]: Sprite::region_enabled
    #[serde(default)]
    pub region_rect: [f32; 4],
    /// Region filter clip — clamps the sampler to [`region_rect`] to stop
    /// neighboring-texel bleed across atlas region edges. Default `true`
    /// for Atlas sprites, `false` for Individual. The conditional default
    /// is set by `migrate_v3_to_v4` / the constructors, NOT by this
    /// `#[serde(default)]` (which returns the Atlas value `true` and is
    /// the wrong value for Individual under a serde-default load — see
    /// anatomia §1.4 critical note; the wrapper enum is the canonical
    /// load path).
    ///
    /// [`region_rect`]: Sprite::region_rect
    #[serde(default = "default_region_filter_clip")]
    pub region_filter_clip: bool,
}

/// Default-helper functions for the v4 `#[serde(default = ...)]`
/// attributes (anatomia §1.4). All documentary under postcard; see the
/// field docs + `sprite_versioned` module docs.
const fn default_version_4() -> u32 {
    4
}
const fn default_white() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
const fn default_per_corner_white() -> [[f32; 4]; 4] {
    [[1.0; 4]; 4]
}
const fn default_one() -> f32 {
    1.0
}
const fn default_true() -> bool {
    true
}
const fn default_one_u32() -> u32 {
    1
}
/// Atlas-style default (`true`). NOTE: incorrect for Individual sprites,
/// which need `false` — the conditional choice lives in
/// `migrate_v3_to_v4` and [`Sprite::individual`], not here. This helper
/// only backstops a hypothetical self-describing-format load.
const fn default_region_filter_clip() -> bool {
    true
}

impl Sprite {
    /// Schema version for the cooked-prefab pipeline (HR-14
    /// mitigation; consumed by `ComponentRegistry` until the
    /// `Saveable` derive macro lands). Bumped to 2 when
    /// `atlas_index` became `source` in M14.5 C; to 3 when the
    /// serialized `anchor` (pivot offset) field landed for the
    /// TOOL_PIVOT + Padding-Keep work; to 4 for Sprite Inspector v2
    /// (14 intrinsic-appearance fields; ADR-0069/0070).
    pub const VERSION: u32 = 4;

    /// Convenience constructor for atlas-backed sprites — the
    /// dominant case after M14.4d. Initializes every v4 field to its
    /// identity/default so callers keep the pre-M14.5 ergonomics
    /// (`Sprite::atlas(key, size, tint)`) while opting into the full
    /// v4 schema. `region_filter_clip` defaults `true` (Atlas anti-bleed).
    pub fn atlas(key: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        Self {
            version: Self::VERSION,
            source: SpriteSource::Atlas { key },
            size,
            tint,
            anchor: [0.0, 0.0],
            premultiplied: false,
            self_tint: [1.0, 1.0, 1.0, 1.0],
            per_corner_tint: [[1.0; 4]; 4],
            tint_fill: false,
            opacity: 1.0,
            flip_x: false,
            flip_y: false,
            centered: true,
            offset: [0.0, 0.0],
            hframes: 1,
            vframes: 1,
            frame: 0,
            region_enabled: false,
            region_rect: [0.0, 0.0, 0.0, 0.0],
            region_filter_clip: true,
        }
    }

    /// Convenience constructor for individual-texture sprites.
    /// `texture_id` must come from
    /// `IndividualTextureStore::acquire`. Identical to [`Sprite::atlas`]
    /// except the source and `region_filter_clip` (`false` — Individual
    /// textures are native-resolution, no atlas-neighbor bleed to clip).
    pub fn individual(texture_id: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        let mut s = Self::atlas(0, size, tint);
        s.source = SpriteSource::Individual { texture_id };
        s.region_filter_clip = false;
        s
    }

    /// Pure v3 → v4 schema migrator (spec
    /// [`Sprite_projeto/10_schema_versionamento.md §10.2`], HR-14
    /// mandatory). Maps every frozen v3 field forward verbatim and
    /// initializes the 14 new v4 intrinsic-appearance fields to their
    /// benign identity defaults (no visual change for a sprite that was
    /// authored under v3). It is the SOLE working back-compat path —
    /// `#[serde(default)]` on the v4 fields is documentary-only under
    /// postcard (positional, non-self-describing; empirically pinned by
    /// `tests/sprite_versioned_postcard.rs`), so loading a v3 blob
    /// REQUIRES routing the deserialized [`SpriteV3`] through here
    /// (ADR-0070-amendment-2). [`crate::sprite_versioned::load_sprite`]
    /// is the wrapper-enum dispatch entry point that calls this.
    ///
    /// ## `region_filter_clip` is the one conditional field
    ///
    /// Atlas sprites share a 4096² texture, so the sampler must clamp to
    /// the region rect or neighboring atlas tiles bleed across the edge
    /// → `true`. Individual sprites own a native-resolution texture with
    /// no neighbor to bleed → `false`. The `#[serde(default = ...)]`
    /// helper on the field returns the Atlas value unconditionally and
    /// is the WRONG value for Individual (anatomia §1.4 critical note),
    /// which is exactly why the migrator — not a serde default — owns
    /// this branch. Mirrors [`Sprite::atlas`]/[`Sprite::individual`].
    ///
    /// ## `premultiplied` is a verbatim value copy, NOT rebuilt here
    ///
    /// `premultiplied` is `#[serde(skip)]` in both [`SpriteV3`] and
    /// [`Sprite`], so a wire-loaded `SpriteV3` always carries `false`
    /// and this migrator faithfully copies that. Rebuilding the runtime
    /// flag from [`crate::individual::IndividualTextureStore`] context
    /// (only BG-Removal-Apply'd individuals are premultiplied — the
    /// naive `matches!(source, Individual)` over-triggers) is the
    /// CALLER's concern at the load/extract boundary, not this pure
    /// transform's. Keeping it a value copy preserves
    /// `migrate(in-memory v3 with premultiplied=true)` round-tripping —
    /// the migrator never silently drops a field a caller set in memory.
    pub fn migrate_v3_to_v4(v3: crate::sprite_versioned::SpriteV3) -> Sprite {
        let region_filter_clip = matches!(v3.source, SpriteSource::Atlas { .. });
        Sprite {
            version: Self::VERSION,
            source: v3.source,
            size: v3.size,
            tint: v3.tint,
            anchor: v3.anchor,
            premultiplied: v3.premultiplied,
            // New v4 intrinsic-appearance fields — benign identity
            // defaults (shared with the constructors' helper fns so the
            // v4 default surface stays single-sourced).
            self_tint: default_white(),
            per_corner_tint: default_per_corner_white(),
            tint_fill: false,
            opacity: default_one(),
            flip_x: false,
            flip_y: false,
            centered: default_true(),
            offset: [0.0, 0.0],
            hframes: default_one_u32(),
            vframes: default_one_u32(),
            frame: 0,
            region_enabled: false,
            region_rect: [0.0, 0.0, 0.0, 0.0],
            region_filter_clip,
        }
    }

    /// CPU-side tint cascade collapse for `RenderInstance.tint`
    /// (anatomia §4.2/§4.3). Multiplies this sprite's own two tint
    /// channels — `self_tint × tint`, per RGBA component. Both default
    /// WHITE, so the collapse is identity for a freshly-migrated v3
    /// sprite (zero render change).
    ///
    /// The ancestor modulate product `Π(ancestors.tint)` from §4.3 is
    /// **NOT** folded here: it needs a `GlobalTint` hierarchy-propagation
    /// pass (analogous to `propagate_transforms`) that does not exist
    /// yet and is W2 work — the 3-level `smoke_w2_color_tint.scene`
    /// validates it. The extract phase calls this so the per-sprite
    /// collapse logic lives (and is unit-tested) in this crate rather
    /// than only in the `shells/desktop` extract closure.
    pub fn collapsed_tint(&self) -> [f32; 4] {
        [
            self.tint[0] * self.self_tint[0],
            self.tint[1] * self.self_tint[1],
            self.tint[2] * self.self_tint[2],
            self.tint[3] * self.self_tint[3],
        ]
    }

    /// Resolve the effective quad-center offset from the pivot, in LOCAL
    /// meters — the value the extract stamps into `RenderInstance.anchor`
    /// (the shader/picking position the quad center there, see
    /// `shaders/sprite.wgsl` and `picking.rs`). It folds the Godot-style
    /// `centered` / `offset` authoring ON TOP of the explicit `anchor`
    /// (tool pivot from TOOL_PIVOT / Padding-Keep), all additive:
    ///
    /// - `centered = true` (default): origin is the quad center — no shift.
    /// - `centered = false`: origin is the texture top-left, so the quad
    ///   center sits a half-size to the right and DOWN of the pivot
    ///   (local frame is Y-up, so "down" is `-y`).
    /// - `offset` (intrinsic pixels, Godot `+x` right / `+y` down):
    ///   converted to local meters via `pixels_per_meter` and added (the
    ///   `+y`-down convention again maps to `-y` local).
    ///
    /// `centered = true` + `offset = [0, 0]` returns `anchor` unchanged,
    /// so every legacy sprite (and the common case) is bit-identical to
    /// the pre-feature behavior. `size` is the LOCAL quad size in meters
    /// (the world basis applies scale separately), matching the shader's
    /// `i.size`.
    pub fn resolve_anchor(&self, pixels_per_meter: f32) -> [f32; 2] {
        // Guard a zero/garbage ppm so the px→m divide can't NaN/inf the
        // quad off-screen; 1 px/m is a harmless fallback.
        let ppm = if pixels_per_meter > f32::EPSILON {
            pixels_per_meter
        } else {
            1.0
        };
        let mut a = self.anchor;
        if !self.centered {
            a[0] += self.size[0] * 0.5;
            a[1] -= self.size[1] * 0.5;
        }
        a[0] += self.offset[0] / ppm;
        a[1] -= self.offset[1] / ppm;
        a
    }
}

impl SimComponent for Sprite {}

/// Per-frame instance data uploaded to the GPU. Layout matches the
/// `InstanceInput` struct in `shaders/sprite.wgsl`. `#[repr(C)]` +
/// `bytemuck::Pod` for zero-copy upload via `Queue::write_buffer`.
///
/// M14.5 C: `texture_id` is CPU-side metadata used by the renderer
/// to group same-texture instances into one draw call each. The
/// shader's vertex layout doesn't reference it, so it's ignored on
/// the GPU side — the byte stride still includes it (Pod size = 52
/// bytes, 4-byte aligned).
///
/// - `texture_id == 0` → the instance reads from the shared atlas
///   bound at material bind group 1.
/// - `texture_id > 0` → the renderer rebinds material 1 to the
///   individually-cached texture handed out by
///   `IndividualTextureStore::acquire`.
#[derive(Component, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RenderInstance {
    pub world_pos: [f32; 2],
    /// Sprite size in LOCAL meters — the raw intrinsic [`Sprite::size`]
    /// (import rect). Scale/rotation/skew are NOT folded here anymore;
    /// the full world linear transform lives in [`Self::basis`], which
    /// the shader applies to the local quad. (Pre-amendment-4 this field
    /// pre-multiplied `GlobalTransform` scale; that decomposition was
    /// lossy under skew — see `basis`.)
    pub size: [f32; 2],
    pub atlas_uv: [f32; 4],
    pub tint: [f32; 4],
    /// The 2×2 world-space linear basis from `GlobalTransform`, column-
    /// major: `[col0.x, col0.y, col1.x, col1.y]` = `[x_basis, y_basis]`.
    /// Carries rotation **and** non-uniform scale **and** skew exactly —
    /// the shader applies `mat2x2(col0, col1) · local` to each quad
    /// corner, so a sheared (non-orthogonal) basis renders as the true
    /// parallelogram (ADR-0025-amendment-1 §2.6 skew render step;
    /// ADR-0070-amendment-4).
    ///
    /// This REPLACES the old `rotation: f32` scalar. The previous extract
    /// decomposed the matrix to `atan2(col0)` + per-column lengths, which
    /// collapsed any shear into a rotated rectangle (skew read as
    /// rotation + stretched scale). Identity = `[1, 0, 0, 1]`.
    pub basis: [f32; 4],
    /// `1.0` → the bound texture is already premultiplied (BG-Removal
    /// Apply); the fragment skips its post-sample premultiply so
    /// bilinear sampling matches the Vello on-canvas preview and the
    /// straight-alpha edge fringe disappears. `0.0` for every other
    /// sprite. CPU-side this is set from [`Sprite::premultiplied`] at
    /// extract time.
    pub premultiplied: f32,
    /// Pivot offset in LOCAL meters (the canonical [`Sprite::anchor`],
    /// no longer scale-folded). The shader adds it to the centered quad
    /// corner BEFORE applying [`Self::basis`], so the quad orbits
    /// `world_pos` (the pivot) rather than its own center.
    /// `[0.0, 0.0]` reproduces the historical strictly-centered sprite.
    pub anchor: [f32; 2],

    // ─── NEW in v4 (Sprite Inspector v2; ADR-0069..0071) ──────────────
    //
    // These three GPU-read fields sit BETWEEN `anchor` and the CPU-only
    // `texture_id`/`z_order` so every vertex attribute stays contiguous
    // in `@location` order before any non-attribute field — the exact
    // invariant `vertex_attr_offsets_match_struct` pins. The §1.7 ABI
    // listing groups them after `z_order` for readability, but the live
    // layout MUST keep GPU fields contiguous (anatomia §1.7 cross-refs
    // the offset gate, which is authoritative). Stride grows 72 → 144 B.
    //
    /// Per-corner tint — a 4-stop bilinear gradient interpolated across
    /// the quad in the vertex stage (`@location(9..12)`, 64 bytes). Order
    /// `[TopLeft, TopRight, BottomLeft, BottomRight]`, each RGBA. Mirrors
    /// [`Sprite::per_corner_tint`]; all-WHITE = identity (zero visual
    /// effect). PresentWorld-only (HR-5 exempt): the bilinear blend is
    /// rasterizer/driver-controlled and may ULP-diverge cross-backend
    /// (anatomia §4.6 Lens-C-M4), so it never lives in SimWorld.
    pub per_corner_tint: [[f32; 4]; 4],
    /// Final opacity multiplier (`@location(13)`), orthogonal to
    /// `tint[3]`: `tint.a` is the color's blend alpha, `opacity` is a
    /// separate visibility multiplier applied last. Mirrors
    /// [`Sprite::opacity`]; `1.0` = identity. Clamped `[0.0, 1.0]` at the
    /// Sprite setter (anatomia §1.6 / §4.10), not here.
    pub opacity: f32,
    /// Packed flip flags (`@location(14)`, u32 bitfield): bit0 = flip_x,
    /// bit1 = flip_y (anatomia §1.7). The shader (W1.T1.11) flips the
    /// sampled UV per bit. `0` = no flip (identity). The extract-phase
    /// bit-encoding from [`Sprite::flip_x`]/[`Sprite::flip_y`] lands in
    /// W1.T1.10; until then this stays `0` (logical no-op, render
    /// identical). A wider flags reconciliation (e.g. packing
    /// `tint_fill`) is a W1.T1.11 contract decision, deferred here.
    pub flip_uv: u32,

    /// CPU-side metadata used by the renderer to group same-texture
    /// instances into one draw call; NOT a vertex attribute. Kept after
    /// every GPU-read field so `vertex_attr_array!`'s sequential offsets
    /// line up exactly with the struct field offsets (the
    /// `vertex_attr_offsets_match_struct` test pins this). `0` = shared
    /// atlas; `> 0` = an `IndividualTextureStore` handle.
    pub texture_id: u32,
    /// Render order key (CPU-side; NOT a vertex attribute). Lower =
    /// painted first (BEHIND). The renderer sorts instances by
    /// `(z_order, texture_id)` so the visual order matches the
    /// Hierarchy panel — without this an image-tool bake that
    /// promotes an Atlas sprite to Individual (`commit_edited_texture`)
    /// would silently float to the front of the scene because the
    /// previous `sort_by_key(|i| i.texture_id)` grouped Atlas (id=0)
    /// before every Individual (id>0). Extract populates a sequential
    /// counter in `propagate_transforms` traversal order, which
    /// mirrors the hierarchy DFS — that's the same order the
    /// Hierarchy panel paints.
    pub z_order: u32,
}

impl PresentComponent for RenderInstance {}

impl RenderInstance {
    pub const VERTEX_ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        2 => Float32x2,  // world_pos
        3 => Float32x2,  // size
        4 => Float32x4,  // atlas_uv
        5 => Float32x4,  // tint
        6 => Float32x4,  // basis (2x2 world linear: col0.xy, col1.xy) — ADR-0070-amendment-4
        7 => Float32,    // premultiplied flag (BG-Removal Apply)
        8 => Float32x2,  // anchor (pivot offset; TOOL_PIVOT + Padding-Keep)
        // v4 (Sprite Inspector v2): per_corner_tint occupies 4 attrs
        // (one Float32x4 per corner) since WGSL has no array-of-vec4
        // vertex attribute — the shader reassembles them into a mat-like
        // 4-corner set. opacity + flip_uv follow.
        9  => Float32x4, // per_corner_tint[0] = TopLeft
        10 => Float32x4, // per_corner_tint[1] = TopRight
        11 => Float32x4, // per_corner_tint[2] = BottomLeft
        12 => Float32x4, // per_corner_tint[3] = BottomRight
        13 => Float32,   // opacity
        14 => Uint32,    // flip_uv bitfield (bit0=flip_x, bit1=flip_y)
    ];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }

    /// Sentinel for `texture_id` meaning "sample from the shared
    /// atlas at material bind group 1".
    pub const ATLAS_TEXTURE_ID: u32 = 0;

    /// Identity 2×2 [`Self::basis`] (`[col0.x, col0.y, col1.x, col1.y]`
    /// = unit x/y axes) — no rotation/scale/skew. Used by legacy /
    /// test construction paths that don't derive a basis from a
    /// `GlobalTransform`.
    pub const IDENTITY_BASIS: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

    // ─── `flip_uv` packed-flags bit layout (ADR-0070-amendment-3) ─────
    //
    // `flip_uv` is a general per-instance flags word, not flip-only. The
    // fragment shader (`shaders/sprite.wgsl`) decodes the SAME masks —
    // keep these constants and the WGSL `& Nu` literals in lockstep.
    // Bits 3..31 are reserved (must be 0) for future per-instance bools.
    /// `flip_uv` bit 0 — mirror the sampled texture U (logical flip_x).
    pub const FLIP_X_BIT: u32 = 1 << 0;
    /// `flip_uv` bit 1 — mirror the sampled texture V (logical flip_y).
    pub const FLIP_Y_BIT: u32 = 1 << 1;
    /// `flip_uv` bit 2 — tint-fill / silhouette mode: the shader ignores
    /// the texel RGB and uses the combined tint RGB (anatomia §4.5).
    pub const TINT_FILL_BIT: u32 = 1 << 2;

    /// Pack the per-instance flip/fill booleans into the `flip_uv` flags
    /// word. Single source of truth for the bit layout shared by the
    /// extract phase (`shells/desktop`) and the WGSL decode, so the two
    /// can't drift. Mirrors [`Sprite::flip_x`]/[`Sprite::flip_y`]/
    /// [`Sprite::tint_fill`].
    pub const fn pack_flip_flags(flip_x: bool, flip_y: bool, tint_fill: bool) -> u32 {
        (if flip_x { Self::FLIP_X_BIT } else { 0 })
            | (if flip_y { Self::FLIP_Y_BIT } else { 0 })
            | (if tint_fill { Self::TINT_FILL_BIT } else { 0 })
    }
}

/// Vertex of the unit quad used as the geometry for every sprite
/// instance. Layout matches `VertexInput` in the shader.
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct QuadVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

impl QuadVertex {
    pub const VERTEX_ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x2,  // quad_pos
        1 => Float32x2,  // quad_uv
    ];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }

    /// Unit quad as triangle strip, centered at origin.
    ///
    /// UV mapping is "natural": world-up vertex (`pos.y = +0.5`)
    /// samples texture top (V=0); world-down vertex samples texture
    /// bottom (V=1). This works directly because
    /// [`Camera2d::view_proj`](crate::camera::Camera2d::view_proj)
    /// uses standard orthographic with no Y-flip (M14.4e v2 removed
    /// the prior `bottom`/`top` swap that had inverted everything).
    /// World Y-up therefore maps to screen Y-up, and a texture
    /// uploaded in image-crate's top-down byte order displays upright.
    pub const QUAD_STRIP: [Self; 4] = [
        Self {
            pos: [-0.5, -0.5],
            uv: [0.0, 1.0],
        },
        Self {
            pos: [0.5, -0.5],
            uv: [1.0, 1.0],
        },
        Self {
            pos: [-0.5, 0.5],
            uv: [0.0, 0.0],
        },
        Self {
            pos: [0.5, 0.5],
            uv: [1.0, 0.0],
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_instance_is_pod_compatible() {
        let inst = RenderInstance {
            world_pos: [1.0, 2.0],
            size: [10.0, 10.0],
            atlas_uv: [0.0, 0.0, 0.25, 0.25],
            tint: [1.0, 1.0, 1.0, 1.0],
            basis: RenderInstance::IDENTITY_BASIS,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            texture_id: RenderInstance::ATLAS_TEXTURE_ID,
            z_order: 0,
        };
        let bytes: &[u8] = bytemuck::bytes_of(&inst);
        assert_eq!(bytes.len(), std::mem::size_of::<RenderInstance>());
        // GPU fields = 76 bytes (world_pos 8 + size 8 + atlas_uv 16 +
        // tint 16 + basis 16 + premultiplied 4 + anchor 8).
        // + per_corner_tint [[f32;4];4] (64) + opacity f32 (4) +
        // flip_uv u32 (4) = +72 → 148 GPU bytes.
        // + texture_id u32 (4) + z_order u32 (4) CPU-only = 156 bytes,
        // 4-byte aligned (no tail padding). ADR-0070-amendment-4 grew
        // `rotation: f32` → `basis: [f32;4]` (+12 B over the 144 freeze).
        assert_eq!(bytes.len(), 156);
    }

    #[test]
    fn vertex_attributes_cover_full_stride() {
        // The flip_uv flags are the last (11th) vertex attribute, at
        // location 14. Confirm the attribute array's last offset+size
        // lands inside the Pod stride so the vertex layout doesn't read
        // past the instance buffer.
        let attrs = RenderInstance::VERTEX_ATTRIBUTES;
        let last = attrs.last().expect("at least one attribute");
        assert_eq!(last.shader_location, 14, "flip_uv is @location(14)");
        // Uint32 == 4 bytes.
        let end = last.offset + 4;
        assert!(
            end <= std::mem::size_of::<RenderInstance>() as u64,
            "attr end {end} must fit in stride {}",
            std::mem::size_of::<RenderInstance>()
        );
    }

    #[test]
    fn vertex_attr_offsets_match_struct() {
        // REGRESSION GUARD. `wgpu::vertex_attr_array!` derives each
        // attribute's byte offset from the running sum of the FORMATS
        // listed — it knows nothing about the Rust struct. So every
        // GPU-read field MUST sit contiguously, in attribute order,
        // before any CPU-only field (`texture_id`, `z_order`). If a future
        // edit interleaves a non-attribute field (as the original
        // `premultiplied` placement did — it sat after `texture_id`, so
        // `@location(7)` silently read `texture_id`'s bytes), this test
        // fails instead of shipping a sampler that reads the wrong word.
        //
        // v4 (Sprite Inspector v2) appended per_corner_tint (4 attrs,
        // @location 9..12), opacity (@13), flip_uv (@14) — still all
        // before the CPU-only tail. 11 attrs total.
        use std::mem::offset_of;
        let expect = [
            (2u32, offset_of!(RenderInstance, world_pos) as u64),
            (3, offset_of!(RenderInstance, size) as u64),
            (4, offset_of!(RenderInstance, atlas_uv) as u64),
            (5, offset_of!(RenderInstance, tint) as u64),
            (6, offset_of!(RenderInstance, basis) as u64),
            (7, offset_of!(RenderInstance, premultiplied) as u64),
            (8, offset_of!(RenderInstance, anchor) as u64),
            // per_corner_tint is one [[f32;4];4] field but FOUR vertex
            // attrs; the macro lays them at +0/+16/+32/+48 from the field
            // base, so locations 9..12 map to the field offset plus the
            // per-corner stride. offset_of! gives the field base for @9;
            // @10..12 follow contiguously (checked by the macro's own sum
            // matching, asserted below via the running offsets).
            (9, offset_of!(RenderInstance, per_corner_tint) as u64),
            (10, offset_of!(RenderInstance, per_corner_tint) as u64 + 16),
            (11, offset_of!(RenderInstance, per_corner_tint) as u64 + 32),
            (12, offset_of!(RenderInstance, per_corner_tint) as u64 + 48),
            (13, offset_of!(RenderInstance, opacity) as u64),
            (14, offset_of!(RenderInstance, flip_uv) as u64),
        ];
        let attrs = RenderInstance::VERTEX_ATTRIBUTES;
        assert_eq!(attrs.len(), expect.len(), "attribute count drifted");
        for (attr, (loc, off)) in attrs.iter().zip(expect) {
            assert_eq!(attr.shader_location, loc, "location order drifted");
            assert_eq!(
                attr.offset, off,
                "@location({loc}) macro offset {} != struct field offset {off} \
                 — a non-attribute field is interleaved with GPU fields",
                attr.offset
            );
        }
    }

    #[test]
    fn collapsed_tint_folds_self_tint_into_tint() {
        // Identity case: default self_tint = WHITE → tint unchanged
        // (zero-regression for v3-migrated sprites).
        let s = Sprite::atlas(0, [1.0, 1.0], [0.4, 0.5, 0.6, 0.7]);
        assert_eq!(s.collapsed_tint(), [0.4, 0.5, 0.6, 0.7]);

        // Non-identity: per-component multiply of tint × self_tint.
        let mut s = Sprite::atlas(0, [1.0, 1.0], [0.5, 1.0, 0.2, 1.0]);
        s.self_tint = [0.5, 0.5, 1.0, 0.8];
        assert_eq!(s.collapsed_tint(), [0.25, 0.5, 0.2, 0.8]);

        // Each channel is independent (no cross-channel bleed).
        let mut s = Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        s.tint = [2.0, 0.0, 0.0, 0.0];
        s.self_tint = [0.5, 7.0, 9.0, 9.0];
        assert_eq!(s.collapsed_tint(), [1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn resolve_anchor_centered_default_is_unchanged() {
        // The common case (centered, no offset, no tool pivot) must be
        // bit-identical to raw `anchor` so every legacy sprite renders
        // and hit-tests exactly as before.
        let s = Sprite::atlas(0, [2.0, 3.0], [1.0; 4]);
        assert_eq!(s.resolve_anchor(100.0), [0.0, 0.0]);

        // A tool-set anchor passes through untouched when centered.
        let mut s = Sprite::atlas(0, [2.0, 3.0], [1.0; 4]);
        s.anchor = [0.25, -0.5];
        assert_eq!(s.resolve_anchor(100.0), [0.25, -0.5]);
    }

    #[test]
    fn resolve_anchor_uncentered_puts_pivot_at_top_left() {
        // centered=false → origin at the texture top-left, so the quad
        // CENTER sits +half-width right and half-height DOWN (-y local).
        let mut s = Sprite::atlas(0, [2.0, 4.0], [1.0; 4]);
        s.centered = false;
        assert_eq!(s.resolve_anchor(100.0), [1.0, -2.0]);
    }

    #[test]
    fn resolve_anchor_offset_is_pixels_over_ppm_with_godot_y_down() {
        // offset is intrinsic px (Godot +x right / +y down); local frame
        // is Y-up so +y offset maps to -y. 50 px @ 100 px/m = 0.5 m.
        let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
        s.offset = [50.0, 100.0];
        assert_eq!(s.resolve_anchor(100.0), [0.5, -1.0]);
    }

    #[test]
    fn resolve_anchor_composes_anchor_centered_and_offset_additively() {
        let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
        s.anchor = [0.1, 0.2];
        s.centered = false; // +[1.0, -1.0]
        s.offset = [100.0, 0.0]; // +[1.0, 0.0] @ 100 ppm
        assert_eq!(s.resolve_anchor(100.0), [0.1 + 1.0 + 1.0, 0.2 - 1.0]);
    }

    #[test]
    fn resolve_anchor_guards_nonpositive_ppm() {
        // A zero/garbage ppm must not NaN/inf the quad off-screen.
        let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
        s.offset = [10.0, 0.0];
        let a = s.resolve_anchor(0.0);
        assert!(a[0].is_finite() && a[1].is_finite());
    }

    #[test]
    fn flip_uv_flag_bits_roundtrip() {
        // Pin the ADR-0070-amendment-3 bit layout (the Rust ENCODE side):
        // a future edit can't silently re-map flip_x/flip_y/tint_fill in
        // `pack_flip_flags`. NOTE: this does NOT mechanically guard the
        // WGSL decode literals (`& 1u`/`& 2u`/`& 4u` in shaders/sprite.wgsl)
        // — those are hand-kept in lockstep and verified by the headless
        // pipeline validation (`pipeline::tests`) + the Enio GPU smoke.
        assert_eq!(RenderInstance::FLIP_X_BIT, 1);
        assert_eq!(RenderInstance::FLIP_Y_BIT, 2);
        assert_eq!(RenderInstance::TINT_FILL_BIT, 4);

        assert_eq!(RenderInstance::pack_flip_flags(false, false, false), 0);
        assert_eq!(RenderInstance::pack_flip_flags(true, false, false), 0b001);
        assert_eq!(RenderInstance::pack_flip_flags(false, true, false), 0b010);
        assert_eq!(RenderInstance::pack_flip_flags(false, false, true), 0b100);
        assert_eq!(RenderInstance::pack_flip_flags(true, true, true), 0b111);
        // Each flag is independent — no bit bleeds into another.
        let f = RenderInstance::pack_flip_flags(true, false, true);
        assert_ne!(f & RenderInstance::FLIP_X_BIT, 0);
        assert_eq!(f & RenderInstance::FLIP_Y_BIT, 0);
        assert_ne!(f & RenderInstance::TINT_FILL_BIT, 0);
        // Reserved bits 3..31 stay zero for any input.
        assert_eq!(
            RenderInstance::pack_flip_flags(true, true, true) & !0b111u32,
            0
        );
    }

    #[test]
    fn sprite_constructors_default_straight_alpha() {
        assert!(!Sprite::atlas(0, [1.0, 1.0], [1.0; 4]).premultiplied);
        assert!(!Sprite::individual(1, [1.0, 1.0], [1.0; 4]).premultiplied);
    }

    #[test]
    fn sprite_source_atlas_zero_const() {
        assert_eq!(SpriteSource::ATLAS_ZERO, SpriteSource::Atlas { key: 0 });
    }

    #[test]
    fn sprite_atlas_constructor_round_trip() {
        let s = Sprite::atlas(7, [1.0, 2.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.source, SpriteSource::Atlas { key: 7 });
        assert_eq!(s.size, [1.0, 2.0]);
    }

    #[test]
    fn sprite_individual_constructor_round_trip() {
        let s = Sprite::individual(42, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.source, SpriteSource::Individual { texture_id: 42 });
    }

    #[test]
    fn quad_strip_winding() {
        // Triangle strip [0,1,2] then [1,3,2] → both CCW when viewed
        // from +Z (Y-up world space). Just sanity that vertex order
        // matches what the shader expects.
        assert_eq!(QuadVertex::QUAD_STRIP.len(), 4);
    }

    #[test]
    fn quad_strip_uv_natural_mapping() {
        // M14.4e v2: camera projection no longer Y-flips, so the UV
        // mapping is straightforward — world-up vertex (pos.y > 0)
        // samples texture top (V=0), world-down samples texture
        // bottom (V=1). X is straight (no flip).
        for v in QuadVertex::QUAD_STRIP {
            let expected_v = if v.pos[1] < 0.0 { 1.0 } else { 0.0 };
            assert_eq!(
                v.uv[1], expected_v,
                "pos {:?} expected V={expected_v} got V={}",
                v.pos, v.uv[1]
            );
            let expected_u = if v.pos[0] < 0.0 { 0.0 } else { 1.0 };
            assert_eq!(v.uv[0], expected_u);
        }
    }
}
