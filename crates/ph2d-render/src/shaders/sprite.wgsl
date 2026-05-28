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
    @location(6) rotation:  f32,        // radians; M14.7 gizmo
    // > 0.5 → this instance's texture is ALREADY premultiplied
    // (BG-Removal Apply bakes premultiplied so bilinear matches the
    // Vello preview). The fragment then skips its post-sample
    // premultiply. 0.0 for every other sprite (atlas + straight
    // individual) so they composite exactly as before.
    @location(7) premultiplied: f32,
    // Pivot offset (world-scaled local meters): the quad CENTER's
    // position relative to `world_pos`, which IS the pivot. Added to
    // the centered corner before rotation so the quad orbits the pivot
    // rather than its own center. [0,0] = strictly-centered (legacy).
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
    //   bit0 = flip_x, bit1 = flip_y, bit2 = tint_fill. bits 3+ reserved.
    @location(14) flip_uv: u32,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv:   vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) premultiplied: f32,
    // Bilinearly-interpolated per-corner tint (anatomia §4.2 step 2).
    @location(3) corner: vec4<f32>,
    @location(4) opacity: f32,
    // tint_fill decoded from flip_uv bit2; >0.5 = silhouette mode.
    // `flat` because it's a per-instance boolean, not a varying.
    @location(5) @interpolate(flat) tint_fill: f32,
};

@vertex
fn vs_main(v: VertexInput, i: InstanceInput) -> VertexOutput {
    // Local-space sprite corner. The quad is centered on its own
    // geometry, then shifted by `anchor` so the quad center sits at
    // `anchor` relative to the pivot (`world_pos`). `size` already
    // carries `Transform.scale` baked at extract time; rotation then
    // orbits the pivot (applies to anchor + corner together) before the
    // world translate. anchor [0,0] = strictly-centered (legacy).
    let local = i.anchor + vec2<f32>(v.quad_pos.x * i.size.x, v.quad_pos.y * i.size.y);
    let cos_r = cos(i.rotation);
    let sin_r = sin(i.rotation);
    let rotated = vec2<f32>(
        local.x * cos_r - local.y * sin_r,
        local.x * sin_r + local.y * cos_r,
    );
    let world = i.world_pos + rotated;

    // Logical flip (ADR-0070-amendment-3): flip the TEXTURE sample UV,
    // not the geometry. bit0 mirrors u, bit1 mirrors v. Per-corner tint
    // stays geometry-locked (uses the UNFLIPPED quad_uv below), so the
    // gradient corners pin to screen corners regardless of texture flip.
    let flip_x = (i.flip_uv & 1u) != 0u;
    let flip_y = (i.flip_uv & 2u) != 0u;
    var quv = v.quad_uv;
    if (flip_x) { quv.x = 1.0 - quv.x; }
    if (flip_y) { quv.y = 1.0 - quv.y; }
    let uv = vec2<f32>(
        i.atlas_uv.x + quv.x * (i.atlas_uv.z - i.atlas_uv.x),
        i.atlas_uv.y + quv.y * (i.atlas_uv.w - i.atlas_uv.y),
    );

    // Bilinear per-corner tint over the UNFLIPPED quad_uv:
    // (0,0)=TL, (1,0)=TR, (0,1)=BL, (1,1)=BR. At each of the 4 quad
    // vertices `quad_uv` is exactly a corner, so the mix yields that
    // corner's color and the rasterizer interpolates across fragments.
    let top = mix(i.corner_tl, i.corner_tr, v.quad_uv.x);
    let bot = mix(i.corner_bl, i.corner_br, v.quad_uv.x);
    let corner = mix(top, bot, v.quad_uv.y);

    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = uv;
    out.tint = i.tint;
    out.premultiplied = i.premultiplied;
    out.corner = corner;
    out.opacity = i.opacity;
    out.tint_fill = select(0.0, 1.0, (i.flip_uv & 4u) != 0u);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // anatomia §4.2 canonical multiplicative math. All channels multiply
    // (no add/lerp/max), so composition is commutative and batch-stable.
    let sample = textureSample(atlas_tex, atlas_sampler, in.uv);

    // Step 4 — tint_fill: ignore the texel RGB (silhouette mode), keep
    // alpha. RGB becomes the combined per-corner × cascade tint.
    var rgb: vec3<f32>;
    if (in.tint_fill > 0.5) {
        rgb = in.corner.rgb * in.tint.rgb;
    } else {
        rgb = sample.rgb * in.corner.rgb * in.tint.rgb;
    }

    // Step 5 — alpha folds every alpha channel + the opacity multiplier.
    let alpha = sample.a * in.corner.a * in.tint.a * in.opacity;

    // Step 6 — premultiply for the PREMULTIPLIED_ALPHA blend (pipeline.rs).
    if (in.premultiplied > 0.5) {
        // The bound texture is already premultiplied (BG-Removal Apply):
        // the bilinear `textureSample` blended premultiplied texels (like
        // Vello's `draw_image_rgba` preview), so partial-alpha edge texels
        // contribute rgb·α and there's no straight-alpha fringe. We must
        // NOT multiply rgb by α again (§4.4 — doing so gives rgb·α² and a
        // dark fringe). Returns the premultiplied rgb as-is with the
        // collapsed alpha — byte-faithful to §4.2 step 6.
        //
        // KNOWN GAP (W2 carry-over, audit H-1): for opacity<1 on a
        // premultiplied sprite, `alpha` is dimmed but this `rgb` is not,
        // so the sprite stays full-brightness at reduced coverage instead
        // of fading. ZERO W1 impact (opacity defaults 1.0; premultiplied
        // is the narrow BG-Removal-Apply set), but the full reconciliation
        // (premult rgb should also scale by opacity — and the §4.4 spec
        // text must be amended to say so) MUST land before opacity ships
        // as an authorable/animatable channel in W2. Left faithful to the
        // ratified §4.2 here rather than diverging from spec mid-W1.
        return vec4<f32>(rgb, alpha);
    }
    // Straight (non-premultiplied) sRGB atlas + premultiplied blend:
    // premultiply here so overlap composites linearly without the
    // dark-fringe artifact. `alpha` already carries opacity, so rgb is
    // dimmed by opacity through this multiply.
    return vec4<f32>(rgb * alpha, alpha);
}
