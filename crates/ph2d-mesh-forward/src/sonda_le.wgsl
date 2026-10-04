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

// ⭐⭐ A BUSCA ao longo do raio (para o reflexo NITIDO), no arco que a recta `p + t r` projecta do
// centro, `lambda(theta) = |q| sen T / sen(T - theta)` (Szirmay-Kalos et al. 2005, os impostores de
// distancia): passos iguais em angulo ate' ao primeiro ponto que fica atras do que a captura guardou;
// bisseccao; e a ESPESSURA (o raio que passa POR TRAS de uma vizinha, vista do centro, nao a acerta).
// Devolve (direccao, t / lambda) — `w < 0` = o raio nao acerta nada. O ponto fixo, num reflexo nitido,
// decidia acertar/falhar pixel a pixel na borda: os DEGRAUS do report do dono (04/10).
fn sonda_dist0(d: vec3<f32>) -> vec2<f32> {
    return textureSampleLevel(sondas, liso, sonda_uv(sky_oct(d), 0u), sonda_camada + 1, 0.0).rg;
}

// O peso do que a busca devolve: `1` num cruzamento NA superficie; perto do contorno (o raio rente,
// por tras da vizinha vista do centro) cai com a distancia a ela — a borda e' continua de pixel a
// pixel em vez de acertar/falhar ao acaso (os pontos claros e as franjas do report).
var<private> marcha_peso: f32 = 0.0;

fn sonda_marcha(p: vec3<f32>, r: vec3<f32>) -> vec4<f32> {
    marcha_peso = 0.0;
    let q = p - objeto.sonda.yzw;
    let ql = length(q);
    let qh = q / max(ql, 1.0e-6);
    let ct = clamp(dot(qh, r), -1.0, 1.0);
    let th = acos(ct);
    if (th < 1.0e-4) {
        return vec4<f32>(r, -1.0);
    }
    let w = normalize(r - ct * qh);
    let st = sin(th);
    // Os passos acompanham o arco: um a cada `SONDA_PASSO` texels do nivel 0, entre o minimo e o maximo.
    let n = clamp(u32(ceil(th / (SONDA_PASSO * SONDA_TEXEL))), SONDA_MARCHA_MIN, SONDA_MARCHA_MAX);
    var ant = 0.0;
    var frente = true;
    var melhor = vec4<f32>(r, -1.0);
    for (var k = 1u; k <= n; k = k + 1u) {
        let a = th * f32(k) / f32(n + 1u);
        let g = sonda_dist0(cos(a) * qh + sin(a) * w);
        let atras = g.y > 0.5 && ql * st / sin(th - a) >= g.x / g.y;
        if (atras && frente) {
            var lo = ant;
            var hi = a;
            for (var b = 0u; b < SONDA_REFINO; b = b + 1u) {
                let mm = 0.5 * (lo + hi);
                let gm = sonda_dist0(cos(mm) * qh + sin(mm) * w);
                if (gm.y > 0.5 && ql * st / sin(th - mm) >= gm.x / gm.y) {
                    hi = mm;
                } else {
                    lo = mm;
                }
            }
            // O cruzamento tem de ser NA superficie: coberta dos dois lados e o raio a espessura dela. No
            // contorno (vista do centro) o raio passa POR TRAS da vizinha, e a busca continua; o peso cai
            // com a distancia a ela (a borda continua de pixel a pixel).
            let u = cos(hi) * qh + sin(hi) * w;
            let gh = sonda_dist0(u);
            let gl = sonda_dist0(cos(lo) * qh + sin(lo) * w);
            let lam = ql * st / sin(th - hi);
            let fora = lam - gh.x / max(gh.y, 1.0e-6);
            if (gl.y > 0.5 && fora <= SONDA_ESPESSURA * lam) {
                marcha_peso = 1.0;
                return vec4<f32>(u, sin(hi) / st);
            }
            let peso = 1.0 - smoothstep(0.0, SONDA_FRANJA * lam, fora);
            if (peso > marcha_peso) {
                marcha_peso = peso;
                melhor = vec4<f32>(u, sin(hi) / st);
            }
        }
        frente = !atras;
        ant = a;
    }
    return melhor;
}

// A MESMA pergunta no mesmo pixel tem a mesma resposta: a direccao da paralaxe nao depende da rugosidade
// (o lobo dieletrico, o metalico e o verniz partilham-na), a leitura sim.
var<private> memo_d: vec4<f32> = vec4<f32>(0.0);
var<private> memo_dr: vec3<f32> = vec3<f32>(0.0);
var<private> memo_dv: bool = false;
var<private> memo_m: vec4<f32> = vec4<f32>(0.0);
var<private> memo_mr: vec3<f32> = vec3<f32>(0.0);
var<private> memo_mv: bool = false;
var<private> memo_mp: f32 = 0.0;
var<private> memo_s: vec4<f32> = vec4<f32>(0.0);
var<private> memo_sd: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

fn sonda_no_pixel(p: vec3<f32>, r: vec3<f32>, alpha: f32) -> vec4<f32> {
    if (memo_sd.w != alpha || any(memo_sd.xyz != r)) {
        let lod0 = sqrt(clamp(alpha, 0.0, 1.0)) * f32(SONDA_NIVEIS - 1u);
        // O reflexo NITIDO (menos de um nivel de borrao) pela busca: a borda e' a do raio.
        if (lod0 < 1.0) {
            if (!memo_mv || any(memo_mr != r)) {
                memo_mv = true;
                memo_m = sonda_marcha(p, r);
                memo_mp = marcha_peso;
                memo_mr = r;
            }
            if (memo_m.w >= 0.0) {
                let lod = sqrt(clamp(alpha * memo_m.w, 0.0, 1.0)) * f32(SONDA_NIVEIS - 1u);
                memo_s = sonda_le(sonda_camada, memo_m.xyz, lod) * memo_mp;
                memo_sd = vec4<f32>(r, alpha);
                return memo_s;
            }
        }
        if (!memo_dv || any(memo_dr != r)) {
            memo_dv = true;
            memo_d = sonda_direcao(p, r);
            memo_dr = r;
        }
        let lod = sqrt(clamp(alpha * memo_d.w, 0.0, 1.0)) * f32(SONDA_NIVEIS - 1u);
        // A busca falhou (o raio nao acerta nada): no nitido, nada; a caminho do aspero, o lobo largo
        // pelo ponto fixo, a crescer com o borrao.
        memo_s = sonda_le(sonda_camada, memo_d.xyz, lod) * clamp(lod0, 0.0, 1.0);
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
