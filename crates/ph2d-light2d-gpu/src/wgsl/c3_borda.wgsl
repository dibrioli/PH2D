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
@group(3) @binding(14) var borda_vizinhos: sampler;

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

// A normal para dentro pelo Sobel (pesos √2) da 3×3 `k` (a cobertura presa a 1); zero sem gradiente.
fn normal_sobel(k: array<f32, 9>) -> vec2<f32> {
    var c: array<f32, 9>;
    for (var i = 0; i < 9; i++) {
        c[i] = min(k[i], 1.0);
    }
    let r2 = 1.4142135;
    let g = vec2<f32>(
        (c[2] + r2 * c[5] + c[8]) - (c[0] + r2 * c[3] + c[6]),
        (c[6] + r2 * c[7] + c[8]) - (c[0] + r2 * c[1] + c[2]),
    );
    let m = length(g);
    if m <= 1e-3 {
        return vec2<f32>(0.0);
    }
    return g / m;
}

// O recuo da reta de normal `u` que deixa dentro a área `a` de um texel.
fn recuo_de(u: vec2<f32>, a: f32) -> f32 {
    return select(-recuo_da_area(u, 1.0 - a), recuo_da_area(u, a), a <= 0.5);
}

// O que o `c3_borda` grava no texel `t` dada a sua 3×3 `k` (as coordenadas presas à textura, como a
// leitura direta): `(normal para dentro, recuo)` de um parcial; na contínua (`BORDA_CONTINUA`) o 3.º é a
// DISTÂNCIA com sinal no centro — o parcial −recuo, o fio cobertura − ½, o cheio/vazio da faixa a reta
// emprestada ([`reta_emprestada`]), o resto ±1.
fn linha_de(t: vec2<i32>, n: vec2<i32>, k: array<f32, 9>) -> vec3<f32> {
    let a = k[4];
    if a > 0.0 && a < 1.0 {
        let u = normal_sobel(k);
        if all(u == vec2<f32>(0.0)) {
            return vec3<f32>(0.0, 0.0, select(0.0, a - 0.5, BORDA_CONTINUA));
        }
        let r = recuo_de(u, a);
        return vec3<f32>(u, select(r, -r, BORDA_CONTINUA));
    }
    if !BORDA_CONTINUA {
        return vec3<f32>(0.0);
    }
    let r = reta_emprestada(t, n, k, a >= 1.0);
    return vec3<f32>(0.0, 0.0, select(-r.z, select(-1.0, 1.0, a >= 1.0), all(r.xy == vec2<f32>(0.0))));
}

// ⭐ W6d — a reta de um texel CHEIO ou VAZIO da faixa (`BORDA_CONTINUA`): a normal pelo Sobel do PRÓPRIO
// 3×3 e, de cada parcial vizinho (dentro da textura), a reta com essa normal que deixa dentro a área
// dele, transladada para o centro deste; fica a que passa mais perto, com o sinal conferido (cheio ⇒
// dentro no centro). Nenhuma ⇒ zero (a distância fica ±1). (A reta de cada vizinho pelo Sobel DELE
// custava +0,20 ms a 1080p: handoff W6d §9.)
fn reta_emprestada(t: vec2<i32>, n: vec2<i32>, k: array<f32, 9>, cheio: bool) -> vec3<f32> {
    var melhor = vec3<f32>(0.0);
    var perto = 1e9;
    var u = vec2<f32>(0.0);
    for (var j = 0; j < 3; j++) {
        for (var i = 0; i < 3; i++) {
            let o = vec2<i32>(i - 1, j - 1);
            let q = t + o;
            let a = k[j * 3 + i];
            if (i == 1 && j == 1) || any(q < vec2<i32>(0)) || any(q >= n) || a <= 0.0 || a >= 1.0 {
                continue;
            }
            if all(u == vec2<f32>(0.0)) {
                u = normal_sobel(k);
                if all(u == vec2<f32>(0.0)) {
                    return melhor;
                }
            }
            // dentro ⟺ (x − c_q)·u ≥ r  ⟺  (x − c_t)·u ≥ r + (c_q − c_t)·u.
            let recuo = recuo_de(u, a) + dot(vec2<f32>(o), u);
            if (-recuo >= 0.0) != cheio {
                continue;
            }
            if abs(recuo) < perto {
                perto = abs(recuo);
                melhor = vec3<f32>(u, recuo);
            }
        }
    }
    return melhor;
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
    if BORDA_CONTINUA {
        // ⭐ W6d: a 3×3 por quatro `textureGather` nos cantos de `t` (o amostrador prende à borda, como o
        // `clamp` da leitura direta); x = (u0, v1), y = (u1, v1), z = (u1, v0), w = (u0, v0).
        let nf = vec2<f32>(n);
        let g00 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t) / nf);
        let g11 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t + 1) / nf);
        let g10 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t + vec2<i32>(1, 0)) / nf);
        let g01 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t + vec2<i32>(0, 1)) / nf);
        // Longe de toda borda (a grande maioria) nenhum parcial na 3×3: ±1, sem listas.
        let parcial = (g00 > vec4<f32>(0.0) & g00 < vec4<f32>(1.0)) | (g11 > vec4<f32>(0.0) & g11 < vec4<f32>(1.0))
            | (g10 > vec4<f32>(0.0) & g10 < vec4<f32>(1.0)) | (g01 > vec4<f32>(0.0) & g01 < vec4<f32>(1.0));
        if any(parcial) {
            let k = array<f32, 9>(g00.w, g00.z, g10.z, g00.x, a, g10.y, g01.x, g01.y, g11.y);
            linha = linha_de(t, n, k);
        } else {
            linha = vec3<f32>(0.0, 0.0, select(-1.0, 1.0, a >= 1.0));
        }
    } else if a > 0.0 && a < 1.0 {
        var k: array<f32, 9>;
        for (var j = 0; j < 3; j++) {
            for (var i = 0; i < 3; i++) {
                let q = clamp(t + vec2<i32>(i - 1, j - 1), vec2<i32>(0), n - 1);
                k[j * 3 + i] = textureLoad(borda_cobertura, q, 0).r;
            }
        }
        linha = linha_de(t, n, k);
    }
    textureStore(borda_saida, t, vec4<f32>(a, linha));
}
