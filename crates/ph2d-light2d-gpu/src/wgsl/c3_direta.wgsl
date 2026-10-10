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
// das CASCATAS (onde vivem a captura e o campo); as lâmpadas de fora da grade também iluminam.

struct Lampada {
    centro: vec2<f32>,
    raio: f32,
    inv_alcance: f32,
    // rgb: a radiância do disco (r > 0) ou a intensidade do ponto (r = 0).
    radiancia: vec4<f32>,
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
@group(2) @binding(0) var<uniform> fina: Fina;
@group(2) @binding(1) var c0_fluencia: texture_2d<f32>;
@group(2) @binding(2) var visibilidades: texture_2d<u32>;
@group(2) @binding(3) var visibilidade_saida: texture_storage_2d<rg32uint, write>;

// A abertura do disco na penumbra: `RAIOS_DIRETOS + 1` raios FIXOS nas fronteiras de
// `RAIOS_DIRETOS` fatias e, numa fatia em que a transmissão muda, `BISSECOES` passos de bissecção
// atrás do ângulo da borda — determinístico (sem sorteio: o «pontilhado» do smoke do dono), contínuo
// e reto (a borda é achada a 1/(RAIOS·2^BISSECOES) da abertura).
override RAIOS_DIRETOS: u32 = 8u;
override BISSECOES: u32 = 5u;
// O refino também nos JOELHOS da penumbra.
override JOELHO: bool = true;
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

// Onde o raio A + d·t (t ∈ [0, len]) entra no PRIMEIRO disco de lâmpada direta; `len + 1` se em
// nenhum. Partir de DENTRO de um disco é t = 0 (a lei da entrada, como nos opacos da captura).
fn primeira_lampada(A: vec2<f32>, d: vec2<f32>, len: f32) -> f32 {
    var melhor = len + 1.0;
    for (var k = 0u; k < lampadas.n; k++) {
        let L = lampadas.itens[k];
        if L.raio <= 0.0 {
            continue;
        }
        let o = A - L.centro;
        let c = dot(o, o) - L.raio * L.raio;
        if c <= 0.0 {
            return 0.0;
        }
        let b = dot(o, d);
        let disc = b * b - c;
        if b < 0.0 && disc >= 0.0 {
            let t = -b - sqrt(disc);
            if t <= len {
                melhor = min(melhor, t);
            }
        }
    }
    return melhor;
}

// A transmissão de P até B (o meio entre eles; o emissor de chegada fica de fora).
fn transmissao(P: vec2<f32>, B: vec2<f32>) -> f32 {
    return f32(marchar(P, P, B).t);
}

// A transmissão de P até o DISCO pelo ângulo `a` (a marcha para antes dele).
fn ate_o_disco(P: vec2<f32>, v: vec2<f32>, d: f32, raio: f32, a: f32) -> f32 {
    let dj = vec2<f32>(cos(a), sin(a));
    let b = dot(dj, v);
    let t = b - sqrt(max(b * b - (d * d - raio * raio), 0.0));
    return transmissao(P, P + dj * max(t - ANTES_DO_DISCO, 0.0));
}

// A fração VISÍVEL da abertura [base − α, base + α]: raios fixos nas fronteiras das fatias e, onde
// a transmissão muda, a bissecção da borda.
fn visivel_do_disco(P: vec2<f32>, v: vec2<f32>, d: f32, raio: f32, base: f32, alfa: f32) -> f32 {
    let n = RAIOS_DIRETOS;
    let largura = 2.0 * alfa / f32(n);
    var a0 = base - alfa;
    var t0 = ate_o_disco(P, v, d, raio, a0);
    var soma = 0.0;
    for (var i = 0u; i < n; i++) {
        let a1 = a0 + largura;
        let t1 = ate_o_disco(P, v, d, raio, a1);
        if abs(t1 - t0) < 1e-4 {
            soma += 0.5 * (t0 + t1) * largura;
        } else {
            var lo = a0;
            var hi = a1;
            for (var k = 0u; k < BISSECOES; k++) {
                let m = 0.5 * (lo + hi);
                let tm = ate_o_disco(P, v, d, raio, m);
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
// `folga` até `fim` (o caminho barato: fora da penumbra a visibilidade é exatamente 1).
fn cone_livre(P: vec2<f32>, dir: vec2<f32>, fim: f32, tg: f32, alvo: u32, folga: f32) -> bool {
    // As OUTRAS lâmpadas diretas (fora da captura): o disco delas toca o cone?
    for (var k = 0u; k < lampadas.n; k++) {
        let L = lampadas.itens[k];
        if k == alvo || L.raio <= 0.0 {
            continue;
        }
        let s_k = clamp(dot(L.centro - P, dir), 0.0, fim);
        if distance(P + dir * s_k, L.centro) < L.raio + s_k * tg + 1.0 + folga {
            return false;
        }
    }
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
            return select(-1.0, 1.0, cone_livre(P, v / max(d, 1e-6), max(d - ANTES_DO_DISCO, 0.0), 0.0, k, folga));
        }
        return transmissao(P, L.centro);
    }
    if d <= L.raio {
        return select(1.0, -1.0, folga > 0.0);
    }
    let alfa = asin(min(L.raio / d, 1.0));
    let dir = v / d;
    let fim = max(d - L.raio - ANTES_DO_DISCO, 0.0);
    if cone_livre(P, dir, fim, tan(alfa), k, folga) {
        return 1.0;
    }
    if folga > 0.0 {
        return -1.0;
    }
    return visivel_do_disco(P, v, d, L.raio, atan2(dir.y, dir.x), alfa);
}

// A lâmpada `k` alcança algum ponto a menos de `folga` de P?
fn alcanca(P: vec2<f32>, L: Lampada, folga: f32) -> bool {
    let d = max(distance(L.centro, P) - L.raio - folga, 0.0);
    return queda(d * cas.texel_m, L.inv_alcance) > 0.0;
}

// A c0 (as cascatas) bilinear no ponto P das cascatas — a conta do `receber.wgsl`, bordas presas.
fn c0_em(P: vec2<f32>) -> vec3<f32> {
    let u = P - 0.5;
    let base = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(base);
    let ultimo = vec2<i32>(cas.tamanho) - 1;
    let q = clamp(base, vec2<i32>(0), ultimo);
    let r = clamp(base + 1, vec2<i32>(0), ultimo);
    let a = mix(textureLoad(c0_fluencia, q, 0).rgb, textureLoad(c0_fluencia, vec2<i32>(r.x, q.y), 0).rgb, fr.x);
    let b = mix(textureLoad(c0_fluencia, vec2<i32>(q.x, r.y), 0).rgb, textureLoad(c0_fluencia, r, 0).rgb, fr.x);
    return mix(a, b, fr.y);
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
    let id = gid.xy + fina.inicio;
    if any(id >= fina.fim) {
        return;
    }
    let P = ponto_fino(id);
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
    var soma = c0_em(P);
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
        if any(c != vec4<u32>(CERTA)) {
            let f = vec4<f32>(min(c, vec4<u32>(INTEIRA))) / f32(INTEIRA);
            let lo = min(min(f.x, f.y), min(f.z, f.w));
            let hi = max(max(f.x, f.y), max(f.z, f.w));
            // O JOELHO da penumbra (onde ela começa ou acaba) não é linear: uma célula que mistura
            // um canto na sombra ou à vista inteira com um canto parcial mede-se no texel.
            let extremo = (f <= vec4<f32>(0.0)) | (f >= vec4<f32>(1.0));
            if hi - lo > fina.limiar || lo >= 1.0 || (JOELHO && any(extremo) && !all(extremo)) {
                v = visibilidade(P, k, 0.0);
            } else {
                v = mix(mix(f.x, f.y, fr.x), mix(f.z, f.w, fr.x), fr.y);
            }
        }
        soma += s * v;
    }
    textureStore(fina_saida, id, vec4<f32>(soma, 1.0));
}

@compute @workgroup_size(8, 8)
fn c3_fina_exata(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.xy + fina.inicio;
    if any(id >= fina.fim) {
        return;
    }
    let P = ponto_fino(id);
    var soma = c0_em(P);
    for (var k = 0u; k < lampadas.n; k++) {
        let s = sem_sombra(P, lampadas.itens[k]);
        if any(s > vec3<f32>(0.0)) {
            soma += s * visibilidade(P, k, 0.0);
        }
    }
    textureStore(fina_saida, id, vec4<f32>(soma, 1.0));
}
