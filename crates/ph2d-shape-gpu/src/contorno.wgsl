
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
    _p: u32,
}

@group(2) @binding(0) var<uniform> contas: Contas;
// Por cópia: quantas arestas (múltiplo de `SEGS_POR_BLOCO`) e, depois do `cs_soma`, onde começam.
// Tem `n + 1` entradas: a última é o total.
@group(2) @binding(1) var<storage, read_write> contagem: array<u32>;
@group(2) @binding(2) var<storage, read_write> contorno_rw: array<vec4<f32>>;
@group(2) @binding(3) var<storage, read_write> cblocos_rw: array<vec4<f32>>;
@group(2) @binding(4) var<storage, read_write> ccopias_rw: array<vec4<u32>>;
@group(2) @binding(5) var<storage, read_write> ccaixas_rw: array<vec4<f32>>;

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

// O índice da cópia: um despacho de `64` por grupo, em duas dimensões quando passa de `65 535`
// grupos (o tecto de uma dimensão do despacho).
fn indice(gid: vec3<u32>, nwg: vec3<u32>) -> u32 {
    return gid.x + gid.y * nwg.x * 64u;
}

@compute @workgroup_size(64)
fn cs_conta(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let ii = indice(gid, nwg);
    if ii >= contas.n {
        return;
    }
    var n = 0u;
    if contas.sem_contorno == 0u {
        let cp = copia_de(ii);
        if cp.r != 0xffffffffu && cp.eixo_rg.y > 0u {
            escrever = false;
            percorre(cp);
            n = (cursor + cursor_b + SEGS_POR_BLOCO - 1u) / SEGS_POR_BLOCO * SEGS_POR_BLOCO;
        }
    }
    contagem[ii] = n;
}

// O prefixo exclusivo das contagens, num grupo só: cada fio soma um pedaço contíguo, o fio `0`
// faz o prefixo dos `256` parciais, e cada fio reescreve o seu pedaço. ⚠️ Determinístico — a ordem
// das somas não depende do escalonamento, e é isso que faz a mesma cena dar os mesmos sítios.
var<workgroup> parcial: array<u32, 256>;

@compute @workgroup_size(256)
fn cs_soma(@builtin(local_invocation_index) li: u32) {
    let n = contas.n;
    let pedaco = (n + 255u) / 256u;
    let i0 = min(li * pedaco, n);
    let i1 = min(i0 + pedaco, n);
    var s = 0u;
    for (var i = i0; i < i1; i += 1u) {
        s += contagem[i];
    }
    parcial[li] = s;
    workgroupBarrier();
    if li == 0u {
        var acc = 0u;
        for (var k = 0u; k < 256u; k += 1u) {
            let v = parcial[k];
            parcial[k] = acc;
            acc += v;
        }
        contagem[n] = acc;
    }
    workgroupBarrier();
    var acc = parcial[li];
    for (var i = i0; i < i1; i += 1u) {
        let v = contagem[i];
        contagem[i] = acc;
        acc += v;
    }
}

@compute @workgroup_size(64)
fn cs_escreve(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let ii = indice(gid, nwg);
    if ii >= contas.n {
        return;
    }
    ccopias_rw[ii] = vec4<u32>(0u);
    let base = contagem[ii];
    let n = contagem[ii + 1u] - base;
    // ⚠️ Fora da capacidade, a cópia fica com o caminho de sempre — nunca um contorno truncado.
    if n == 0u || base + n > contas.cap {
        return;
    }
    let cp = copia_de(ii);
    escrever = true;
    base_saida = base;
    limite_saida = n;
    cmin = vec2<f32>(3.0e38);
    cmax = vec2<f32>(-3.0e38);
    ultimo = vec2<f32>(0.0);
    cabeca_b = vec2<f32>(0.0);
    percorre(cp);
    // ⚠️ A contagem e a escrita correm o MESMO código, e um compilador que os arredondasse
    // diferente (dois pontos de entrada) daria outra decisão numa bissectriz no limiar. ⇒ uma
    // escrita que não bate com a contagem é deitada fora e a cópia segue pelo caminho de sempre.
    let total = cursor + cursor_b;
    if total > n || total + SEGS_POR_BLOCO <= n {
        return;
    }
    // O enchimento, ENTRE as duas correntes: arestas de comprimento zero no último ponto da da
    // frente (contribuem `0`, não alargam caixa nenhuma — no `(0, 0)` alargariam a do bloco até à
    // origem do ecrã — e continuam a corrente; sem corrente da frente, no primeiro ponto da de trás).
    let enche = select(cabeca_b, ultimo, cursor > 0u);
    for (var i = cursor; i < n - cursor_b; i += 1u) {
        contorno_rw[base + i] = vec4<f32>(enche, enche);
    }
    // Por bloco, DOIS `vec4`: a caixa, e `(y₀, y₈, encadeado, 0)` — `encadeado` quando cada aresta
    // começa EXACTAMENTE onde a anterior acabou (a igualdade é de bits: a soma telescopa só então).
    let b0 = base / SEGS_POR_BLOCO;
    let nb = n / SEGS_POR_BLOCO;
    for (var bl = 0u; bl < nb; bl += 1u) {
        var lo = vec2<f32>(3.0e38);
        var hi = vec2<f32>(-3.0e38);
        var corrente = true;
        var antes = contorno_rw[base + bl * SEGS_POR_BLOCO].xy;
        for (var j = 0u; j < SEGS_POR_BLOCO; j += 1u) {
            let e = contorno_rw[base + bl * SEGS_POR_BLOCO + j];
            lo = min(lo, min(e.xy, e.zw));
            hi = max(hi, max(e.xy, e.zw));
            corrente = corrente && all(e.xy == antes);
            antes = e.zw;
        }
        let y0 = contorno_rw[base + bl * SEGS_POR_BLOCO].y;
        cblocos_rw[2u * (b0 + bl)] = vec4<f32>(lo, hi);
        cblocos_rw[2u * (b0 + bl) + 1u] = vec4<f32>(y0, antes.y, select(0.0, 1.0, corrente), 0.0);
    }
    ccaixas_rw[ii] = vec4<f32>(cmin, cmax);
    ccopias_rw[ii] = vec4<u32>(b0, nb, 0u, 0u);
}
