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
    // doc 121 §9.5: o CONTORNO calculado da cópia — x = o primeiro bloco, y = quantos; `y = 0` ⇒ o
    // traço sai do eixo pixel a pixel (`traco_do_eixo`), como antes.
    @location(8) @interpolate(flat) contorno_rg: vec2<u32>,
}

// ⭐ doc 121 §9.5 — **O CONTORNO DE CADA CÓPIA, CALCULADO UMA VEZ** (`contorno.wgsl` escreve-os, o
// desenho só os lê): as arestas no ECRÃ, em blocos de `SEGS_POR_BLOCO`; a caixa de cada bloco; por
// cópia `(primeiro bloco, blocos, 0, 0)`; e a caixa da cópia inteira.
@group(1) @binding(0) var<storage, read> contorno: array<vec4<f32>>;
@group(1) @binding(1) var<storage, read> cblocos: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read> ccopias: array<vec4<u32>>;
@group(1) @binding(3) var<storage, read> ccaixas: array<vec4<f32>>;

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
    let caneta = bitcast<f32>(eixo_rg.w);
    // O quad: a caixa da forma no ECRÃ, arredondada PARA FORA ao pixel inteiro.
    // ⚠️ **Sem margem, e é medido:** a caixa é a dos SEGMENTOS aplanados, e um pixel só tem
    // cobertura se um segmento (ou o interior entre eles) lhe toca — logo o `floor`/`ceil` já
    // inclui todo pixel de borda. A 1.ª redacção alargava um pixel de cada lado, e a mutação que o
    // apagava SOBREVIVEU à paridade de pixel: não era lei, era trabalho a mais.
    let c0 = aplica(lin, t, rec.bbox.xy);
    let c1 = aplica(lin, t, rec.bbox.zy);
    let c2 = aplica(lin, t, rec.bbox.xw);
    let c3 = aplica(lin, t, rec.bbox.zw);
    var lo_f = min(min(c0, c1), min(c2, c3));
    var hi_f = max(max(c0, c1), max(c2, c3));
    // ⭐ doc 121 §9.5: o contorno desta cópia já foi CALCULADO (`contorno.wgsl`) — a caixa dele é a
    // exacta, e o fragmento lê as arestas prontas em vez de as refazer.
    var contorno_rg = vec2<u32>(0u, 0u);
    if eixo_rg.y > 0u {
        contorno_rg = ccopias[ii].xy;
    }
    if contorno_rg.y > 0u {
        let cx = ccaixas[ii];
        lo_f = min(lo_f, cx.xy);
        hi_f = max(hi_f, cx.zw);
    } else if eixo_rg.y > 0u {
        // O traço do eixo vai até `ext_fora × caneta` para FORA dos pontos do eixo, no ecrã — a caixa
        // local do contorno expandido não o cobre sob escala não uniforme.
        let e0 = aplica(lin, t, rec.eixo_bbox.xy);
        let e1 = aplica(lin, t, rec.eixo_bbox.zy);
        let e2 = aplica(lin, t, rec.eixo_bbox.xw);
        let e3 = aplica(lin, t, rec.eixo_bbox.zw);
        let m = vec2<f32>(rec.ext_fora * caneta);
        lo_f = min(lo_f, min(min(e0, e1), min(e2, e3)) - m);
        hi_f = max(hi_f, max(max(e0, e1), max(e2, e3)) + m);
    }
    let lo = floor(lo_f);
    let hi = ceil(hi_f);
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
    out.contorno_rg = contorno_rg;
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

// ⭐ doc 121 §9.5 — **o traço a partir do CONTORNO CALCULADO**: as arestas já estão no ecrã, logo
// um bloco acima, abaixo ou à direita soma zero; um todo à ESQUERDA soma a faixa de cada aresta
// (`clamp(y₀) − clamp(y₁)`, sem divisão); o resto, a conta do Vello aresta a aresta. ⚠️ As arestas
// partilhadas da FAIXA não estão lá (as duas metades cancelavam-se), logo um bloco já não é FECHADO e
// um à esquerda NÃO se pode saltar — por isso a faixa das arestas, que é barata, e num bloco
// ENCADEADO (`cs_escreve`) ela telescopa em dois números já guardados ao lado da caixa.
fn traco_do_contorno(b0: u32, nb: u32, xy: vec2<f32>) -> f32 {
    var s = 0.0;
    for (var b = b0; b < b0 + nb; b += 1u) {
        let cx = cblocos[2u * b];
        if cx.w <= xy.y || cx.y >= xy.y + 1.0 || cx.x >= xy.x + 1.0 {
            continue;
        }
        let i0 = b * SEGS_POR_BLOCO;
        if cx.z <= xy.x {
            let ex = cblocos[2u * b + 1u];
            if ex.z != 0.0 {
                s += clamp(ex.x - xy.y, 0.0, 1.0) - clamp(ex.y - xy.y, 0.0, 1.0);
                continue;
            }
            for (var i = i0; i < i0 + SEGS_POR_BLOCO; i += 1u) {
                let e = contorno[i];
                s += clamp(e.y - xy.y, 0.0, 1.0) - clamp(e.w - xy.y, 0.0, 1.0);
            }
            continue;
        }
        for (var i = i0; i < i0 + SEGS_POR_BLOCO; i += 1u) {
            let e = contorno[i];
            s += contribuicao(e.xy, e.zw, xy);
        }
    }
    return min(abs(s), 1.0);
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
        if bhi.x < xa.x || blo.x > xb.x || bhi.y < xa.y || blo.y > xb.y {
            i = fim;
            continue;
        }
        i += 1u;
        for (; i < fim; i += 1u) {
            s += peca_do_eixo(eixo[inicio + i], lin, t, caneta, xy);
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
    if (!quina && dot(m, m) > fora * fora) || recuo > 0.5 * min(l0, l1) {
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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let xy = floor(in.pos.xy);
    var af = area(in.fill.x, in.fill.y, in.lin, in.t, xy);
    // As duas regras, à letra do Vello.
    if in.even_odd != 0u {
        af = abs(af - 2.0 * round(0.5 * af));
    } else {
        af = min(abs(af), 1.0);
    }
    // O traço é sempre não-nulo: o contorno expandido é um preenchimento.
    var as_ = 0.0;
    if in.eixo_rg.y > 0u {
        // Sob escala não uniforme: as MARCAS (as primeiras `z` peças do traço) mais o traço do eixo.
        let marcas = area(in.stroke.x, in.eixo_rg.z, in.lin, in.t, xy);
        var eixo_cob = 0.0;
        if in.contorno_rg.y > 0u {
            eixo_cob = traco_do_contorno(in.contorno_rg.x, in.contorno_rg.y, xy);
        } else {
            eixo_cob = traco_do_eixo(in.eixo_rg.x, in.eixo_rg.y, in.lin, in.t, bitcast<f32>(in.eixo_rg.w), xy);
        }
        as_ = min(min(abs(marcas), 1.0) + eixo_cob, 1.0);
    } else {
        as_ = min(abs(area(in.stroke.x, in.stroke.y, in.lin, in.t, xy)), 1.0);
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
