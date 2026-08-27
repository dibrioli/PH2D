// Motion glow (bloom) — the Call of Duty / Jimenez mip-chain (SIGGRAPH 2014),
// the technique Unity/Unreal ship and the reference LearnOpenGL "Physically Based
// Bloom" documents. A SINGLE wide box blur keeps the SQUARE of the source quad;
// this doesn't. The bright-passed image is progressively DOWNSAMPLED (13-tap) into
// a mip chain, then UPSAMPLED back (9-tap tent) with additive accumulation — the
// repeated bilinear halving dissolves the source's corners into a ROUND, smooth
// falloff with energy at every scale (a tight core halo AND a soft wide glow).
//
// HDR throughout (Rgba16Float): highlights above 1.0 survive the chain instead of
// clipping to white the way an 8-bit round-trip would. No transcendentals (HR-5):
// every pass is weighted texture taps.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle from the vertex index — no vertex buffer (matches tonemap).
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);
    let y = f32(vid & 2u);
    out.uv = vec2<f32>(x, 1.0 - y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct Params {
    // Meaning is per-pass; see each fragment:
    //  prefilter  → v = (threshold, threshold-knee, 2·knee, 0.25/knee), v2.x = teto
    //  downsample → v = (srcTexel.x, srcTexel.y, _, _)
    //  upsample   → v = (du.x, du.y, dv.x, dv.y)  (a base 2x2 da tenda, em UV)
    //  composite  → v = (intensity, saturation, tem_rampa, dirt_intensity),
    //               v2 = tint rgba, v3 = (scale.xy, offset.xy) da mascara de sujidade
    v: vec4<f32>,
    v2: vec4<f32>,
    v3: vec4<f32>,
};
@group(0) @binding(2) var<uniform> P: Params;
// A LUT da RAMPA do halo (doc 89 folha 11): uma tira de `HALO_LUT_TEXELS x 1` em
// Rgba16Float, assada na CPU a partir da rampa que o artista desenhou. Ela esta' no
// layout partilhado, entao viaja nos quatro passes -- so' o composite a le^.
@group(0) @binding(3) var halo_lut: texture_2d<f32>;
// A MASCARA DE SUJIDADE (doc 89 folha 11): o `Dirt Texture` do Unity / o `Bloom Dirt
// Mask` do Unreal. Como a LUT, esta' no layout partilhado e so' o composite a le^.
// Sem imagem escolhida o binding fica com uma textura PRETA de 1x1 -- e' ela que
// carrega a identidade do quadro, e nao um ramo aqui dentro.
@group(0) @binding(4) var dirt: texture_2d<f32>;

// Bright-pass with a soft knee (COD/Karis): below `threshold-knee` nothing
// contributes, above `threshold` full, quadratic ramp between — no hard edge.
// Premultiplied HDR in (Motion is over transparent black), so max(r,g,b) is a
// premult-safe brightness. Also downsamples (mip0 is half-res, bilinear).
// `P.v2.x` e' o TETO do bright-pass (doc 89 folha 11). O lado Rust manda o maior
// finito do Rgba16Float quando o knob esta' desligado, entao o `min` nao pode
// morder nada que o RT consiga guardar -- o caminho neutro fica intacto sem um
// ramo. O clamp e' POR CANAL de proposito: limitar a luminancia e reescalar
// mudaria o MATIZ do pixel que estourou, e o antidoto do firefly nao pode
// recolorir a cena.
// `P.v2.y` escolhe DE QUE o bright-pass se alimenta (doc 89 folha 11, o *Glow Based
// On* do AE): `0` = luminancia (o de sempre, `max(r,g,b)` premultiplicado), `1` = o
// ALFA. A diferenca e' o que a referencia oferece: com luma uma silhueta PRETA opaca
// nao tem nada acima do limiar e nunca acende; com alfa ela acende pela COBERTURA,
// que e' o que se quer quando o halo e' uma aura e nao uma emissao. O `select` esta'
// escrito com a condicao no formato do WGSL (`f, t, cond`), e o ramo falso e' o
// literal de sempre -- em `source = 0` o numero e' o mesmo, bit a bit.
@fragment
fn fs_prefilter(in: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(src, samp, in.uv);
    let c = min(texel.rgb, vec3<f32>(P.v2.x));
    let luma = max(c.r, max(c.g, c.b));
    let brightness = select(luma, min(texel.a, P.v2.x), P.v2.y >= 0.5);
    var soft = brightness - P.v.y;
    soft = clamp(soft, 0.0, P.v.z);
    soft = soft * soft * P.v.w;
    let contribution = max(soft, brightness - P.v.x) / max(brightness, 1e-4);
    return vec4<f32>(c * contribution, 1.0);
}

// 13-tap downsample (COD): centre + inner 2×2 + outer ring, weighted so the
// filter is a smooth wide tent — this is where the corners get rounded off. `x`,
// `y` are the SOURCE texel size, so the taps land on the source's texel grid.
@fragment
fn fs_downsample(in: VsOut) -> @location(0) vec4<f32> {
    let x = P.v.x;
    let y = P.v.y;
    let uv = in.uv;
    let a = textureSample(src, samp, uv + vec2<f32>(-2.0 * x, 2.0 * y)).rgb;
    let b = textureSample(src, samp, uv + vec2<f32>(0.0, 2.0 * y)).rgb;
    let c = textureSample(src, samp, uv + vec2<f32>(2.0 * x, 2.0 * y)).rgb;
    let d = textureSample(src, samp, uv + vec2<f32>(-2.0 * x, 0.0)).rgb;
    let e = textureSample(src, samp, uv).rgb;
    let f = textureSample(src, samp, uv + vec2<f32>(2.0 * x, 0.0)).rgb;
    let g = textureSample(src, samp, uv + vec2<f32>(-2.0 * x, -2.0 * y)).rgb;
    let h = textureSample(src, samp, uv + vec2<f32>(0.0, -2.0 * y)).rgb;
    let i = textureSample(src, samp, uv + vec2<f32>(2.0 * x, -2.0 * y)).rgb;
    let j = textureSample(src, samp, uv + vec2<f32>(-x, y)).rgb;
    let k = textureSample(src, samp, uv + vec2<f32>(x, y)).rgb;
    let l = textureSample(src, samp, uv + vec2<f32>(-x, -y)).rgb;
    let m = textureSample(src, samp, uv + vec2<f32>(x, -y)).rgb;
    var o = e * 0.125;
    o += (a + c + g + i) * 0.03125;
    o += (b + d + f + h) * 0.0625;
    o += (j + k + l + m) * 0.125;
    return vec4<f32>(o, 1.0);
}

// 9-tap tent upsample (COD): a 3×3 [1 2 1; 2 4 2; 1 2 1]/16 kernel. Blended
// ADDITIVELY onto the finer mip (which already holds its own downsample), so the
// accumulation across the chain is what builds the round multi-scale glow.
//
// A tenda le' uma BASE 2x2 e nao dois raios (doc 89 folha 11): `du = P.v.xy` e
// `dv = P.v.zw`. Com `stretch = 1` o lado Rust manda `du = (fr, 0)` e
// `dv = (0, fr*aspect)`, e os nove offsets reconstroem tap a tap os de sempre --
// a mesma soma, ao bit. Com anamorfose a base roda e estica, e o halo vira o
// streak anamorfico.
@fragment
fn fs_upsample(in: VsOut) -> @location(0) vec4<f32> {
    let du = P.v.xy;
    let dv = P.v.zw;
    let uv = in.uv;
    let a = textureSample(src, samp, uv - du + dv).rgb;
    let b = textureSample(src, samp, uv + dv).rgb;
    let c = textureSample(src, samp, uv + du + dv).rgb;
    let d = textureSample(src, samp, uv - du).rgb;
    let e = textureSample(src, samp, uv).rgb;
    let f = textureSample(src, samp, uv + du).rgb;
    let g = textureSample(src, samp, uv - du - dv).rgb;
    let h = textureSample(src, samp, uv - dv).rgb;
    let i = textureSample(src, samp, uv + du - dv).rgb;
    var o = e * 4.0;
    o += (b + d + f + h) * 2.0;
    o += (a + c + g + i);
    o *= 1.0 / 16.0;
    return vec4<f32>(o, 1.0);
}

// The accumulated glow (mip0), coloured then scaled: desaturate toward luminance
// by `saturation` (0 = a white bloom), multiply by the tint (default white =
// no-op), then by intensity. The pipeline blends this ADDITIVELY over the scene
// (color One/One) — emitted light only brightens, so it bleeds over whatever is
// in front. Alpha 0 + the pipeline keeps the destination alpha, so the opaque
// scene stays opaque for the compositor.
// A RAMPA DO HALO (doc 89 folha 11, o *Glow Colors A & B* do AE): a cor do halo deixa
// de ser uma constante e passa a ser escolhida pelo BRILHO dele. `P.v.z` diz se ha'
// rampa; sem ela o caminho e' o `tint` literal de sempre, ao bit.
//
// ⚠️ O indice e' a LUMINANCIA do halo, saturada em [0,1] -- e' o valor do brilho a
// escolher a cor, que e' o que a referencia faz. A LUT tem um texel por celula e o
// sampler interpola entre vizinhas; a tabela ja' vem com a interpolacao e o espaco de
// cor que o artista escolheu, resolvidos na CPU.
@fragment
fn fs_composite(in: VsOut) -> @location(0) vec4<f32> {
    let intensity = P.v.x;
    let saturation = P.v.y;
    let tint = P.v2;
    var glow = textureSample(src, samp, in.uv).rgb;
    // Rec.709 luminance; mix toward grey for a white bloom at saturation 0.
    let lum = dot(glow, vec3<f32>(0.2126, 0.7152, 0.0722));
    glow = mix(vec3<f32>(lum), glow, saturation);
    var colour = tint.rgb;
    if (P.v.z >= 0.5) {
        colour = textureSample(halo_lut, samp, vec2<f32>(clamp(lum, 0.0, 1.0), 0.5)).rgb;
    }
    // A SUJIDADE SOMA-SE A' TINTA, que e' a forma da referencia (`bloom * (tint + dirt)`
    // no UberPost do Unity): a mascara nao pinta nada sozinha, ela acende ONDE o halo ja'
    // esta'. Somar em vez de multiplicar tambem e' o que deixa a cor da imagem passar --
    // um mapa de sujidade e' uma fotografia colorida de po' e riscos, nao uma mascara de
    // cobertura.
    //
    // O `v3` traz `scale`+`offset` ja' compostos na CPU: o `cover` que preserva o aspecto
    // da imagem E o sub-rect da celula quando ela vem do atlas partilhado. Ver
    // `motion_fx_dirt::scale_offset` -- a lei toda mora la', e este passe faz UMA afim.
    let dirt_uv = in.uv * P.v3.xy + P.v3.zw;
    colour = colour + textureSample(dirt, samp, dirt_uv).rgb * P.v.w;
    glow = glow * colour * (intensity * tint.a);
    return vec4<f32>(glow, 0.0);
}
