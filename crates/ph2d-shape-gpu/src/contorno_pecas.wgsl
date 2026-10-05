
// ⭐ doc 121 §9.18 (E) — **A EMISSÃO TRACEJADA POR PEÇA.** O fio de cada cópia anda só os TROÇOS de um
// sub-caminho tracejado (o arco, onde o troço começa, o 1.º traço) e escreve uma LINHA por troço; o
// `cs_soma_pecas` faz o prefixo das peças das cópias; o `cs_pecas` corre UM FIO POR PEÇA, que acha a
// cópia e o troço dela por busca binária (o desenho do `aresta_de`) e emite o pedaço; o `cs_fecha` acaba
// cada cópia (o bloco, a caixa, a capacidade). Os números de cada troço são os do passeio, os MESMOS bits
// que o fio da cópia usava; a ordem das arestas não importa (as células somam inteiros, §9.12).
//
// `POR_PECA`: `0` um fio por cópia emite tudo (`emite_tracejado`); `1` a PROVA do modelo — o passeio, a
// tabela e o `pedaco` por peça, SEM arestas; `2` por peça, cada aresta reserva o seu sítio na cópia
// (`atomicAdd`); `3` por peça, contada antes e reservada de uma vez. As ABLAÇÕES da prova: `4` o fio da
// peça sai logo · `5` sai depois das buscas e das leituras · `6` o passeio sem escrever as linhas.
override POR_PECA: u32 = 0u;

// As linhas: por cópia, a partir do bloco onde a reserva dela começa (`2` `vec4` por linha), a CABEÇA — o
// afim, a caneta e o ajuste — e uma linha por troço tracejado: `(troço, início do sub-caminho, k, peças
// antes)` · `(s0, arco, 1.º traço, total do fechado)`. ⚠️ Cabe na reserva: cada troço tracejado reserva
// `≥ 12` arestas (`limite_de_arestas`: `2` peças de `4` e a junta), e a reserva dá uma linha por bloco de `8`.
@group(2) @binding(9) var<storage, read_write> trocos_rw: array<vec4<u32>>;
// Por cópia (`PORCOPIA` palavras): `0` as arestas do contorno já escritas (a reserva atómica) · `1..5` a
// caixa em bits ORDENADOS (`ordena`) · `5` quantas linhas · `6` quantas arestas cabem · `7` onde começam.
@group(2) @binding(10) var<storage, read_write> porcopia_rw: array<atomic<u32>>;
const PORCOPIA: u32 = 8u;
// Os argumentos do despacho de um fio por PEÇA (`despacha`, a seguir aos dois desenhos).
const POR_PECA_ARGS: u32 = 17u;

const EMITE_FILA: u32 = 0u;
const EMITE_CONTA: u32 = 1u;
const EMITE_RESERVA: u32 = 2u;
var<private> modo_emite: u32;
var<private> copia_emite: u32;

var<private> linha0: u32;
var<private> linhas: u32;
var<private> linhas_cap: u32;
var<private> pecas: u32;

// Um `f32` em `u32` com a MESMA ordem (para `atomicMin`/`atomicMax`), e de volta.
fn ordena(x: f32) -> u32 {
    let b = bitcast<u32>(x);
    return select(b | 0x80000000u, ~b, (b & 0x80000000u) != 0u);
}

fn desordena(u: u32) -> f32 {
    return bitcast<f32>(select(~u, u & 0x7fffffffu, (u & 0x80000000u) != 0u));
}

// A cabeça das linhas da cópia cuja reserva começa em `base`.
fn abre_as_linhas(base: u32, reservado: u32, cp: Copia, ajuste: f32) {
    linha0 = 2u * (base / SEGS_POR_BLOCO);
    linhas = 0u;
    linhas_cap = reservado / SEGS_POR_BLOCO - 1u;
    pecas = 0u;
    trocos_rw[linha0] = bitcast<vec4<u32>>(cp.lin);
    trocos_rw[linha0 + 1u] = vec4<u32>(bitcast<u32>(cp.t.x), bitcast<u32>(cp.t.y), cp.eixo_rg.w, bitcast<u32>(ajuste));
}

// O passeio de um sub-caminho tracejado SÓ pelos troços — o de `emite_tracejado`, sem o laço dos pedaços.
fn regista_tracejado(i0: u32, lin: vec4<f32>, t: vec2<f32>, caneta: f32, ajuste: f32) {
    let sub = cabeca_do_tracejado(i0, caneta, ajuste);
    if sub.per <= 0.0 {
        return;
    }
    let primeira = linhas;
    var i = i0;
    var s0 = 0.0;
    for (var k = 0u; k < sub.n; k += 1u) {
        i = proximo_troco(i);
        let len = arco(eixo[i], lin, t);
        let fim = s0 + len;
        let n0 = floor(s0 / sub.per);
        let n1 = min(floor(fim / sub.per), n0 + TRACOS_POR_TROCO_MAX);
        if linhas < linhas_cap && POR_PECA != 6u {
            let r = linha0 + 2u + 2u * linhas;
            trocos_rw[r] = vec4<u32>(i, i0, k, pecas);
            trocos_rw[r + 1u] = vec4<u32>(bitcast<u32>(s0), bitcast<u32>(len), bitcast<u32>(n0), 0u);
        }
        linhas += 1u;
        pecas += u32(n1 - n0) + 1u;
        s0 = fim;
        i += 1u;
    }
    // A emenda de um fechado só a lê o 1.º traço do 1.º troço.
    if sub.fechado && primeira < linhas_cap {
        trocos_rw[linha0 + 3u + 2u * primeira].w = bitcast<u32>(s0);
    }
}

// As peças da cópia `ii` passam a existir para o `cs_pecas`; `false` ⇒ as linhas não couberam.
fn publica_as_linhas(ii: u32, bc: u32, limite: u32) -> bool {
    if POR_PECA == 6u {
        // A ablação: nada publicado, e uma escrita que nunca acontece segura o passeio.
        if pecas == 0xffffffffu {
            contagem[quinto(5u) + ii] = linhas;
        }
        return true;
    }
    if pecas == 0u {
        return true;
    }
    if linhas > linhas_cap {
        return false;
    }
    contagem[quinto(5u) + ii] = pecas;
    let pc = PORCOPIA * ii;
    atomicStore(&porcopia_rw[pc + 5u], linhas);
    if POR_PECA == 2u || POR_PECA == 3u {
        atomicStore(&porcopia_rw[pc], cursor);
        atomicStore(&porcopia_rw[pc + 1u], ordena(cmin.x));
        atomicStore(&porcopia_rw[pc + 2u], ordena(cmin.y));
        atomicStore(&porcopia_rw[pc + 3u], ordena(cmax.x));
        atomicStore(&porcopia_rw[pc + 4u], ordena(cmax.y));
        atomicStore(&porcopia_rw[pc + 6u], limite);
        atomicStore(&porcopia_rw[pc + 7u], bc);
    }
    return true;
}

// O prefixo das peças das cópias (o 6.º quinto) e o despacho de um fio por peça — o desenho do `cs_soma`.
@compute @workgroup_size(256)
fn cs_soma_pecas(@builtin(local_invocation_index) li: u32) {
    let n = contas.n;
    let q = quinto(5u);
    let pedaco = (n + 255u) / 256u;
    let i0 = min(li * pedaco, n);
    let i1 = min(i0 + pedaco, n);
    var s = 0u;
    for (var i = i0; i < i1; i += 1u) {
        s += contagem[q + i];
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
        contagem[q + n] = acc;
        despacha(POR_PECA_ARGS, acc);
    }
    workgroupBarrier();
    var acc = parcial[li];
    for (var i = i0; i < i1; i += 1u) {
        let v = contagem[q + i];
        contagem[q + i] = acc;
        acc += v;
    }
}

@compute @workgroup_size(64)
fn cs_pecas(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let g = indice(gid, nwg);
    let n = contas.n;
    let q = quinto(5u);
    if g >= contagem[q + n] || POR_PECA == 4u {
        return;
    }
    // A cópia: a ÚLTIMA cujo prefixo começa em `≤ g` (uma cópia sem peças perde para a seguinte).
    var lo = 0u;
    var hi = n;
    while hi - lo > 1u {
        let m = (lo + hi) / 2u;
        if contagem[q + m] <= g {
            lo = m;
        } else {
            hi = m;
        }
    }
    let j = g - contagem[q + lo];
    let pc = PORCOPIA * lo;
    let cab = 2u * (contagem[lo] / SEGS_POR_BLOCO);
    // O troço: a ÚLTIMA linha cujas peças antes são `≤ j`.
    var a = 0u;
    var b = atomicLoad(&porcopia_rw[pc + 5u]);
    while b - a > 1u {
        let m = (a + b) / 2u;
        if trocos_rw[cab + 2u + 2u * m].w <= j {
            a = m;
        } else {
            b = m;
        }
    }
    let l0 = trocos_rw[cab + 2u + 2u * a];
    let l1 = bitcast<vec4<f32>>(trocos_rw[cab + 3u + 2u * a]);
    let lin = bitcast<vec4<f32>>(trocos_rw[cab]);
    let h1 = bitcast<vec4<f32>>(trocos_rw[cab + 1u]);
    if POR_PECA == 5u {
        if l1.x + h1.x + lin.x < -1.0e30 {
            contorno_rw[0] = vec4<f32>(f32(l0.x));
        }
        return;
    }
    var sub = cabeca_do_tracejado(l0.y, h1.z, h1.w);
    if sub.fechado && l0.z == 0u {
        fecha_o_tracejado(&sub, l1.w);
    }
    let it = eixo[l0.x];
    let tr = troco_da_linha(it, sub, l0.z, l1.x, l1.y, l1.z, lin, h1.xy);
    let p = pedaco(tr, sub, l1.z + f32(j - l0.w));
    if !p.valido {
        return;
    }
    if POR_PECA == 1u {
        // A PROVA: só o pedaço. Uma escrita que nunca acontece segura o cálculo contra o compilador.
        if p.x1 < p.x0 - 1.0e30 {
            contorno_rw[0] = vec4<f32>(p.x0);
        }
        return;
    }
    cmin = vec2<f32>(3.0e38);
    cmax = vec2<f32>(-3.0e38);
    base_saida = atomicLoad(&porcopia_rw[pc + 7u]);
    let limite = atomicLoad(&porcopia_rw[pc + 6u]);
    if POR_PECA == 2u {
        modo_emite = EMITE_RESERVA;
        copia_emite = lo;
        limite_saida = limite;
        emite_pedaco(it, lin, h1.xy, h1.z, tr, p);
    } else {
        modo_emite = EMITE_CONTA;
        cursor = 0u;
        emite_pedaco(it, lin, h1.xy, h1.z, tr, p);
        if cursor == 0u {
            return;
        }
        let s = atomicAdd(&porcopia_rw[pc], cursor);
        modo_emite = EMITE_FILA;
        cursor = 0u;
        base_saida += s;
        // Passar do limite apaga a cópia inteira no `cs_fecha` (o total atómico fica acima dele).
        limite_saida = select(0u, limite - s, limite > s);
        emite_pedaco(it, lin, h1.xy, h1.z, tr, p);
    }
    if cmin.x <= cmax.x {
        atomicMin(&porcopia_rw[pc + 1u], ordena(cmin.x));
        atomicMin(&porcopia_rw[pc + 2u], ordena(cmin.y));
        atomicMax(&porcopia_rw[pc + 3u], ordena(cmax.x));
        atomicMax(&porcopia_rw[pc + 4u], ordena(cmax.y));
    }
}

// O fim de uma cópia com peças: o que o `escreve` fazia depois do `percorre` — o limite, o bloco inteiro,
// a caixa e o registo.
@compute @workgroup_size(64)
fn cs_fecha(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let ii = indice(gid, nwg);
    let q = quinto(5u);
    if ii >= contas.n || contagem[q + ii + 1u] == contagem[q + ii] {
        return;
    }
    let pc = PORCOPIA * ii;
    let total = atomicLoad(&porcopia_rw[pc]);
    if total > atomicLoad(&porcopia_rw[pc + 6u]) {
        ccopias_rw[3u * ii] = vec4<u32>(0u);
        ccopias_rw[3u * ii + 1u] = vec4<u32>(0u);
        ccopias_rw[3u * ii + 2u] = vec4<u32>(0u);
        pede_a_completa(ii);
        return;
    }
    let bc = atomicLoad(&porcopia_rw[pc + 7u]);
    let nc = (total + SEGS_POR_BLOCO - 1u) / SEGS_POR_BLOCO * SEGS_POR_BLOCO;
    for (var i = total; i < nc; i += 1u) {
        contorno_rw[bc + i] = vec4<f32>(0.0);
    }
    let c0 = ccopias_rw[3u * ii];
    ccopias_rw[3u * ii] = vec4<u32>(c0.x, c0.y, c0.z, nc / SEGS_POR_BLOCO);
    ccaixas_rw[ii] = vec4<f32>(
        desordena(atomicLoad(&porcopia_rw[pc + 1u])),
        desordena(atomicLoad(&porcopia_rw[pc + 2u])),
        desordena(atomicLoad(&porcopia_rw[pc + 3u])),
        desordena(atomicLoad(&porcopia_rw[pc + 4u])),
    );
    if c0.y + c0.z + nc == 0u {
        pede_a_completa(ii);
    }
}
