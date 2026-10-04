// AS CAPTURAS DE REFLEXO no quadro (`gpu_sondas.rs`): a peca le a SUA captura — as vizinhas vistas do
// centro dela, num octaedro com borda, um nivel por rugosidade — com a paralaxe corrigida pela
// distancia guardada ao lado. Dentro de uma captura nenhuma peca le capturas (um so' reflexo).
var<private> sonda_camada: i32 = -1;
// O peso da parte do ceu no REFLEXO com captura: so' a oclusao propria (as vizinhas tapam pela captura).
var<private> peso_espec: f32 = 1.0;

fn sonda_uv(ab: vec2<f32>, k: u32) -> vec2<f32> {
    let w = f32(SONDA_LADO >> k);
    return (vec2<f32>(1.0) + (ab * 0.5 + vec2<f32>(0.5)) * (w - 2.0)) / w;
}

fn sonda_le(camada: i32, d: vec3<f32>, lod: f32) -> vec4<f32> {
    let l = clamp(lod, 0.0, f32(SONDA_NIVEIS - 1u));
    let k0 = min(u32(l), SONDA_NIVEIS - 2u);
    let ab = sky_oct(d);
    let a = textureSampleLevel(sondas, liso, sonda_uv(ab, k0), camada, f32(k0));
    let b = textureSampleLevel(sondas, liso, sonda_uv(ab, k0 + 1u), camada, f32(k0 + 1u));
    return mix(a, b, l - f32(k0));
}

// A direccao, vista do centro da captura, do ponto onde o raio `p + t r` encontra as vizinhas: a esfera
// do raio que a captura mediu na direccao corrente, um passo por nivel (do grosso ao fino). Em `w`, a
// razao das distancias ao ponto acertado (do pixel / do centro): o lobo visto do centro e' essa fraccao
// do lobo do pixel (Lagarde e Zanuttini 2012, os cubos com paralaxe).
fn sonda_direcao(p: vec3<f32>, r: vec3<f32>) -> vec4<f32> {
    let q = p - objeto.sonda.yzw;
    var d = r;
    var razao = 1.0;
    for (var i = 0u; i < SONDA_PASSOS; i = i + 1u) {
        let g = sonda_le(sonda_camada + 1, d, SONDA_PARALAXE[i]);
        let dist = g.r / max(g.g, 1.0e-6);
        let b = dot(q, r);
        let disc = b * b - (dot(q, q) - dist * dist);
        let t = sqrt(max(disc, 0.0)) - b;
        if (g.g >= 1.0e-3 && disc >= 0.0 && t > 0.0) {
            d = normalize(q + t * r);
            razao = t / max(dist, 1.0e-6);
        }
    }
    return vec4<f32>(d, razao);
}

// A MESMA pergunta no mesmo pixel tem a mesma resposta: a direccao da paralaxe nao depende da rugosidade
// (o lobo dieletrico, o metalico e o verniz partilham-na), a leitura sim.
var<private> memo_d: vec4<f32> = vec4<f32>(0.0);
var<private> memo_dr: vec3<f32> = vec3<f32>(0.0);
var<private> memo_dv: bool = false;
var<private> memo_s: vec4<f32> = vec4<f32>(0.0);
var<private> memo_sd: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

fn sonda_no_pixel(p: vec3<f32>, r: vec3<f32>, alpha: f32) -> vec4<f32> {
    if (memo_sd.w != alpha || any(memo_sd.xyz != r)) {
        if (!memo_dv || any(memo_dr != r)) {
            memo_dv = true;
            memo_d = sonda_direcao(p, r);
            memo_dr = r;
        }
        let lod = sqrt(clamp(alpha * memo_d.w, 0.0, 1.0)) * f32(SONDA_NIVEIS - 1u);
        memo_s = sonda_le(sonda_camada, memo_d.xyz, lod);
        memo_sd = vec4<f32>(r, alpha);
    }
    return memo_s;
}

// As faces de uma captura: a luz da cena (antes da exposicao e do olhar) com cobertura 1, e a distancia
// ao centro da captura.
struct SondaOut {
    @location(0) cor: vec4<f32>,
    @location(1) dist: vec4<f32>,
};

@fragment
fn fs_sonda(i: VsOut) -> SondaOut {
    var o: SondaOut;
    o.cor = vec4<f32>(luz_de_cena(i), 1.0);
    o.dist = vec4<f32>(length(i.mundo - quadro.olho.xyz), 1.0, 0.0, 1.0);
    return o;
}
