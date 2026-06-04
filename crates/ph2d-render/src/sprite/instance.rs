use bevy_ecs::component::Component;
use ph2d_ecs::PresentComponent;

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
    /// Sprite size in LOCAL meters — the raw intrinsic [`Sprite::size`](crate::sprite::Sprite::size)
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
    /// sprite. CPU-side this is set from [`Sprite::premultiplied`](crate::sprite::Sprite::premultiplied) at
    /// extract time.
    pub premultiplied: f32,
    /// Pivot offset in LOCAL meters (the canonical [`Sprite::anchor`](crate::sprite::Sprite::anchor),
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
    /// [`Sprite::per_corner_tint`](crate::sprite::Sprite::per_corner_tint); all-WHITE = identity (zero visual
    /// effect). PresentWorld-only (HR-5 exempt): the bilinear blend is
    /// rasterizer/driver-controlled and may ULP-diverge cross-backend
    /// (anatomia §4.6 Lens-C-M4), so it never lives in SimWorld.
    pub per_corner_tint: [[f32; 4]; 4],
    /// Final opacity multiplier (`@location(13)`), orthogonal to
    /// `tint[3]`: `tint.a` is the color's blend alpha, `opacity` is a
    /// separate visibility multiplier applied last. Mirrors
    /// [`Sprite::opacity`](crate::sprite::Sprite::opacity); `1.0` = identity. Clamped `[0.0, 1.0]` at the
    /// Sprite setter (anatomia §1.6 / §4.10), not here.
    pub opacity: f32,
    /// Packed flip flags (`@location(14)`, u32 bitfield): bit0 = flip_x,
    /// bit1 = flip_y (anatomia §1.7). The shader (W1.T1.11) flips the
    /// sampled UV per bit. `0` = no flip (identity). The extract-phase
    /// bit-encoding from [`Sprite::flip_x`](crate::sprite::Sprite::flip_x)/[`Sprite::flip_y`](crate::sprite::Sprite::flip_y) lands in
    /// W1.T1.10; until then this stays `0` (logical no-op, render
    /// identical). A wider flags reconciliation (e.g. packing
    /// `tint_fill`) is a W1.T1.11 contract decision, deferred here.
    pub flip_uv: u32,
    /// UV tiling/scroll transform (`@location(15)`, ADR-0070-amendment-6):
    /// `[scale.x, scale.y, offset.x, offset.y]`. The fragment samples
    /// `wrap(quad_uv * scale + offset)` INSIDE the sprite's own sub-rect
    /// (no atlas bleed), where the wrap mode is decoded from
    /// [`Self::flip_uv`] bits 3–4. `[1, 1, 0, 0]` = identity (W3.T3.11
    /// UvTransform; spec §9.2 tiling/scroll). GPU-read, so it sits before
    /// the CPU-only tail to keep the vertex attributes contiguous.
    pub uv_xform: [f32; 4],

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
    /// Packed per-node sampling key (CPU-side; NOT a vertex attribute).
    /// `filter (low byte) | repeat << 8` from the hierarchically-resolved
    /// `TextureFilter`/`TextureRepeat` (W3.T3.11). The renderer groups
    /// runs by `(z_order, sampling, texture_id)` and binds the matching
    /// sampler so per-node filter/wrap works without a shader change.
    /// `0` = `Inherit/Inherit` → the renderer's default sampler (the
    /// project `ImageFilterMode`). ADR-0070-amendment-5: this grows the
    /// CPU-only tail (GPU vertex layout unchanged).
    pub sampling: u32,
    /// Clip-stencil grouping key (CPU-side; NOT a vertex attribute).
    /// `0` = the instance takes no part in any [`ClipChildren`] silhouette
    /// clip — the renderer paints it in the normal pass exactly as before
    /// (zero-regression identity). A non-zero value is the *clip-group id*,
    /// defined as `clip_parent.z_order + 1` (so it is unique per clip-parent
    /// and never collides with the `0` sentinel). Both the clip-parent
    /// (mask source) and every clipped descendant of one subtree carry the
    /// SAME `clip_group`; the renderer batches them into a stencil
    /// mark→test→draw triple. ADR-0070-amendment-7 (grows the CPU-only
    /// tail; the GPU vertex layout is unchanged, so no attribute moves).
    pub clip_group: u32,
    /// Clip role + quantized cutoff for a clip-group member (CPU-side; NOT
    /// a vertex attribute). Only meaningful when [`Self::clip_group`] != 0.
    /// Bit layout (ADR-0070-amendment-7):
    /// - bits 0–1 = role: `0` member · `1` mask source (`ClipOnly`) ·
    ///   `2` mask source (`ClipAndDraw`),
    /// - bits 8–15 = `alpha_cutoff` quantized to `u8` as
    ///   `round(cutoff * 255)` (read for the clip mask-source AND the
    ///   Mask2D source to threshold its silhouette in the mark pass),
    /// - bits 16–17 = Mask2D/MaskInteraction role (`MASK_ROLE_*`): `0`
    ///   none · `1` Mask2D source · `2` responder VisibleInside · `3`
    ///   responder VisibleOutside. Orthogonal to the clip bits — the mask
    ///   feature is global, so it does NOT use `clip_group`.
    ///
    /// Use [`Self::pack_clip_meta`] / [`Self::clip_role`] /
    /// [`Self::clip_cutoff`] / [`Self::with_mask_role`] /
    /// [`Self::mask_role`] — never hand-pack.
    pub clip_meta: u32,
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
        14 => Uint32,    // flip_uv bitfield (bit0=flip_x, bit1=flip_y, bit2=tint_fill, bits3-4=repeat, bits5-7=blend[CPU-only])
        15 => Float32x4, // uv_xform (scale.xy, offset.xy) — ADR-0070-amendment-6
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

    /// High-bit tag carving a distinct `texture_id` namespace for cooked
    /// KTX2 textures (W2.T4): an id with this bit set binds the
    /// [`crate::cooked_texture::CookedTextureStore`] entry, NOT an
    /// `IndividualTextureStore` one. This keeps the additive cooked-texture
    /// path off the Individual id space (which allocates `1..` and never
    /// reaches `2^31`) without changing the [`RenderInstance`] ABI —
    /// `texture_id` is CPU-only metadata sitting in the tail AFTER the last
    /// vertex attribute, so the frozen layout (184 B struct, 12 GPU vertex
    /// attributes @location 2..15; ADR-0070) is untouched and the high bit
    /// can never reach a `@location`. The renderer's `material_bg`
    /// dispatch is the only reader: `0` → atlas, high-bit set → cooked, else
    /// individual. Cooked ids sort *after* individuals within a `z_order`
    /// slice (they're large `u32`s), which is harmless since `z_order` is
    /// the primary sort key.
    pub const COOKED_TEXTURE_ID_BIT: u32 = 1 << 31;

    /// `true` if `texture_id` is in the cooked-texture namespace
    /// ([`Self::COOKED_TEXTURE_ID_BIT`] set). The atlas sentinel (`0`) and
    /// individual ids (`1..2^31`) both return `false`.
    #[must_use]
    pub const fn is_cooked_texture_id(texture_id: u32) -> bool {
        texture_id & Self::COOKED_TEXTURE_ID_BIT != 0
    }

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
    // Bits 3-4 = repeat; bits 5-7 = blend tag (CPU-only, shader ignores);
    // bits 8..31 reserved (must be 0) for future per-instance flags.
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
    /// can't drift. Mirrors [`Sprite::flip_x`](crate::sprite::Sprite::flip_x)/[`Sprite::flip_y`](crate::sprite::Sprite::flip_y)/
    /// [`Sprite::tint_fill`](crate::sprite::Sprite::tint_fill).
    pub const fn pack_flip_flags(flip_x: bool, flip_y: bool, tint_fill: bool) -> u32 {
        (if flip_x { Self::FLIP_X_BIT } else { 0 })
            | (if flip_y { Self::FLIP_Y_BIT } else { 0 })
            | (if tint_fill { Self::TINT_FILL_BIT } else { 0 })
    }

    /// Identity [`Self::uv_xform`] — `scale [1,1]`, `offset [0,0]` (no
    /// tiling, no scroll). Used by every non-tiling construction site.
    pub const IDENTITY_UV_XFORM: [f32; 4] = [1.0, 1.0, 0.0, 0.0];

    /// Bit offset of the 2-bit resolved-repeat field packed into
    /// [`Self::flip_uv`] (W3.T3.11): `0 Inherit · 1 Disabled · 2 Enabled
    /// · 3 Mirror`. The fragment decodes it to pick the in-rect UV wrap.
    pub const REPEAT_SHIFT: u32 = 3;

    /// Pack a resolved `RepeatMode` tag (`0..=3`) into the `flip_uv`
    /// repeat bits (OR into the flip-flag word).
    pub const fn pack_repeat_bits(repeat_tag: u8) -> u32 {
        ((repeat_tag as u32) & 0b11) << Self::REPEAT_SHIFT
    }

    /// Bit offset of the 3-bit blend-mode tag packed into
    /// [`Self::flip_uv`] (§10, ADR-0070-amendment-3 free-bit budget):
    /// bits 5-7 hold `BlendMode::tag()` (`0..=5`). **CPU-only** — the
    /// renderer reads it to key draw runs onto the matching blend
    /// pipeline; the WGSL fragment never decodes these bits (blend is
    /// pipeline state, not shader logic). Zero ABI cost (no new field).
    pub const BLEND_SHIFT: u32 = 5;

    /// Pack a resolved `BlendMode` tag (`0..=5`) into the `flip_uv`
    /// blend bits (OR into the flags word). Mirrors [`pack_repeat_bits`].
    pub const fn pack_blend_bits(blend_tag: u8) -> u32 {
        ((blend_tag as u32) & 0b111) << Self::BLEND_SHIFT
    }

    /// Unpack the blend-mode tag (`0..=5`) from a `flip_uv` flags word.
    pub const fn unpack_blend(flip_uv: u32) -> u8 {
        ((flip_uv >> Self::BLEND_SHIFT) & 0b111) as u8
    }

    /// Default [`Self::sampling`] key — `Inherit/Inherit`, i.e. the
    /// renderer's project-default sampler. Used by every non-extract
    /// construction site (tests, picking, benches).
    pub const SAMPLING_DEFAULT: u32 = 0;

    /// Pack a resolved `(filter, repeat)` mode pair (each a small enum
    /// tag) into the [`Self::sampling`] key: `filter | repeat << 8`.
    pub const fn pack_sampling(filter: u8, repeat: u8) -> u32 {
        (filter as u32) | ((repeat as u32) << 8)
    }

    /// Unpack [`Self::sampling`] into `(filter_tag, repeat_tag)`.
    pub const fn unpack_sampling(sampling: u32) -> (u8, u8) {
        ((sampling & 0xFF) as u8, ((sampling >> 8) & 0xFF) as u8)
    }

    // ─── `clip_group` / `clip_meta` (ADR-0070-amendment-7) ────────────
    //
    /// Sentinel [`Self::clip_group`] meaning "no clip" — the instance
    /// renders in the normal pass (identity / zero-regression path).
    pub const CLIP_GROUP_NONE: u32 = 0;

    /// [`Self::clip_meta`] role (bits 0–1) — a clipped descendant of a
    /// clip-parent; painted with the stencil test `Equal ref`.
    pub const CLIP_ROLE_MEMBER: u8 = 0;
    /// [`Self::clip_meta`] role — the mask source for a `ClipOnly` parent
    /// (silhouette marks the stencil; the parent itself does NOT draw).
    pub const CLIP_ROLE_MASK_CLIP_ONLY: u8 = 1;
    /// [`Self::clip_meta`] role — the mask source for a `ClipAndDraw`
    /// parent (silhouette marks the stencil AND the parent draws normally).
    pub const CLIP_ROLE_MASK_CLIP_AND_DRAW: u8 = 2;

    const CLIP_ROLE_MASK: u32 = 0b11;
    const CLIP_CUTOFF_SHIFT: u32 = 8;

    /// Pack a clip role (`CLIP_ROLE_*`) and an `alpha_cutoff` in `[0, 1]`
    /// into the [`Self::clip_meta`] word. Cutoff is quantized to `u8`
    /// (`round(cutoff * 255)`) — exact enough for a binary silhouette
    /// threshold and keeps the field GPU-uploadable if ever promoted.
    pub fn pack_clip_meta(role: u8, alpha_cutoff: f32) -> u32 {
        let q = (alpha_cutoff.clamp(0.0, 1.0) * 255.0).round() as u32;
        ((role as u32) & Self::CLIP_ROLE_MASK) | ((q & 0xFF) << Self::CLIP_CUTOFF_SHIFT)
    }

    /// Extract the clip role (`CLIP_ROLE_*`) from [`Self::clip_meta`].
    pub const fn clip_role(clip_meta: u32) -> u8 {
        (clip_meta & Self::CLIP_ROLE_MASK) as u8
    }

    /// Extract the `alpha_cutoff` in `[0, 1]` from [`Self::clip_meta`]
    /// (dequantizes the stored `u8`). Shared by the ClipChildren mark pass
    /// AND the Mask2D mark pass (a sprite is a clip-parent OR a mask source,
    /// never both, so the same cutoff slot serves whichever).
    pub fn clip_cutoff(clip_meta: u32) -> f32 {
        ((clip_meta >> Self::CLIP_CUTOFF_SHIFT) & 0xFF) as f32 / 255.0
    }

    // ─── Mask2D / MaskInteraction roles, packed in `clip_meta` bits 16-17
    //     (no new ABI field — the mask feature is GLOBAL, so it doesn't
    //     reuse the per-subtree `clip_group`; only a 2-bit role is needed).
    //
    /// [`Self::clip_meta`] mask role (bits 16-17) — not part of any mask.
    pub const MASK_ROLE_NONE: u8 = 0;
    /// Mask role — a [`ph2d_ecs::Mask2D`] SOURCE: marks the shared mask
    /// stencil at its silhouette (cutoff from bits 8-15), draws no color.
    pub const MASK_ROLE_SOURCE: u8 = 1;
    /// Mask role — a `VisibleInside` responder: drawn where `stencil == ref`.
    pub const MASK_ROLE_INSIDE: u8 = 2;
    /// Mask role — a `VisibleOutside` responder: drawn where `stencil != ref`.
    pub const MASK_ROLE_OUTSIDE: u8 = 3;

    const MASK_ROLE_SHIFT: u32 = 16;
    const MASK_ROLE_MASK: u32 = 0b11;

    /// OR a mask role (`MASK_ROLE_*`) into a `clip_meta` word (preserving
    /// any clip role / cutoff already packed in the low bits).
    pub const fn with_mask_role(clip_meta: u32, role: u8) -> u32 {
        clip_meta | (((role as u32) & Self::MASK_ROLE_MASK) << Self::MASK_ROLE_SHIFT)
    }

    /// Extract the mask role (`MASK_ROLE_*`) from [`Self::clip_meta`].
    pub const fn mask_role(clip_meta: u32) -> u8 {
        ((clip_meta >> Self::MASK_ROLE_SHIFT) & Self::MASK_ROLE_MASK) as u8
    }
}
