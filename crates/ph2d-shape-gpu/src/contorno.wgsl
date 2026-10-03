
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
    // Quantas palavras cabem em `ccelulas_rw`.
    cap_celulas: u32,
    // A área no ecrã (px², da caixa estimada) a partir da qual uma cópia CONFORME vai pelas arestas
    // no ecrã (`contorno.rs`, `AREA_MINIMA_CONFORME`).
    area_minima_conforme: f32,
    // Quantas arestas cabem em `listas_rw` (doc 121 §9.8).
    cap_listas: u32,
    _p1: u32,
    _p2: u32,
}

@group(2) @binding(0) var<uniform> contas: Contas;
// Três contagens por cópia, e depois do `cs_soma` onde cada uma começa: as ARESTAS (múltiplo de
// `SEGS_POR_BLOCO`) nas `n + 1` primeiras entradas, as PALAVRAS DE MÁSCARA nas `n + 1` seguintes e
// as LINHAS de ecrã nas `n + 1` últimas — a última de cada terço é o total.
@group(2) @binding(1) var<storage, read_write> contagem: array<u32>;
@group(2) @binding(2) var<storage, read_write> contorno_rw: array<vec4<f32>>;
@group(2) @binding(4) var<storage, read_write> ccopias_rw: array<vec4<u32>>;
@group(2) @binding(5) var<storage, read_write> ccaixas_rw: array<vec4<f32>>;
// Os registos de célula (`REGISTO` palavras): atómicos, porque os passes por ARESTA (doc 121 §9.8)
// somam-lhes fundos e contagens de muitos fios ao mesmo tempo.
@group(2) @binding(6) var<storage, read_write> ccelulas_rw: array<atomic<u32>>;
// ⭐ doc 121 §9.7–§9.8 — os argumentos dos despachos INDIRECTOS (`x, y, z` grupos), escritos pelo
// `cs_soma`: `[0, 3)` um fio por LINHA de ecrã, `[3, 6)` um fio por ARESTA. Os totais só existem na
// placa, e lê-los no CPU custaria dois quadros.
@group(2) @binding(7) var<storage, read_write> despacho_rw: array<u32>;
// ⭐ doc 121 §9.8 — as LISTAS de arestas das células (cópias das arestas, para o desenho as ler
// seguidas) e o total pedido, que cada fileira reserva de uma vez. ⚠️ O total passa da capacidade
// quando ela não chega: é esse número que o CPU lê para a fazer crescer (`contorno.rs`).
@group(2) @binding(8) var<storage, read_write> listas_rw: array<vec4<f32>>;
@group(2) @binding(9) var<storage, read_write> lista_total: array<atomic<u32>>;

// O estado da emissão de UMA cópia (um fio por cópia). A ordem das arestas não importa a ninguém: as
// células as tomam uma a uma (doc 121 §9.8).
var<private> cursor: u32;
var<private> base_saida: u32;
var<private> limite_saida: u32;
var<private> cmin: vec2<f32>;
var<private> cmax: vec2<f32>;

fn emite(p0: vec2<f32>, p1: vec2<f32>) {
    if cursor < limite_saida {
        contorno_rw[base_saida + cursor] = vec4<f32>(p0, p1);
        cmin = min(cmin, min(p0, p1));
        cmax = max(cmax, max(p0, p1));
    }
    cursor += 1u;
}

// Uma aresta com o sentido da peça: o `orienta` da soma por pixel inverte a peça inteira quando
// ela gira ao contrário, e inverter uma aresta inverte o sinal da contribuição dela.
fn aresta(p0: vec2<f32>, p1: vec2<f32>, positivo: bool) {
    if positivo {
        emite(p0, p1);
    } else {
        emite(p1, p0);
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
        let q0 = a + m0;
        let q1 = b + m1;
        let q2 = b - m1;
        let q3 = a - m0;
        let s = positivo(q0, q1, q2);
        if !faixa0 {
            aresta(q3, q0, s);
        }
        aresta(q0, q1, s);
        aresta(q2, q3, s);
        if !faixa1 {
            aresta(q1, q2, s);
        }
        return;
    }
    if it.tipo == 2u {
        emite_tampa(b, u, r, it.ponta);
    }
}

// A ponta de estilo `tampa` em `c`, virada para `u` — o `tampa_px` da soma por pixel, como arestas.
fn emite_tampa(c: vec2<f32>, u: vec2<f32>, r: f32, tampa: u32) {
    let nr = perp(u) * r;
    if tampa == 1u {
        emite_quad(c + nr, c + nr + u * r, c - nr + u * r, c - nr);
    } else if tampa == 2u {
        let dir = sign(nr.x * u.y - nr.y * u.x);
        emite_leque(c, nr, -nr, -1.0, dir, r);
    }
}

// doc 121 §9.9 — o `pedaco_px` da soma por pixel, como arestas.
fn emite_pedaco(it: Eixo, lin: vec4<f32>, t: vec2<f32>, caneta: f32, tr: TrocoTracejado, p: Pedaco) {
    let r = it.meia * caneta;
    if tr.corda <= 0.0 || tr.len <= 0.0 || r <= 0.0 {
        return;
    }
    let a = aplica(lin, t, it.a);
    let b = aplica(lin, t, it.b);
    let q = cantos_do_pedaco(a, b, tr, p);
    let u = (b - a) / tr.corda;
    let nr = perp(u) * r;
    var m0 = nr;
    var m1 = nr;
    var faixa0 = false;
    var faixa1 = false;
    if p.liga0 {
        let e = bissectriz_ate(aplica(lin, t, it.d), a, b, r, (it.ponta & 4u) != 0u, it.junta, it.limite, p.recuo0);
        if e.z > 0.0 {
            m0 = e.xy;
            faixa0 = true;
        }
    } else {
        emite_tampa(q.xy, -u, r, (it.ponta >> 6u) & 3u);
    }
    if p.liga1 {
        let quina = (it.ponta & 8u) != 0u;
        let cf = aplica(lin, t, it.c);
        let e = bissectriz_ate(a, b, cf, r, quina, it.junta, it.limite, p.recuo1);
        if e.z > 0.0 {
            m1 = e.xy;
            faixa1 = true;
        } else {
            emite_junta(u, b, cf, r, select(2u, it.junta, quina), it.limite);
        }
    } else {
        emite_tampa(q.zw, u, r, (it.ponta >> 8u) & 3u);
    }
    let q0 = q.xy + m0;
    let q1 = q.zw + m1;
    let q2 = q.zw - m1;
    let q3 = q.xy - m0;
    let s = positivo(q0, q1, q2);
    if !faixa0 {
        aresta(q3, q0, s);
    }
    aresta(q0, q1, s);
    aresta(q2, q3, s);
    if !faixa1 {
        aresta(q1, q2, s);
    }
}

// O sub-caminho tracejado que começa no troço `i0` — o `tracejado_px` da soma por pixel.
fn emite_tracejado(i0: u32, lin: vec4<f32>, t: vec2<f32>, caneta: f32, ajuste: f32) {
    let sub = sub_tracejado(i0, lin, t, caneta, ajuste);
    if sub.per <= 0.0 {
        return;
    }
    var i = i0;
    var s0 = 0.0;
    for (var k = 0u; k < sub.n; k += 1u) {
        i = proximo_troco(i);
        let it = eixo[i];
        let tr = troco_tracejado(it, sub, k, s0, lin, t);
        for (var n = tr.n0; n <= tr.n1; n += 1.0) {
            let p = pedaco(tr, sub, n);
            if p.valido {
                emite_pedaco(it, lin, t, caneta, tr, p);
            }
        }
        s0 = tr.fim;
        i += 1u;
    }
}

// As peças do eixo de uma cópia (os cabeçalhos de bloco não desenham nada; um sub-caminho tracejado
// percorre-se inteiro a partir do primeiro troço dele).
fn percorre(cp: Copia) {
    cursor = 0u;
    let caneta = bitcast<f32>(cp.eixo_rg.w);
    var ajuste = 0.0;
    for (var i = cp.eixo_rg.x; i < cp.eixo_rg.x + cp.eixo_rg.y; i += 1u) {
        let it = eixo[i];
        if it.tipo == 3u {
            continue;
        }
        if !tracejado(it) {
            emite_peca(it, cp.lin, cp.t, caneta);
        } else if (it.ponta & SUB_INICIO) != 0u {
            if ajuste == 0.0 {
                ajuste = ajuste_do_tracejado(cp.eixo_rg.x, cp.eixo_rg.y, cp.lin, cp.t, caneta);
            }
            emite_tracejado(i, cp.lin, cp.t, caneta, ajuste);
        }
    }
}

// ⭐ doc 121 §9.6 — **O QUE UMA CÓPIA ESCREVE**, decidido igual nas duas passagens: os segmentos
// LOCAIS do preenchimento e das marcas (que se transformam um a um) e, com o eixo, o contorno
// gerado; e as LINHAS de ecrã das células, que saem da caixa ESTIMADA (a exacta só se conhece
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
    // A primeira linha do ecrã com células e quantas; a primeira coluna e quantas CÉLULAS por linha.
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
    // usa, e as células não podem ficar curtas da exacta por um arredondamento.
    let cx = caixa_estimada(p.cp, rec);
    let lo = floor(cx.xy) - vec2<f32>(1.0);
    let hi = ceil(cx.zw) + vec2<f32>(1.0);
    // Fora do ecrã nenhum pixel corre: a cópia não paga cálculo nenhum.
    if hi.x <= 0.0 || lo.x >= view.alvo.x || hi.y <= 0.0 || lo.y >= view.alvo.y {
        return p;
    }
    // ⭐ Uma cópia CONFORME só paga o cálculo quando é GRANDE: o caminho de sempre já a desenha bem
    // pequena (sem eixo, os segmentos locais por pixel), e o custo do cálculo é por cópia — medido
    // na escada de `32 768` estrelas pequenas, `+3,9 ms` de cálculo por zero ganho no desenho.
    let tam = cx.zw - cx.xy;
    if !p.eixo && tam.x * tam.y < contas.area_minima_conforme {
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

// ⭐ doc 121 §9.6 — **QUANTAS ARESTAS UMA PEÇA PODE EMITIR, sem a geometria dela.** A contagem
// corria o `percorre` inteiro (bissectrizes, juntas, leques) só para saber um número, e a escrita
// corria-o outra vez. ⇒ a contagem usa este LIMITE SUPERIOR — o pior caso de cada ramo de
// `emite_peca`, que só depende do tipo e do raio no ecrã —, e a escrita usa só o que precisou
// (`cs_escreve`). O leque de `k` passos emite no máximo `3k` arestas (a primeira, a de cada
// passo, as duas de uma troca de sentido e a de fecho); a junta é o pior entre o quadrilátero (4) e
// o leque; o troço são 4 mais a junta de quem chega.
fn arestas_do_leque(r: f32) -> u32 {
    if r <= 0.25 {
        return 3u;
    }
    let q = 1.0 - 0.25 / r;
    let passo_max = 2.0 * acos(q);
    let k = u32(clamp(ceil(3.14159274 / max(passo_max, 1.0e-4)), 1.0, 64.0));
    return 3u * k;
}

fn arestas_da_tampa(tampa: u32, r: f32) -> u32 {
    return select(select(0u, 4u, tampa == 1u), arestas_do_leque(r), tampa == 2u);
}

fn limite_de_arestas(cp: Copia) -> u32 {
    let caneta = bitcast<f32>(cp.eixo_rg.w);
    var n = 0u;
    var ajuste = 0.0;
    for (var i = cp.eixo_rg.x; i < cp.eixo_rg.x + cp.eixo_rg.y; i += 1u) {
        let it = eixo[i];
        let r = it.meia * caneta;
        if it.tipo == 0u && tracejado(it) {
            // doc 121 §9.9 — cada traço que toca o troço: o quadrilátero, as duas pontas e a junta.
            if ajuste == 0.0 {
                ajuste = ajuste_do_tracejado(cp.eixo_rg.x, cp.eixo_rg.y, cp.lin, cp.t, caneta);
            }
            // ⚠️ `0,99`: o período ajustado é o que o percurso usa; a folga cobre o arredondamento.
            let per = (it.traco + it.vao) * caneta * ajuste * 0.99;
            let len = arco(it, cp.lin, cp.t);
            let pecas = u32(min(ceil(len / max(per, 1.0e-30)), TRACOS_POR_TROCO_MAX)) + 2u;
            let tampa = max(arestas_da_tampa((it.ponta >> 6u) & 3u, r), arestas_da_tampa((it.ponta >> 8u) & 3u, r));
            n = min(n + pecas * (4u + 2u * tampa + max(4u, arestas_do_leque(r))), 0x3fffffffu);
        } else if it.tipo == 0u {
            n += 4u;
            if (it.ponta & 2u) != 0u {
                n += max(4u, arestas_do_leque(r));
            }
        } else if it.tipo == 2u {
            n += select(arestas_do_leque(r), 4u, it.ponta == 1u);
        }
    }
    return n;
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
    var ne = 0u;
    var nmask = 0u;
    var linhas = 0u;
    let p = plano_de(ii);
    if p.valido {
        var nc = 0u;
        if p.eixo {
            nc = (limite_de_arestas(p.cp) + SEGS_POR_BLOCO - 1u) / SEGS_POR_BLOCO * SEGS_POR_BLOCO;
        }
        ne = p.nf + p.nm + nc;
        nmask = p.linhas * p.celulas * REGISTO;
        if nmask == 0u {
            ne = 0u;
        } else {
            linhas = p.linhas;
        }
    }
    contagem[ii] = ne;
    contagem[contas.n + 1u + ii] = nmask;
    contagem[2u * (contas.n + 1u) + ii] = linhas;
}

// Os prefixos exclusivos das DUAS contagens, num grupo só: cada fio soma um pedaço contíguo, o fio
// `0` faz o prefixo dos `256` parciais, e cada fio reescreve o seu pedaço. ⚠️ Determinístico — a
// ordem das somas não depende do escalonamento, e é isso que faz a mesma cena dar os mesmos sítios.
var<workgroup> parcial: array<u32, 256>;

// Os argumentos de um despacho indirecto de `fios` fios em grupos de `64`, a partir de `em`: em duas
// dimensões quando passa de `65 535` grupos (o tecto de uma dimensão), como o de `indice`.
fn despacha(em: u32, fios: u32) {
    let grupos = (fios + 63u) / 64u;
    let gx = min(grupos, 65535u);
    despacho_rw[em] = gx;
    despacho_rw[em + 1u] = select(1u, (grupos + gx - 1u) / max(gx, 1u), gx > 0u);
    despacho_rw[em + 2u] = 1u;
}
var<workgroup> parcial_m: array<u32, 256>;
var<workgroup> parcial_l: array<u32, 256>;

@compute @workgroup_size(256)
fn cs_soma(@builtin(local_invocation_index) li: u32) {
    let n = contas.n;
    let m0 = n + 1u;
    let l0 = 2u * (n + 1u);
    let pedaco = (n + 255u) / 256u;
    let i0 = min(li * pedaco, n);
    let i1 = min(i0 + pedaco, n);
    var s = 0u;
    var sm = 0u;
    var sl = 0u;
    for (var i = i0; i < i1; i += 1u) {
        s += contagem[i];
        sm += contagem[m0 + i];
        sl += contagem[l0 + i];
    }
    parcial[li] = s;
    parcial_m[li] = sm;
    parcial_l[li] = sl;
    workgroupBarrier();
    if li == 0u {
        var acc = 0u;
        var acc_m = 0u;
        var acc_l = 0u;
        for (var k = 0u; k < 256u; k += 1u) {
            let v = parcial[k];
            parcial[k] = acc;
            acc += v;
            let vm = parcial_m[k];
            parcial_m[k] = acc_m;
            acc_m += vm;
            let vl = parcial_l[k];
            parcial_l[k] = acc_l;
            acc_l += vl;
        }
        contagem[n] = acc;
        contagem[m0 + n] = acc_m;
        contagem[l0 + n] = acc_l;
        // ⭐ doc 121 §9.7–§9.8 — os despachos das células: um fio por LINHA e um por ARESTA.
        despacha(0u, acc_l);
        despacha(3u, acc);
        // O total das listas recomeça a cada quadro (o `cs_celulas` corre depois deste passe).
        atomicStore(&lista_total[0], 0u);
    }
    workgroupBarrier();
    var acc = parcial[li];
    var acc_m = parcial_m[li];
    var acc_l = parcial_l[li];
    for (var i = i0; i < i1; i += 1u) {
        let v = contagem[i];
        contagem[i] = acc;
        acc += v;
        let vm = contagem[m0 + i];
        contagem[m0 + i] = acc_m;
        acc_m += vm;
        let vl = contagem[l0 + i];
        contagem[l0 + i] = acc_l;
        acc_l += vl;
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
    // O que a contagem RESERVOU: com o eixo, o limite superior do contorno; o que se usa é ≤.
    let reservado = contagem[ii + 1u] - base;
    let mbase = contagem[m0 + ii];
    let nmask_reservado = contagem[m0 + ii + 1u] - mbase;
    // ⚠️ Fora da capacidade, a cópia fica com o caminho de sempre — nunca um contorno truncado.
    if reservado == 0u || base + reservado > contas.cap
        || mbase + nmask_reservado > contas.cap_celulas {
        return;
    }
    let p = plano_de(ii);
    if !p.valido || p.nf + p.nm > reservado
        || p.linhas * p.celulas * REGISTO != nmask_reservado {
        return;
    }
    cmin = vec2<f32>(3.0e38);
    cmax = vec2<f32>(-3.0e38);
    transforma(p.cp, p.f0, p.nf, base);
    transforma(p.cp, p.m0, p.nm, base + p.nf);
    let nc_reservado = reservado - p.nf - p.nm;
    var nc = 0u;
    if nc_reservado > 0u {
        let bc = base + p.nf + p.nm;
        base_saida = bc;
        limite_saida = nc_reservado;
        percorre(p.cp);
        // ⚠️ O limite da contagem é um pior caso: uma escrita que o passasse seria um contorno
        // truncado. ⇒ é deitada fora e a cópia segue pelo caminho de sempre.
        if cursor > nc_reservado {
            return;
        }
        nc = (cursor + SEGS_POR_BLOCO - 1u) / SEGS_POR_BLOCO * SEGS_POR_BLOCO;
        // O enchimento até ao bloco inteiro: arestas de comprimento zero (`dy = 0` em toda a fileira —
        // as células não as vêem).
        for (var i = cursor; i < nc; i += 1u) {
            contorno_rw[bc + i] = vec4<f32>(0.0);
        }
    }
    // O que se usa: as arestas e o registo de célula do que foi DE FACTO escrito.
    let ne = p.nf + p.nm + nc;
    ccaixas_rw[ii] = vec4<f32>(cmin, cmax);
    ccopias_rw[3u * ii] = vec4<u32>(base / SEGS_POR_BLOCO, p.nf / SEGS_POR_BLOCO, p.nm / SEGS_POR_BLOCO, nc / SEGS_POR_BLOCO);
    ccopias_rw[3u * ii + 1u] = vec4<u32>(mbase, p.linhas, 0u, bitcast<u32>(p.y0));
    ccopias_rw[3u * ii + 2u] = vec4<u32>(bitcast<u32>(p.x0), p.celulas, 0u, 0u);
}

// ⭐⭐ doc 121 §9.8 — **AS CÉLULAS E AS LISTAS, EM QUATRO PASSES LARGOS.** Cada fileira de pixels de uma
// cópia é partida em células de `LARGURA_DA_CELULA` px; cada célula guarda o FUNDO (as arestas que,
// nessa fileira, ficam todas à esquerda dela — o `backdrop` do Vello) e a LISTA das que a tocam — a
// lista de segmentos que o Vello guarda por ladrilho.
//
// Um fio por FILEIRA, como no §9.7, eram `~2 900` fios na sonda das estrelas grandes, cada um a
// percorrer em série centenas de arestas — a placa integrada ficava à espera da memória (`1,5 ms` de
// cálculo). Aqui o trabalho é por ARESTA (`~70 000` fios, cada um com uma ou duas fileiras):
//
// 1. `cs_zera` — um fio por fileira apaga os registos dela;
// 2. `cs_conta_listas` — um fio por aresta soma o fundo e conta a lista de cada célula (atómicos);
// 3. `cs_lugar_das_listas` — um fio por fileira reserva o lugar das listas dela de uma vez, faz o
//    prefixo do fundo e põe cada contagem como cursor;
// 4. `cs_escreve_listas` — um fio por aresta escreve-a na lista de cada célula que toca.
//
// ⭐ **Determinístico por construção:** o fundo soma-se em PONTO FIXO (`ESCALA_FIXA`), e a soma de
// inteiros não depende da ordem — e o fragmento soma a lista também em ponto fixo, logo a ORDEM em que
// os atómicos arrumaram as arestas numa lista não muda um bit da imagem.
//
// ⚠️ Uma fileira que não cabe na capacidade fica `SEM_LISTA` e o desenho refaz-lhe os pixels pelo
// caminho de sempre. Uma cópia que o `cs_escreve` recusou (capacidade, contagem que não bateu) tem
// `(0, 0, 0)` nos blocos e é saltada: os registos dela não se lêem.

// Uma aresta na fileira `[y, y + 1)`: `(dy, x mínimo, x máximo)`. `dy` é o da `contribuicao` (zero ⇒ a
// aresta não soma nada a pixel nenhum da fileira); os `x` são os dos dois PONTOS, sem recortar à
// fileira — um intervalo que contém o do pedaço. ⚠️ É exacto mesmo assim: um pixel cujo canto + 1 não
// passa do `x` mínimo lê `0` da `contribuicao`, um cujo canto não fica antes do máximo lê `dy`, e entre
// os dois a lista leva a aresta e o fragmento faz a conta inteira — numa aresta que atravessa várias
// fileiras a lista só fica mais longa.
fn na_fileira(e: vec4<f32>, y: f32) -> vec3<f32> {
    let y0 = clamp(e.y - y, 0.0, 1.0);
    let y1 = clamp(e.w - y, 0.0, 1.0);
    return vec3<f32>(y0 - y1, min(e.x, e.z), max(e.x, e.z));
}

// As células `[ka, kb)` de uma fileira que um pedaço `[x mín, x máx]` toca: a partir da `kb` ele fica
// todo à esquerda de todo pixel (o canto do pixel é `≥` o máximo) e é fundo; antes da `ka` todo à
// direita (o canto + 1 é `≤` o mínimo) e soma zero.
fn celulas_do_pedaco(f: vec3<f32>, x0: f32, celulas: u32) -> vec2<u32> {
    let ka = u32(clamp(floor((f.y - x0) / LARGURA_DA_CELULA), 0.0, f32(celulas)));
    let kb = u32(clamp(ceil((f.z - x0) / LARGURA_DA_CELULA), 0.0, f32(celulas)));
    return vec2<u32>(ka, kb);
}

// O que um fio de FILEIRA sabe: a cópia, a fileira dentro dela e o primeiro registo dela.
struct Fileira {
    valida: bool,
    ii: u32,
    r: u32,
    celulas: u32,
    registo: u32,
}

fn fileira_de(g: u32) -> Fileira {
    var f: Fileira;
    f.valida = false;
    let n = contas.n;
    let l0 = 2u * (n + 1u);
    if g >= contagem[l0 + n] {
        return f;
    }
    // A cópia: a ÚLTIMA cujo início de linhas é `≤ g` (uma cópia sem linhas partilha o início da
    // seguinte e perde para ela, que é a que tem a linha).
    var a = 0u;
    var b = n;
    while b - a > 1u {
        let m = (a + b) / 2u;
        if contagem[l0 + m] <= g {
            a = m;
        } else {
            b = m;
        }
    }
    let c0 = ccopias_rw[3u * a];
    let c1 = ccopias_rw[3u * a + 1u];
    f.r = g - contagem[l0 + a];
    if c0.y + c0.z + c0.w == 0u || f.r >= c1.y {
        return f;
    }
    f.valida = true;
    f.ii = a;
    f.celulas = ccopias_rw[3u * a + 2u].y;
    f.registo = c1.x + f.r * f.celulas * REGISTO;
    return f;
}

// O que um fio de ARESTA sabe: a aresta (o índice dela é o do fio — as arestas de uma cópia começam
// onde a reserva dela começa), a família e a geometria das fileiras da cópia.
struct ArestaDoFio {
    valida: bool,
    e: vec4<f32>,
    fam: u32,
    y0: f32,
    linhas: u32,
    x0: f32,
    celulas: u32,
    registo0: u32,
}

fn aresta_de(g: u32) -> ArestaDoFio {
    var a: ArestaDoFio;
    a.valida = false;
    let n = contas.n;
    if g >= contagem[n] {
        return a;
    }
    // A cópia: a ÚLTIMA cuja reserva começa em `≤ g`.
    var lo = 0u;
    var hi = n;
    while hi - lo > 1u {
        let m = (lo + hi) / 2u;
        if contagem[m] <= g {
            lo = m;
        } else {
            hi = m;
        }
    }
    let c0 = ccopias_rw[3u * lo];
    // ⚠️ A reserva é um pior caso: só as arestas DE FACTO escritas contam.
    let bl = (g - contagem[lo]) / SEGS_POR_BLOCO;
    if bl >= c0.y + c0.z + c0.w {
        return a;
    }
    let c1 = ccopias_rw[3u * lo + 1u];
    let c2 = ccopias_rw[3u * lo + 2u];
    a.valida = true;
    a.e = contorno_rw[g];
    a.fam = select(select(2u, 1u, bl < c0.y + c0.z), 0u, bl < c0.y);
    a.y0 = bitcast<f32>(c1.w);
    a.linhas = c1.y;
    a.x0 = bitcast<f32>(c2.x);
    a.celulas = c2.y;
    a.registo0 = c1.x;
    return a;
}

// As fileiras da cópia que a aresta pode tocar: `[floor(y mín), ceil(y máx))`, recortadas às da cópia.
fn fileiras_da_aresta(a: ArestaDoFio) -> vec2<u32> {
    let ra = u32(clamp(floor(min(a.e.y, a.e.w)) - a.y0, 0.0, f32(a.linhas)));
    let rb = u32(clamp(ceil(max(a.e.y, a.e.w)) - a.y0, 0.0, f32(a.linhas)));
    return vec2<u32>(ra, rb);
}

@compute @workgroup_size(64)
fn cs_zera(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let f = fileira_de(indice(gid, nwg));
    if !f.valida {
        return;
    }
    for (var k = 0u; k < f.celulas * REGISTO; k += 1u) {
        atomicStore(&ccelulas_rw[f.registo + k], 0u);
    }
}

@compute @workgroup_size(64)
fn cs_conta_listas(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let a = aresta_de(indice(gid, nwg));
    if !a.valida {
        return;
    }
    let rs = fileiras_da_aresta(a);
    for (var r = rs.x; r < rs.y; r += 1u) {
        let f = na_fileira(a.e, a.y0 + f32(r));
        if f.x == 0.0 {
            continue;
        }
        let ks = celulas_do_pedaco(f, a.x0, a.celulas);
        let q0 = a.registo0 + r * a.celulas * REGISTO;
        if ks.y < a.celulas {
            atomicAdd(&ccelulas_rw[q0 + ks.y * REGISTO + a.fam], bitcast<u32>(fixo(f.x)));
        }
        for (var k = ks.x; k < ks.y; k += 1u) {
            atomicAdd(&ccelulas_rw[q0 + k * REGISTO + 4u + a.fam], 1u);
        }
    }
}

@compute @workgroup_size(64)
fn cs_lugar_das_listas(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let f = fileira_de(indice(gid, nwg));
    if !f.valida {
        return;
    }
    var total = 0u;
    for (var k = 0u; k < f.celulas; k += 1u) {
        let q = f.registo + k * REGISTO;
        total += atomicLoad(&ccelulas_rw[q + 4u]) + atomicLoad(&ccelulas_rw[q + 5u])
            + atomicLoad(&ccelulas_rw[q + 6u]);
    }
    var inicio = 0u;
    if total > 0u {
        inicio = atomicAdd(&lista_total[0], total);
    }
    if inicio > contas.cap_listas || total > contas.cap_listas - inicio {
        for (var k = 0u; k < f.celulas; k += 1u) {
            atomicStore(&ccelulas_rw[f.registo + k * REGISTO + 3u], SEM_LISTA);
        }
        return;
    }
    // Os registos passam a `(fundos, inicio, cursor_f, cursor_m, cursor_c)`: cada cursor começa onde a
    // sua família começa e, escritas as listas, acaba onde ela acaba — `(inicio, fim_f, fim_m, fim)`.
    // O fundo de uma célula é o de TODAS as arestas que acabam antes dela: o prefixo ao longo da
    // fileira, em inteiros.
    var acc = inicio;
    var fundo = vec3<i32>(0);
    for (var k = 0u; k < f.celulas; k += 1u) {
        let q = f.registo + k * REGISTO;
        fundo += vec3<i32>(
            bitcast<i32>(atomicLoad(&ccelulas_rw[q])),
            bitcast<i32>(atomicLoad(&ccelulas_rw[q + 1u])),
            bitcast<i32>(atomicLoad(&ccelulas_rw[q + 2u])),
        );
        atomicStore(&ccelulas_rw[q], bitcast<u32>(fundo.x));
        atomicStore(&ccelulas_rw[q + 1u], bitcast<u32>(fundo.y));
        atomicStore(&ccelulas_rw[q + 2u], bitcast<u32>(fundo.z));
        let nf = atomicLoad(&ccelulas_rw[q + 4u]);
        let nm = atomicLoad(&ccelulas_rw[q + 5u]);
        let nc = atomicLoad(&ccelulas_rw[q + 6u]);
        atomicStore(&ccelulas_rw[q + 3u], acc);
        atomicStore(&ccelulas_rw[q + 4u], acc);
        atomicStore(&ccelulas_rw[q + 5u], acc + nf);
        atomicStore(&ccelulas_rw[q + 6u], acc + nf + nm);
        acc += nf + nm + nc;
    }
}

@compute @workgroup_size(64)
fn cs_escreve_listas(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let a = aresta_de(indice(gid, nwg));
    if !a.valida {
        return;
    }
    let rs = fileiras_da_aresta(a);
    for (var r = rs.x; r < rs.y; r += 1u) {
        let f = na_fileira(a.e, a.y0 + f32(r));
        if f.x == 0.0 {
            continue;
        }
        let q0 = a.registo0 + r * a.celulas * REGISTO;
        if atomicLoad(&ccelulas_rw[q0 + 3u]) == SEM_LISTA {
            continue;
        }
        let ks = celulas_do_pedaco(f, a.x0, a.celulas);
        for (var k = ks.x; k < ks.y; k += 1u) {
            let i = atomicAdd(&ccelulas_rw[q0 + k * REGISTO + 4u + a.fam], 1u);
            listas_rw[i] = a.e;
        }
    }
}
