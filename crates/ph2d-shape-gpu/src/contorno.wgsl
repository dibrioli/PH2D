
// ⭐⭐ doc 121 §9.5 — **O CONTORNO DE CADA CÓPIA, CALCULADO UMA VEZ.**
//
// Medido no proxy de telemóvel com o arranjo da `=127` densa (a sonda `PH2D_SONDA_DENSO=2`): de
// `6,67 ms` de passe, `5,76` eram o traço do eixo, e dentro dele `4,5` eram a GEOMETRIA de cada
// peça refeita em CADA pixel (as bissectrizes, a junta, o quadrilátero — tudo no ecrã). Aqui ela é
// feita uma vez por cópia: um fio por cópia percorre as peças do eixo dela e escreve as ARESTAS do
// contorno no ecrã, e o desenho só soma arestas prontas (`traco_do_contorno`).
//
// ⭐ **E as arestas que a FAIXA cancela não chegam a existir:** dois troços que acabam na MESMA
// bissectriz têm a aresta partilhada nos dois sentidos, e a soma por pixel pagava as duas para
// chegar a zero. Aqui nenhuma das duas é escrita (as duas metades decidem com os MESMOS argumentos
// — `bissectriz` — logo saltam juntas).
//
// ⚠️ **A soma é a MESMA, não outra lei:** cada aresta é a de uma peça de `peca_do_eixo`, com o
// sentido que o `orienta` lhe dava; tirar um par que se anula não muda o total, só o arredondamento.
//
// Três passes (`ShapePass::draw`): `cs_conta` (quantas arestas cada cópia escreve), `cs_soma` (o
// prefixo — onde cada uma começa) e `cs_escreve`. A contagem e a escrita correm o MESMO código
// (`percorre`), e uma cópia cuja escrita não coube na capacidade ou não bateu na contagem fica com
// `(0, 0)` e é desenhada pelo caminho de sempre, pixel a pixel — nunca um contorno errado.

struct Contas {
    // Quantas cópias.
    n: u32,
    // Quantas arestas cabem em `contorno_rw`.
    cap: u32,
    // `1` ⇒ nenhuma cópia ganha contorno (o caminho de antes, para os gates o compararem).
    sem_contorno: u32,
    // Quantas palavras cabem em `cmascaras_rw`.
    cap_mascaras: u32,
}

@group(2) @binding(0) var<uniform> contas: Contas;
// Duas contagens por cópia, e depois do `cs_soma` onde cada uma começa: as ARESTAS (múltiplo de
// `SEGS_POR_BLOCO`) nas `n + 1` primeiras entradas e as PALAVRAS DE MÁSCARA nas `n + 1` seguintes —
// a última de cada metade é o total.
@group(2) @binding(1) var<storage, read_write> contagem: array<u32>;
@group(2) @binding(2) var<storage, read_write> contorno_rw: array<vec4<f32>>;
@group(2) @binding(3) var<storage, read_write> cblocos_rw: array<vec4<f32>>;
@group(2) @binding(4) var<storage, read_write> ccopias_rw: array<vec4<u32>>;
@group(2) @binding(5) var<storage, read_write> ccaixas_rw: array<vec4<f32>>;
@group(2) @binding(6) var<storage, read_write> cmascaras_rw: array<u32>;

// O estado da emissão de UMA cópia (um fio por cópia).
//
// ⭐ **DUAS correntes, para um bloco à esquerda custar DOIS números e não oito arestas.** Os lados
// longos dos troços de um eixo encadeiam-se: o de cima de um troço acaba onde o do seguinte começa
// (a mesma bissectriz), e o de baixo também — mas no sentido CONTRÁRIO. ⇒ as arestas que andam com
// o eixo escrevem-se para a FRENTE a partir do início da cópia (`cursor`), e as que andam contra ele
// para TRÁS a partir do fim (`cursor_b`) — lidas por ordem de memória, as duas são correntes. Um
// bloco cujas oito arestas se tocam ponta a ponta (conferido AO BIT em `cs_escreve`) e que fica todo
// à esquerda do pixel soma `clamp(y₀) − clamp(y₈)`: a faixa telescopa.
var<private> cursor: u32;
var<private> cursor_b: u32;
var<private> escrever: bool;
var<private> base_saida: u32;
var<private> limite_saida: u32;
var<private> cmin: vec2<f32>;
var<private> cmax: vec2<f32>;
// O último ponto da corrente da frente e o primeiro da de trás (o enchimento põe-se entre as duas).
var<private> ultimo: vec2<f32>;
var<private> cabeca_b: vec2<f32>;

fn emite_em(p0: vec2<f32>, p1: vec2<f32>, atras: bool) {
    if escrever && cursor + cursor_b < limite_saida {
        if atras {
            contorno_rw[base_saida + limite_saida - 1u - cursor_b] = vec4<f32>(p0, p1);
            cabeca_b = p0;
        } else {
            contorno_rw[base_saida + cursor] = vec4<f32>(p0, p1);
            ultimo = p1;
        }
        cmin = min(cmin, min(p0, p1));
        cmax = max(cmax, max(p0, p1));
    }
    if atras {
        cursor_b += 1u;
    } else {
        cursor += 1u;
    }
}

fn emite(p0: vec2<f32>, p1: vec2<f32>) {
    emite_em(p0, p1, false);
}

// Uma aresta com o sentido da peça: o `orienta` da soma por pixel inverte a peça inteira quando
// ela gira ao contrário, e inverter uma aresta inverte o sinal da contribuição dela.
fn aresta(p0: vec2<f32>, p1: vec2<f32>, positivo: bool) {
    aresta_em(p0, p1, positivo, false);
}

fn aresta_em(p0: vec2<f32>, p1: vec2<f32>, positivo: bool, atras: bool) {
    if positivo {
        emite_em(p0, p1, atras);
    } else {
        emite_em(p1, p0, atras);
    }
}

// O sinal que o `orienta(v, a, b, c)` daria.
fn positivo(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> bool {
    return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0;
}

fn emite_tri(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) {
    let s = positivo(a, b, c);
    aresta(a, b, s);
    aresta(b, c, s);
    aresta(c, a, s);
}

fn emite_quad(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, d: vec2<f32>) {
    let s = positivo(a, b, c);
    aresta(a, b, s);
    aresta(b, c, s);
    aresta(c, d, s);
    aresta(d, a, s);
}

// O `leque` da soma por pixel, como arestas: cada triângulo `(centro, p, w)` com o sentido dele, e
// os raios interiores partilhados por dois triângulos do MESMO sentido não se escrevem (anulam-se).
fn emite_leque(centro: vec2<f32>, n0: vec2<f32>, n_fim: vec2<f32>, cos_alpha: f32, dir: f32, r: f32) {
    if r <= 0.25 {
        emite_tri(centro, centro + n0, centro + n_fim);
        return;
    }
    let q = 1.0 - 0.25 / r;
    if cos_alpha >= 2.0 * q * q - 1.0 {
        emite_tri(centro, centro + n0, centro + n_fim);
        return;
    }
    let alpha = acos(clamp(cos_alpha, -1.0, 1.0));
    let passo_max = 2.0 * acos(q);
    let k = u32(clamp(ceil(alpha / max(passo_max, 1.0e-4)), 1.0, 64.0));
    let ang = dir * alpha / f32(k);
    let c = cos(ang);
    let sn = sin(ang);
    var v = n0;
    var p = centro + n0;
    var s_ant = false;
    for (var j = 0u; j < k; j += 1u) {
        var w = centro + n_fim;
        if j + 1u < k {
            v = vec2<f32>(c * v.x - sn * v.y, sn * v.x + c * v.y);
            w = centro + v;
        }
        let s = positivo(centro, p, w);
        if j == 0u {
            aresta(centro, p, s);
        } else if s != s_ant {
            aresta(p, centro, s_ant);
            aresta(centro, p, s);
        }
        aresta(p, w, s);
        s_ant = s;
        p = w;
    }
    aresta(p, centro, s_ant);
}

// O `junta_em` da soma por pixel, como arestas.
fn emite_junta(u: vec2<f32>, b: vec2<f32>, c: vec2<f32>, r: f32, junta: u32, limite: f32) {
    let dbc = c - b;
    let lbc = length(dbc);
    if lbc <= 0.0 {
        return;
    }
    let v = dbc / lbc;
    let cr = u.x * v.y - u.y * v.x;
    let dt = clamp(dot(u, v), -1.0, 1.0);
    if abs(cr) < 1.0e-7 && dt > 0.0 {
        return;
    }
    let lado = select(1.0, -1.0, cr > 0.0);
    let n0 = perp(u) * r * lado;
    let n1 = perp(v) * r * lado;
    if junta == 0u && 2.0 <= (1.0 + dt) * limite * limite {
        let m = b + (n0 + n1) / (1.0 + dt);
        emite_quad(b, b + n0, m, b + n1);
        return;
    }
    if junta == 2u {
        var dir = sign(n0.x * n1.y - n0.y * n1.x);
        if dir == 0.0 {
            dir = sign(n0.x * u.y - n0.y * u.x);
        }
        emite_leque(b, n0, n1, dt, dir, r);
        return;
    }
    emite_tri(b, b + n0, b + n1);
}

// O `peca_do_eixo` da soma por pixel, como arestas — sem a caixa (ela só saltava pixels).
fn emite_peca(it: Eixo, lin: vec4<f32>, t: vec2<f32>, caneta: f32) {
    let r = it.meia * caneta;
    let a = aplica(lin, t, it.a);
    let b = aplica(lin, t, it.b);
    let dab = b - a;
    let lab = length(dab);
    if lab <= 0.0 || r <= 0.0 {
        return;
    }
    let u = dab / lab;
    let nr = perp(u) * r;
    if it.tipo == 0u {
        var m0 = nr;
        var m1 = nr;
        var faixa0 = false;
        var faixa1 = false;
        if (it.ponta & 1u) != 0u {
            let e = bissectriz(aplica(lin, t, it.d), a, b, r, (it.ponta & 4u) != 0u, it.junta, it.limite);
            if e.z > 0.0 {
                m0 = e.xy;
                faixa0 = true;
            }
        }
        if (it.ponta & 2u) != 0u {
            let quina = (it.ponta & 8u) != 0u;
            let cf = aplica(lin, t, it.c);
            let e = bissectriz(a, b, cf, r, quina, it.junta, it.limite);
            if e.z > 0.0 {
                m1 = e.xy;
                faixa1 = true;
            } else {
                emite_junta(u, b, cf, r, select(2u, it.junta, quina), it.limite);
            }
        }
        // O quadrilátero `a+m0 → b+m1 → b−m1 → a−m0`, sem as arestas de ponta que a faixa partilha
        // com o vizinho (o vizinho também não escreve a dele: as duas anulavam-se).
        //
        // ⚠️ A ORDEM é a das correntes: a ponta de trás chega ao início do lado que anda com o eixo,
        // ele, e a ponta da frente sai do fim dele — as três encadeiam na frente. O lado que anda
        // CONTRA o eixo vai para trás. Com `s` negativo os papéis trocam (`aresta` inverte cada uma).
        let q0 = a + m0;
        let q1 = b + m1;
        let q2 = b - m1;
        let q3 = a - m0;
        let s = positivo(q0, q1, q2);
        if !faixa0 {
            aresta(q3, q0, s);
        }
        aresta_em(q0, q1, s, !s);
        aresta_em(q2, q3, s, s);
        if !faixa1 {
            aresta(q1, q2, s);
        }
        return;
    }
    if it.tipo == 2u {
        if it.ponta == 1u {
            emite_quad(b + nr, b + nr + u * r, b - nr + u * r, b - nr);
        } else if it.ponta == 2u {
            let dir = sign(nr.x * u.y - nr.y * u.x);
            emite_leque(b, nr, -nr, -1.0, dir, r);
        }
    }
}

// As peças do eixo de uma cópia (os cabeçalhos de bloco não desenham nada).
fn percorre(cp: Copia) {
    cursor = 0u;
    cursor_b = 0u;
    let caneta = bitcast<f32>(cp.eixo_rg.w);
    for (var i = cp.eixo_rg.x; i < cp.eixo_rg.x + cp.eixo_rg.y; i += 1u) {
        let it = eixo[i];
        if it.tipo == 3u {
            continue;
        }
        emite_peca(it, cp.lin, cp.t, caneta);
    }
}

// ⭐ doc 121 §9.6 — **O QUE UMA CÓPIA ESCREVE**, decidido igual nas duas passagens: os segmentos
// LOCAIS do preenchimento e das marcas (que se transformam um a um) e, com o eixo, o contorno
// gerado; e as LINHAS de ecrã das máscaras, que saem da caixa ESTIMADA (a exacta só se conhece
// depois de escrever, e a contagem tem de lhe chegar antes).
struct Plano {
    cp: Copia,
    valido: bool,
    // O preenchimento: o primeiro segmento em `segs` e quantos.
    f0: u32,
    nf: u32,
    // As marcas — sob afim conforme, o traço inteiro (as marcas e o contorno expandido).
    m0: u32,
    nm: u32,
    eixo: bool,
    // A primeira linha do ecrã com máscara e quantas; a primeira coluna e quantas CÉLULAS por linha.
    y0: f32,
    linhas: u32,
    x0: f32,
    celulas: u32,
}

fn plano_de(ii: u32) -> Plano {
    var p: Plano;
    p.valido = false;
    if contas.sem_contorno != 0u {
        return p;
    }
    p.cp = copia_de(ii);
    if p.cp.r == 0xffffffffu {
        return p;
    }
    let rec = records[p.cp.r];
    let rg = rec.ranges[p.cp.nivel];
    p.f0 = rg.x;
    p.nf = rg.y;
    p.m0 = rg.z;
    p.eixo = p.cp.eixo_rg.y > 0u;
    p.nm = select(rg.w, p.cp.eixo_rg.z, p.eixo);
    // ⚠️ Um pixel de folga de cada lado: a caixa estimada é a mesma que o quad do caminho de sempre
    // usa, e a máscara não pode ficar curta da exacta por um arredondamento.
    let cx = caixa_estimada(p.cp, rec);
    let lo = floor(cx.xy) - vec2<f32>(1.0);
    let hi = ceil(cx.zw) + vec2<f32>(1.0);
    // Fora do ecrã nenhum pixel corre: a cópia não paga cálculo nenhum.
    if hi.x <= 0.0 || lo.x >= view.alvo.x || hi.y <= 0.0 || lo.y >= view.alvo.y {
        return p;
    }
    p.y0 = max(lo.y, 0.0);
    p.linhas = u32(min(hi.y, ceil(view.alvo.y)) - p.y0);
    p.x0 = max(lo.x, 0.0);
    let x1 = min(hi.x, ceil(view.alvo.x));
    p.celulas = u32(ceil((x1 - p.x0) / LARGURA_DA_CELULA));
    p.valido = true;
    return p;
}

// O índice da cópia: um despacho de `64` por grupo, em duas dimensões quando passa de `65 535`
// grupos (o tecto de uma dimensão do despacho).
fn indice(gid: vec3<u32>, nwg: vec3<u32>) -> u32 {
    return gid.x + gid.y * nwg.x * 64u;
}

fn palavras_por_linha(arestas: u32) -> u32 {
    return (arestas / SEGS_POR_BLOCO + 31u) / 32u;
}

// Quantas palavras um REGISTO de célula ocupa: os três fundos (`f32`) e a máscara.
fn registo_de_celula(arestas: u32) -> u32 {
    return 3u + palavras_por_linha(arestas);
}

@compute @workgroup_size(64)
fn cs_conta(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let ii = indice(gid, nwg);
    if ii >= contas.n {
        return;
    }
    var ne = 0u;
    var nmask = 0u;
    let p = plano_de(ii);
    if p.valido {
        var nc = 0u;
        if p.eixo {
            escrever = false;
            percorre(p.cp);
            nc = (cursor + cursor_b + SEGS_POR_BLOCO - 1u) / SEGS_POR_BLOCO * SEGS_POR_BLOCO;
        }
        ne = p.nf + p.nm + nc;
        nmask = p.linhas * p.celulas * registo_de_celula(ne);
        if nmask == 0u {
            ne = 0u;
        }
    }
    contagem[ii] = ne;
    contagem[contas.n + 1u + ii] = nmask;
}

// Os prefixos exclusivos das DUAS contagens, num grupo só: cada fio soma um pedaço contíguo, o fio
// `0` faz o prefixo dos `256` parciais, e cada fio reescreve o seu pedaço. ⚠️ Determinístico — a
// ordem das somas não depende do escalonamento, e é isso que faz a mesma cena dar os mesmos sítios.
var<workgroup> parcial: array<u32, 256>;
var<workgroup> parcial_m: array<u32, 256>;

@compute @workgroup_size(256)
fn cs_soma(@builtin(local_invocation_index) li: u32) {
    let n = contas.n;
    let m0 = n + 1u;
    let pedaco = (n + 255u) / 256u;
    let i0 = min(li * pedaco, n);
    let i1 = min(i0 + pedaco, n);
    var s = 0u;
    var sm = 0u;
    for (var i = i0; i < i1; i += 1u) {
        s += contagem[i];
        sm += contagem[m0 + i];
    }
    parcial[li] = s;
    parcial_m[li] = sm;
    workgroupBarrier();
    if li == 0u {
        var acc = 0u;
        var acc_m = 0u;
        for (var k = 0u; k < 256u; k += 1u) {
            let v = parcial[k];
            parcial[k] = acc;
            acc += v;
            let vm = parcial_m[k];
            parcial_m[k] = acc_m;
            acc_m += vm;
        }
        contagem[n] = acc;
        contagem[m0 + n] = acc_m;
    }
    workgroupBarrier();
    var acc = parcial[li];
    var acc_m = parcial_m[li];
    for (var i = i0; i < i1; i += 1u) {
        let v = contagem[i];
        contagem[i] = acc;
        acc += v;
        let vm = contagem[m0 + i];
        contagem[m0 + i] = acc_m;
        acc_m += vm;
    }
}

// Os segmentos LOCAIS `[s0, s0 + k)` pelo afim da cópia, para `[saida, saida + k)`.
fn transforma(cp: Copia, s0: u32, k: u32, saida: u32) {
    for (var i = 0u; i < k; i += 1u) {
        let seg = segs[s0 + i];
        let a = aplica(cp.lin, cp.t, seg.xy);
        let b = aplica(cp.lin, cp.t, seg.zw);
        contorno_rw[saida + i] = vec4<f32>(a, b);
        cmin = min(cmin, min(a, b));
        cmax = max(cmax, max(a, b));
    }
}

@compute @workgroup_size(64)
fn cs_escreve(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let ii = indice(gid, nwg);
    if ii >= contas.n {
        return;
    }
    ccopias_rw[3u * ii] = vec4<u32>(0u);
    ccopias_rw[3u * ii + 1u] = vec4<u32>(0u);
    ccopias_rw[3u * ii + 2u] = vec4<u32>(0u);
    let m0 = contas.n + 1u;
    let base = contagem[ii];
    let ne = contagem[ii + 1u] - base;
    let mbase = contagem[m0 + ii];
    let nmask = contagem[m0 + ii + 1u] - mbase;
    // ⚠️ Fora da capacidade, a cópia fica com o caminho de sempre — nunca um contorno truncado.
    if ne == 0u || base + ne > contas.cap || mbase + nmask > contas.cap_mascaras {
        return;
    }
    let p = plano_de(ii);
    let palavras = palavras_por_linha(ne);
    let registo = 3u + palavras;
    if !p.valido || p.nf + p.nm > ne || p.linhas * p.celulas * registo != nmask {
        return;
    }
    cmin = vec2<f32>(3.0e38);
    cmax = vec2<f32>(-3.0e38);
    transforma(p.cp, p.f0, p.nf, base);
    transforma(p.cp, p.m0, p.nm, base + p.nf);
    let nc = ne - p.nf - p.nm;
    if nc > 0u {
        let bc = base + p.nf + p.nm;
        escrever = true;
        base_saida = bc;
        limite_saida = nc;
        ultimo = vec2<f32>(0.0);
        cabeca_b = vec2<f32>(0.0);
        percorre(p.cp);
        // ⚠️ A contagem e a escrita correm o MESMO código, e um compilador que os arredondasse
        // diferente (dois pontos de entrada) daria outra decisão numa bissectriz no limiar. ⇒ uma
        // escrita que não bate com a contagem é deitada fora e a cópia segue pelo caminho de sempre.
        let total = cursor + cursor_b;
        if total > nc || total + SEGS_POR_BLOCO <= nc {
            return;
        }
        // O enchimento, ENTRE as duas correntes: arestas de comprimento zero no último ponto da da
        // frente (contribuem `0`, não alargam caixa nenhuma — no `(0, 0)` alargariam a do bloco até
        // à origem do ecrã — e continuam a corrente; sem corrente da frente, no primeiro ponto da de
        // trás).
        let enche = select(cabeca_b, ultimo, cursor > 0u);
        for (var i = cursor; i < nc - cursor_b; i += 1u) {
            contorno_rw[bc + i] = vec4<f32>(enche, enche);
        }
    }
    // Os registos começam vazios: fundos a `0,0` (os bits de `0.0` são `0`) e máscaras apagadas.
    for (var k = 0u; k < nmask; k += 1u) {
        cmascaras_rw[mbase + k] = 0u;
    }
    let fim_f = p.nf / SEGS_POR_BLOCO;
    let fim_m = (p.nf + p.nm) / SEGS_POR_BLOCO;
    // Por bloco, DOIS `vec4`: a caixa, e `(y₀, y₈, encadeado, 0)` — `encadeado` quando cada aresta
    // começa EXACTAMENTE onde a anterior acabou (a igualdade é de bits: a soma telescopa só então).
    let b0 = base / SEGS_POR_BLOCO;
    let nb = ne / SEGS_POR_BLOCO;
    for (var bl = 0u; bl < nb; bl += 1u) {
        let e0 = base + bl * SEGS_POR_BLOCO;
        var lo = vec2<f32>(3.0e38);
        var hi = vec2<f32>(-3.0e38);
        var corrente = true;
        var antes = contorno_rw[e0].xy;
        for (var j = 0u; j < SEGS_POR_BLOCO; j += 1u) {
            let e = contorno_rw[e0 + j];
            lo = min(lo, min(e.xy, e.zw));
            hi = max(hi, max(e.xy, e.zw));
            corrente = corrente && all(e.xy == antes);
            antes = e.zw;
        }
        let y0b = contorno_rw[e0].y;
        let y8b = antes.y;
        cblocos_rw[2u * (b0 + bl)] = vec4<f32>(lo, hi);
        cblocos_rw[2u * (b0 + bl) + 1u] = vec4<f32>(y0b, y8b, select(0.0, 1.0, corrente), 0.0);
        // ⭐ A fileira de pixels `r` (do canto `y0 + r` ao `y0 + r + 1`) é tocada pelo bloco sse
        // `lo.y < y0 + r + 1` e `hi.y > y0 + r` — de `floor(lo.y)` a `ceil(hi.y) − 1`, exactos em
        // `f32`. Um bloco de altura zero (só arestas horizontais ou o enchimento) soma `0` em toda a
        // parte e não acende nada.
        if hi.y <= lo.y {
            continue;
        }
        let ra = u32(clamp(floor(lo.y) - p.y0, 0.0, f32(p.linhas)));
        let rb = u32(clamp(ceil(hi.y) - p.y0, 0.0, f32(p.linhas)));
        // ⭐⭐ As CÉLULAS: a caixa toca as células `ka .. kb` da linha (a máscara) e fica TODA À
        // ESQUERDA de todo pixel a partir da `kb` — lá a soma dela é o FUNDO, que não depende do
        // `x` do pixel e se soma uma vez aqui em vez de uma vez por pixel (o `backdrop` do Vello).
        let ka = u32(clamp(floor((lo.x - p.x0) / LARGURA_DA_CELULA), 0.0, f32(p.celulas)));
        let kb = u32(clamp(ceil((hi.x - p.x0) / LARGURA_DA_CELULA), 0.0, f32(p.celulas)));
        let bit = 1u << (bl % 32u);
        let palavra = 3u + bl / 32u;
        let cat = select(select(2u, 1u, bl < fim_m), 0u, bl < fim_f);
        for (var r = ra; r < rb; r += 1u) {
            let linha = mbase + r * p.celulas * registo;
            for (var k = ka; k < kb; k += 1u) {
                let i = linha + k * registo + palavra;
                cmascaras_rw[i] = cmascaras_rw[i] | bit;
            }
            if kb < p.celulas {
                let y = p.y0 + f32(r);
                var v = 0.0;
                if corrente {
                    v = clamp(y0b - y, 0.0, 1.0) - clamp(y8b - y, 0.0, 1.0);
                } else {
                    for (var j = 0u; j < SEGS_POR_BLOCO; j += 1u) {
                        let e = contorno_rw[e0 + j];
                        v += clamp(e.y - y, 0.0, 1.0) - clamp(e.w - y, 0.0, 1.0);
                    }
                }
                let i = linha + kb * registo + cat;
                cmascaras_rw[i] = bitcast<u32>(bitcast<f32>(cmascaras_rw[i]) + v);
            }
        }
    }
    // O fundo de uma célula é o de TODOS os blocos que acabam antes dela: o prefixo ao longo da linha.
    for (var r = 0u; r < p.linhas; r += 1u) {
        let linha = mbase + r * p.celulas * registo;
        var acc = vec3<f32>(0.0);
        for (var k = 0u; k < p.celulas; k += 1u) {
            let i = linha + k * registo;
            acc += vec3<f32>(
                bitcast<f32>(cmascaras_rw[i]),
                bitcast<f32>(cmascaras_rw[i + 1u]),
                bitcast<f32>(cmascaras_rw[i + 2u]),
            );
            cmascaras_rw[i] = bitcast<u32>(acc.x);
            cmascaras_rw[i + 1u] = bitcast<u32>(acc.y);
            cmascaras_rw[i + 2u] = bitcast<u32>(acc.z);
        }
    }
    ccaixas_rw[ii] = vec4<f32>(cmin, cmax);
    ccopias_rw[3u * ii] = vec4<u32>(b0, p.nf / SEGS_POR_BLOCO, p.nm / SEGS_POR_BLOCO, nc / SEGS_POR_BLOCO);
    ccopias_rw[3u * ii + 1u] = vec4<u32>(mbase, p.linhas, palavras, bitcast<u32>(p.y0));
    ccopias_rw[3u * ii + 2u] = vec4<u32>(bitcast<u32>(p.x0), p.celulas, 0u, 0u);
}
