// ⭐ W5d — a BORDA EXATA da captura que a C3 lê, um texel de cada vez — a reconstrução do AAEDT
// (Gustavson & Strand, «Anti-aliased Euclidean distance transform», Pattern Recognition Letters
// 32(2), 2011): a cobertura é a ÁREA exata, logo um texel cheio está todo dentro, um vazio todo fora,
// e num parcial a borda é a reta de normal = o gradiente da cobertura (Sobel de pesos √2) que deixa
// dentro a área igual à cobertura. Saída: `(cobertura, normal para dentro, recuo)` — dentro ⟺
// `(p − centro)·normal ≥ recuo`; normal nula = sem gradiente (um fio de um texel: o iso-½).
//
// Concatenado depois de `c2_rc.wgsl` (o `dentro_exato` dele lê esta saída no lugar da cobertura).

@group(3) @binding(12) var borda_cobertura: texture_2d<f32>;
@group(3) @binding(13) var borda_saida: texture_storage_2d<rgba16float, write>;

// O `t ≥ 0` tal que a área do texel unitário (centrado) além da reta `u·x = t` vale `area ≤ ½` — a
// inversa da conta do trapézio (o `area_alem` da C3).
fn recuo_da_area(u: vec2<f32>, area: f32) -> f32 {
    let p = max(0.5 * min(abs(u.x), abs(u.y)), 1e-6);
    let q = 0.5 * max(abs(u.x), abs(u.y));
    if area <= p / (2.0 * q) {
        return p + q - sqrt(8.0 * p * q * area);
    }
    return q - 2.0 * q * area;
}

@compute @workgroup_size(8, 8)
fn c3_borda(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = vec2<i32>(textureDimensions(borda_cobertura));
    let t = vec2<i32>(gid.xy);
    if any(t >= n) {
        return;
    }
    let a = textureLoad(borda_cobertura, t, 0).r;
    var linha = vec3<f32>(0.0);
    if a > 0.0 && a < 1.0 {
        var k: array<f32, 9>;
        for (var j = 0; j < 3; j++) {
            for (var i = 0; i < 3; i++) {
                let q = clamp(t + vec2<i32>(i - 1, j - 1), vec2<i32>(0), n - 1);
                k[j * 3 + i] = min(textureLoad(borda_cobertura, q, 0).r, 1.0);
            }
        }
        let r2 = 1.4142135;
        let g = vec2<f32>(
            (k[2] + r2 * k[5] + k[8]) - (k[0] + r2 * k[3] + k[6]),
            (k[6] + r2 * k[7] + k[8]) - (k[0] + r2 * k[1] + k[2]),
        );
        let m = length(g);
        if m > 1e-3 {
            let u = g / m;
            if a <= 0.5 {
                linha = vec3<f32>(u, recuo_da_area(u, a));
            } else {
                linha = vec3<f32>(u, -recuo_da_area(u, 1.0 - a));
            }
        }
    }
    textureStore(borda_saida, t, vec4<f32>(a, linha));
}
