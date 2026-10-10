// C0 REBOTE (W5c, a lei do plano §2.4) — a emissão ganha `min(albedo, A_MAX) × S`: a luz que a
// superfície recebeu no quadro ANTERIOR (`c4_rebote.wgsl`), lida no MUNDO — a grade anda (pan,
// oitava) e o texel de hoje lê a fonte onde ela estava no mundo, nunca no mesmo índice. Mistura
// ADITIVA na emissão; o alfa (1/alcance) fica.

struct Rebote {
    // Texel de hoje (centro em i + ½) → texel do quadro anterior: q = p·escala + desloc.
    escala: vec2<f32>,
    desloc: vec2<f32>,
    // As dimensões da grade anterior.
    anterior: vec2<f32>,
    // 0 = não há fonte (o 1.º quadro, ou o solver acabou de nascer).
    valida: f32,
    albedo_max: f32,
}

@group(0) @binding(0) var<uniform> reb: Rebote;
@group(0) @binding(1) var albedo: texture_2d<f32>;
@group(0) @binding(2) var fonte: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) v: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

// A fonte em `q` (texels da grade anterior): a bilinear só entre os texels que a TÊM (alfa 1).
fn fonte_em(q: vec2<f32>) -> vec3<f32> {
    let base = floor(q - 0.5);
    let fr = q - 0.5 - base;
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    for (var j = 0u; j < 4u; j++) {
        let o = vec2<f32>(f32(j & 1u), f32(j >> 1u));
        let t = base + o;
        if any(t < vec2<f32>(0.0)) || any(t >= reb.anterior) {
            continue;
        }
        let s = textureLoad(fonte, vec2<i32>(t), 0);
        let w2 = mix(1.0 - fr, fr, o);
        let w = w2.x * w2.y * s.a;
        soma += s.rgb * w;
        peso += w;
    }
    if peso <= 0.0 {
        return vec3<f32>(0.0);
    }
    return soma / peso;
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let id = vec2<i32>(floor(pos.xy));
    let a = min(textureLoad(albedo, id, 0).rgb, vec3<f32>(reb.albedo_max));
    if reb.valida < 0.5 || all(a <= vec3<f32>(0.0)) {
        discard;
    }
    let q = (vec2<f32>(id) + 0.5) * reb.escala + reb.desloc;
    return vec4<f32>(a * fonte_em(q), 0.0);
}
