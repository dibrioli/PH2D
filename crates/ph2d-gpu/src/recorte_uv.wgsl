// O uv de uma sprite preso ao RECORTE dela (célula, região, lugar no atlas): o filtro nunca lê um texel de fora.
// `rect` é `[u0, v0, u1, v1]` em qualquer ordem; `dim` são os texels do nível 0. O uv fica entre os centros dos
// texels do nível lido (`passo` texels do nível 0) que estão a pelo menos `folga` texels da borda.
// Nível 0: `passo = 1`, `folga = 0`; um rect já recuado meio texel (`AtlasRegion::uv`) cai no mesmo lugar e o bit
// não muda. Doc: docs/Sprite_projeto/22_o_recorte_nao_le_o_vizinho.md
fn recorte_uv(uv: vec2<f32>, rect: vec4<f32>, dim: vec2<f32>, passo: vec2<f32>, folga: f32) -> vec2<f32> {
    let de = ceil(min(rect.xy, rect.zw) * dim - 0.5);
    let ate = floor(max(rect.xy, rect.zw) * dim + 0.5);
    let lo = (ceil((de + folga) / passo - 0.5) + 0.5) * passo;
    let hi = (floor((ate - folga) / passo + 0.5) - 0.5) * passo;
    let meio = (floor(0.5 * (de + ate)) + 0.5) / dim;
    return select(clamp(uv, lo / dim, hi / dim), meio, hi < lo);
}
