// A SOMBRA DA CAIXA no mapa de sombra (PCSS em tres niveis) — ranhura do `forward.wgsl`.

// ⭐⭐ A SOMBRA DA CAIXA — penumbra de tamanho FISICO (PCSS): a caixa de luz tem raio angular, logo a
// sombra e' dura onde o objecto toca o chao e mole longe dele. Pontos fixos (nada de ruido rodado por
// pixel): a mesma imagem em todo quadro.
const DISCO: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(0.17677670, 0.00000000),
    vec2<f32>(-0.22577219, 0.20682582),
    vec2<f32>(0.03455805, -0.39377118),
    vec2<f32>(0.28457122, 0.37117276),
    vec2<f32>(-0.52222319, -0.09237393),
    vec2<f32>(0.49469539, -0.31468471),
    vec2<f32>(-0.16546593, 0.61552500),
    vec2<f32>(-0.31556147, -0.60759440),
    vec2<f32>(0.68464216, 0.25003022),
    vec2<f32>(-0.71225609, 0.29400896),
    vec2<f32>(0.34335450, -0.73372862),
    vec2<f32>(0.25373024, 0.80893199),
    vec2<f32>(-0.76474589, -0.44318588),
    vec2<f32>(0.89713398, -0.19723239),
    vec2<f32>(-0.54750690, 0.77877223),
    vec2<f32>(-0.12648677, -0.97608970),
);

// ⭐ O raio maximo do PCSS em texels do NIVEL: 16 amostras numa janela maior mostram degraus. Uma
// penumbra maior le um nivel mais grosso do mapa (o mesmo enquadramento redesenhado a 1/4 do lado).
const PCSS_MAX: f32 = 48.0;
const PCSS_MIN: f32 = 12.0;

fn nivel_do_raio(raio0: f32) -> u32 {
    if (raio0 <= PCSS_MAX) {
        return 0u;
    }
    if (raio0 <= PCSS_MAX * 4.0) {
        return 1u;
    }
    return 2u;
}

fn prof_no_nivel(k: u32, q: vec2<i32>) -> f32 {
    if (k == 0u) {
        return textureLoad(mapa_sombra, q, 0);
    }
    if (k == 1u) {
        return textureLoad(mapa_sombra_1, q, 0);
    }
    return textureLoad(mapa_sombra_2, q, 0);
}

fn textureLoad_centro(k: u32, uv: vec2<f32>, dims: vec2<f32>) -> f32 {
    return prof_no_nivel(k, vec2<i32>(clamp(uv * dims, vec2<f32>(0.0), dims - vec2<f32>(1.0))));
}

fn compara_no_nivel(k: u32, uv: vec2<f32>, z: f32) -> f32 {
    if (k == 0u) {
        return textureSampleCompareLevel(mapa_sombra, compara, uv, z);
    }
    if (k == 1u) {
        return textureSampleCompareLevel(mapa_sombra_1, compara, uv, z);
    }
    return textureSampleCompareLevel(mapa_sombra_2, compara, uv, z);
}

fn visibilidade_da_caixa(p: vec3<f32>) -> f32 {
    if (quadro.sombra.z < 0.5) {
        return 1.0;
    }
    let c = quadro.sombra_vp * vec4<f32>(p, 1.0);
    let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return 1.0;
    }
    let dims0 = vec2<f32>(textureDimensions(mapa_sombra));
    let zr = c.z;
    // Alem do fundo do mapa nao ha' sombra: o fundo e' a sombra mais funda que os objetos deitam no
    // chao, e o texel vazio (1) leria-se como bloqueador.
    if (zr >= 1.0) {
        return 1.0;
    }
    let fundo = quadro.sombra.x;
    let texel = quadro.chao.w;
    let tan_p = quadro.chao.z;
    // 1) quem tapa: a profundidade media dos bloqueadores numa janela do tamanho da caixa vista daqui
    // (em texels do nivel 0; o nivel onde ela cabe).
    let busca0 = max(tan_p * zr * fundo / texel, 1.0);
    let ks = nivel_do_raio(busca0);
    let es = f32(1u << (2u * ks));
    let dims_s = dims0 / es;
    let busca = clamp(busca0 / es, 1.0, PCSS_MAX);
    let vies_s = quadro.sombra.y * es;
    // ⭐ O proprio raio do pixel conta primeiro: o bloqueador que esta' sobre ele esta' sempre dentro do
    // cone, por menor que seja (junto do contacto o cone tem milimetros e nenhuma amostra da janela
    // cai nele — medido no oraculo, 03/10: o chao saia aceso logo atras da caixa).
    let centro = textureLoad_centro(ks, uv, dims_s);
    var soma = 0.0;
    var n = 0.0;
    if (centro < zr - vies_s) {
        soma = centro;
        n = 1.0;
    }
    for (var i = 0u; i < 16u; i = i + 1u) {
        let q = clamp(uv * dims_s + DISCO[i] * busca, vec2<f32>(0.0), dims_s - vec2<f32>(1.0));
        let d = prof_no_nivel(ks, vec2<i32>(q));
        // ⛔ Medido (03/10, sol de 10 graus): o filtro «so' quem cai dentro do cone» cortava a ponta da
        // penumbra (0 onde o Cycles da' 0,40); com as faces de TRAS no mapa ele ja' nao fazia falta.
        if (d < zr - vies_s) {
            soma = soma + d;
            n = n + 1.0;
        }
    }
    if (n < 0.5) {
        return 1.0;
    }
    let zb = soma / n;
    // 2) a penumbra: a distancia ao bloqueador vezes a tangente da caixa, em texels do nivel 0, lida
    // no nivel CONTINUO onde o raio fica em [PCSS_MIN, PCSS_MAX): entre dois niveis mistura-se (sem
    // degrau onde um pixel troca de nivel).
    let raio0 = max(tan_p * (zr - zb) * fundo / texel, 1.0);
    let l = max(log2(raio0 / PCSS_MIN) * 0.5, 0.0);
    let k0 = min(u32(l), 2u);
    var vis = filtra_no_nivel(k0, uv, zr, raio0, dims0);
    let t = clamp(l - f32(k0), 0.0, 1.0);
    if (k0 < 2u && t > 0.0) {
        vis = mix(vis, filtra_no_nivel(k0 + 1u, uv, zr, raio0, dims0), t);
    }
    return vis;
}

// O PCF de 16 amostras no nivel `k`, com o raio dado em texels do nivel 0.
fn filtra_no_nivel(k: u32, uv: vec2<f32>, zr: f32, raio0: f32, dims0: vec2<f32>) -> f32 {
    let e = f32(1u << (2u * k));
    let dims = dims0 / e;
    let raio = clamp(raio0 / e, 1.0, PCSS_MAX);
    let vies = quadro.sombra.y * e;
    var vis = 0.0;
    for (var i = 0u; i < 16u; i = i + 1u) {
        vis = vis + compara_no_nivel(k, uv + DISCO[i] * raio / dims, zr - vies);
    }
    return vis / 16.0;
}
