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
// ⭐ W5a (F3): por célula do preenchimento, o texel fino exterior mais próximo (`x | y << 16`).
@group(2) @binding(5) var preenchido: texture_2d<u32>;

override PREENCHE: bool = false;
// Texels finos por célula do preenchimento.
override CELULA_PREENCHE: u32 = 1u;
// O preenchimento correu na grade inteira (senão só nas células da região).
override PREENCHE_A_GRADE_INTEIRA: bool = true;
const SEM_SEMENTE: u32 = 0xffffffffu;

// A abertura do disco na penumbra: `RAIOS_DIRETOS + 1` raios FIXOS nas fronteiras de
// `RAIOS_DIRETOS` fatias e, numa fatia em que a transmissão muda, `BISSECOES` passos de bissecção
// atrás do ângulo da borda — determinístico (sem sorteio: o «pontilhado» do smoke do dono), contínuo
// e reto (a borda é achada a 1/(RAIOS·2^BISSECOES) da abertura).
override RAIOS_DIRETOS: u32 = 8u;
override BISSECOES: u32 = 5u;
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
            return select(-1.0, 1.0, cone_livre(P, v / max(d, 1e-6), max(d - ANTES_DO_DISCO, 0.0), 0.0, folga));
        }
        return transmissao(P, L.centro);
    }
    if d <= L.raio {
        return select(1.0, -1.0, folga > 0.0);
    }
    let alfa = asin(min(L.raio / d, 1.0));
    let dir = v / d;
    let fim = max(d - L.raio - ANTES_DO_DISCO, 0.0);
    if cone_livre(P, dir, fim, tan(alfa), folga) {
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
            * vec4<f32>(c00.a, c10.a, c01.a, c11.a);
        let soma = w.x + w.y + w.z + w.w;
        if soma > 0.0 {
            return (c00.rgb * w.x + c10.rgb * w.y + c01.rgb * w.z + c11.rgb * w.w) / soma;
        }
    }
    let a = mix(c00.rgb, c10.rgb, fr.x);
    let b = mix(c01.rgb, c11.rgb, fr.x);
    return mix(a, b, fr.y);
}

// ⭐ O ponto da FRONTEIRA entre `dentro` (num opaco) e `fora` (um texel exterior), do lado de fora —
// por bissecção do `meio`: a lei do gabarito é o exterior mais próximo, não o centro de um texel.
fn na_fronteira(dentro: vec2<f32>, fora: vec2<f32>) -> vec2<f32> {
    var a = dentro;
    var b = fora;
    for (var i = 0u; i < 8u; i++) {
        let m = 0.5 * (a + b);
        if meio(m).tau <= 0.0 {
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
}

// ⭐ Onde o texel fino `id` se calcula: no centro dele, ou — dentro de um opaco, com o preenchimento —
// no ponto da fronteira mais próximo: perto da borda, entre os texels exteriores de uma janela de 5×5
// o de FRONTEIRA mais perto (no canto o eixo medial chega à borda e o texel exterior mais perto pode
// estar na face errada); mais fundo, a semente do JFA mais perto entre a da célula e as 8 vizinhas.
fn onde_calcular(id: vec2<u32>) -> Onde {
    let P0 = ponto_fino(id);
    if !PREENCHE || meio(P0).tau > 0.0 {
        return Onde(P0, false);
    }
    let n = vec2<i32>(cas.tamanho * fina.fina);
    var melhor = P0;
    var d_melhor = 3.4e38;
    for (var dy = -2; dy <= 2; dy++) {
        for (var dx = -2; dx <= 2; dx++) {
            let t = vec2<i32>(id) + vec2<i32>(dx, dy);
            if any(t < vec2<i32>(0)) || any(t >= n) {
                continue;
            }
            let Pt = ponto_fino(vec2<u32>(t));
            if meio(Pt).tau <= 0.0 {
                continue;
            }
            let b = na_fronteira(P0, Pt);
            let d = distance(b, P0);
            if d < d_melhor {
                d_melhor = d;
                melhor = b;
            }
        }
    }
    if d_melhor < 3.4e38 {
        return Onde(melhor, true);
    }
    let c = vec2<i32>(id / CELULA_PREENCHE);
    var lo = vec2<i32>(0);
    var hi = vec2<i32>(textureDimensions(preenchido));
    if !PREENCHE_A_GRADE_INTEIRA {
        lo = vec2<i32>(fina.inicio / CELULA_PREENCHE);
        hi = vec2<i32>((fina.fim + CELULA_PREENCHE - 1u) / CELULA_PREENCHE);
    }
    // Entre as sementes da célula e das 8 vizinhas, a de ponto de FRONTEIRA mais perto (a mesma regra
    // da janela: a semente de uma célula é UM texel exterior dela, e o desempate entre texels
    // equidistantes não pode decidir a face).
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
            let b = na_fronteira(P0, ponto_fino(vec2<u32>(s & 0xffffu, s >> 16u)));
            let d = distance(b, P0);
            if d < d_melhor {
                d_melhor = d;
                melhor = b;
            }
        }
    }
    if d_melhor < 3.4e38 {
        return Onde(melhor, true);
    }
    return Onde(P0, false);
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
    // O texel remapeado (dentro de um opaco) MEDE a visibilidade: as sondas em volta do exterior mais
    // próximo podem estar fora da região.
    let onde = onde_calcular(alvo);
    let remapeado = onde.remapeado;
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
        if remapeado {
            v = visibilidade(P, k, 0.0);
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
    textureStore(fina_saida, alvo, vec4<f32>(soma, 1.0));
}

@compute @workgroup_size(8, 8)
fn c3_fina_exata(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.xy + fina.inicio;
    if any(id >= fina.fim) {
        return;
    }
    let P = onde_calcular(id).p;
    var soma = c0_em(P);
    for (var k = 0u; k < lampadas.n; k++) {
        let s = sem_sombra(P, lampadas.itens[k]);
        if any(s > vec3<f32>(0.0)) {
            soma += s * visibilidade(P, k, 0.0);
        }
    }
    textureStore(fina_saida, id, vec4<f32>(soma, 1.0));
}
