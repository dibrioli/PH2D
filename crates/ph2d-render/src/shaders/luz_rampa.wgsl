// P13 L — a luz na rampa (gabarito CPU: `ph2d_pixel_rampa::luz`). LUT: linha = rampa, texel =
// (L OKLab, r, g, b sRGB 0..1); L < 0 = fim da rampa. Pede o `pc_*` da `ph2d_color::wgsl_perceptual!`.
fn luz_rampa_lum(luz: vec3<f32>) -> f32 {
    return pc_to_oklab(max(luz, vec3<f32>(0.0))).x;
}

// O degrau aceso: `L_alvo = L(degrau pos) · lum(luz)`, o de L mais próximo na MESMA linha; empate =
// o mais escuro. Devolve o texel da LUT (`.yzw` = a cor da paleta).
fn luz_receber_rampa(lut: texture_2d<f32>, rampa: u32, pos: u32, luz: vec3<f32>) -> vec4<f32> {
    let w = textureDimensions(lut).x;
    let alvo = textureLoad(lut, vec2<u32>(pos, rampa), 0).x * luz_rampa_lum(luz);
    var melhor = textureLoad(lut, vec2<u32>(0u, rampa), 0);
    var dist = abs(melhor.x - alvo);
    for (var i = 1u; i < w; i = i + 1u) {
        let g = textureLoad(lut, vec2<u32>(i, rampa), 0);
        if (g.x < 0.0) {
            break;
        }
        let e = abs(g.x - alvo);
        if (e < dist) {
            dist = e;
            melhor = g;
        }
    }
    return melhor;
}
