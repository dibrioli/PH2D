// ⭐ P14 «L» — o primeiro momento angular `M = ∫L(θ)(cos θ, sin θ)dθ` por canal, na placa
// (`docs/PixelArt/P14_plano.md` §4; gabarito `ph2d_light2d_ref::momento`). Dois passes PRÓPRIOS, que só
// correm com receptor de normal à vista: os passes de hoje não mudam um bit (o `MOMENTO` deles é `false`).
//
// · `cascata_momento`: a c0 refeita (as mesmas 4 direções da `cascata`, o mesmo `0.25·TAU`), `ω̂·L`.
// · `c3_momento`: o `luz_do_texel` da fina com o `MOMENTO` ligado — cada lâmpada soma `s·v·ω̂·sin(h)/h`
//   (o disco: `∫ω̂` sobre a abertura `2h` é `2 sin h·ω̂`; o ponto: `h → 0`; dentro do disco `M = 0`) e a
//   c0 do momento interpolada como a `c0_em`.

override MOMENTO: bool = false;

@group(2) @binding(30) var c0_momento_x: texture_2d<f32>;
@group(2) @binding(31) var c0_momento_y: texture_2d<f32>;
@group(2) @binding(32) var fina_momento_x: texture_storage_2d<rgba16float, write>;
@group(2) @binding(33) var fina_momento_y: texture_storage_2d<rgba16float, write>;
@group(2) @binding(34) var c0_momento_x_saida: texture_storage_2d<rgba16float, write>;
@group(2) @binding(35) var c0_momento_y_saida: texture_storage_2d<rgba16float, write>;

// O momento do passo corrente do `luz_do_texel` (x: `Mx` rgb · y: `My` rgb).
var<private> momento_x: vec3<f32>;
var<private> momento_y: vec3<f32>;
// O do texel: as duas faces misturadas como a luz (`mix` por `f2`).
var<private> momento_fina_x: vec3<f32>;
var<private> momento_fina_y: vec3<f32>;

// A parcela de uma lâmpada `L` vista de `P` com luz `l = s·v` (o [`luz_das_lampadas`]).
fn momento_da_lampada(P: vec2<f32>, L: Lampada, l: vec3<f32>) {
    let v = L.centro - P;
    let d = length(v);
    if d <= max(L.raio, 0.0) || d <= 0.0 {
        return;
    }
    var k = 1.0;
    if L.raio > 0.0 {
        let h = asin(min(L.raio / d, 1.0));
        k = select(1.0, sin(h) / h, h > 0.0);
    }
    let w = v / d * k;
    momento_x += l * w.x;
    momento_y += l * w.y;
}

// A c0 do momento no ponto `P` das cascatas: a bilinear da `c0_em` (bordas presas).
fn c0_momento_em(P: vec2<f32>) {
    let u = P - 0.5;
    let base = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(base);
    let ultimo = vec2<i32>(cas.tamanho) - 1;
    let q = clamp(base, vec2<i32>(0), ultimo);
    let r = clamp(base + 1, vec2<i32>(0), ultimo);
    let a = vec2<i32>(r.x, q.y);
    let b = vec2<i32>(q.x, r.y);
    momento_x = mix(
        mix(textureLoad(c0_momento_x, q, 0).rgb, textureLoad(c0_momento_x, a, 0).rgb, fr.x),
        mix(textureLoad(c0_momento_x, b, 0).rgb, textureLoad(c0_momento_x, r, 0).rgb, fr.x),
        fr.y,
    );
    momento_y = mix(
        mix(textureLoad(c0_momento_y, q, 0).rgb, textureLoad(c0_momento_y, a, 0).rgb, fr.x),
        mix(textureLoad(c0_momento_y, b, 0).rgb, textureLoad(c0_momento_y, r, 0).rgb, fr.x),
        fr.y,
    );
}

@compute @workgroup_size(8, 8)
fn cascata_momento(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= cas.tamanho.x || id.y >= cas.tamanho.y || cas.nivel != 0u {
        return;
    }
    let P = vec2<f32>(id.xy) + 0.5;
    var mx = vec3<Real>(Real(0));
    var my = vec3<Real>(Real(0));
    for (var s = 0u; s < 4u; s++) {
        let ang = TAU * (f32(s) + 0.5) / 4.0;
        let l = radiancia(P, id.xy, vec2<f32>(cos(ang), sin(ang)), s, s);
        mx += l * Real(cos(ang));
        my += l * Real(sin(ang));
    }
    textureStore(c0_momento_x_saida, id.xy, vec4<f32>(vec3<f32>(mx) * (0.25 * TAU), 1.0));
    textureStore(c0_momento_y_saida, id.xy, vec4<f32>(vec3<f32>(my) * (0.25 * TAU), 1.0));
}

@compute @workgroup_size(8, 8)
fn c3_momento(@builtin(global_invocation_id) gid: vec3<u32>) {
    let alvo = gid.xy + fina.inicio;
    if !all(alvo < fina.fim) {
        return;
    }
    borda_aqui = true;
    let tau = meio(ponto_fino(alvo)).tau;
    _ = luz_do_texel(alvo, onde_calcular(alvo), tau, false, false);
    textureStore(fina_momento_x, alvo, vec4<f32>(momento_fina_x, 1.0));
    textureStore(fina_momento_y, alvo, vec4<f32>(momento_fina_y, 1.0));
}
