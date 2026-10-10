// C2 HOLOGRÁFICA (W5c, a do produto em todo nível) — Holographic Radiance Cascades (Freeman, Sannikov, Margel,
// arXiv 2505.02041; descrição para implementar em `docs/Luz2D/pesquisa/research_notes/
// hrc_para_implementar.md`). Substitui as cascatas: escreve a MESMA c0 (fluência por texel das
// cascatas, alfa = fora dos opacos) que a C3, o rebote e os receptores leem.
//
// Por quadrante (o canónico olha para +u, ângulos [−45°, 45°]; os outros por rotação das coordenadas,
// nunca das texturas), sondas da cascata n em (xi·2ⁿ, v): espaçamento 2ⁿ no eixo principal, 1 no
// transversal. T_n(p, k) = Traço(p, p + v_n(k)), v_n(k) = (2ⁿ, 2k − 2ⁿ), k = 0..2ⁿ: T0–T2 traçados (a
// MESMA lei do meio das cascatas: × (1 − α) a cada entrada, a emissão na entrada); T3..TN pela fusão
// (Eq. 18–20). R_n(p, m): a fluência angular do cone m (arestas por p + v_n(m) e p + v_n(m+1)), de cima
// para baixo (Eq. 14–17); R_N = o céu × o arco (nota 1). A fluência do texel: Σ quadrantes
// R0((u+1, v), 0) — o «+1» do artigo (Fig. 12). Um traço conta as entradas em (A, B]: o estado em A é
// o «anterior» — a composição de dois não conta a fronteira duas vezes.
//
// Concatenado depois de `c2_rc.wgsl` (usa `cas`, `meio`, `queda`, `TAU`). Grupo 3, bindings 4–11 (os
// 0–3 são do preenchimento e do rebote).

struct Hrc {
    // O quadrante (0 = +x, 1 = +y, 2 = −x, 3 = −y) e o nível n deste passe.
    quadrante: u32,
    nivel: u32,
    // A grade das cascatas (W, H) e o quadrante no canónico: X ao longo de u, Y ao longo de v.
    grade: vec2<u32>,
    eixo: vec2<u32>,
    // As sondas ao longo de u nos níveis n, n − 1 e n + 1.
    nx: u32,
    nx_baixo: u32,
    nx_cima: u32,
    // A cascata de cima é o topo (o R dela é o céu).
    topo: u32,
    // Quantas entradas este passe calcula (o despacho é 1D).
    total: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

@group(3) @binding(4) var<uniform> hrc: Hrc;
@group(3) @binding(5) var<storage, read> t_a: array<vec2<u32>>;
@group(3) @binding(6) var<storage, read> t_b: array<vec2<u32>>;
@group(3) @binding(7) var<storage, read_write> t_saida: array<vec2<u32>>;
@group(3) @binding(8) var<storage, read> r_cima: array<vec2<u32>>;
@group(3) @binding(9) var<storage, read_write> r_saida: array<vec2<u32>>;
@group(3) @binding(10) var<storage, read_write> acumulado: array<vec4<f32>>;
// (o binding 11, a c0 de saída, vem do prelúdio `saida_*`: o formato segue o armazenamento.)

// O passo do traço curto, em texels.
override PASSO_HRC: f32 = 0.5;
// O filtro em cruz da Eq. 21 (sondas de v par e ímpar não conversam para n ≥ 1).
override BORRAO_HRC: bool = true;
// O «+1» da Fig. 12: a fluência do texel u lê-se na sonda u + 1 (evita as cruzes das diagonais
// partilhadas entre quadrantes) — salvo quando a sonda adiante cai DENTRO de um opaco (junto a uma
// face: lá lê-se no próprio u; com o «+1» cego o texel colado à face saía −43 %, medido W5c).
override MAIS_UM_HRC: bool = true;

struct Par {
    r: vec3<f32>,
    t: f32,
}

fn empacotar(p: Par) -> vec2<u32> {
    return vec2<u32>(pack2x16float(p.r.xy), pack2x16float(vec2<f32>(p.r.z, p.t)));
}

fn desempacotar(e: vec2<u32>) -> Par {
    let a = unpack2x16float(e.x);
    let b = unpack2x16float(e.y);
    return Par(vec3<f32>(a, b.x), b.y);
}

// Merge (Eq. 7): ⟨r_n + t_n·r_f, t_n·t_f⟩.
fn fundir(perto: Par, longe: Par) -> Par {
    return Par(perto.r + perto.t * longe.r, perto.t * longe.t);
}

const VACUO: Par = Par(vec3<f32>(0.0), 1.0);

// O ponto canónico (centro do texel (u, v)) na grade.
fn na_grade(u: f32, v: f32) -> vec2<f32> {
    let c = vec2<f32>(u + 0.5, v + 0.5);
    let w = vec2<f32>(hrc.grade);
    switch hrc.quadrante {
        case 1u: { return vec2<f32>(w.x - c.y, c.x); }
        case 2u: { return vec2<f32>(w.x - c.x, w.y - c.y); }
        case 3u: { return vec2<f32>(c.y, w.y - c.x); }
        default: { return c; }
    }
}

// O texel da grade do texel canónico (u, v).
fn texel_da_grade(u: u32, v: u32) -> vec2<u32> {
    return vec2<u32>(floor(na_grade(f32(u), f32(v))));
}

// Traço(A, B) na lei do meio, as entradas em (A, B]; a queda conta-se de A.
fn tracar(A: vec2<f32>, B: vec2<f32>) -> Par {
    var par = VACUO;
    let seg = B - A;
    let len = length(seg);
    if len <= 0.0 {
        return par;
    }
    let dir = seg / len;
    let m0 = meio(A);
    var tau_ant = m0.tau;
    var luz_ant = m0.luz;
    var s = min(PASSO_HRC, len);
    loop {
        let x = A + dir * s;
        let m = meio(x);
        if m.luz && !luz_ant {
            var l = m.e.rgb;
            if m.e.a > 0.0 {
                l *= queda(s * cas.texel_m, m.e.a);
            }
            par.r += par.t * l;
        }
        if m.tau < tau_ant {
            par.t *= m.tau / tau_ant;
            if par.t <= 0.0 {
                break;
            }
        }
        tau_ant = m.tau;
        luz_ant = m.luz;
        if s >= len {
            break;
        }
        s = min(s + PASSO_HRC, len);
    }
    return par;
}

// O arco do cone m da cascata n (Eq. 13): entre as arestas v_n(m) e v_n(m+1).
fn arco(n: u32, m: u32) -> f32 {
    let lado = f32(1u << n);
    return atan((2.0 * f32(m + 1u) - lado) / lado) - atan((2.0 * f32(m) - lado) / lado);
}

fn indice_t(nx: u32, n: u32, xi: u32, v: u32, k: u32) -> u32 {
    return (v * nx + xi) * ((1u << n) + 1u) + k;
}

fn indice_r(nx: u32, n: u32, xi: u32, v: u32, m: u32) -> u32 {
    return (v * nx + xi) * (1u << n) + m;
}

// T_{n−1} em (xi, v, k) — fora do que se guarda, vácuo.
fn t_de_baixo(xi: u32, v: i32, k: u32) -> Par {
    if xi >= hrc.nx_baixo || v < 0 || v >= i32(hrc.eixo.y) {
        return VACUO;
    }
    return desempacotar(t_a[indice_t(hrc.nx_baixo, hrc.nivel - 1u, xi, u32(v), k)]);
}

// T_n (t_a) e T_{n+1} (t_b) na fusão de cima para baixo.
fn t_deste(xi: u32, v: u32, k: u32) -> Par {
    return desempacotar(t_a[indice_t(hrc.nx, hrc.nivel, xi, v, k)]);
}

fn t_de_cima(xi: u32, v: u32, k: u32) -> Par {
    if xi >= hrc.nx_cima {
        return VACUO;
    }
    return desempacotar(t_b[indice_t(hrc.nx_cima, hrc.nivel + 1u, xi, v, k)]);
}

// R_{n+1} em (xi, v, j) — no topo, ou fora do que se guarda, o céu × o arco (o que está além).
fn r_de_cima(xi: u32, v: i32, j: u32) -> vec3<f32> {
    let n = hrc.nivel + 1u;
    if hrc.topo == 1u || xi >= hrc.nx_cima || v < 0 || v >= i32(hrc.eixo.y) {
        return cas.ceu.rgb * arco(n, j);
    }
    return desempacotar(r_cima[indice_r(hrc.nx_cima, n, xi, u32(v), j)]).r;
}

// A entrada do despacho 1D (dobrado em 2D: o limite de 65 535 grupos por eixo).
fn entrada(gid: vec3<u32>, nw: vec3<u32>) -> u32 {
    return gid.x + gid.y * nw.x * 64u;
}

// ── T0, T1, T2: traçados ──
@compute @workgroup_size(64)
fn hrc_traca(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let n = hrc.nivel;
    let ks = (1u << n) + 1u;
    let k = e % ks;
    let resto = e / ks;
    let xi = resto % hrc.nx;
    let v = resto / hrc.nx;
    let u = f32(xi << n);
    let lado = f32(1u << n);
    let A = na_grade(u, f32(v));
    let B = na_grade(u + lado, f32(v) + 2.0 * f32(k) - lado);
    t_saida[e] = empacotar(tracar(A, B));
}

// ── T3..TN: a fusão de baixo para cima (Eq. 18–20) ──
@compute @workgroup_size(64)
fn hrc_sobe(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let n = hrc.nivel;
    let ks = (1u << n) + 1u;
    let k = e % ks;
    let resto = e / ks;
    let xi = resto % hrc.nx;
    let v = i32(resto / hrc.nx);
    let h = i32(1u << (n - 1u));
    var par: Par;
    if k % 2u == 0u {
        let c = k / 2u;
        par = fundir(t_de_baixo(2u * xi, v, c), t_de_baixo(2u * xi + 1u, v + 2 * i32(c) - h, c));
    } else {
        let a = (k - 1u) / 2u;
        let b = (k + 1u) / 2u;
        let mais = fundir(t_de_baixo(2u * xi, v, b), t_de_baixo(2u * xi + 1u, v + 2 * i32(b) - h, a));
        let menos = fundir(t_de_baixo(2u * xi, v, a), t_de_baixo(2u * xi + 1u, v + 2 * i32(a) - h, b));
        par = Par(0.5 * (mais.r + menos.r), 0.5 * (mais.t + menos.t));
    }
    t_saida[e] = empacotar(par);
}

// ── R_{N−1}..R0: a fusão de cima para baixo (Eq. 14–17) ──
@compute @workgroup_size(64)
fn hrc_desce(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let n = hrc.nivel;
    let cones = 1u << n;
    let m = e % cones;
    let resto = e / cones;
    let xi = resto % hrc.nx;
    let v = resto / hrc.nx;
    let lado = i32(cones);
    var soma = vec3<f32>(0.0);
    for (var lado_do_cone = 0u; lado_do_cone < 2u; lado_do_cone++) {
        // A metade de cima (+, aresta m+1, cone fino 2m+1) e a de baixo (−, aresta m, cone 2m).
        let aresta = m + lado_do_cone;
        let j = 2u * m + lado_do_cone;
        let a = arco(n + 1u, j);
        if xi % 2u == 1u {
            let tr = t_deste(xi, v, aresta);
            let q = i32(v) + 2 * i32(aresta) - lado;
            soma += a * tr.r + tr.t * r_de_cima((xi + 1u) / 2u, q, j);
        } else {
            let f0 = r_de_cima(xi / 2u, i32(v), j);
            let grande = 2u * aresta;
            let tr = t_de_cima(xi / 2u, v, grande);
            let q = i32(v) + 2 * i32(grande) - 2 * lado;
            let f1 = a * tr.r + tr.t * r_de_cima(xi / 2u + 1u, q, j);
            soma += 0.5 * (f0 + f1);
        }
    }
    r_saida[e] = empacotar(Par(soma, 0.0));
}

// ── A fluência do quadrante: L(u, v) += R0((u + 1, v), 0) ──
@compute @workgroup_size(64)
fn hrc_soma(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let u = e % hrc.eixo.x;
    let v = e / hrc.eixo.x;
    var onde = u;
    if MAIS_UM_HRC && meio(na_grade(f32(u + 1u), f32(v))).tau > 0.0 {
        onde = u + 1u;
    }
    var f = cas.ceu.rgb * arco(0u, 0u);
    if onde < hrc.nx {
        f = desempacotar(r_cima[indice_r(hrc.nx, 0u, onde, v, 0u)]).r;
    }
    // Partir de DENTRO de um translúcido conta como entrada (a lei do gabarito): os traços da sonda
    // começam «já dentro» e não a contam. A sonda dentro de um OPACO fica com a luz que a rodeia: a
    // máscara tira-a da bilinear, mas onde as quatro de um texel caem dentro (um enxame denso) a
    // bilinear crua lê-a — com zero, pintas pretas (medido na foto da cena 2, W5c).
    let tau = meio(na_grade(f32(onde), f32(v))).tau;
    if tau > 0.0 {
        f *= tau;
    }
    let t = texel_da_grade(u, v);
    let i = t.y * hrc.grade.x + t.x;
    if hrc.quadrante == 0u {
        acumulado[i] = vec4<f32>(f, 0.0);
    } else {
        acumulado[i] += vec4<f32>(f, 0.0);
    }
}

// ── A c0: o acumulado (com o filtro em cruz da Eq. 21) e a máscara do preenchimento ──
@compute @workgroup_size(8, 8)
fn hrc_saida(@builtin(global_invocation_id) gid: vec3<u32>) {
    if any(gid.xy >= hrc.grade) {
        return;
    }
    let P = vec2<f32>(gid.xy) + 0.5;
    let fora = meio(P).tau > 0.0;
    let i = gid.y * hrc.grade.x + gid.x;
    var f = acumulado[i].rgb;
    if BORRAO_HRC {
        var soma = 4.0 * f;
        var peso = 4.0;
        let d = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
        for (var j = 0u; j < 4u; j++) {
            let q = vec2<i32>(gid.xy) + d[j];
            if any(q < vec2<i32>(0)) || any(q >= vec2<i32>(hrc.grade)) {
                continue;
            }
            // O vizinho do OUTRO lado de uma borda não entra (a opacidade difere).
            if (meio(vec2<f32>(q) + 0.5).tau > 0.0) != fora {
                continue;
            }
            soma += acumulado[u32(q.y) * hrc.grade.x + u32(q.x)].rgb;
            peso += 1.0;
        }
        f = soma / peso;
    }
    var alfa = 1.0;
    if MASCARA_C0 && !fora {
        alfa = 0.0;
    }
    textureStore(c0_saida, gid.xy, vec4<f32>(f, alfa));
}
