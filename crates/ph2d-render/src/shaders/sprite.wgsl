// Sprite pipeline (M5; v4 channels = Sprite Inspector v2 W1.T1.11).
//
// Bind groups (per LLM1 audit + toji.dev convention):
//   @group(0) frame:    camera view+proj uniform
//   @group(1) material: atlas texture + sampler
// Per-instance data goes via vertex attributes (instance step mode),
// not a third bind group — cheaper.

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var atlas_tex: texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,  // unit quad corner in [-0.5, 0.5]
    @location(1) quad_uv:  vec2<f32>,  // [0, 1]; (0,0)=top-left, (1,1)=bottom-right
};

struct InstanceInput {
    @location(2) world_pos: vec2<f32>,
    @location(3) size:      vec2<f32>,
    @location(4) atlas_uv:  vec4<f32>,  // u_min, v_min, u_max, v_max
    // Cascade tint (CPU-collapsed self_tint × tint × Π ancestor modulates,
    // anatomia §4.3). The cascade collapse over ancestors lands in a later
    // wave; in W1 this is `self_tint × tint` (both default WHITE → identity).
    @location(5) tint:      vec4<f32>,
    // 2x2 world linear basis (column-major): basis.xy = col0 (x axis),
    // basis.zw = col1 (y axis). Carries rotation + scale + skew exactly;
    // a non-orthogonal basis renders the true sheared parallelogram
    // (ADR-0070-amendment-4; ADR-0025-amendment-1 §2.6). Replaces the old
    // decomposed `rotation` scalar that collapsed skew into rot+scale.
    @location(6) basis:     vec4<f32>,
    // > 0.5 → this instance's texture is ALREADY premultiplied
    // (BG-Removal Apply bakes premultiplied so bilinear matches the
    // Vello preview). The fragment then skips its post-sample
    // premultiply. 0.0 for every other sprite (atlas + straight
    // individual) so they composite exactly as before.
    @location(7) premultiplied: f32,
    // Pivot offset (LOCAL meters): the quad CENTER's position relative
    // to `world_pos` (the pivot), in the sprite's own local frame. Added
    // to the centered corner before the basis maps it to world, so the
    // quad orbits the pivot. [0,0] = strictly-centered (legacy).
    @location(8) anchor: vec2<f32>,
    // v4 per-corner tint (anatomia §4.1/§4.6) — a 4-stop bilinear
    // gradient. Order [TopLeft, TopRight, BottomLeft, BottomRight]; the
    // vertex stage bilinearly resolves the per-vertex color and the
    // rasterizer interpolates it across fragments. All-WHITE = identity.
    @location(9)  corner_tl: vec4<f32>,
    @location(10) corner_tr: vec4<f32>,
    @location(11) corner_bl: vec4<f32>,
    @location(12) corner_br: vec4<f32>,
    // v4 final opacity multiplier (anatomia §4.1), orthogonal to tint.a.
    // 1.0 = identity.
    @location(13) opacity: f32,
    // v4 packed flags bitfield (ADR-0070-amendment-3):
    //   bit0 = flip_x, bit1 = flip_y, bit2 = tint_fill,
    //   bits3-4 = resolved RepeatMode (0 Inherit/clamp · 1 Disabled/clamp
    //   · 2 Enabled/repeat · 3 Mirror) — ADR-0070-amendment-6.
    @location(14) flip_uv: u32,
    // UV tiling/scroll transform (ADR-0070-amendment-6): scale.xy,
    // offset.xy. The fragment samples wrap(quad_uv*scale+offset) inside
    // the sprite's own sub-rect. [1,1,0,0] = identity.
    @location(15) uv_xform: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    // Flipped quad UV in [0,1], interpolated. The fragment turns it into
    // the texture UV via the tiling transform + in-rect wrap (so tiling
    // wraps WITHIN the sprite sub-rect — no atlas bleed).
    @location(0) quv:  vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) premultiplied: f32,
    // Bilinearly-interpolated per-corner tint (anatomia §4.2 step 2).
    @location(3) corner: vec4<f32>,
    @location(4) opacity: f32,
    // tint_fill decoded from flip_uv bit2; >0.5 = silhouette mode.
    // `flat` because it's a per-instance boolean, not a varying.
    @location(5) @interpolate(flat) tint_fill: f32,
    // Per-instance constants for the fragment's tiling math (flat).
    @location(6) @interpolate(flat) atlas_uv: vec4<f32>,
    @location(7) @interpolate(flat) uv_xform: vec4<f32>,
    @location(8) @interpolate(flat) repeat_mode: u32,
};

// Wrap `t` into [0,1] per the resolved RepeatMode (W3.T3.11): 2 Enabled
// → repeat (fract), 3 Mirror → triangle wave, else (Inherit/Disabled) →
// clamp. Applied per-fragment so tiling stays inside the sprite rect.
fn wrap_uv(t: vec2<f32>, mode: u32) -> vec2<f32> {
    switch mode {
        case 2u: { return fract(t); }
        case 3u: {
            let m = t - 2.0 * floor(t * 0.5);  // t mod 2 ∈ [0,2)
            return 1.0 - abs(m - 1.0);          // 0→0, 1→1, 2→0
        }
        default: { return clamp(t, vec2<f32>(0.0), vec2<f32>(1.0)); }
    }
}

@vertex
fn vs_main(v: VertexInput, i: InstanceInput) -> VertexOutput {
    // Local-space sprite corner (the quad is centered on its own
    // geometry, then shifted by `anchor` so the quad center sits at
    // `anchor` relative to the pivot `world_pos`). `size` and `anchor`
    // are LOCAL now — the full world linear transform (rotation + scale
    // + skew) lives in `basis`. anchor [0,0] = strictly-centered (legacy).
    let local = i.anchor + vec2<f32>(v.quad_pos.x * i.size.x, v.quad_pos.y * i.size.y);
    // Apply the 2x2 world basis: col0 = basis.xy, col1 = basis.zw.
    // A sheared (non-orthogonal) basis maps the axis-aligned local quad
    // to the correct parallelogram — true skew, not a rotated rectangle.
    let mapped = vec2<f32>(
        local.x * i.basis.x + local.y * i.basis.z,
        local.x * i.basis.y + local.y * i.basis.w,
    );
    let world = i.world_pos + mapped;

    // Logical flip (ADR-0070-amendment-3): flip the TEXTURE sample UV,
    // not the geometry. bit0 mirrors u, bit1 mirrors v. Per-corner tint
    // stays geometry-locked (uses the UNFLIPPED quad_uv below), so the
    // gradient corners pin to screen corners regardless of texture flip.
    let flip_x = (i.flip_uv & 1u) != 0u;
    let flip_y = (i.flip_uv & 2u) != 0u;
    var quv = v.quad_uv;
    if (flip_x) { quv.x = 1.0 - quv.x; }
    if (flip_y) { quv.y = 1.0 - quv.y; }
    // The texture UV is resolved in the fragment (tiling + in-rect wrap),
    // so the vertex just forwards the flipped quad UV + the per-instance
    // constants the fragment needs.

    // Bilinear per-corner tint over the UNFLIPPED quad_uv:
    // (0,0)=TL, (1,0)=TR, (0,1)=BL, (1,1)=BR. At each of the 4 quad
    // vertices `quad_uv` is exactly a corner, so the mix yields that
    // corner's color and the rasterizer interpolates across fragments.
    let top = mix(i.corner_tl, i.corner_tr, v.quad_uv.x);
    let bot = mix(i.corner_bl, i.corner_br, v.quad_uv.x);
    let corner = mix(top, bot, v.quad_uv.y);

    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.quv = quv;
    out.tint = i.tint;
    out.premultiplied = i.premultiplied;
    out.corner = corner;
    out.opacity = i.opacity;
    out.tint_fill = select(0.0, 1.0, (i.flip_uv & 4u) != 0u);
    out.atlas_uv = i.atlas_uv;
    out.uv_xform = i.uv_xform;
    out.repeat_mode = (i.flip_uv >> 3u) & 3u;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // W3.T3.11 tiling + scroll (inside the sprite's own sub-rect — never
    // bleeds into atlas neighbours):
    //  1. TILE by `scale`; the RepeatMode governs how the tiling wraps at
    //     the rect edge (repeat / mirror / clamp).
    //  2. SCROLL by `offset`; a scroll always WRAPS (fract) so it stays
    //     continuous and never clamps the sprite off-screen (spec §9.2
    //     background-scroll use case) — even in Clamp mode. `offset == 0`
    //     skips the fract so the default (and Clamp tiling) is unchanged,
    //     keeping identity (scale 1, offset 0) bit-for-bit equal to legacy.
    let tiled = wrap_uv(in.quv * in.uv_xform.xy, in.repeat_mode);
    let scrolled = tiled + in.uv_xform.zw;
    let local = select(scrolled, fract(scrolled), in.uv_xform.zw != vec2<f32>(0.0));
    let uv = vec2<f32>(
        mix(in.atlas_uv.x, in.atlas_uv.z, local.x),
        mix(in.atlas_uv.y, in.atlas_uv.w, local.y),
    );
    // anatomia §4.2 canonical multiplicative math. All channels multiply
    // (no add/lerp/max), so composition is commutative and batch-stable.
    let sample = textureSample(atlas_tex, atlas_sampler, uv);

    // Step 4 — tint_fill: ignore the texel RGB (silhouette mode), keep
    // alpha. RGB becomes the combined per-corner × cascade tint.
    var rgb: vec3<f32>;
    if (in.tint_fill > 0.5) {
        rgb = in.corner.rgb * in.tint.rgb;
    } else {
        rgb = sample.rgb * in.corner.rgb * in.tint.rgb;
    }

    // Step 5 — `extra_alpha` is every alpha multiplier OTHER than the
    // texel's own α (corner.a · tint.a · opacity). Full alpha folds the
    // texel α on top.
    let extra_alpha = in.corner.a * in.tint.a * in.opacity;
    let alpha = sample.a * extra_alpha;

    // Step 6 — premultiply for the PREMULTIPLIED_ALPHA blend (pipeline.rs).
    if (in.premultiplied > 0.5) {
        // The bound texture is already premultiplied (BG-Removal Apply):
        // the bilinear `textureSample` blended premultiplied texels (like
        // Vello's `draw_image_rgba` preview), so partial-alpha edge texels
        // contribute rgb·α and there's no straight-alpha fringe. We must
        // NOT multiply rgb by the *texel* α again (§4.4 — doing so gives
        // rgb·α² and a dark fringe). But we MUST scale by `extra_alpha`
        // (opacity, tint.a, corner.a) so a premultiplied sprite fades
        // identically to a straight one — the texel α is already baked in,
        // the authored alpha factors are not (§4.4 amended; audit H-1/E-2).
        // extra_alpha defaults 1.0 (opacity/tint.a/corner.a all 1) → the
        // BG-Removal fringe fix is preserved byte-for-byte.
        return vec4<f32>(rgb * extra_alpha, alpha);
    }
    // Straight (non-premultiplied) sRGB atlas + premultiplied blend:
    // premultiply here so overlap composites linearly without the
    // dark-fringe artifact. `alpha` already carries opacity + every alpha
    // factor, so rgb is dimmed correctly through this multiply.
    return vec4<f32>(rgb * alpha, alpha);
}
