// Tonemap pass (M14.5).
//
// Samples the HDR game RT (Rgba16Float, linear-light) and writes
// the tonemapped result into an LDR sRGB target. AgX LUT-based
// implementation: the 3D LUT (33³ Rgba8Unorm) is bound at
// @group(0) @binding(2). When no LUT is supplied the host can use
// `Tonemap::new_identity()` which binds a 2×2×2 identity volume —
// the shader is the same.
//
// Coordinate convention: the LUT input domain is AgX's "Log2"
// encoding remapped to [0, 1]. We map linear HDR → log2 → [0, 1]
// via the standard AgX log encoding (min EV = -12.47, max EV = +4.03,
// Blender 4.0+ reference values), sample the LUT trilinearly, then
// return the LDR result. The output texture format is *Srgb so wgpu
// re-encodes the linear output to sRGB on write — there is **no
// manual gamma** in this shader.
//
// ⭐ O MAPEAMENTO (`BYPASS_LUT`, o estado do AgX, a codificação log) vive em `tonemap_conta.wgsl`,
// concatenado à frente: a colagem do vetor ACESO pela luz 2D (`band_blit.wgsl`) aplica a MESMA função.


@group(0) @binding(0)
var game_rt: texture_2d<f32>;
@group(0) @binding(1)
var game_sampler: sampler;
@group(0) @binding(2)
var agx_lut: texture_3d<f32>;
@group(0) @binding(3)
var lut_sampler: sampler;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle — three verts cover the whole NDC quad with a
// single triangle, cheaper than two-triangle quads (no shared edge).
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);
    let y = f32(vid & 2u);
    out.uv = vec2<f32>(x, 1.0 - y);
    out.clip_pos = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

// ─── ⛔ RECUSA MEDIDA: dither NESTE passe (2026-08-21) ───────────
//
// Este É o último sítio onde o conteúdo do jogo tem mais de 8 bits (o
// `game_rt` é `Rgba16Float`, a saída é `Bgra8UnormSrgb`), e por isso
// parece o sítio óbvio para o dither que mata as faixas de um degradê.
// Foi construído, medido, e **recusado**. Medido em 2026-08-21 por
// `tonemap_descent_gpu` (RTX, wgpu 28), sobre os 256 bytes × 64 células:
//
//   folga máxima que não move byte nenhum:  ~0,0283 LSB  (de 0,5)
//   com o pico que a CPU usa (0,4311 LSB):   5,98% dos pixels movidos
//
// Ou seja: um dither aqui teria de caber em **7%** da amplitude que o
// caminho de software usa — e a 7% ele não espalha nada. A alternativa é
// aceitar que 6% de uma cor chapada vire mosquito, que numa ferramenta
// de arte por pixels é o defeito pior de todos: *o que eu pintei não é
// o que está no ecrã*.
//
// O MECANISMO, que é o que impede reconstruir isto sem pensar: o valor
// que chega aqui é `hw_decode(byte)`, e a tabela sRGB do hardware **não
// é a curva ideal** — `precision_parity_gpu` mediu-a a afastar-se até
// `0,00195` em linear, que naquele brilho vale ~0,34 de um código.
// `hw_encode(hw_decode(N)) == N` é garantido pelas especificações, mas
// só enquanto ninguém empurra o valor pelo meio. Um shader que
// re-codifique com a curva *ideal*, some o viés e volte a descodificar
// está a medir a distância à fronteira com uma régua que não é a do
// hardware, e o que sobra de folga é propriedade DA PLACA — não do
// formato, não da representação, não de nada que caiba numa constante
// portátil.
//
// ⛔ Encolher a amplitude até passar seria trocar um defeito visível por
// um número ajustado a uma placa só. A sonda `tonemap_descent_gpu`
// ficou no repo a MEDIR essa folga em qualquer máquina: quem quiser
// reabrir isto começa por a correr, não por escrever um shader.
//
// ✅ O que ship*ou* foi o dither da descida que o autor COMANDA (o botão
// `RGBA8` do Inspector), onde a conversão é em software de ponta a ponta
// e a amplitude sai de uma deriva medida e portátil (`ph2d_color::dither`).
//
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return tonemap_mapear(textureSample(game_rt, game_sampler, in.uv), agx_lut, lut_sampler);
}
