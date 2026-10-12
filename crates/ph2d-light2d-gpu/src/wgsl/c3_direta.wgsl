// C3 — a luz DIRETA das lâmpadas (`Luz`), numa grade FINA própria (smoke do dono, 2026-10-09/10:
// «a sombra não fica reta», «o tamanho afeta a qualidade», e a ESCADA da sombra nítida no nível
// baixo). A visibilidade do DISCO é medida por raios até ele, pela MESMA marcha da C2 (`marchar`: a
// mesma lei da entrada e dos opacos). Fluência de um disco uniforme no plano: F = L · ∫ T(θ) dθ
// sobre a abertura dele (2·asin(r/d)); de um PONTO (r = 0): F = I·queda/r.
//
// Dois passes: `c3_visibilidade` mede, em cada sonda da c0, a fração à vista de cada lâmpada; a
// `c3_fina` escreve cada texel fino = a c0 (as cascatas) interpolada + Σ a parte SUAVE da lâmpada
// (analítica no próprio texel) × a visibilidade — interpolada das sondas onde ela é suave, medida no
// texel onde não é (as bordas e os joelhos da penumbra). `c3_fina_exata` mede-a em todo texel (a régua do refino, e `fina = 1`).
//
// Concatenado DEPOIS de `c2_rc.wgsl` (usa `cas`, `marchar`, `livre`, `queda`, `TAU`). Tudo em texels
// das CASCATAS (onde vivem o campo e as sondas); as lâmpadas de fora da grade também iluminam. A
// marcha lê a captura ligada no grupo 0: a FINA dos opacos (W5a), na escala dela.
//
// ⭐ W5a (F3) — o PREENCHIMENTO: um texel fino DENTRO de um opaco calcula-se no texel exterior mais
// próximo (o JFA do complemento, `c3_preenche.wgsl`), e a c0 interpolada deixa de fora as sondas de
// dentro de um opaco (o alfa 0 da c0): a face acesa não mistura zeros e o interior recebe a luz da
// face — a mesma regra do gabarito.

struct Lampada {
    centro: vec2<f32>,
    raio: f32,
    inv_alcance: f32,
    // rgb: a radiância do disco (r > 0) ou a intensidade do ponto (r = 0).
    radiancia: vec4<f32>,
    // ⭐ W5b (F4): os objetos FORA da grade no caminho até ela — `[primeiro, quantos]` em `indices`.
    lista: vec4<u32>,
}

struct Lampadas {
    n: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
    itens: array<Lampada>,
}

// A grade fina e a REGIÃO deste quadro (a da janela): só ela se calcula, em texels finos e nas
// sondas da c0 que a cobrem.
struct Fina {
    tamanho: vec2<u32>,
    inicio: vec2<u32>,
    fim: vec2<u32>,
    inicio_grosso: vec2<u32>,
    fim_grosso: vec2<u32>,
    // Texels finos por texel das cascatas.
    fina: u32,
    limiar: f32,
}

@group(1) @binding(0) var<storage, read> lampadas: Lampadas;

// ⭐ W5b (F4) — o mundo FORA da grade (`distantes.rs`): por lâmpada, os opacos do caminho até ela que
// a captura não tem inteiros, cada um na sua moldura; testados só no trecho do raio fora da grade.
struct Distante {
    // O paralelogramo que o envolve (texels das cascatas): o, a | b, e o canto da janela na atlas.
    quad_o_a: vec4<f32>,
    quad_b_janela: vec4<f32>,
    // Texel das cascatas → moldura [0, 1]² da máscara (`w`: o lado da janela · τ dentro).
    s_: vec4<f32>,
    t_: vec4<f32>,
    tipo: u32,
    primeira: u32,
    n: u32,
    _p: u32,
}
const MASCARA: u32 = 0u;
// O passo da marcha numa máscara, em texels dela (a captura marcha a meio texel junto à borda).
const PASSO_DA_MASCARA: f32 = 0.5;

@group(1) @binding(1) var<storage, read> distantes: array<Distante>;
// a.xy, b.xy: o interior à ESQUERDA de a → b.
@group(1) @binding(2) var<storage, read> arestas: array<vec4<f32>>;
@group(1) @binding(3) var<storage, read> indices: array<u32>;
@group(1) @binding(4) var mascaras: texture_2d<f32>;
@group(2) @binding(0) var<uniform> fina: Fina;
@group(2) @binding(1) var c0_fluencia: texture_2d<f32>;
@group(2) @binding(2) var visibilidades: texture_2d<u32>;
@group(2) @binding(3) var visibilidade_saida: texture_storage_2d<rg32uint, write>;
// ⭐ W5a (F3): por célula do preenchimento, o texel fino exterior mais próximo (`x | y << 16`).
@group(2) @binding(5) var preenchido: texture_2d<u32>;

override PREENCHE: bool = false;
// Texels finos por célula do preenchimento.
override CELULA_PREENCHE: u32 = 1u;
// O preenchimento correu na grade inteira (senão só nas células da região).
override PREENCHE_A_GRADE_INTEIRA: bool = true;
// ⭐ O texel que o eixo medial corta guarda a média das faces (senão o ponto de fronteira mais perto).
override MEDIA_NO_EIXO: bool = true;
// ⭐ W5d — o ponto de fronteira que um texel do INTERIOR lê toma a c0 pela média AO LONGO da face,
// nesta meia-largura (texels das cascatas; 0 = no ponto): ver [`c0_ao_longo_da_face`].
override C0_AO_LONGO_DA_FACE: f32 = 0.0;
// ⭐ W5d — a MEDIANA (por canal) dos pontos ao longo da face, não a média: ver [`c0_ao_longo_da_face`].
override C0_AO_LONGO_MEDIANA: bool = false;
const SEM_SEMENTE: u32 = 0xffffffffu;
// ⭐ W5h: o alfa da fluência marca o texel do INTERIOR (0) para o `c3_miolo.wgsl`.
override MIOLO: bool = false;

// A abertura do disco na penumbra: `RAIOS_DIRETOS + 1` raios FIXOS nas fronteiras de
// `RAIOS_DIRETOS` fatias e, numa fatia em que a transmissão muda, `BISSECOES` passos de bissecção
// atrás do ângulo da borda — determinístico (sem sorteio: o «pontilhado» do smoke do dono), contínuo
// e reto (a borda é achada a 1/(RAIOS·2^BISSECOES) da abertura).
override RAIOS_DIRETOS: u32 = 8u;
override BISSECOES: u32 = 5u;
// ⭐ W5b (F4): a luz direta vê o mundo FORA da grade (`fora_da_grade`).
override DISTANTES: bool = true;
// O refino também nos JOELHOS da penumbra.
override JOELHO: bool = true;
// ⭐ W5a (F2): e onde a visibilidade das sondas CURVA (o vale de um oclusor fino).
override CURVATURA: bool = true;
// Quanto antes do disco a marcha para (a lâmpada direta não está na captura; meio texel de folga
// contra o que estiver colado a ela).
const ANTES_DO_DISCO: f32 = 0.5;
// Um texel fino interpola as QUATRO sondas da célula dele: está a menos de √2 texels de cada uma.
const UMA_CELULA: f32 = 1.4142136;
// O cone livre «sem folga» (pedir uma folga de uma célula não mudou nada: o `livre` já é
// conservador em ~2 texels — medido na W4b, o fio de sombra entre as sondas).
const SEM_FOLGA: f32 = 1e-6;
// O código de uma sonda: 0..=254 = a fração à vista × 254; `CERTA` = à vista com o cone LIVRE (sem
// nada perto do caminho: os texels finos da célula interpolam-na sem medir).
const INTEIRA: u32 = 254u;
const CERTA: u32 = 255u;

// ⭐ Onde o segmento A + u·(B − A) sai da grade (A dentro dela): u ≥ 1 = não sai.
fn saida_da_grade(A: vec2<f32>, B: vec2<f32>) -> f32 {
    let n = vec2<f32>(cas.tamanho);
    let d = B - A;
    var u = 1.0;
    for (var e = 0; e < 2; e++) {
        if d[e] > 0.0 {
            u = min(u, (n[e] - A[e]) / d[e]);
        } else if d[e] < 0.0 {
            u = min(u, -A[e] / d[e]);
        }
    }
    return max(u, 0.0);
}

// A transmissão do segmento E → B por um POLÍGONO: × τ a cada ENTRADA (o interior à esquerda da
// aresta: entra onde o raio cruza contra a normal de fora).
fn pelo_poligono(o: Distante, E: vec2<f32>, B: vec2<f32>) -> f32 {
    let d = B - E;
    var t = 1.0;
    for (var i = 0u; i < o.n; i++) {
        let ar = arestas[o.primeira + i];
        let e = ar.zw - ar.xy;
        let den = d.x * e.y - d.y * e.x;
        // Entra ⟺ o raio vai para a esquerda da aresta: cross(e, d) > 0 ⟺ den < 0.
        if den >= 0.0 {
            continue;
        }
        let w = ar.xy - E;
        let u = (w.x * e.y - w.y * e.x) / den;
        let v = (w.x * d.y - w.y * d.x) / den;
        if u > 0.0 && u <= 1.0 && v >= 0.0 && v < 1.0 {
            t *= o.t_.w;
            if t <= 0.0 {
                return 0.0;
            }
        }
    }
    return t;
}

// A cobertura da máscara em `st` (a moldura [0, 1]²): bilinear entre os centros, presa à janela.
fn cobertura_da_mascara(o: Distante, st: vec2<f32>) -> f32 {
    let lado = o.s_.w;
    let p = st * lado - 0.5;
    let b = floor(p);
    let f = p - b;
    let ultimo = vec2<i32>(i32(lado) - 1);
    let q = clamp(vec2<i32>(b), vec2<i32>(0), ultimo) + vec2<i32>(o.quad_b_janela.zw);
    let r = clamp(vec2<i32>(b) + 1, vec2<i32>(0), ultimo) + vec2<i32>(o.quad_b_janela.zw);
    let c00 = textureLoad(mascaras, q, 0).r;
    let c10 = textureLoad(mascaras, vec2<i32>(r.x, q.y), 0).r;
    let c01 = textureLoad(mascaras, vec2<i32>(q.x, r.y), 0).r;
    let c11 = textureLoad(mascaras, r, 0).r;
    return mix(mix(c00, c10, f.x), mix(c01, c11, f.x), f.y);
}

// A transmissão do segmento E → B por uma MÁSCARA: a lei da captura (dentro ⟺ cobertura ≥ ½, × τ a
// cada entrada) na moldura do objeto. Partindo de dentro (E na borda da grade), a entrada já contou.
fn pela_mascara(o: Distante, E: vec2<f32>, B: vec2<f32>) -> f32 {
    let a = vec2<f32>(dot(o.s_.xy, E) + o.s_.z, dot(o.t_.xy, E) + o.t_.z);
    let b = vec2<f32>(dot(o.s_.xy, B) + o.s_.z, dot(o.t_.xy, B) + o.t_.z);
    let d = b - a;
    var lo = 0.0;
    var hi = 1.0;
    for (var e = 0; e < 2; e++) {
        if abs(d[e]) < 1e-12 {
            if a[e] < 0.0 || a[e] > 1.0 {
                return 1.0;
            }
        } else {
            let u0 = (0.0 - a[e]) / d[e];
            let u1 = (1.0 - a[e]) / d[e];
            lo = max(lo, min(u0, u1));
            hi = min(hi, max(u0, u1));
        }
    }
    if lo >= hi {
        return 1.0;
    }
    let n = max(u32(ceil(length(d) * o.s_.w * (hi - lo) / PASSO_DA_MASCARA)), 1u);
    var t = 1.0;
    var dentro_ant = lo <= 0.0 && cobertura_da_mascara(o, a) >= 0.5;
    for (var j = 0u; j <= n; j++) {
        let dentro = cobertura_da_mascara(o, a + d * mix(lo, hi, f32(j) / f32(n))) >= 0.5;
        if dentro && !dentro_ant {
            t *= o.t_.w;
            if t <= 0.0 {
                return 0.0;
            }
        }
        dentro_ant = dentro;
    }
    return t;
}

// ⭐ A transmissão do trecho de P → B FORA da grade, pelos objetos distantes da lâmpada `k`.
fn fora_da_grade(P: vec2<f32>, B: vec2<f32>, k: u32) -> f32 {
    let lista = lampadas.itens[k].lista;
    if lista.y == 0u {
        return 1.0;
    }
    let u = saida_da_grade(P, B);
    if u >= 1.0 {
        return 1.0;
    }
    let E = mix(P, B, u);
    var t = 1.0;
    for (var i = 0u; i < lista.y; i++) {
        let o = distantes[indices[lista.x + i]];
        if o.tipo == MASCARA {
            t *= pela_mascara(o, E, B);
        } else {
            t *= pelo_poligono(o, E, B);
        }
        if t <= 0.0 {
            return 0.0;
        }
    }
    return t;
}

// ⭐ O cone de P (eixo `dir`, até `fim`, meia-largura `s·tg + folga`) não toca nenhum objeto
// distante da lâmpada `k`? Conservador (o trapézio do cone contra o paralelogramo de cada um, pelos
// eixos separadores): tocar só obriga a medir.
fn cone_livre_fora(P: vec2<f32>, dir: vec2<f32>, fim: f32, tg: f32, folga: f32, k: u32) -> bool {
    let lista = lampadas.itens[k].lista;
    if lista.y == 0u {
        return true;
    }
    let lado = vec2<f32>(-dir.y, dir.x);
    let largo = fim * tg + folga;
    var cone = array<vec2<f32>, 4>(
        P - lado * folga,
        P + dir * fim - lado * largo,
        P + dir * fim + lado * largo,
        P + lado * folga,
    );
    for (var i = 0u; i < lista.y; i++) {
        let o = distantes[indices[lista.x + i]];
        let q0 = o.quad_o_a.xy;
        let a = o.quad_o_a.zw;
        let b = o.quad_b_janela.xy;
        var quad = array<vec2<f32>, 4>(q0, q0 + a, q0 + a + b, q0 + b);
        var eixos = array<vec2<f32>, 5>(
            vec2<f32>(-a.y, a.x),
            vec2<f32>(-b.y, b.x),
            dir,
            vec2<f32>(-(cone[1] - cone[0]).y, (cone[1] - cone[0]).x),
            vec2<f32>(-(cone[3] - cone[2]).y, (cone[3] - cone[2]).x),
        );
        var separa = false;
        for (var e = 0u; e < 5u; e++) {
            let n = eixos[e];
            var c_lo = 3.4e38;
            var c_hi = -3.4e38;
            var q_lo = 3.4e38;
            var q_hi = -3.4e38;
            for (var j = 0u; j < 4u; j++) {
                let x = dot(n, cone[j]);
                c_lo = min(c_lo, x);
                c_hi = max(c_hi, x);
                let y = dot(n, quad[j]);
                q_lo = min(q_lo, y);
                q_hi = max(q_hi, y);
            }
            if c_hi < q_lo || q_hi < c_lo {
                separa = true;
                break;
            }
        }
        if !separa {
            return false;
        }
    }
    return true;
}

// A transmissão de P até B (o meio entre eles; o emissor de chegada fica de fora), e o trecho fora
// da grade pelos objetos distantes da lâmpada `k`.
fn transmissao(P: vec2<f32>, B: vec2<f32>, k: u32) -> f32 {
    let t = f32(marchar(P, P, B).t);
    if t <= 0.0 || !DISTANTES {
        return t;
    }
    return t * fora_da_grade(P, B, k);
}

// A transmissão de P até o DISCO pelo ângulo `a` (a marcha para antes dele).
fn ate_o_disco(P: vec2<f32>, v: vec2<f32>, d: f32, raio: f32, a: f32, k: u32) -> f32 {
    let dj = vec2<f32>(cos(a), sin(a));
    let b = dot(dj, v);
    let t = b - sqrt(max(b * b - (d * d - raio * raio), 0.0));
    return transmissao(P, P + dj * max(t - ANTES_DO_DISCO, 0.0), k);
}

// A fração VISÍVEL da abertura [base − α, base + α]: raios fixos nas fronteiras das fatias e, onde
// a transmissão muda, a bissecção da borda.
fn visivel_do_disco(P: vec2<f32>, v: vec2<f32>, d: f32, raio: f32, base: f32, alfa: f32, k: u32) -> f32 {
    let n = RAIOS_DIRETOS;
    let largura = 2.0 * alfa / f32(n);
    var a0 = base - alfa;
    var t0 = ate_o_disco(P, v, d, raio, a0, k);
    var soma = 0.0;
    for (var i = 0u; i < n; i++) {
        let a1 = a0 + largura;
        let t1 = ate_o_disco(P, v, d, raio, a1, k);
        if abs(t1 - t0) < 1e-4 {
            soma += 0.5 * (t0 + t1) * largura;
        } else {
            var lo = a0;
            var hi = a1;
            for (var b = 0u; b < BISSECOES; b++) {
                let m = 0.5 * (lo + hi);
                let tm = ate_o_disco(P, v, d, raio, m, k);
                if abs(tm - t0) <= abs(tm - t1) {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let borda = 0.5 * (lo + hi);
            soma += t0 * (borda - a0) + t1 * (a1 - borda);
        }
        a0 = a1;
        t0 = t1;
    }
    return soma / (2.0 * alfa);
}

// O cone de P ao disco está LIVRE? — nenhum ponto «dentro» a menos da meia-largura do cone mais
// `folga` até `fim` (o caminho barato: fora da penumbra a visibilidade é exatamente 1). As outras
// lâmpadas não contam: uma lâmpada é luz, não corpo (smoke do dono, 10/10).
fn cone_livre(P: vec2<f32>, dir: vec2<f32>, fim: f32, tg: f32, folga: f32) -> bool {
    var s = 0.0;
    loop {
        if s >= fim {
            break;
        }
        let f = livre(P + dir * s) - s * tg - folga;
        if f <= 1.0 {
            return false;
        }
        s += f;
    }
    return true;
}

// A luz da lâmpada em P SEM sombra (o disco inteiro à vista): a parte SUAVE, analítica.
fn sem_sombra(P: vec2<f32>, L: Lampada) -> vec3<f32> {
    let d = distance(L.centro, P);
    if L.raio <= 0.0 {
        if d <= 0.0 {
            return vec3<f32>(0.0);
        }
        let r = d * cas.texel_m;
        return L.radiancia.rgb * (queda(r, L.inv_alcance) / r);
    }
    if d <= L.raio {
        // Dentro do disco: ele ocupa a volta inteira.
        return TAU * L.radiancia.rgb * queda(0.0, L.inv_alcance);
    }
    return L.radiancia.rgb * (2.0 * asin(min(L.raio / d, 1.0)) * queda(d * cas.texel_m, L.inv_alcance));
}

// A fração À VISTA da lâmpada `k` de P (o ponto: a transmissão do raio até ele). `folga` > 0 pede o
// cone livre com essa folga; sem ele devolve -1 (o passe grosso: a sonda à vista sem obstáculo perto).
fn visibilidade(P: vec2<f32>, k: u32, folga: f32) -> f32 {
    let L = lampadas.itens[k];
    let v = L.centro - P;
    let d = length(v);
    if L.raio <= 0.0 {
        if folga > 0.0 {
            let dir = v / max(d, 1e-6);
            let fim = max(d - ANTES_DO_DISCO, 0.0);
            let livre = cone_livre(P, dir, fim, 0.0, folga)
                && (!DISTANTES || cone_livre_fora(P, dir, fim, 0.0, folga, k));
            return select(-1.0, 1.0, livre);
        }
        return transmissao(P, L.centro, k);
    }
    if d <= L.raio {
        return select(1.0, -1.0, folga > 0.0);
    }
    let alfa = asin(min(L.raio / d, 1.0));
    let dir = v / d;
    let fim = max(d - L.raio - ANTES_DO_DISCO, 0.0);
    if cone_livre(P, dir, fim, tan(alfa), folga)
        && (!DISTANTES || cone_livre_fora(P, dir, fim, tan(alfa), folga, k)) {
        return 1.0;
    }
    if folga > 0.0 {
        return -1.0;
    }
    return visivel_do_disco(P, v, d, L.raio, atan2(dir.y, dir.x), alfa, k);
}

// A lâmpada `k` alcança algum ponto a menos de `folga` de P?
fn alcanca(P: vec2<f32>, L: Lampada, folga: f32) -> bool {
    let d = max(distance(L.centro, P) - L.raio - folga, 0.0);
    return queda(d * cas.texel_m, L.inv_alcance) > 0.0;
}

// ⭐ W5k: o τ do meio do ponto onde a `c0_em` se pede — 1 FORA dos corpos (o exterior; a omissão), o
// do translúcido na transmitida de dentro dele (só à volta dela).
var<private> c0_tau: f32 = 1.0;

// ⭐ W5k — o peso de uma sonda vista de FORA: o τ dela se cai num translúcido (a 0,999 ≈ o opaco que o
// alfa da c0 já tira; a 0,001 ≈ nenhum corpo), 1 no resto (o opaco é do alfa: o bit de antes). Só PESO,
// nunca `v / τ`: a sonda dentro de um corpo que reflete apanha a própria emissão sem o τ, e dividi-la
// fazia o rebote divergir (a sala a 0,99: 10⁴, medido).
fn peso_de_fora(t: vec2<i32>) -> f32 {
    if c0_tau < 1.0 {
        return 1.0;
    }
    let tau = meio(vec2<f32>(t) + 0.5).tau;
    return select(1.0, tau, tau > 0.0 && tau < 1.0);
}

// A c0 (as cascatas) bilinear no ponto P das cascatas — a conta do `receber.wgsl`, bordas presas.
// Com o preenchimento, só as sondas de FORA dos opacos (o alfa da c0), renormalizadas.
fn c0_em(P: vec2<f32>) -> vec3<f32> {
    let u = P - 0.5;
    let base = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(base);
    let ultimo = vec2<i32>(cas.tamanho) - 1;
    let q = clamp(base, vec2<i32>(0), ultimo);
    let r = clamp(base + 1, vec2<i32>(0), ultimo);
    let c00 = textureLoad(c0_fluencia, q, 0);
    let c10 = textureLoad(c0_fluencia, vec2<i32>(r.x, q.y), 0);
    let c01 = textureLoad(c0_fluencia, vec2<i32>(q.x, r.y), 0);
    let c11 = textureLoad(c0_fluencia, r, 0);
    if PREENCHE {
        let w = vec4<f32>((1.0 - fr.x) * (1.0 - fr.y), fr.x * (1.0 - fr.y), (1.0 - fr.x) * fr.y, fr.x * fr.y)
            * vec4<f32>(c00.a, c10.a, c01.a, c11.a)
            * vec4<f32>(peso_de_fora(q), peso_de_fora(vec2<i32>(r.x, q.y)), peso_de_fora(vec2<i32>(q.x, r.y)), peso_de_fora(r));
        let soma = w.x + w.y + w.z + w.w;
        if soma > 0.0 {
            return (c00.rgb * w.x + c10.rgb * w.y + c01.rgb * w.z + c11.rgb * w.w) / soma;
        }
        // ⭐ Nenhuma das quatro é de fora: P está fora pela captura fina, mas a borda dela e a das
        // sondas diferem por centésimos de célula — as de fora estão no anel seguinte. Voltar às de
        // DENTRO lia o interior (o rebote da face inteira 6× na oitava grossa, 2.º smoke do dono).
        var fora = vec3<f32>(0.0);
        var peso = 0.0;
        for (var dy = -1; dy <= 2; dy++) {
            for (var dx = -1; dx <= 2; dx++) {
                let t = clamp(base + vec2<i32>(dx, dy), vec2<i32>(0), ultimo);
                let c = textureLoad(c0_fluencia, t, 0);
                let o = abs(vec2<f32>(base + vec2<i32>(dx, dy)) - u);
                let wt = max(1.0 - 0.5 * o.x, 0.0) * max(1.0 - 0.5 * o.y, 0.0) * c.a * peso_de_fora(t);
                fora += c.rgb * wt;
                peso += wt;
            }
        }
        if peso > 0.0 {
            return fora / peso;
        }
    }
    let a = mix(c00.rgb, c10.rgb, fr.x);
    let b = mix(c01.rgb, c11.rgb, fr.x);
    return mix(a, b, fr.y);
}

// ⭐ W5d — a c0 no ponto de fronteira `P` que o texel `P0` (dentro) lê: a média de até cinco pontos ao
// longo da face (a tangente ⟂ `P − P0`), a [`C0_AO_LONGO_DA_FACE`] de cada lado — só os que continuam
// sobre ELA (fora do corpo, com ele um texel atrás: a ponta de uma face não lê o outro lado do canto).
// As sondas COLADAS a uma superfície que reflete variam ±15 % de uma para a outra (o brilho dela visto
// por poucos raios curtos: a fração que a acerta salta com a posição da sonda contra a curva) e a lei do
// interior projetava esse ruído até ao centro do corpo (as fatias do disco com o rebote, cena 5). Ao
// longo da face, e não em volta: as sondas mais afastadas veem menos o próprio corpo (a tenda de raio 2
// tirava 4–9 % da face).
fn c0_ao_longo_da_face(P: vec2<f32>, P0: vec2<f32>) -> vec3<f32> {
    let d = P - P0;
    let m = length(d);
    if C0_AO_LONGO_DA_FACE <= 0.0 || m <= 1e-6 {
        return c0_em(P);
    }
    let n = d / m;
    let t = vec2<f32>(-n.y, n.x) * (0.5 * C0_AO_LONGO_DA_FACE);
    var v: array<vec3<f32>, 5>;
    v[0] = c0_em(P);
    var k = 1u;
    for (var i = -2; i <= 2; i++) {
        let q = P + t * f32(i);
        if i == 0 || num_corpo(q + 0.25 * n) || !num_corpo(q - n) {
            continue;
        }
        v[k] = c0_em(q);
        k++;
    }
    if !C0_AO_LONGO_MEDIANA {
        var s = vec3<f32>(0.0);
        for (var i = 0u; i < k; i++) {
            s += v[i];
        }
        return s / f32(k);
    }
    // A mediana por canal (ordenação por inserção de ≤ 5): tira o ruído de sonda para sonda sem
    // arrastar um DEGRAU ao longo da face — junto a um canto, a cascata já vaza a luz do outro lado por
    // ~2 texels, e a média espalhava-o mais 2.
    var r = vec3<f32>(0.0);
    for (var c = 0; c < 3; c++) {
        var o: array<f32, 5>;
        for (var i = 0u; i < k; i++) {
            let x = v[i][c];
            var j = i;
            loop {
                if j == 0u || o[j - 1u] <= x {
                    break;
                }
                o[j] = o[j - 1u];
                j--;
            }
            o[j] = x;
        }
        r[c] = select(o[k / 2u], 0.5 * (o[k / 2u - 1u] + o[k / 2u]), k % 2u == 0u);
    }
    return r;
}

// ⭐ O ponto da FRONTEIRA entre `dentro` (num opaco) e `fora` (um texel exterior), do lado de fora —
// por bissecção do `meio`: a lei do gabarito é o exterior mais próximo, não o centro de um texel.
fn na_fronteira(dentro: vec2<f32>, fora: vec2<f32>) -> vec2<f32> {
    var a = dentro;
    var b = fora;
    for (var i = 0u; i < 8u; i++) {
        let m = 0.5 * (a + b);
        if num_corpo(m) {
            a = m;
        } else {
            b = m;
        }
    }
    return b;
}

struct Onde {
    // O ponto (texels das cascatas) onde o texel fino se calcula.
    p: vec2<f32>,
    // Dentro de um opaco, calcula-se no exterior mais próximo.
    remapeado: bool,
    // ⭐ O ponto da OUTRA face e a fração da área do texel do lado dela (o texel que o eixo medial
    // corta guarda a média da lei sobre a área dele, não o valor do centro); 0 longe do eixo.
    p2: vec2<f32>,
    f2: f32,
}

// Duas faces são outras quando as direções aos pontos de fronteira se afastam mais que isto.
const COS_OUTRA_FACE: f32 = 0.5;
const SEM_FACE: u32 = 0xffffffffu;

// ⭐ A fração da área de um quadrado de lado `h` (centrado) além da reta `g·x = t` (`g` unitária,
// `t ≥ 0`): a projeção do quadrado em `g` é um trapézio.
fn area_alem(g: vec2<f32>, t: f32, h: f32) -> f32 {
    let p = max(0.5 * h * min(abs(g.x), abs(g.y)), 1e-6);
    let q = 0.5 * h * max(abs(g.x), abs(g.y));
    if t >= p + q {
        return 0.0;
    }
    if t >= q - p {
        return (p + q - t) * (p + q - t) / (8.0 * p * q);
    }
    return (q - t) / (2.0 * q);
}

// A fração do texel além do eixo entre a face mais perto `a` e a face `b` (`(normal, distância)` de
// cada uma): o eixo é onde as distâncias se igualam, e o gradiente da diferença é `na − nb`.
fn alem_do_eixo(a: vec3<f32>, b: vec3<f32>, h: f32) -> f32 {
    let g = a.xy - b.xy;
    let m = length(g);
    if m <= 1e-4 {
        return 0.0;
    }
    return area_alem(g / m, max(b.z - a.z, 0.0) / m, h);
}

// ⭐ Onde o texel fino `id` se calcula: no centro dele, ou — dentro de um opaco, com o preenchimento —
// no ponto da fronteira mais próximo. Os candidatos: os pontos de fronteira a partir dos texels
// exteriores de uma janela de 5×5 (no canto o eixo medial chega à borda e o texel exterior mais perto
// pode estar na face errada) e — mais fundo, ou se a janela não alcança uma segunda face — das
// sementes do JFA da célula e das 8 vizinhas. Junto ao eixo medial, também o ponto da outra face mais
// perto e a área do texel além do eixo, pelas distâncias PERPENDICULARES às duas faces.
fn onde_calcular(id: vec2<u32>) -> Onde {
    borda_aqui = true;
    let P0 = ponto_fino(id);
    if !PREENCHE || !num_corpo(P0) {
        return Onde(P0, false, P0, 0.0);
    }
    let h = 1.0 / f32(fina.fina);
    var cand: array<vec2<f32>, 34>;
    var n_cand = 0u;
    let n = vec2<i32>(cas.tamanho * fina.fina);
    for (var dy = -2; dy <= 2; dy++) {
        for (var dx = -2; dx <= 2; dx++) {
            let t = vec2<i32>(id) + vec2<i32>(dx, dy);
            if any(t < vec2<i32>(0)) || any(t >= n) {
                continue;
            }
            let Pt = ponto_fino(vec2<u32>(t));
            if num_corpo(Pt) {
                continue;
            }
            cand[n_cand] = na_fronteira(P0, Pt);
            n_cand++;
        }
    }
    var i1 = mais_perto(P0, &cand, n_cand);
    let na_janela = n_cand;
    if n_cand == 0u || (MEDIA_NO_EIXO && outra_face(P0, &cand, n_cand, i1, h) == SEM_FACE) {
        var u1 = vec2<f32>(0.0);
        var d1 = 0.0;
        if na_janela > 0u {
            d1 = distance(cand[i1], P0);
            u1 = (cand[i1] - P0) / max(d1, 1e-6);
        }
        let c = vec2<i32>(id / CELULA_PREENCHE);
        var lo = vec2<i32>(0);
        var hi = vec2<i32>(textureDimensions(preenchido));
        if !PREENCHE_A_GRADE_INTEIRA {
            lo = vec2<i32>(fina.inicio / CELULA_PREENCHE);
            hi = vec2<i32>((fina.fim + CELULA_PREENCHE - 1u) / CELULA_PREENCHE);
        }
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let q = c + vec2<i32>(dx, dy);
                if any(q < lo) || any(q >= hi) {
                    continue;
                }
                let s = textureLoad(preenchido, q, 0).x;
                if s == SEM_SEMENTE {
                    continue;
                }
                let ps = ponto_fino(vec2<u32>(s & 0xffffu, s >> 16u));
                // Junto à borda, só a semente de OUTRA face ao alcance (a janela já tem a desta).
                if na_janela > 0u {
                    let ds = distance(ps, P0);
                    if ds >= d1 + 3.0 * h || dot((ps - P0) / max(ds, 1e-6), u1) >= COS_OUTRA_FACE {
                        continue;
                    }
                }
                cand[n_cand] = na_fronteira(P0, ps);
                n_cand++;
            }
        }
        i1 = mais_perto(P0, &cand, n_cand);
    }
    if n_cand == 0u {
        return Onde(P0, false, P0, 0.0);
    }
    cand[i1] = pe_na_borda(P0, cand[i1]);
    if !MEDIA_NO_EIXO {
        return Onde(cand[i1], true, cand[i1], 0.0);
    }
    // Longe do eixo (nenhuma outra face a dois texels) o texel é o de sempre. Junto a ele a fração vem
    // da diferença das distâncias, e o ponto de fronteira cruza a face na OBLÍQUA (a semente é um
    // texel): as distâncias pela PERPENDICULAR.
    let i2 = outra_face(P0, &cand, n_cand, i1, h);
    if i2 == SEM_FACE {
        return Onde(cand[i1], true, cand[i1], 0.0);
    }
    cand[i2] = pe_na_borda(P0, cand[i2]);
    let e1 = pe_na_face(P0, cand[i1], h);
    let e2 = pe_na_face(P0, cand[i2], h);
    // Um ponto da MESMA face visto de lado também se afasta em direção: conta a normal.
    if e1.z < 0.0 || e2.z < 0.0 || dot(e1.xy, e2.xy) >= COS_OUTRA_FACE {
        return Onde(cand[i1], true, cand[i1], 0.0);
    }
    if e2.z < e1.z {
        return Onde(cand[i2], true, cand[i1], alem_do_eixo(e2, e1, h));
    }
    return Onde(cand[i1], true, cand[i2], alem_do_eixo(e1, e2, h));
}

// O candidato mais perto de `P0` (`SEM_FACE` se não há nenhum).
fn mais_perto(P0: vec2<f32>, cand: ptr<function, array<vec2<f32>, 34>>, n_cand: u32) -> u32 {
    var melhor = SEM_FACE;
    var d_melhor = 3.4e38;
    for (var i = 0u; i < n_cand; i++) {
        let d = distance((*cand)[i], P0);
        if d < d_melhor {
            d_melhor = d;
            melhor = i;
        }
    }
    return melhor;
}

// O candidato mais perto numa face OUTRA que a de `a` (a direção afasta-se), a menos de dois texels
// além do `a` — `SEM_FACE` se não há.
fn outra_face(P0: vec2<f32>, cand: ptr<function, array<vec2<f32>, 34>>, n_cand: u32, a: u32, h: f32) -> u32 {
    if a == SEM_FACE {
        return SEM_FACE;
    }
    let da = distance((*cand)[a], P0);
    let ua = ((*cand)[a] - P0) / max(da, 1e-6);
    var melhor = SEM_FACE;
    var d_melhor = da + 2.0 * h;
    for (var i = 0u; i < n_cand; i++) {
        let d = distance((*cand)[i], P0);
        let u = ((*cand)[i] - P0) / max(d, 1e-6);
        if d < d_melhor && dot(u, ua) < COS_OUTRA_FACE {
            d_melhor = d;
            melhor = i;
        }
    }
    return melhor;
}

// ⭐ A normal de fora da face no ponto de fronteira `b` (achado de `P0`, dentro) e a distância de `P0`
// à face pela perpendicular: a tangente pelos cruzamentos de duas retas paralelas a `P0 → b`, de cada
// lado a meia distância (no máximo um texel — junto a um canto, mais longe cruzaria a outra face).
// `(normal, distância)`; sem os dois cruzamentos, distância NEGATIVA (a face não se mede).
fn pe_na_face(P0: vec2<f32>, b: vec2<f32>, h: f32) -> vec3<f32> {
    let d = max(distance(b, P0), 1e-6);
    let u = (b - P0) / d;
    let w = vec2<f32>(-u.y, u.x) * min(h, 0.5 * d);
    let fim = b + u * h;
    let a0 = P0 + w;
    let a1 = P0 - w;
    if !num_corpo(a0) || !num_corpo(a1) || num_corpo(fim + w) || num_corpo(fim - w) {
        return vec3<f32>(u, -1.0);
    }
    let c0 = na_fronteira_fina(a0, fim + w, h);
    let c1 = na_fronteira_fina(a1, fim - w, h);
    let t = c0 - c1;
    var n = normalize(vec2<f32>(t.y, -t.x));
    if dot(n, u) < 0.0 {
        n = -n;
    }
    return vec3<f32>(n, dot(0.5 * (c0 + c1) - P0, n));
}

// [`na_fronteira`] até um centésimo de texel `h` (o meio do último intervalo): os passos contam-se
// pelo comprimento do segmento.
fn na_fronteira_fina(dentro: vec2<f32>, fora: vec2<f32>, h: f32) -> vec2<f32> {
    var a = dentro;
    var b = fora;
    let passos = u32(ceil(log2(max(distance(a, b) / (0.01 * h), 2.0))));
    for (var i = 0u; i < passos; i++) {
        let m = 0.5 * (a + b);
        if num_corpo(m) {
            a = m;
        } else {
            b = m;
        }
    }
    return 0.5 * (a + b);
}

// A visibilidade da lâmpada `k` na sonda `q` da c0 (presa à grade; `CERTA` = 1).
fn vis_sonda(q: vec2<i32>, k: u32) -> f32 {
    let w = textureLoad(visibilidades, clamp(q, vec2<i32>(0), vec2<i32>(cas.tamanho) - 1), 0).xy;
    let c = (select(w.y, w.x, k < 4u) >> (8u * (k % 4u))) & 0xffu;
    return f32(min(c, INTEIRA)) / f32(INTEIRA);
}

// ⭐ A CURVATURA da visibilidade em volta da célula `q..r` (cantos `f`): a maior 2.ª diferença das
// sondas, por eixo, em cada canto. Um oclusor FINO deixa um vale mais estreito que a célula: os
// cantos ficam iguais (a amplitude não o vê) e a interpolação apagava o fundo (F2, W5a).
fn curvatura(q: vec2<i32>, r: vec2<i32>, f: vec4<f32>, k: u32) -> f32 {
    let x = vec4<f32>(
        vis_sonda(vec2<i32>(q.x - 1, q.y), k),
        vis_sonda(vec2<i32>(r.x + 1, q.y), k),
        vis_sonda(vec2<i32>(q.x - 1, r.y), k),
        vis_sonda(vec2<i32>(r.x + 1, r.y), k),
    );
    let y = vec4<f32>(
        vis_sonda(vec2<i32>(q.x, q.y - 1), k),
        vis_sonda(vec2<i32>(r.x, q.y - 1), k),
        vis_sonda(vec2<i32>(q.x, r.y + 1), k),
        vis_sonda(vec2<i32>(r.x, r.y + 1), k),
    );
    let dx = abs(vec4<f32>(x.x - 2.0 * f.x + f.y, f.x - 2.0 * f.y + x.y, x.z - 2.0 * f.z + f.w, f.z - 2.0 * f.w + x.w));
    let dy = abs(vec4<f32>(y.x - 2.0 * f.x + f.z, y.y - 2.0 * f.y + f.w, f.x - 2.0 * f.z + y.z, f.y - 2.0 * f.w + y.w));
    let m = max(dx, dy);
    return max(max(m.x, m.y), max(m.z, m.w));
}

// O texel fino `id` no espaço das cascatas.
fn ponto_fino(id: vec2<u32>) -> vec2<f32> {
    return (vec2<f32>(id) + 0.5) / f32(fina.fina);
}

@compute @workgroup_size(8, 8)
fn c3_visibilidade(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.xy + fina.inicio_grosso;
    if any(id >= fina.fim_grosso) {
        return;
    }
    let Q = vec2<f32>(id) + 0.5;
    borda_aqui = false;
    var w = vec2<u32>(0u);
    for (var k = 0u; k < lampadas.n; k++) {
        var codigo = CERTA;
        if alcanca(Q, lampadas.itens[k], UMA_CELULA) && visibilidade(Q, k, SEM_FOLGA) < 0.0 {
            codigo = u32(round(clamp(visibilidade(Q, k, 0.0), 0.0, 1.0) * f32(INTEIRA)));
        }
        let b = codigo << (8u * (k % 4u));
        w = select(vec2<u32>(w.x, w.y | b), vec2<u32>(w.x | b, w.y), k < 4u);
    }
    textureStore(visibilidade_saida, id, vec4<u32>(w, 0u, 0u));
}

@compute @workgroup_size(8, 8)
fn c3_fina(@builtin(global_invocation_id) gid: vec3<u32>) {
    let alvo = gid.xy + fina.inicio;
    if any(alvo >= fina.fim) {
        return;
    }
    borda_aqui = true;
    let tau = meio(ponto_fino(alvo)).tau;
    let onde = onde_calcular(alvo);
    var l = luz_fina(alvo, onde);
    var t = vec3<f32>(0.0);
    if onde.remapeado && tau > 0.0 {
        c0_tau = tau;
        t = luz_fina(alvo, Onde(ponto_fino(alvo), false, ponto_fino(alvo), 0.0));
        c0_tau = 1.0;
    }
    guardar_fina(alvo, onde, l, t, tau);
}

// ⭐ W5k — a fluência do texel e, dentro de um corpo de opacidade `α = 1 − τ < 1`, a TRANSMITIDA `t`
// (a luz que o atravessa até ele): `mix(t, l, α)` aqui, ou — com o miolo, que troca o `l` do interior
// pela difusão — na textura que o `miolo_final` mistura (`α = 1` no opaco: o bit de antes).
fn guardar_fina(alvo: vec2<u32>, onde: Onde, l: vec3<f32>, t: vec3<f32>, tau: f32) {
    let translucido = onde.remapeado && tau > 0.0;
    var saida = l;
    if MIOLO {
        if onde.remapeado {
            textureStore(transmitida_saida, alvo, vec4<f32>(t, select(1.0, 1.0 - tau, translucido)));
        }
    } else if translucido {
        saida = mix(t, l, 1.0 - tau);
    }
    textureStore(fina_saida, alvo, vec4<f32>(saida, alfa_do_miolo(onde)));
}

// A luz do texel fino `alvo` calculada em `onde`, com as duas faces.
fn luz_fina(alvo: vec2<u32>, onde: Onde) -> vec3<f32> {
    // O texel remapeado (dentro de um opaco) MEDE a visibilidade: as sondas em volta do exterior mais
    // próximo podem estar fora da região.
    let remapeado = onde.remapeado;
    borda_aqui = remapeado;
    let P = onde.p;
    let u = P - 0.5;
    let base = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(base);
    let ultimo = vec2<i32>(cas.tamanho) - 1;
    let q = clamp(base, vec2<i32>(0), ultimo);
    let r = clamp(base + 1, vec2<i32>(0), ultimo);
    // (q.x, q.y) · (r.x, q.y) · (q.x, r.y) · (r.x, r.y)
    let w0 = textureLoad(visibilidades, q, 0).xy;
    let w1 = textureLoad(visibilidades, vec2<i32>(r.x, q.y), 0).xy;
    let w2 = textureLoad(visibilidades, vec2<i32>(q.x, r.y), 0).xy;
    let w3 = textureLoad(visibilidades, r, 0).xy;
    var soma: vec3<f32>;
    if remapeado {
        soma = c0_ao_longo_da_face(P, ponto_fino(alvo));
    } else {
        soma = c0_em(P);
    }
    for (var k = 0u; k < lampadas.n; k++) {
        let L = lampadas.itens[k];
        let s = sem_sombra(P, L);
        if all(s <= vec3<f32>(0.0)) {
            continue;
        }
        let desl = 8u * (k % 4u);
        let lado = k < 4u;
        let c = (vec4<u32>(
            select(w0.y, w0.x, lado),
            select(w1.y, w1.x, lado),
            select(w2.y, w2.x, lado),
            select(w3.y, w3.x, lado),
        ) >> vec4<u32>(desl)) & vec4<u32>(0xffu);
        var v = 1.0;
        if remapeado {
            v = visibilidade_remapeada(P, ponto_fino(alvo), k);
        } else if any(c != vec4<u32>(CERTA)) {
            let f = vec4<f32>(min(c, vec4<u32>(INTEIRA))) / f32(INTEIRA);
            let lo = min(min(f.x, f.y), min(f.z, f.w));
            let hi = max(max(f.x, f.y), max(f.z, f.w));
            // O JOELHO da penumbra (onde ela começa ou acaba) não é linear: uma célula que mistura
            // um canto na sombra ou à vista inteira com um canto parcial mede-se no texel.
            let extremo = (f <= vec4<f32>(0.0)) | (f >= vec4<f32>(1.0));
            if hi - lo > fina.limiar || lo >= 1.0 || (JOELHO && any(extremo) && !all(extremo))
                || (CURVATURA && curvatura(q, r, f, k) > fina.limiar) {
                v = visibilidade(P, k, 0.0);
            } else {
                v = mix(mix(f.x, f.y, fr.x), mix(f.z, f.w, fr.x), fr.y);
            }
        }
        soma += s * v;
    }
    return nas_faces(soma, onde);
}

fn alfa_do_miolo(onde: Onde) -> f32 {
    return select(1.0, 0.0, MIOLO && onde.remapeado);
}

// A luz do texel: `l` (a da face mais perto) misturada com a da outra face na área dela.
fn nas_faces(l: vec3<f32>, onde: Onde) -> vec3<f32> {
    if onde.f2 <= 0.0 {
        return l;
    }
    return mix(l, luz_exata(onde.p2), onde.f2);
}

// A luz em P com a visibilidade de cada lâmpada medida no ponto.
fn luz_exata(P: vec2<f32>) -> vec3<f32> {
    var soma = c0_em(P);
    for (var k = 0u; k < lampadas.n; k++) {
        let s = sem_sombra(P, lampadas.itens[k]);
        if any(s > vec3<f32>(0.0)) {
            soma += s * visibilidade(P, k, 0.0);
        }
    }
    return soma;
}

@compute @workgroup_size(8, 8)
fn c3_fina_exata(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.xy + fina.inicio;
    if any(id >= fina.fim) {
        return;
    }
    borda_aqui = true;
    let tau = meio(ponto_fino(id)).tau;
    let onde = onde_calcular(id);
    borda_aqui = onde.remapeado;
    var l = luz_exata(onde.p);
    if onde.remapeado {
        l = luz_exata_remapeada(onde.p, ponto_fino(id));
        l += c0_ao_longo_da_face(onde.p, ponto_fino(id)) - c0_em(onde.p);
    }
    var t = vec3<f32>(0.0);
    if onde.remapeado && tau > 0.0 {
        borda_aqui = false;
        c0_tau = tau;
        t = luz_exata(ponto_fino(id));
        c0_tau = 1.0;
    }
    guardar_fina(id, onde, nas_faces(l, onde), t, tau);
}
