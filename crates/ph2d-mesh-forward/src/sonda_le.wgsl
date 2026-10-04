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
// O PRIMEIRO passo le a distancia media das vizinhas em toda a volta (o nivel mais largo nos seis eixos e
// em `r`): a esfera das vizinhas, que nenhum nivel sozinho da' a 512.
fn sonda_esfera(r: vec3<f32>) -> vec2<f32> {
    let k = f32(SONDA_NIVEIS - 1u);
    var g = sonda_le(sonda_camada + 1, r, k).rg;
    for (var e = 0u; e < 6u; e = e + 1u) {
        g = g + sonda_le(sonda_camada + 1, SONDA_FACE_W[e], k).rg;
    }
    return g;
}

// `fino`: o nivel mais fino onde ainda se refina — o do borrao do lobo (num reflexo aspero refinar abaixo
// dele so' traz os saltos do nivel fino: a aresta dura e os rastros no aspero, report do dono 04/10).
fn sonda_direcao(p: vec3<f32>, r: vec3<f32>, fino: f32) -> vec4<f32> {
    let q = p - objeto.sonda.yzw;
    var d = r;
    var razao = 1.0;
    for (var i = 0u; i <= SONDA_PASSOS; i = i + 1u) {
        var g = vec4<f32>(0.0);
        if (i == 0u) {
            g = vec4<f32>(sonda_esfera(r), 0.0, 0.0);
        } else {
            g = sonda_le(sonda_camada + 1, d, max(SONDA_PARALAXE[i - 1u], fino));
        }
        let dist = g.r / max(g.g, 1.0e-6);
        let b = dot(q, r);
        let disc = b * b - (dot(q, q) - dist * dist);
        let t = sqrt(max(disc, 0.0)) - b;
        // O passo pesa pela cobertura (sem limiar: um limiar e' uma aresta no reflexo aspero).
        if (disc >= 0.0 && t > 0.0) {
            let w = smoothstep(0.0, SONDA_COBERTURA_PLENA, g.g);
            d = normalize(mix(d, normalize(q + t * r), w));
            // A razao das distancias (o lobo visto do centro) so' da esfera das vizinhas: os passos finos
            // mudavam-na so' onde havia cobertura, e o borrao saltava de nivel (a aresta dura no aspero).
            if (i == 0u) {
                razao = mix(razao, t / max(dist, 1.0e-6), w);
            }
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
fn sonda_dist0(d: vec3<f32>) -> vec3<f32> {
    return textureSampleLevel(sondas, liso, sonda_uv(sky_oct(d), 0u), sonda_camada + 1, 0.0).rgb;
}

// A leitura mistura superficies de distancias diferentes (o desvio pelo 2.o momento passa `SONDA_ARESTA`
// da media): a distancia dela e' FANTASMA — a junta entre duas vizinhas, report do dono 04/10.
// So' se pergunta onde ha' cobertura (o lado de la' de um cruzamento).
fn sonda_aresta(g: vec3<f32>) -> bool {
    let m = g.x / max(g.y, 1.0e-6);
    let v = g.z / max(g.y, 1.0e-6) - m * m;
    return v > SONDA_ARESTA * SONDA_ARESTA * m * m;
}

// O peso do que a busca devolve: `1` num cruzamento NA superficie; perto do contorno (o raio rente,
// por tras da vizinha vista do centro) cai com a distancia a ela — a borda e' continua de pixel a
// pixel em vez de acertar/falhar ao acaso (os pontos claros e as franjas do report).
var<private> marcha_peso: f32 = 0.0;

// Numa ARESTA em `hi` (o arco `cos a qh + sin a w`, `lambda(a) = k / sen(th - a)`): quanto o raio passa atras
// da vizinha da FRENTE na silhueta dela. A da frente le-se pura adiante; a de tras sai dos dois momentos da
// leitura em `hi` (numa mistura de duas, `d_tras = media + variancia / (media - d_frente)` — a de tras pode
// nao ter leitura pura nenhuma: uma lasca); a silhueta fica onde a distancia lida e' a media das duas.
fn sonda_fora_na_aresta(qh: vec3<f32>, w: vec3<f32>, th: f32, k: f32, hi: f32) -> f32 {
    var af = hi;
    var gf = sonda_dist0(cos(af) * qh + sin(af) * w);
    let g0 = gf;
    for (var s = 1u; s <= SONDA_ARESTA_PASSOS && sonda_aresta(gf); s = s + 1u) {
        af = hi + f32(s) * SONDA_TEXEL;
        gf = sonda_dist0(cos(af) * qh + sin(af) * w);
    }
    if (sonda_aresta(gf) || gf.y < 0.5) {
        return 1.0e9;
    }
    // So' duas superficies: com ceu no texel (tres) a conta nao vale — um fiapo onde a de tras acaba.
    if (g0.y < SONDA_CHEIA) {
        return 1.0e9;
    }
    let dn = gf.x / gf.y;
    let m = g0.x / max(g0.y, 1.0e-6);
    if (m <= dn) {
        return k / sin(th - hi) - dn;
    }
    let meio = 0.5 * (dn + m + max(g0.z / max(g0.y, 1.0e-6) - m * m, 0.0) / (m - dn));
    var ab = max(hi - f32(SONDA_ARESTA_PASSOS) * SONDA_TEXEL, 0.0);
    for (var b = 0u; b < SONDA_REFINO; b = b + 1u) {
        let c = 0.5 * (ab + af);
        let g = sonda_dist0(cos(c) * qh + sin(c) * w);
        if (g.x / max(g.y, 1.0e-6) > meio) {
            ab = c;
        } else {
            af = c;
        }
    }
    return k / sin(th - 0.5 * (ab + af)) - dn;
}

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
            let gl = sonda_dist0(cos(lo) * qh + sin(lo) * w);
            let lam = ql * st / sin(th - hi);
            // Numa ARESTA a distancia lida e' fantasma (a junta): a da vizinha da FRENTE le-se pura uns texels
            // adiante (o lado de la' do cruzamento), a da de TRAS uns atras, e a silhueta da da frente fica
            // onde a mistura e' meio a meio — ali se mede quanto o raio passa atras dela, como contra o fundo
            // (a cobertura a meio). A de tras nao muda a da frente (o buraco, report do dono 04/10).
            var u = cos(hi) * qh + sin(hi) * w;
            var gh = sonda_dist0(u);
            var fora = lam - gh.x / max(gh.y, 1.0e-6);
            if (sonda_aresta(gh)) {
                fora = sonda_fora_na_aresta(qh, w, th, ql * st, hi);
            }
            // A cor le-se adiante, onde a vizinha ja' cobre o texel inteiro: na silhueta (a meia cobertura
            // do cruzamento) ela e' meio ceu, e de raspao essa meia cor fazia uma borda mole e roida de varios
            // pixels (report do dono 04/10). Para se a cobertura cair (uma vizinha fina).
            for (var s = 1u; s <= SONDA_ARESTA_PASSOS && (sonda_aresta(gh) || gh.y < SONDA_CHEIA); s = s + 1u) {
                let av = hi + f32(s) * SONDA_TEXEL;
                let g = sonda_dist0(cos(av) * qh + sin(av) * w);
                if (!sonda_aresta(g) && g.y < gh.y) {
                    break;
                }
                u = cos(av) * qh + sin(av) * w;
                gh = g;
            }
            if (!sonda_aresta(gh)) {
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
        }
        frente = !atras;
        ant = a;
    }
    return melhor;
}

// A MESMA pergunta no mesmo pixel tem a mesma resposta: a direccao da paralaxe nao depende da rugosidade
// (o lobo dieletrico, o metalico e o verniz partilham-na), a leitura sim.
var<private> memo_d: vec4<f32> = vec4<f32>(0.0);
var<private> memo_dr: vec4<f32> = vec4<f32>(0.0);
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
        // Um nivel ACIMA do borrao do lobo: no nivel dele a cobertura ainda muda em 2 texels e a direccao
        // saltava (a aresta dura no aspero).
        let fino = select(0.0, lod0 + SONDA_ACIMA, lod0 >= 1.0);
        if (!memo_dv || any(memo_dr != vec4<f32>(r, fino))) {
            memo_dv = true;
            memo_d = sonda_direcao(p, r, fino);
            memo_dr = vec4<f32>(r, fino);
        }
        // O ponto fixo: o lobo largo (e, quando a busca falha, o que sobra dele: nada no nitido, a crescer
        // com o borrao na passagem).
        let lod_f = sqrt(clamp(alpha * memo_d.w, 0.0, 1.0)) * f32(SONDA_NIVEIS - 1u);
        let fixo = sonda_le(sonda_camada, memo_d.xyz, lod_f);
        // A BUSCA ate' `SONDA_LOD_BUSCA` niveis de borrao, a passar ao ponto fixo nos ultimos
        // `SONDA_PASSAGEM` (sem degrau: uma lasca clara onde a troca era seca; e so' na passagem: o ponto fixo
        // misturado no quase nitido punha a junta translucida a 0,05 — reports do dono de 04/10).
        let passa = smoothstep(SONDA_LOD_BUSCA - SONDA_PASSAGEM, SONDA_LOD_BUSCA, lod0);
        let busca = 1.0 - passa;
        var s = fixo;
        if (busca > 0.0) {
            if (!memo_mv || any(memo_mr != r)) {
                memo_mv = true;
                memo_m = sonda_marcha(p, r);
                memo_mp = marcha_peso;
                memo_mr = r;
            }
            var m = fixo * passa;
            if (memo_m.w >= 0.0) {
                let lod = sqrt(clamp(alpha * memo_m.w, 0.0, 1.0)) * f32(SONDA_NIVEIS - 1u);
                m = mix(m, sonda_le(sonda_camada, memo_m.xyz, lod), memo_mp);
            }
            s = mix(fixo, m, busca);
        }
        memo_s = s;
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
    let d = length(i.mundo - quadro.olho.xyz);
    o.dist = vec4<f32>(d, 1.0, d * d, 1.0);
    return o;
}
