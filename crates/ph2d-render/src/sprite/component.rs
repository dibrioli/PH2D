use bevy_ecs::component::Component;
use ph2d_ecs::SimComponent;

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
    /// A tier-cooked KTX2 texture (KTX2 Fase 2, W2.T2). Stores the
    /// **tier-agnostic** [`ph2d_asset::LogicalTextureId`] so the sprite
    /// stays portable across devices; extract / the loader (W2.T4)
    /// resolve `logical_id` + the active `DeviceTier` to the concrete
    /// `AssetId` → `Asset::TextureKtx2` for upload via
    /// [`crate::compressed_pipeline`] (W2.T3).
    ///
    /// Appended as postcard discriminant `2` — purely additive, so
    /// existing v4 blobs (which only ever encode `Atlas`/`Individual`)
    /// keep loading and `Sprite::VERSION` stays `4` (the `Sprite`
    /// struct field count is unchanged, frozen by ADR-0070). No
    /// `#[non_exhaustive]`: like [`crate::sprite_versioned::SpriteVersioned`],
    /// the postcard discriminant is the stability contract and all
    /// consumers are in-workspace, so exhaustive matches are wanted.
    CookedTexture {
        logical_id: ph2d_asset::LogicalTextureId,
    },
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
/// the `PrefabDoc` / `SceneDoc` postcard pipeline (M14.3). `SpriteSource`'s
/// payloads are fixed-size (`u32` for `Atlas`/`Individual`, a 32-byte
/// [`ph2d_asset::LogicalTextureId`] for `CookedTexture`), and postcard
/// encodes the variant via an append-only varint discriminant — so the
/// wire format stays stable across rustc versions as long as variants are
/// only appended (never reordered or inserted before existing ones).
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
    // (positional, non-self-describing): the attribute never fires.
    /// Self tint — does NOT cascade to children (Godot `self_modulate`).
    /// Multiplies [`Sprite::tint`] for this sprite only. Default WHITE
    /// (identity, zero visual effect).
    #[serde(default = "default_white")]
    pub self_tint: [f32; 4],
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
const fn default_one() -> f32 {
    1.0
}
const fn default_true() -> bool {
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
    pub const VERSION: u32 = 5;

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
            tint_fill: false,
            opacity: 1.0,
            flip_x: false,
            flip_y: false,
            centered: true,
            offset: [0.0, 0.0],
        }
    }

    /// Convenience constructor for individual-texture sprites.
    /// `texture_id` must come from `IndividualTextureStore::acquire`.
    ///
    /// ⚠️ **O `region_filter_clip = false` que morava aqui mudou-se para
    /// [`ph2d_ecs::SpriteRegion::individual`]** (ADR-0164 F1 passo 6): a escolha é de quem
    /// CRIA a região, porque é quem sabe de que fonte ela é. Um sprite Individual sem região
    /// nenhuma — o caso comum — deixou de carregar um bool que não se aplicava a ele.
    pub fn individual(texture_id: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        let mut s = Self::atlas(0, size, tint);
        s.source = SpriteSource::Individual { texture_id };
        s
    }

    /// Convenience constructor for tier-cooked KTX2 sprites (KTX2 Fase 2,
    /// W2.T2). `logical_id` is the tier-agnostic
    /// [`ph2d_asset::LogicalTextureId`]; the loader (W2.T4) resolves it
    /// against the active `DeviceTier` to the concrete cooked asset.
    /// Como a [`Sprite::individual`], uma textura cozida é de resolução nativa (não há
    /// vizinho de quem sangrar) — ver lá a nota sobre onde essa escolha vive hoje.
    pub fn cooked_texture(
        logical_id: ph2d_asset::LogicalTextureId,
        size: [f32; 2],
        tint: [f32; 4],
    ) -> Self {
        let mut s = Self::atlas(0, size, tint);
        s.source = SpriteSource::CookedTexture { logical_id };
        s
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

    /// `Π(opacity)` of this sprite and its sprite ancestors (`ChildOf`): a parent's «Opacity» fades
    /// the whole subtree (Rive, Figma, Unity `CanvasGroup`) —
    /// [doc 04](../../../../docs/Sprite_projeto/04_color_tint_canais.md). O(depth), no allocation.
    pub fn cascaded_opacity(
        &self,
        world: &bevy_ecs::world::World,
        entity: bevy_ecs::entity::Entity,
    ) -> f32 {
        let pai = |e| world.get::<ph2d_ecs::ChildOf>(e).map(|c| c.parent());
        let (mut opacity, mut cur) = (self.opacity, pai(entity));
        while let Some(p) = cur {
            opacity *= world.get::<Self>(p).map_or(1.0, |s| s.opacity);
            cur = pai(p);
        }
        opacity
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

#[cfg(test)]
#[path = "component_opacity_tests.rs"]
mod opacity_tests;
