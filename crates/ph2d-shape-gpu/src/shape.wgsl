// O PASSE DE FORMAS INSTANCIADO (doc 121 do Motion) — N cópias de geometrias vectoriais numa
// chamada, com a área de cada pixel calculada EXACTAMENTE como o rasterizador fino do Vello.
//
// A conta da área (`contribuicao`) é portada de `vello_shaders/shader/fine.wgsl` (`fill_path`,
// o modo `AaConfig::Area` que esta casa usa):
//   Copyright 2022 the Vello Authors — SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// A única mudança é a de CONTEXTO: o Vello corre por LADRILHO, com um `backdrop` e um `y_edge` que
// carregam a contribuição dos segmentos cortados à esquerda do ladrilho; aqui cada pixel soma TODOS
// os segmentos da geometria, e um segmento que acaba à esquerda do pixel dá `a = 1` pela mesma
// fórmula (`xmax < 0`), logo os dois termos de ladrilho desaparecem.

struct View {
    // pixel = lin · mundo + t, com lin = [[a, c], [b, d]] guardado como (a, b, c, d).
    lin: vec4<f32>,
    t: vec2<f32>,
    alvo: vec2<f32>,
}

struct Instance {
    pos: vec2<f32>,
    size: vec2<f32>,
    basis: vec4<f32>,
    anchor: vec2<f32>,
    geometry: u32,
    _pad: u32,
    tint: vec4<f32>,
}

struct Record {
    bbox: vec4<f32>,
    stroke_color: vec4<f32>,
    tol: array<vec4<f32>, 2>,
    ranges: array<vec4<u32>, 8>,
    // Por nível: eixo_start, eixo_count, marcas_count, 0 (doc 121 W4).
    eixo: array<vec4<u32>, 8>,
    eixo_bbox: vec4<f32>,
    flags: u32,
    ext_fora: f32,
    _pad0: u32,
    _pad1: u32,
}

// Um item do EIXO do traço (`eixo.rs`): troço a→b · junta em b entre a→b e b→c · ponta em b.
struct Eixo {
    a: vec2<f32>,
    b: vec2<f32>,
    c: vec2<f32>,
    d: vec2<f32>,
    meia: f32,
    limite: f32,
    tipo: u32,
    junta: u32,
    ponta: u32,
    _pad: u32,
    // doc 121 §9.9 — o tracejado em unidades LOCAIS (`traço`, `vão`); `(0, 0)` ⇒ contínuo.
    traco: f32,
    vao: f32,
    // A flecha da corda (local): o meio da curva menos o meio da corda.
    flecha: vec2<f32>,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> instances: array<Instance>;
@group(0) @binding(2) var<storage, read> records: array<Record>;
@group(0) @binding(3) var<storage, read> segs: array<vec4<f32>>;
// Os handles de geometria, ORDENADOS — a posição de um handle é o índice do registo dele.
@group(0) @binding(4) var<storage, read> handles: array<u32>;
@group(0) @binding(5) var<storage, read> eixo: array<Eixo>;

// Um bloco de `SEGS_POR_BLOCO` segmentos (`blocos.rs`): a caixa LOCAL e se é uma corrente ligada.
struct Bloco {
    caixa: vec4<f32>,
    encadeado: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}
const SEGS_POR_BLOCO: u32 = 8u;
@group(0) @binding(6) var<storage, read> blocos: array<Bloco>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    // O afim local → pixel da cópia: (a, b, c, d) e a translação.
    @location(0) @interpolate(flat) lin: vec4<f32>,
    @location(1) @interpolate(flat) t: vec2<f32>,
    @location(2) @interpolate(flat) fill: vec2<u32>,
    @location(3) @interpolate(flat) stroke: vec2<u32>,
    @location(4) @interpolate(flat) tint: vec4<f32>,
    @location(5) @interpolate(flat) stroke_color: vec4<f32>,
    @location(6) @interpolate(flat) even_odd: u32,
    // doc 121 W4: a cópia é NÃO conforme e o traço dela sai do EIXO (x = início, y = contagem), com
    // as marcas (z) e a caneta `√|det|` (w, em bits). `y = 0` ⇒ o caminho conforme de sempre.
    @location(7) @interpolate(flat) eixo_rg: vec4<u32>,
    // doc 121 §9.6: `ii + 1` quando a cópia tem as arestas no ECRÃ (`cobertura_de_ecra`); `0` ⇒ o
    // caminho de sempre, com os segmentos LOCAIS e o traço do eixo pixel a pixel.
    @location(8) @interpolate(flat) tela: u32,
}

// ⭐ doc 121 §9.5–§9.12 — **AS ARESTAS DE CADA CÓPIA, NO ECRÃ, CALCULADAS UMA VEZ** (`contorno.wgsl`
// escreve-as, o desenho só as lê): por cópia TRÊS `vec4` — `(primeiro bloco, blocos do
// preenchimento, das marcas, do contorno)`, `(primeira célula, linhas, 0, a primeira linha em bits
// de f32)` e `(a primeira coluna em bits, células por linha, a regra, 0)`; a caixa da cópia inteira;
// e a acumulação das células, cuja 1.ª palavra de cada pixel o cálculo deixou com a COBERTURA acabada
// (`pack2x16unorm` do preenchimento e do traço).
@group(1) @binding(0) var<storage, read> ccopias: array<vec4<u32>>;
@group(1) @binding(1) var<storage, read> ccaixas: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read> ccobertura: array<u32>;

// O índice do registo de um handle, ou `0xffffffff` se a geometria não existe.
fn registo_de(handle: u32) -> u32 {
    var lo = 0u;
    var hi = arrayLength(&handles);
    while lo < hi {
        let meio = (lo + hi) / 2u;
        let h = handles[meio];
        if h == handle {
            return meio;
        }
        if h < handle {
            lo = meio + 1u;
        } else {
            hi = meio;
        }
    }
    return 0xffffffffu;
}

fn aplica(lin: vec4<f32>, t: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(lin.x * p.x + lin.z * p.y, lin.y * p.x + lin.w * p.y) + t;
}

// ⭐ doc 121 §9.5 — **O QUE UMA CÓPIA É, NO ECRÃ**: o registo da geometria, o afim local → pixel, o
// nível de aplanamento e o eixo do traço. Lido pelo shader de vértice E pelo cálculo do CONTORNO
// (`contorno.wgsl`) — escrito duas vezes, os dois escolheriam níveis diferentes no dia em que um
// mudasse, e o contorno calculado seria o de outra forma.
struct Copia {
    // `0xffffffff` ⇒ a geometria não existe.
    r: u32,
    nivel: u32,
    lin: vec4<f32>,
    t: vec2<f32>,
    // x = início, y = contagem, z = marcas, w = a caneta `√|det|` em bits; `y = 0` ⇒ sem eixo.
    eixo_rg: vec4<u32>,
}

fn copia_de(ii: u32) -> Copia {
    var cp: Copia;
    let inst = instances[ii];
    cp.r = registo_de(inst.geometry);
    if cp.r == 0xffffffffu {
        return cp;
    }
    let rec = records[cp.r];
    // mundo = pos + basis · (anchor + q · size) — a MESMA pose da sprite e do `instance_pose`.
    let b = inst.basis;
    let w_lin = vec4<f32>(b.x * inst.size.x, b.y * inst.size.x, b.z * inst.size.y, b.w * inst.size.y);
    let w_t = inst.pos + vec2<f32>(b.x * inst.anchor.x + b.z * inst.anchor.y, b.y * inst.anchor.x + b.w * inst.anchor.y);
    let v = view.lin;
    let lin = vec4<f32>(
        v.x * w_lin.x + v.z * w_lin.y,
        v.y * w_lin.x + v.w * w_lin.y,
        v.x * w_lin.z + v.z * w_lin.w,
        v.y * w_lin.z + v.w * w_lin.w,
    );
    let t = aplica(view.lin, view.t, w_t);
    // A maior escala do afim (o maior valor singular): quantos pixels vale uma unidade local.
    let e = 0.5 * (lin.x + lin.w);
    let f = 0.5 * (lin.x - lin.w);
    let g = 0.5 * (lin.y + lin.z);
    let h = 0.5 * (lin.y - lin.z);
    let escala = sqrt(e * e + h * h) + sqrt(f * f + g * g);
    // O nível mais GROSSO cujo erro no ecrã cabe na tolerância do Vello (0,25 px).
    var nivel = 7u;
    for (var k = 0u; k < 8u; k += 1u) {
        if rec.tol[k / 4u][k % 4u] * escala <= 0.25 {
            nivel = k;
            break;
        }
    }
    // ⭐ doc 121 W4 — **CONFORME?** A mesma pergunta do `stroke_uniform::is_conformal` da casa: as
    // duas colunas do afim com o mesmo comprimento e perpendiculares. Sob escala NÃO uniforme o
    // traço sai do eixo, com a caneta REDONDA de largura `w·√|det|` (bug #27).
    let l1 = lin.x * lin.x + lin.y * lin.y;
    let l2 = lin.z * lin.z + lin.w * lin.w;
    let esc2 = max(max(l1, l2), 1.0e-30);
    let conforme = abs(l1 - l2) <= esc2 * 1.0e-5 && abs(lin.x * lin.z + lin.y * lin.w) <= esc2 * 1.0e-5;
    let caneta = sqrt(abs(lin.x * lin.w - lin.z * lin.y));
    let ex = rec.eixo[nivel];
    var eixo_rg = vec4<u32>(0u, 0u, 0u, 0u);
    if !conforme && ex.y > 0u && (rec.flags & 2u) == 0u {
        eixo_rg = vec4<u32>(ex.x, ex.y, ex.z, bitcast<u32>(caneta));
    }
    cp.nivel = nivel;
    cp.lin = lin;
    cp.t = t;
    cp.eixo_rg = eixo_rg;
    return cp;
}

// A caixa no ECRÃ que cobre tudo o que a cópia pode desenhar, sem a percorrer: a caixa LOCAL da
// forma pelo afim e, com o traço do eixo, a caixa dos pontos do eixo alargada por `ext_fora × caneta`
// (a caixa local do contorno expandido não o cobre sob escala não uniforme). Lida pelo quad do
// caminho de sempre e pelas LINHAS das células (`contorno.wgsl`) — as duas passagens de cálculo
// têm de chegar ao MESMO número de linhas.
fn caixa_estimada(cp: Copia, rec: Record) -> vec4<f32> {
    let lin = cp.lin;
    let t = cp.t;
    let c0 = aplica(lin, t, rec.bbox.xy);
    let c1 = aplica(lin, t, rec.bbox.zy);
    let c2 = aplica(lin, t, rec.bbox.xw);
    let c3 = aplica(lin, t, rec.bbox.zw);
    var lo = min(min(c0, c1), min(c2, c3));
    var hi = max(max(c0, c1), max(c2, c3));
    if cp.eixo_rg.y > 0u {
        let e0 = aplica(lin, t, rec.eixo_bbox.xy);
        let e1 = aplica(lin, t, rec.eixo_bbox.zy);
        let e2 = aplica(lin, t, rec.eixo_bbox.xw);
        let e3 = aplica(lin, t, rec.eixo_bbox.zw);
        let m = vec2<f32>(rec.ext_fora * bitcast<f32>(cp.eixo_rg.w));
        lo = min(lo, min(min(e0, e1), min(e2, e3)) - m);
        hi = max(hi, max(max(e0, e1), max(e2, e3)) + m);
    }
    return vec4<f32>(lo, hi);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    var out: VsOut;
    let cp = copia_de(ii);
    if cp.r == 0xffffffffu {
        // Sem geometria: um triângulo degenerado, fora de tudo.
        out.pos = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        return out;
    }
    let inst = instances[ii];
    let rec = records[cp.r];
    let lin = cp.lin;
    let t = cp.t;
    let rg = rec.ranges[cp.nivel];
    let eixo_rg = cp.eixo_rg;
    // O quad: a caixa da forma no ECRÃ, arredondada PARA FORA ao pixel inteiro.
    // ⚠️ **Sem margem, e é medido:** a caixa é a dos SEGMENTOS aplanados, e um pixel só tem
    // cobertura se um segmento (ou o interior entre eles) lhe toca — logo o `floor`/`ceil` já
    // inclui todo pixel de borda. A 1.ª redacção alargava um pixel de cada lado, e a mutação que o
    // apagava SOBREVIVEU à paridade de pixel: não era lei, era trabalho a mais.
    var caixa = caixa_estimada(cp, rec);
    // ⭐ doc 121 §9.6: as arestas desta cópia já estão no ECRÃ (`contorno.wgsl`) — a caixa delas é a
    // exacta, e o fragmento lê-as pelas células em vez de as refazer.
    var tela = 0u;
    let c0 = ccopias[3u * ii];
    if c0.y + c0.z + c0.w > 0u {
        caixa = ccaixas[ii];
        tela = ii + 1u;
    }
    let lo = floor(caixa.xy);
    let hi = ceil(caixa.zw);
    var canto = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let px = mix(lo, hi, canto[vi]);
    out.pos = vec4<f32>(px.x / view.alvo.x * 2.0 - 1.0, 1.0 - px.y / view.alvo.y * 2.0, 0.0, 1.0);
    out.lin = lin;
    out.t = t;
    out.fill = rg.xy;
    out.stroke = rg.zw;
    out.tint = inst.tint;
    out.stroke_color = rec.stroke_color;
    out.even_odd = rec.flags & 1u;
    out.eixo_rg = eixo_rg;
    out.tela = tela;
    return out;
}

// A contribuição de UM segmento (em pixels) para a área do pixel cujo canto é `xy` — portada do
// `fill_path` do Vello (ver o cabeçalho).
fn contribuicao(p0: vec2<f32>, p1: vec2<f32>, xy: vec2<f32>) -> f32 {
    let y = p0.y - xy.y;
    let delta = p1 - p0;
    let y0 = clamp(y, 0.0, 1.0);
    let y1 = clamp(y + delta.y, 0.0, 1.0);
    let dy = y0 - y1;
    if dy == 0.0 {
        return 0.0;
    }
    // ⭐ doc 121 §9.5 — **os dois lados sem divisão**: com os DOIS pontos à esquerda do pixel o
    // troço recortado também está (o `x` dele fica entre os dos pontos), e a contribuição é a faixa
    // inteira — o mesmo `dy` que o ramo de baixo devolve, sem o recíproco; com os dois à DIREITA é
    // zero (a fórmula de baixo daria `~1e-6/(xmax − 1)`, o resto do `−1e-6` do Vello).
    let px0 = p0.x - xy.x;
    let px1 = p1.x - xy.x;
    if max(px0, px1) <= 0.0 {
        return dy;
    }
    if min(px0, px1) >= 1.0 {
        return 0.0;
    }
    let vec_y_recip = 1.0 / delta.y;
    let t0 = (y0 - y) * vec_y_recip;
    let t1 = (y1 - y) * vec_y_recip;
    let startx = p0.x - xy.x;
    let x0 = startx + t0 * delta.x;
    let x1 = startx + t1 * delta.x;
    let xmin0 = min(x0, x1);
    let xmax0 = max(x0, x1);
    // ⛔ doc 121 W4 (a linha da `=127`): o segmento todo à ESQUERDA do pixel conta a faixa inteira.
    // A fórmula abaixo dá `1` aí só enquanto `xmax − xmin ≠ 0` — e o `−1e-6` que a garante é do
    // Vello, cujas coordenadas são relativas a um ladrilho de 16 px. Aqui elas são relativas ao
    // PIXEL e chegam a centenas: numa aresta VERTICAL longe à esquerda o `−1e-6` perde-se no `f32`,
    // a conta vira `0/0 = NaN`, e o `min(abs(NaN), 1)` pinta a fileira inteira.
    if xmax0 <= 0.0 {
        return dy;
    }
    let xmin = min(xmin0, 1.0) - 1.0e-6;
    let xmax = xmax0;
    let b = min(xmax, 1.0);
    let c = max(b, 0.0);
    let d = max(xmin, 0.0);
    let a = (b + 0.5 * (d * d - c * c) - xmin) / (xmax - xmin);
    return a * dy;
}

// A caixa no ECRÃ de uma caixa LOCAL `(x0, y0, x1, y1)`: o centro pelo afim e a meia-extensão
// pelo valor absoluto dele — a mesma caixa que os quatro cantos dão, com uma transformação só.
fn caixa_no_ecra(lin: vec4<f32>, t: vec2<f32>, c: vec4<f32>) -> vec4<f32> {
    let m = aplica(lin, t, 0.5 * (c.xy + c.zw));
    let h = 0.5 * (c.zw - c.xy);
    let e = vec2<f32>(abs(lin.x) * h.x + abs(lin.z) * h.y, abs(lin.y) * h.x + abs(lin.w) * h.y);
    return vec4<f32>(m - e, m + e);
}

// ⭐ doc 121 §9.3 — **a área por BLOCOS** (`blocos.rs`): um bloco acima, abaixo ou à direita do
// pixel soma zero e salta-se; um todo à ESQUERDA e encadeado soma `clamp(y₀) − clamp(yₙ)` (a
// contribuição de um segmento à esquerda é a faixa dele, e numa corrente ela telescopa); o resto,
// segmento a segmento. `inicio` e `n` são múltiplos de `SEGS_POR_BLOCO`.
fn area(inicio: u32, n: u32, lin: vec4<f32>, t: vec2<f32>, xy: vec2<f32>) -> f32 {
    var s = 0.0;
    let b0 = inicio / SEGS_POR_BLOCO;
    let b1 = (inicio + n) / SEGS_POR_BLOCO;
    for (var b = b0; b < b1; b += 1u) {
        let bl = blocos[b];
        let cx = caixa_no_ecra(lin, t, bl.caixa);
        if cx.w <= xy.y || cx.y >= xy.y + 1.0 || cx.x >= xy.x + 1.0 {
            continue;
        }
        let i0 = b * SEGS_POR_BLOCO;
        if cx.z <= xy.x && bl.encadeado != 0u {
            let p0 = aplica(lin, t, segs[i0].xy);
            let p1 = aplica(lin, t, segs[i0 + SEGS_POR_BLOCO - 1u].zw);
            s += clamp(p0.y - xy.y, 0.0, 1.0) - clamp(p1.y - xy.y, 0.0, 1.0);
            continue;
        }
        for (var i = i0; i < i0 + SEGS_POR_BLOCO; i += 1u) {
            let seg = segs[i];
            s += contribuicao(aplica(lin, t, seg.xy), aplica(lin, t, seg.zw), xy);
        }
    }
    return s;
}

// ⭐ doc 121 W4 — **O TRAÇO DE UMA CÓPIA NÃO CONFORME, construído no ECRÃ a partir do eixo.** Cada
// peça (o quadrilátero de um troço, a junta de uma quina, a ponta de um extremo) soma a área dela com
// a orientação POSITIVA, e a soma passa pela regra não-nula — a mesma com que o Vello preenche o
// contorno que o kurbo expande. As peças do lado de FORA não se sobrepõem (a junta vive entre as
// duas normais, a ponta à frente do fim), logo a borda anti-serrilhada é a da área exacta.

fn orienta(v: f32, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
    let x = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    return select(-v, v, x >= 0.0);
}

fn tri(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, xy: vec2<f32>) -> f32 {
    let v = contribuicao(a, b, xy) + contribuicao(b, c, xy) + contribuicao(c, a, xy);
    return orienta(v, a, b, c);
}

// Um quadrilátero CONVEXO a→b→c→d.
fn quad(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, d: vec2<f32>, xy: vec2<f32>) -> f32 {
    let v = contribuicao(a, b, xy) + contribuicao(b, c, xy) + contribuicao(c, d, xy)
        + contribuicao(d, a, xy);
    return orienta(v, a, b, c);
}

fn perp(u: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(-u.y, u.x);
}

// Um leque de `centro` sobre o arco de `n0` a `n_fim` (`cos_alpha` = o cosseno do ângulo entre
// os dois, no sentido `dir`), com os passos que mantêm a flecha do arco em `0,25 px` (a tolerância
// do Vello).
//
// ⭐ **Sem trigonometria no caso comum** (doc 121 §9.3): o passo máximo `2·acos(1 − 0,25/r)` cabe
// num teste de COSSENO (`cos(2·acos q) = 2q² − 1`), e um arco que cabe num passo é o triângulo
// `centro, n0, n_fim` — o fim já é conhecido, não se roda nada. Numa caneta de um pixel quase toda
// junta é assim, e no proxy de telemóvel a junta redonda era o grosso do traço esticado. Quando o
// arco pede `k > 1` passos, roda-se por UM par `cos/sin` (o passo) em vez de um por passo, e o
// último ponto é `n_fim` exacto.
fn leque(centro: vec2<f32>, n0: vec2<f32>, n_fim: vec2<f32>, cos_alpha: f32, dir: f32, r: f32, xy: vec2<f32>) -> f32 {
    if r <= 0.25 {
        return tri(centro, centro + n0, centro + n_fim, xy);
    }
    let q = 1.0 - 0.25 / r;
    if cos_alpha >= 2.0 * q * q - 1.0 {
        return tri(centro, centro + n0, centro + n_fim, xy);
    }
    let alpha = acos(clamp(cos_alpha, -1.0, 1.0));
    let passo_max = 2.0 * acos(q);
    let k = u32(clamp(ceil(alpha / max(passo_max, 1.0e-4)), 1.0, 64.0));
    let ang = dir * alpha / f32(k);
    let c = cos(ang);
    let sn = sin(ang);
    var s = 0.0;
    var v = n0;
    var p = centro + n0;
    for (var j = 1u; j < k; j += 1u) {
        v = vec2<f32>(c * v.x - sn * v.y, sn * v.x + c * v.y);
        let w = centro + v;
        s += tri(centro, p, w, xy);
        p = w;
    }
    return s + tri(centro, p, centro + n_fim, xy);
}

// A largura de uma CÉLULA, em pixels (doc 121 §9.6; re-medida para as listas no §9.8). A altura é UMA
// fileira: mais alta piora em monotonia (§9.7, recusa medida).
const PIXELS_DA_CELULA: u32 = 32u;
const LARGURA_DA_CELULA: f32 = f32(PIXELS_DA_CELULA);
// ⭐ doc 121 §9.12 — as palavras de um REGISTO de célula: os três FUNDOS (preenchimento, marcas,
// contorno, em ponto fixo) e a regra da cópia.
const REGISTO: u32 = 4u;
// As palavras de ACUMULAÇÃO de uma célula: as três famílias, cada uma com um depósito por pixel.
const ACUMULA: u32 = 3u * PIXELS_DA_CELULA;
// ⭐ doc 121 §9.8 — **as somas das células em PONTO FIXO**: o fundo e os depósitos são somados por muitos
// fios ao mesmo tempo (atómicos de inteiros) — em inteiros a soma não depende da ordem, e a imagem não
// depende do escalonamento. `2¹⁶` por unidade de cobertura: cada parcela erra `≤ 7,6e-6`, e um pixel
// soma dezenas, longe dos `1/255` de um passo de alfa.
const ESCALA_FIXA: f32 = 65536.0;

fn fixo(v: f32) -> i32 {
    return i32(round(v * ESCALA_FIXA));
}

// ⭐⭐ doc 121 §9.6–§9.12 — **A COBERTURA PELAS CÉLULAS**: cada fileira da cópia é partida em células
// de `PIXELS_DA_CELULA` px, e o cálculo (`contorno.wgsl`) deixou em cada pixel delas a cobertura
// ACABADA — o preenchimento com a regra da cópia e o traço (as marcas mais o contorno do eixo). O pixel
// faz UMA leitura; a mistura continua aqui, no hardware, na ordem das cópias.
fn cobertura_de_ecra(ii: u32, xy: vec2<f32>) -> vec2<f32> {
    let c1 = ccopias[3u * ii + 1u];
    let c2 = ccopias[3u * ii + 2u];
    let r = xy.y - bitcast<f32>(c1.w);
    let x = xy.x - bitcast<f32>(c2.x);
    if r < 0.0 || r >= f32(c1.y) || x < 0.0 || x >= f32(c2.y) * LARGURA_DA_CELULA {
        return vec2<f32>(0.0);
    }
    // A coluna `x` corre a fileira inteira: a célula dela e, dentro da célula, a palavra do pixel.
    let xi = u32(x);
    let cel = c1.x + u32(r) * c2.y + xi / PIXELS_DA_CELULA;
    return unpack2x16unorm(ccobertura[cel * ACUMULA + xi % PIXELS_DA_CELULA]);
}

fn traco_do_eixo(inicio: u32, n: u32, lin: vec4<f32>, t: vec2<f32>, caneta: f32, xy: vec2<f32>) -> f32 {
    // ⭐ Cada peça é FECHADA, logo uma que não toca no pixel soma ZERO (as faixas das arestas à
    // esquerda cancelam-se). ⇒ salta-se pela caixa: primeiro a do BLOCO (`ITEM_BLOCO`, as peças
    // seguintes), depois a de cada peça — as duas no ECRÃ e alargadas pelo que a peça vai para fora
    // do eixo NESTA cópia. ⛔ No espaço local o teste teria de alargar pela PIOR direcção do afim, e
    // sob escala não uniforme isso engolia meia estrela (medido: `1,09 → 0,99 ms`, quase nada).
    // No ecrã: `1,34 → 0,40 ms`, contra `0,37` do Vello (ver `PECAS_POR_BLOCO`).
    let xa = xy;
    let xb = xy + vec2<f32>(1.0);
    var s = 0.0;
    var i = 0u;
    // O ajuste do tracejado é da CÓPIA (o contorno mais longo dela): calcula-se ao 1.º tracejado.
    var ajuste = 0.0;
    loop {
        if i >= n {
            break;
        }
        let cab = eixo[inicio + i];
        let fim = i + 1u + cab._pad;
        let cx = caixa_no_ecra(lin, t, vec4<f32>(cab.a, cab.b));
        let fb = cab.meia * caneta + FAIXA_FOLGA;
        let blo = cx.xy - vec2<f32>(fb);
        let bhi = cx.zw + vec2<f32>(fb);
        // ⚠️ Um bloco com troços TRACEJADOS não se salta: o comprimento de arco passa por todos.
        if (cab.ponta & 1u) == 0u && (bhi.x < xa.x || blo.x > xb.x || bhi.y < xa.y || blo.y > xb.y) {
            i = fim;
            continue;
        }
        i += 1u;
        for (; i < fim; i += 1u) {
            let it = eixo[inicio + i];
            if !tracejado(it) {
                s += peca_do_eixo(it, lin, t, caneta, xy);
            } else if (it.ponta & SUB_INICIO) != 0u {
                if ajuste == 0.0 {
                    ajuste = ajuste_do_tracejado(inicio, n, lin, t, caneta);
                }
                s += tracejado_px(inicio + i, lin, t, caneta, ajuste, xy);
            }
        }
    }
    return min(abs(s), 1.0);
}

// Quantas meias larguras a peça vai para FORA do eixo: a esquadria até ao limite (só num troço que
// chega a uma quina em esquadria), a ponta quadrada até `√2`, o resto até `1`. ⚠️ O mesmo teste que o
// `eixo::alcanca_a_esquadria` faz para a caixa do bloco.
fn alcance_da_peca(it: Eixo) -> f32 {
    if it.tipo == 0u && it.junta == 0u && (it.ponta & 12u) != 0u {
        return max(it.limite, 1.0);
    }
    if it.tipo == 2u && it.ponta == 1u {
        return 1.5;
    }
    return 1.0;
}

// Quanto a bissectriz de um vértice LISO pode sair do arco verdadeiro, em pixels: a faixa troca o
// leque redondo pela esquadria só quando ela fica a esta distância do contorno exacto. É o que as
// caixas das peças (`peca_do_eixo`) e dos blocos (`traco_do_eixo`) alargam além da caneta.
const FAIXA_FOLGA: f32 = 0.1;

// A BISSECTRIZ do vértice `p1` entre `p0 → p1` e `p1 → p2`, do lado `+perp(u)`, ou `z = 0` quando
// ela não serve. Num ponto LISO (`quina = false`) serve se a esquadria fica a `FAIXA_FOLGA` do arco
// verdadeiro; numa QUINA só se a junta autorada é a esquadria (`junta == 0`) dentro do `limite` — e
// então ela É a junta, ao vértice. Nos dois casos não pode recuar mais de metade de um dos troços
// pelo lado de dentro (o quadrilátero deixaria de ser convexo).
//
// ⚠️ Os DOIS troços que se encontram em `p1` chamam isto com os MESMOS argumentos, e é isso que faz a
// aresta partilhada cancelar-se: a decisão e a bissectriz saem iguais nos dois lados, e uma junta
// que um desse e o outro não ficaria como uma cunha por pintar.
fn bissectriz(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, r: f32, quina: bool, junta: u32, limite: f32) -> vec3<f32> {
    return bissectriz_ate(p0, p1, p2, r, quina, junta, limite, 0.5 * min(length(p1 - p0), length(p2 - p1)));
}

// A mesma, com o recuo máximo dado: num traço TRACEJADO (doc 121 §9.9) o que limita o recuo são os
// PEDAÇOS de traço dos dois lados do vértice, e não os troços inteiros.
fn bissectriz_ate(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, r: f32, quina: bool, junta: u32, limite: f32, recuo_max: f32) -> vec3<f32> {
    let d0 = p1 - p0;
    let d1 = p2 - p1;
    let l0 = length(d0);
    let l1 = length(d1);
    if l0 <= 0.0 || l1 <= 0.0 {
        return vec3<f32>(0.0);
    }
    let u0 = d0 / l0;
    let u1 = d1 / l1;
    let dt = dot(u0, u1);
    if quina {
        // A esquadria do Vello: `1 / cos(θ/2) ≤ limite` ⇔ `2 ≤ (1 + cos θ)·limite²`.
        if junta != 0u || 2.0 > (1.0 + dt) * limite * limite {
            return vec3<f32>(0.0);
        }
    } else if dt <= 0.0 {
        return vec3<f32>(0.0);
    }
    let m = (perp(u0) + perp(u1)) * (r / (1.0 + dt));
    let fora = r + FAIXA_FOLGA;
    let recuo = r * sqrt(max(1.0 - dt, 0.0) / (1.0 + dt));
    if (!quina && dot(m, m) > fora * fora) || recuo > recuo_max {
        return vec3<f32>(0.0);
    }
    return vec3<f32>(m, 1.0);
}

// A junta em `b` entre `b − u` e `b → c`, do lado de FORA, com o estilo `junta` (`0` esquadria ·
// `1` chanfro · `2` redonda). Quem a chama é o troço que CHEGA a um vértice que a faixa não cobre.
fn junta_em(u: vec2<f32>, b: vec2<f32>, c: vec2<f32>, r: f32, junta: u32, limite: f32, xy: vec2<f32>) -> f32 {
    let dbc = c - b;
    let lbc = length(dbc);
    if lbc <= 0.0 {
        return 0.0;
    }
    let v = dbc / lbc;
    let cr = u.x * v.y - u.y * v.x;
    let dt = clamp(dot(u, v), -1.0, 1.0);
    if abs(cr) < 1.0e-7 && dt > 0.0 {
        return 0.0;
    }
    // O lado de FORA é o oposto ao da viragem.
    let lado = select(1.0, -1.0, cr > 0.0);
    let n0 = perp(u) * r * lado;
    let n1 = perp(v) * r * lado;
    if junta == 0u && 2.0 <= (1.0 + dt) * limite * limite {
        let m = b + (n0 + n1) / (1.0 + dt);
        return quad(b, b + n0, m, b + n1, xy);
    }
    if junta == 2u {
        var dir = sign(n0.x * n1.y - n0.y * n1.x);
        if dir == 0.0 {
            dir = sign(n0.x * u.y - n0.y * u.x);
        }
        return leque(b, n0, n1, dt, dir, r, xy);
    }
    return tri(b, b + n0, b + n1, xy);
}

// Uma peça do eixo, no ecrã: o quadrilátero de um troço, a junta de uma quina, a ponta de um extremo.
fn peca_do_eixo(it: Eixo, lin: vec4<f32>, t: vec2<f32>, caneta: f32, xy: vec2<f32>) -> f32 {
    let r = it.meia * caneta;
    let a = aplica(lin, t, it.a);
    let b = aplica(lin, t, it.b);
    let fp = r * alcance_da_peca(it) + FAIXA_FOLGA;
    let plo = min(a, b) - vec2<f32>(fp);
    let phi = max(a, b) + vec2<f32>(fp);
    if phi.x < xy.x || plo.x > xy.x + 1.0 || phi.y < xy.y || plo.y > xy.y + 1.0 {
        return 0.0;
    }
    let dab = b - a;
    let lab = length(dab);
    if lab <= 0.0 || r <= 0.0 {
        return 0.0;
    }
    let u = dab / lab;
    if it.tipo == 0u {
        // ⭐ A FAIXA: o troço acaba na bissectriz que o vizinho também usa, e a aresta partilhada
        // cancela-se — sem peça de junta nenhuma. Onde ela não serve, a normal simples, e quem
        // CHEGA ao vértice põe a junta: a autorada numa quina, a redonda num ponto liso.
        let nr = perp(u) * r;
        var m0 = nr;
        var m1 = nr;
        var s = 0.0;
        if (it.ponta & 1u) != 0u {
            let e = bissectriz(aplica(lin, t, it.d), a, b, r, (it.ponta & 4u) != 0u, it.junta, it.limite);
            if e.z > 0.0 {
                m0 = e.xy;
            }
        }
        if (it.ponta & 2u) != 0u {
            let quina = (it.ponta & 8u) != 0u;
            let cf = aplica(lin, t, it.c);
            let e = bissectriz(a, b, cf, r, quina, it.junta, it.limite);
            if e.z > 0.0 {
                m1 = e.xy;
            } else {
                s = junta_em(u, b, cf, r, select(2u, it.junta, quina), it.limite, xy);
            }
        }
        return s + quad(a + m0, b + m1, b - m1, a - m0, xy);
    }
    let nr = perp(u) * r;
    if it.ponta == 1u {
        return quad(b + nr, b + nr + u * r, b - nr + u * r, b - nr, xy);
    }
    if it.ponta == 2u {
        let dir = sign(nr.x * u.y - nr.y * u.x);
        return leque(b, nr, -nr, -1.0, dir, r, xy);
    }
    return 0.0;
}

// ⭐⭐ doc 121 §9.9 — **O TRACEJADO NO ECRÃ.** A lei da casa traceja a geometria JÁ transformada com o
// padrão `× √|det|` (`stroke_uniform::pen_for`) e o `kurbo::dash` corta-o pelo comprimento de arco: a
// fase recomeça em cada sub-caminho, cada traço tem as suas pontas, e num fechado o último traço
// EMENDA no primeiro quando atravessa o início. Aqui cada sub-caminho é percorrido somando o
// comprimento de cada troço no ecrã; o traço `n` ocupa `[n·período, n·período + traço]`.
//
// ⚠️ Os dois troços de um vértice decidem a faixa com os MESMOS números (o comprimento de cada troço
// sai de `comprimento`, com os mesmos argumentos dos dois lados, e o arco é uma soma sequencial) —
// uma decisão diferente deixava uma cunha por pintar.
const SUB_INICIO: u32 = 16u;
const SUB_FECHADO: u32 = 32u;
// Quantos traços um troço corta, no máximo — o tecto é o do VIGIA do dispositivo (um laço sem fim
// perde a placa), não uma escolha de desenho: `2¹⁶` traços num troço é um traço abaixo do pixel.
const TRACOS_POR_TROCO_MAX: f32 = 65536.0;

// ⭐ doc 121 §9.10 — **a variante ENXUTA**: com `TRACEJADO = false` o driver apaga todo o ramo do
// tracejado. Inline, ele dobrava os registos do fragmento e do `cs_escreve` (iGPU: `56 → 128` VGPRs,
// `18 → 8` ondas por SIMD) e toda a cena pagava, com ou sem tracejado. O passe escolhe a variante
// pelo eixo carregado (`EixoItem::tracejado`).
override TRACEJADO: bool = true;

fn tracejado(it: Eixo) -> bool {
    return TRACEJADO && it.traco + it.vao > 0.0;
}

fn comprimento(lin: vec4<f32>, t: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    return length(aplica(lin, t, b) - aplica(lin, t, a));
}

// O ARCO do troço no ecrã — a corda mais `8h²/3c`, com `h` a flecha levada pelo afim (o arco de uma
// parábola pela corda e pelo ponto do meio). É ele que anda o tracejado: só com as cordas, um canto
// arredondado ficava curto e o padrão escorregava ao longo do contorno (medido: alfa `134` → ver
// doc 121 §9.9). As decisões de faixa ficam nas CORDAS, que os dois lados de um vértice recalculam.
fn arco(it: Eixo, lin: vec4<f32>, t: vec2<f32>) -> f32 {
    let d = aplica(lin, t, it.b) - aplica(lin, t, it.a);
    let c = length(d);
    if c <= 0.0 {
        return 0.0;
    }
    let f = vec2<f32>(lin.x * it.flecha.x + lin.z * it.flecha.y, lin.y * it.flecha.x + lin.w * it.flecha.y);
    let h = (d.x * f.y - d.y * f.x) / c;
    return c + 8.0 * h * h / (3.0 * c);
}

// O primeiro TROÇO a partir de `i` (salta os cabeçalhos de bloco).
fn proximo_troco(i: u32) -> u32 {
    var j = i;
    while eixo[j].tipo != 0u {
        j += 1u;
    }
    return j;
}

// ⭐ doc 121 §9.9 — **O AJUSTE DO TRACEJADO NO ECRÃ**, o `stroke_uniform::ajusta_no_ecra` da casa: o
// padrão (já `× √|det|`) estica o mínimo para caber um número inteiro de vezes no sub-caminho
// tracejado mais LONGO da cópia — `n` períodos num fechado, `n` mais um traço num aberto (a
// `dash_fit::fit`) — e num fechado alonga `FOLGA_DO_AJUSTE`, para o fim cair DENTRO do último vão.
// Devolve o factor do período (`1` sem tracejado). ⚠️ `floor(x + 0,5)` e não `round`: o do WGSL
// arredonda as metades para o PAR, o `f64::round` da casa para longe do zero.
const FOLGA_DO_AJUSTE: f32 = 1.0e-4;

fn arredonda(x: f32) -> f32 {
    return floor(x + 0.5);
}

// ⭐ doc 121 §9.13 — UMA volta: um sub-caminho são os `_pad` troços a partir do de início (só
// cabeçalhos de bloco no meio), somados pela mesma ordem de sempre. ⭐ §9.15 — a volta é um
// ACUMULADOR, para a contagem (que já anda o eixo com o `arco` de cada troço) o levar consigo.
struct Ajuste {
    melhor: f32,
    fechado: bool,
    tr: f32,
    per: f32,
    tot: f32,
    restantes: u32,
    sub_fechado: bool,
    sub_tr: f32,
    sub_per: f32,
}

fn ajuste_novo() -> Ajuste {
    return Ajuste(0.0, false, 0.0, 0.0, 0.0, 0u, false, 0.0, 0.0);
}

// Um TROÇO tracejado (`tipo 0`) de arco `len`, na ordem do eixo.
fn ajuste_passo(a: ptr<function, Ajuste>, it: Eixo, len: f32, caneta: f32) {
    if (*a).restantes == 0u {
        if (it.ponta & SUB_INICIO) == 0u || it._pad == 0u {
            return;
        }
        (*a).restantes = it._pad;
        (*a).tot = 0.0;
        (*a).sub_fechado = (it.ponta & SUB_FECHADO) != 0u;
        (*a).sub_tr = it.traco * caneta;
        (*a).sub_per = (it.traco + it.vao) * caneta;
    }
    (*a).tot = (*a).tot + len;
    (*a).restantes -= 1u;
    if (*a).restantes == 0u && (*a).tot > (*a).melhor {
        (*a).melhor = (*a).tot;
        (*a).fechado = (*a).sub_fechado;
        (*a).tr = (*a).sub_tr;
        (*a).per = (*a).sub_per;
    }
}

fn ajuste_fim(a: Ajuste) -> f32 {
    if a.melhor <= 0.0 || a.per <= 0.0 {
        return 1.0;
    }
    var denom = max(arredonda(a.melhor / a.per), 1.0) * a.per;
    if !a.fechado {
        denom = max(arredonda((a.melhor - a.tr) / a.per), 0.0) * a.per + a.tr;
    }
    if denom <= 0.0 {
        return 1.0;
    }
    return a.melhor / denom * select(1.0, 1.0 + FOLGA_DO_AJUSTE, a.fechado);
}

fn ajuste_do_tracejado(inicio: u32, n: u32, lin: vec4<f32>, t: vec2<f32>, caneta: f32) -> f32 {
    var a = ajuste_novo();
    for (var i = inicio; i < inicio + n; i += 1u) {
        let it = eixo[i];
        if it.tipo == 0u && tracejado(it) {
            ajuste_passo(&a, it, arco(it, lin, t), caneta);
        }
    }
    return ajuste_fim(a);
}

struct SubTracejado {
    n: u32,
    fechado: bool,
    // O traço e o período no ecrã.
    tr: f32,
    per: f32,
    // Num fechado: o comprimento inteiro, o início do traço que o atravessa no fim, e se ele EMENDA.
    tot: f32,
    a_fim: f32,
    emenda: bool,
}

// O sub-caminho sem o total (`tot`, `a_fim`, `emenda` a zero) — o que [`sub_tracejado`] sabe sem andar.
fn cabeca_do_tracejado(i0: u32, caneta: f32, ajuste: f32) -> SubTracejado {
    let it = eixo[i0];
    var s: SubTracejado;
    s.n = it._pad;
    s.fechado = (it.ponta & SUB_FECHADO) != 0u;
    s.tr = it.traco * caneta * ajuste;
    s.per = (it.traco + it.vao) * caneta * ajuste;
    s.tot = 0.0;
    s.emenda = false;
    return s;
}

// A emenda de um FECHADO de comprimento `tot`.
fn fecha_o_tracejado(s: ptr<function, SubTracejado>, tot: f32) {
    (*s).tot = tot;
    (*s).a_fim = floor(tot / (*s).per) * (*s).per;
    (*s).emenda = (*s).a_fim < tot && tot < (*s).a_fim + (*s).tr;
}

fn sub_tracejado(i0: u32, lin: vec4<f32>, t: vec2<f32>, caneta: f32, ajuste: f32) -> SubTracejado {
    var s = cabeca_do_tracejado(i0, caneta, ajuste);
    if s.fechado && s.per > 0.0 {
        var i = i0;
        var tot = 0.0;
        for (var k = 0u; k < s.n; k += 1u) {
            i = proximo_troco(i);
            let e = eixo[i];
            tot = tot + arco(e, lin, t);
            i += 1u;
        }
        fecha_o_tracejado(&s, tot);
    }
    return s;
}

struct TrocoTracejado {
    s0: f32,
    fim: f32,
    // O arco (anda o tracejado) e a corda (a direcção e as decisões de faixa).
    len: f32,
    corda: f32,
    lprev: f32,
    lnext: f32,
    tem_ant: bool,
    tem_seg: bool,
    // O primeiro e o último troço de um FECHADO (a emenda).
    primeiro: bool,
    ultimo: bool,
    n0: f32,
    n1: f32,
}

fn troco_tracejado(it: Eixo, sub: SubTracejado, k: u32, s0: f32, lin: vec4<f32>, t: vec2<f32>) -> TrocoTracejado {
    var tr: TrocoTracejado;
    tr.s0 = s0;
    tr.len = arco(it, lin, t);
    tr.corda = comprimento(lin, t, it.a, it.b);
    tr.fim = s0 + tr.len;
    tr.lprev = comprimento(lin, t, it.d, it.a);
    tr.lnext = comprimento(lin, t, it.b, it.c);
    tr.tem_ant = k > 0u || sub.fechado;
    tr.tem_seg = k + 1u < sub.n || sub.fechado;
    tr.primeiro = k == 0u && sub.fechado;
    tr.ultimo = k + 1u == sub.n && sub.fechado;
    tr.n0 = floor(s0 / sub.per);
    tr.n1 = min(floor(tr.fim / sub.per), tr.n0 + TRACOS_POR_TROCO_MAX);
    return tr;
}

// O traço `n` dentro de um troço: o arco `[x0, x1]`, se liga ao troço de trás e ao da frente, e o
// recuo máximo da faixa em cada vértice ligado (metade do menor dos dois pedaços que lá se tocam).
struct Pedaco {
    valido: bool,
    x0: f32,
    x1: f32,
    liga0: bool,
    liga1: bool,
    recuo0: f32,
    recuo1: f32,
}

fn pedaco(tr: TrocoTracejado, sub: SubTracejado, n: f32) -> Pedaco {
    var p: Pedaco;
    p.valido = false;
    let a = n * sub.per;
    let b = a + sub.tr;
    // Um traço é do troço onde COMEÇA (`[s0, fim)`), e continua nos seguintes.
    if a >= tr.fim || (a < tr.s0 && b <= tr.s0) {
        return p;
    }
    p.valido = true;
    p.x0 = max(a, tr.s0);
    p.x1 = min(b, tr.fim);
    p.liga0 = a < tr.s0 && tr.tem_ant;
    var la = tr.s0 - a;
    if tr.primeiro && n == 0.0 {
        p.liga0 = sub.emenda;
        la = sub.tot - sub.a_fim;
    }
    p.recuo0 = 0.5 * min(min(tr.lprev, la), min(tr.corda, b - tr.s0));
    p.liga1 = b > tr.fim && tr.tem_seg;
    var ld = b - tr.fim;
    if tr.ultimo {
        ld = sub.tr;
    }
    p.recuo1 = 0.5 * min(min(tr.corda, tr.fim - a), min(tr.lnext, ld));
    return p;
}

// Os cantos do pedaço no ecrã — no VÉRTICE quando o pedaço chega lá (o vizinho usa os mesmos bits).
fn cantos_do_pedaco(a: vec2<f32>, b: vec2<f32>, tr: TrocoTracejado, p: Pedaco) -> vec4<f32> {
    let q0 = select(mix(a, b, (p.x0 - tr.s0) / tr.len), a, p.x0 <= tr.s0);
    let q1 = select(mix(a, b, (p.x1 - tr.s0) / tr.len), b, p.x1 >= tr.fim);
    return vec4<f32>(q0, q1);
}

// A ponta de estilo `tampa` em `c`, virada para `u`.
fn tampa_px(c: vec2<f32>, u: vec2<f32>, r: f32, tampa: u32, xy: vec2<f32>) -> f32 {
    let nr = perp(u) * r;
    if tampa == 1u {
        return quad(c + nr, c + nr + u * r, c - nr + u * r, c - nr, xy);
    }
    if tampa == 2u {
        let dir = sign(nr.x * u.y - nr.y * u.x);
        return leque(c, nr, -nr, -1.0, dir, r, xy);
    }
    return 0.0;
}

// Um pedaço de traço, pixel a pixel: o quadrilátero, a faixa nos vértices ligados, a junta de quem
// chega a um vértice ligado que a faixa não cobre, e as pontas onde o traço começa ou acaba.
fn pedaco_px(it: Eixo, lin: vec4<f32>, t: vec2<f32>, caneta: f32, tr: TrocoTracejado, p: Pedaco, xy: vec2<f32>) -> f32 {
    let r = it.meia * caneta;
    let a = aplica(lin, t, it.a);
    let b = aplica(lin, t, it.b);
    let q = cantos_do_pedaco(a, b, tr, p);
    let fp = r * max(alcance_da_peca(it), 1.5) + FAIXA_FOLGA;
    let plo = min(q.xy, q.zw) - vec2<f32>(fp);
    let phi = max(q.xy, q.zw) + vec2<f32>(fp);
    if phi.x < xy.x || plo.x > xy.x + 1.0 || phi.y < xy.y || plo.y > xy.y + 1.0 {
        return 0.0;
    }
    if tr.corda <= 0.0 || tr.len <= 0.0 || r <= 0.0 {
        return 0.0;
    }
    let u = (b - a) / tr.corda;
    let nr = perp(u) * r;
    var m0 = nr;
    var m1 = nr;
    var s = 0.0;
    if p.liga0 {
        let e = bissectriz_ate(aplica(lin, t, it.d), a, b, r, (it.ponta & 4u) != 0u, it.junta, it.limite, p.recuo0);
        if e.z > 0.0 {
            m0 = e.xy;
        }
    } else {
        s += tampa_px(q.xy, -u, r, (it.ponta >> 6u) & 3u, xy);
    }
    if p.liga1 {
        let quina = (it.ponta & 8u) != 0u;
        let cf = aplica(lin, t, it.c);
        let e = bissectriz_ate(a, b, cf, r, quina, it.junta, it.limite, p.recuo1);
        if e.z > 0.0 {
            m1 = e.xy;
        } else {
            s += junta_em(u, b, cf, r, select(2u, it.junta, quina), it.limite, xy);
        }
    } else {
        s += tampa_px(q.zw, u, r, (it.ponta >> 8u) & 3u, xy);
    }
    return s + quad(q.xy + m0, q.zw + m1, q.zw - m1, q.xy - m0, xy);
}

// O sub-caminho tracejado que começa no troço `i0`, pixel a pixel.
fn tracejado_px(i0: u32, lin: vec4<f32>, t: vec2<f32>, caneta: f32, ajuste: f32, xy: vec2<f32>) -> f32 {
    let sub = sub_tracejado(i0, lin, t, caneta, ajuste);
    if sub.per <= 0.0 {
        return 0.0;
    }
    var s = 0.0;
    var i = i0;
    var s0 = 0.0;
    for (var k = 0u; k < sub.n; k += 1u) {
        i = proximo_troco(i);
        let it = eixo[i];
        let tr = troco_tracejado(it, sub, k, s0, lin, t);
        for (var n = tr.n0; n <= tr.n1; n += 1.0) {
            let p = pedaco(tr, sub, n);
            if p.valido {
                s += pedaco_px(it, lin, t, caneta, tr, p, xy);
            }
        }
        s0 = tr.fim;
        i += 1u;
    }
    return s;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let xy = floor(in.pos.xy);
    var af = 0.0;
    // O traço é sempre não-nulo: o contorno expandido é um preenchimento.
    var as_ = 0.0;
    if in.tela > 0u {
        // ⭐ doc 121 §9.12: a cobertura acabada pelas células — a mesma conta do ramo de baixo, com as
        // regras já aplicadas.
        let s = cobertura_de_ecra(in.tela - 1u, xy);
        af = s.x;
        as_ = s.y;
    } else {
        af = area(in.fill.x, in.fill.y, in.lin, in.t, xy);
        if in.eixo_rg.y > 0u {
            // Sob escala não uniforme: as MARCAS (as primeiras `z` peças do traço) mais o traço do eixo.
            let marcas = area(in.stroke.x, in.eixo_rg.z, in.lin, in.t, xy);
            let eixo_cob = traco_do_eixo(in.eixo_rg.x, in.eixo_rg.y, in.lin, in.t, bitcast<f32>(in.eixo_rg.w), xy);
            as_ = min(min(abs(marcas), 1.0) + eixo_cob, 1.0);
        } else {
            as_ = min(abs(area(in.stroke.x, in.stroke.y, in.lin, in.t, xy)), 1.0);
        }
        // As duas regras, à letra do Vello (o `cs_varre` aplica as mesmas às células).
        if in.even_odd != 0u {
            af = abs(af - 2.0 * round(0.5 * af));
        } else {
            af = min(abs(af), 1.0);
        }
    }
    let f = vec4<f32>(in.tint.rgb * in.tint.a, in.tint.a) * af;
    let s = vec4<f32>(in.stroke_color.rgb * in.stroke_color.a, in.stroke_color.a) * as_;
    // O traço POR CIMA do preenchimento — a ordem dos dois `fill` do Vello, composta aqui.
    let c = s + f * (1.0 - s.a);
    if c.a <= 0.0 {
        discard;
    }
    return c;
}
