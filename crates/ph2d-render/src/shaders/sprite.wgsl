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

// ⭐⭐⭐ **A PELE (F9 W2)** — o que faz a PLACA posar o que a CPU posava vértice a vértice, todo
// quadro. Medido na arte do dono: a CPU custava `35,9 %` de um quadro a 8 imagens presas
// (`--release`), e o dispositivo desenha `100 352` triângulos em `0,73 ms` (`4,4 %`).
//
// `skin_w[i]` / `skin_b[i]` são PARALELOS ao buffer de vértices das MALHAS e indexados pelo
// `@builtin(vertex_index)`, que numa chamada NÃO-INDEXADA é o índice absoluto nesse buffer.
// `skin_b[i].x == SEM_PELE` quer dizer *«este vértice não é posado»* — e é isso que torna o caminho
// de toda malha que não é uma pele byte-idêntico POR CONSTRUÇÃO, em vez de multiplicar por uma
// identidade (`y · 0` deixa de ser `0` num `y` infinito).
//
// ⚠️ **Os afins chegam já conjugados para o espaço do QUAD** pelo lado da CPU — o payload guarda-os
// em LOCAL→LOCAL, porque o mesmo bind serve nove instâncias num 9-slice e cada uma tem o seu quad.
struct SkinAfim {
    lin: vec4<f32>,
    tra: vec2<f32>,
    // `(cos θ, sin θ)` da pose CRUA deste osso — o `atan2` do afim conjugado dava outro ângulo.
    ang: vec2<f32>,
    // `(índice local, ossos da malha, base da tabela de juntas, 0)`.
    info: vec4<u32>,
    // `(sx/sy, sy/sx, 0, 0)` da instância: conjugar uma ROTAÇÃO por um `size` não-uniforme não dá
    // uma rotação, e o `θ̄` só existe depois de o shader misturar os ângulos.
    razao: vec4<f32>,
};
@group(2) @binding(0)
var<storage, read> skin_w: array<vec4<f32>>;
@group(2) @binding(1)
var<storage, read> skin_b: array<vec4<u32>>;
@group(2) @binding(2)
var<storage, read> skin_a: array<SkinAfim>;
@group(2) @binding(3)
var<storage, read> skin_j: array<vec2<f32>>;

const SEM_PELE: u32 = 0xFFFFFFFFu;

fn aplica_afim(a: SkinAfim, p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        a.lin.x * p.x + a.lin.z * p.y + a.tra.x,
        a.lin.y * p.x + a.lin.w * p.y + a.tra.y,
    );
}

// A mistura LINEAR `Σ ŵ_i · (A_i · p)` — o CONTROLO, e o que as três degenerescências devolvem.
// ⛔ Soma zero devolve o ponto INTACTO e nunca a origem — a lei do `Skin::blend`.
fn mistura_linear(w: vec4<f32>, b: vec4<u32>, p: vec2<f32>) -> vec2<f32> {
    var out = vec2<f32>(0.0, 0.0);
    var soma = 0.0;
    for (var k = 0u; k < 4u; k = k + 1u) {
        let wk = w[k];
        if (wk == 0.0) { continue; }
        out = out + wk * aplica_afim(skin_a[b[k]], p);
        soma = soma + wk;
    }
    if (soma == 0.0) { return p; }
    return out;
}

// ⭐⭐⭐ **A LEI, no dispositivo** — `p' = R(θ̄)·(p − c) + Σ ŵ_i·(A_i·c)`, a MESMA que a
// `ph2d_render::SpriteMeshSkin::posa` corre na CPU para as dez costuras. *Dois motores, uma lei*,
// com gate de paridade entre eles (o molde é o do Flip).
//
// ⛔⛔ Ela NÃO é uma mistura linear de afins. A mistura linear é a lei ANTIGA, que o
// `ph2d_skeleton::Skin::blend_linear` guarda como controlo — a CPU passou a rodar em torno da
// JUNTA em 2026-09-19 para curar o entalhe do cotovelo, e a primeira redacção deste shader
// implementou a antiga: o gate de paridade leu `2,315e-3 m`.
//
// ⚠️ Tudo aqui vive no espaço do QUAD. Os afins e as juntas chegam conjugados pela CPU; a rotação
// `R(θ̄)` não pode, porque `θ̄` só nasce da mistura dos ângulos — daí `S⁻¹RS`, feito à mão com as
// duas razões do `size`.
fn posa_pela_pele(vi: u32, qp: vec2<f32>) -> vec2<f32> {
    let b = skin_b[vi];
    if (b.x == SEM_PELE) { return qp; }
    let w = skin_w[vi];
    let a0 = skin_a[b.x];
    let n = a0.info.y;
    let jb = a0.info.z;

    // O CENTRO: a junta de cada PAR, pesada por `w_i·w_j`. Sem par não há junta ⇒ a lei é rígida e
    // o centro é irrelevante (o `Skin::centro_de_rotacao` devolve `None` no mesmo sítio).
    var num = vec2<f32>(0.0, 0.0);
    var den = 0.0;
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (w[i] <= 0.0) { continue; }
        let li = skin_a[b[i]].info.x;
        for (var j = i + 1u; j < 4u; j = j + 1u) {
            if (w[j] <= 0.0) { continue; }
            let q = w[i] * w[j];
            num = num + q * skin_j[jb + li * n + skin_a[b[j]].info.x];
            den = den + q;
        }
    }
    if (den <= 0.0) { return mistura_linear(w, b, qp); }
    let c = num / den;

    // O ÂNGULO: média em CÍRCULO, nunca `Σ w·θ` — os ângulos vêm de um `atan2` e saltam em meia
    // volta. ⛔ A degenerescência dela (duas poses a `180°` com pesos iguais) cai na linear.
    var sx = 0.0;
    var sy = 0.0;
    var soma = 0.0;
    for (var k = 0u; k < 4u; k = k + 1u) {
        if (w[k] == 0.0) { continue; }
        let a = skin_a[b[k]];
        sx = sx + w[k] * a.ang.x;
        sy = sy + w[k] * a.ang.y;
        soma = soma + w[k];
    }
    if (soma == 0.0 || (sx == 0.0 && sy == 0.0)) { return mistura_linear(w, b, qp); }
    let nrm = sqrt(sx * sx + sy * sy);
    let co = sx / nrm;
    let si = sy / nrm;

    let base = mistura_linear(w, b, c);
    let d = qp - c;
    return vec2<f32>(
        co * d.x - si * a0.razao.y * d.y + base.x,
        si * a0.razao.x * d.x + co * d.y + base.y,
    );
}

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
fn vs_main(@builtin(vertex_index) vi: u32, v: VertexInput, i: InstanceInput) -> VertexOutput {
    // Local-space sprite corner (the quad is centered on its own
    // geometry, then shifted by `anchor` so the quad center sits at
    // `anchor` relative to the pivot `world_pos`). `size` and `anchor`
    // are LOCAL now — the full world linear transform (rotation + scale
    // + skew) lives in `basis`. anchor [0,0] = strictly-centered (legacy).
    // ⭐⭐⭐ **A PELE entra AQUI, antes de o quad virar local** — o `quad_pos` que chega é o de
    // REPOUSO e o que sai é o POSADO. ⚠️ A UV **não** é tocada: a tinta está pintada na forma de
    // repouso, e é isso que faz a imagem viajar COM a malha em vez de deslizar sobre ela.
    let qp = posa_pela_pele(vi, v.quad_pos);
    let local = i.anchor + vec2<f32>(qp.x * i.size.x, qp.y * i.size.y);
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

// ─── ClipChildren stencil-mark pass (W3 §8; ADR-0070/0074-amendment) ───
//
// The mark pass writes the clip-parent SILHOUETTE into the stencil buffer
// (the pipeline does the `Replace ref` where this fragment survives). It
// emits no color (the pipeline masks color writes off). The 16-attribute
// device limit is full, so the mark pipeline uses a MINIMAL instance
// layout (only the geometry + UV inputs it needs) and REPURPOSES the freed
// @location(5) — normally `tint`, unused here — to carry `clip_meta` from
// its real offset. Bits 8–15 hold the alpha cutoff quantized to u8, so a
// fragment whose texel alpha is ≤ the cutoff is discarded (carving the
// binary mask, spec §6.2/§6.4).

struct MarkInstanceInput {
    @location(2) world_pos: vec2<f32>,
    @location(3) size:      vec2<f32>,
    @location(4) atlas_uv:  vec4<f32>,
    @location(6) basis:     vec4<f32>,
    @location(8) anchor:    vec2<f32>,
    @location(14) flip_uv:  u32,
    @location(15) uv_xform: vec4<f32>,
    // Repurposed location (see comment above): clip_meta, NOT tint.
    @location(5) clip_meta: u32,
};

struct StencilMarkOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) quv: vec2<f32>,
    @location(1) @interpolate(flat) atlas_uv: vec4<f32>,
    @location(2) @interpolate(flat) uv_xform: vec4<f32>,
    @location(3) @interpolate(flat) repeat_mode: u32,
    @location(4) @interpolate(flat) cutoff: f32,
};

@vertex
fn vs_stencil_mark(@builtin(vertex_index) vi: u32, v: VertexInput, i: MarkInstanceInput) -> StencilMarkOutput {
    // Position EXACTLY like vs_main so the silhouette covers the same
    // pixels the parent would draw (anchor + size + 2x2 basis).
    // ⭐⭐⭐ **A PELE entra AQUI, antes de o quad virar local** — o `quad_pos` que chega é o de
    // REPOUSO e o que sai é o POSADO. ⚠️ A UV **não** é tocada: a tinta está pintada na forma de
    // repouso, e é isso que faz a imagem viajar COM a malha em vez de deslizar sobre ela.
    let qp = posa_pela_pele(vi, v.quad_pos);
    let local = i.anchor + vec2<f32>(qp.x * i.size.x, qp.y * i.size.y);
    let mapped = vec2<f32>(
        local.x * i.basis.x + local.y * i.basis.z,
        local.x * i.basis.y + local.y * i.basis.w,
    );
    let world = i.world_pos + mapped;

    // Same logical flip as vs_main so the sampled silhouette matches.
    let flip_x = (i.flip_uv & 1u) != 0u;
    let flip_y = (i.flip_uv & 2u) != 0u;
    var quv = v.quad_uv;
    if (flip_x) { quv.x = 1.0 - quv.x; }
    if (flip_y) { quv.y = 1.0 - quv.y; }

    var out: StencilMarkOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.quv = quv;
    out.atlas_uv = i.atlas_uv;
    out.uv_xform = i.uv_xform;
    out.repeat_mode = (i.flip_uv >> 3u) & 3u;
    // Dequantize the u8 cutoff stored in clip_meta bits 8–15.
    out.cutoff = f32((i.clip_meta >> 8u) & 0xffu) / 255.0;
    return out;
}

@fragment
fn fs_stencil_mark(in: StencilMarkOutput) -> @location(0) vec4<f32> {
    // Resolve the texture UV with the SAME tiling/scroll math as fs_main
    // so a tiled / scrolled parent masks by its visible silhouette.
    let tiled = wrap_uv(in.quv * in.uv_xform.xy, in.repeat_mode);
    let scrolled = tiled + in.uv_xform.zw;
    let local = select(scrolled, fract(scrolled), in.uv_xform.zw != vec2<f32>(0.0));
    let uv = vec2<f32>(
        mix(in.atlas_uv.x, in.atlas_uv.z, local.x),
        mix(in.atlas_uv.y, in.atlas_uv.w, local.y),
    );
    // The parent's texel alpha is the silhouette. Below the cutoff = OUTSIDE
    // the mask → discard so the pipeline does NOT write the stencil there.
    let a = textureSample(atlas_tex, atlas_sampler, uv).a;
    if (a <= in.cutoff) {
        discard;
    }
    // Color is masked off by the pipeline; the stencil `Replace` is what
    // matters. Return a defined value to satisfy the fragment signature.
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
