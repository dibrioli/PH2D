// A CONTA do tonemap — HDR linear pré-multiplicado → LDR linear pré-multiplicado. UMA função, que o
// passe de tonemap (`tonemap.wgsl`) e a colagem do vetor aceso pela luz 2D (`band_blit.wgsl`) concatenam:
// o sprite e o vetor sob a mesma luz dão o mesmo pixel (gate G7). O LUT entra por parâmetro porque cada
// módulo o liga no seu grupo; `textureSampleLevel` (o LUT não tem mips) deixa-a correr em qualquer ramo.
//
// ─── State of the AgX path (M14.5, audit 2026-05-11) ─────────────
//
// `BYPASS_LUT = true` is the **intentional** default until the editor
// canvas hosts genuine HDR content (e.g. additive lights > 1.0,
// particle emitters with bloom, HDR sprite imports). Reasons:
//   1. Current editor content is in-gamut sRGB. Tonemapping does
//      nothing perceptually useful for [0, 1] colors.
//   2. The bundled identity LUT stores raw RGB 0..1 in voxels, but
//      the shader samples it at log2-encoded coords. That mismatch
//      applies an unintended log curve — i.e. the path is NOT
//      mathematically identity. Routing through it would tint mid-
//      tones (the "dull look" reported during M14.5 round 7).
//
// ─── Migration trigger ──────────────────────────────────────────
//
// Flip `BYPASS_LUT = false` AND replace the identity LUT with a real
// AgX bake when ANY of the following ships:
//   • Light/particle component emitting linear values > 1.0 to game_rt
//   • HDR texture import path (EXR / HDR / 16-bit PNG sprites)  ⚠️ DISPAROU
//   • Bloom or glow post-process pass that exceeds [0, 1] range
//
// ⚠️ **O SEGUNDO GATILHO DISPAROU** (2026-08-21, plano
// `docs/Sprite_projeto/18`): a importação de PNG de 16 bits ship*ou* e
// uma sprite pode agora ser `Rgba16Float` de ponta a ponta. Esta nota
// dizia «quando isto acontecer», e aconteceu — quem move o número que
// tornava algo inalcançável tem de reconferir a nota (`CLAUDE.md` §0.0).
//
// ⛔ **E mesmo assim NÃO se acende o LUT.** O que falta não é o gatilho:
// é o bake, que não existe (`tools/bake_agx_lut/` continua TBD), e
// encaminhar pelo LUT identidade aplica a curva log não-intencional que
// o parágrafo acima descreve — o «dull look» que o Enio já recusou na
// ronda 7 do M14.5. *Um gatilho disparado sem a peça que ele pedia é
// uma nota a corrigir, não um interruptor a ligar.*
//
// O que a chegada dos 16 bits mudou de facto neste passe foi o dither
// da descida (abaixo), que é ortogonal ao LUT e ship*a* sem ele.
//
// Bake script lives at (TBD) `tools/bake_agx_lut/` — outputs
// `assets/luts/agx_default.cube` for `Tonemap::new()` to load. Until
// the trigger fires, this code path is intentionally cold but kept
// compile-clean so the wiring (bind groups, samplers, pipeline) stays
// validated each build.
const BYPASS_LUT: bool = true;

// AgX log-encoding constants (Troy Sobotka's reference values used
// by Blender 4.0+). Linear → Log2 → [0, 1].
const AGX_MIN_EV: f32 = -12.47393;
const AGX_MAX_EV: f32 = 4.026069;

fn agx_log2_encode(linear: vec3<f32>) -> vec3<f32> {
    // Guard against log2(0) → -inf.
    let safe = max(linear, vec3<f32>(1e-10));
    let log2v = log2(safe);
    return clamp(
        (log2v - vec3<f32>(AGX_MIN_EV)) / (AGX_MAX_EV - AGX_MIN_EV),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
}

fn tonemap_mapear(hdr: vec4<f32>, lut: texture_3d<f32>, lut_s: sampler) -> vec4<f32> {
    if (BYPASS_LUT) {
        // Pure passthrough: clamp HDR to LDR, let the sRGB target encode.
        // Keeps premultiplied alpha invariant (rgb already scaled by a).
        //
        // ⚠️ **A passagem é BYTE-EXACTA para conteúdo de 8 bits**, e há
        // gate a medi-la (`tonemap_descent_gpu`). É a invariante que a
        // recusa lá em cima protege: uma cor chapada entra e sai igual.
        return vec4<f32>(clamp(hdr.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), hdr.a);
    }

    // Premultiplied alpha throughout: divide-out alpha BEFORE tonemap
    // so highlights of semi-transparent sprites don't collapse to
    // mid-grey.
    let safe_a = max(hdr.a, 1e-6);
    let straight_rgb = hdr.rgb / safe_a;
    let coord = agx_log2_encode(straight_rgb);
    // Trilinear LUT sample. The 33³ identity LUT produces hdr_clamped
    // (the log2-decoded straight color, clamped to [0,1]) — visually
    // identical to no-op for in-gamut content, and the place to swap
    // in AgX's full sigmoid + per-channel curves once the LUT is baked.
    let mapped_rgb = textureSampleLevel(lut, lut_s, coord, 0.0).rgb;
    // Re-premultiply so the compositor's blend math stays consistent.
    return vec4<f32>(mapped_rgb * hdr.a, hdr.a);
}
