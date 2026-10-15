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
    // ⭐ W5d — o MIOLO (texels da grade, `[x0, y0, x1, y1]`) cuja emissão este solver NÃO recolhe: ele é
    // o ENTORNO de outro, e só a luz que vem de fora da grade daquele entra nas cascatas dele.
    miolo: vec4<f32>,
    // ⭐ W5d — a c0 do ENTORNO somada na saída: `(desloc, escala, ligado)` — o texel da grade daquele
    // é `P · escala + desloc`.
    entorno: vec4<f32>,
}

@group(3) @binding(4) var<uniform> hrc: Hrc;
@group(3) @binding(5) var<storage, read> t_a: array<vec2<u32>>;
@group(3) @binding(6) var<storage, read> t_b: array<vec2<u32>>;
@group(3) @binding(7) var<storage, read_write> t_saida: array<vec2<u32>>;
@group(3) @binding(8) var<storage, read> r_cima: array<vec2<u32>>;
@group(3) @binding(9) var<storage, read_write> r_saida: array<vec2<u32>>;
@group(3) @binding(10) var<storage, read_write> acumulado: array<vec4<f32>>;
// ⭐ W5r: o arco de cada cone (`arcos[2ⁿ + m]`), escrito uma vez pelo `hrc_arcos` (binding 13).
@group(3) @binding(12) var<storage, read> arcos: array<f32>;
@group(3) @binding(13) var<storage, read_write> arcos_escrita: array<f32>;
// ⭐ W5s: um por ladrilho LADO_DO_VAZIO² da captura — 1 se algum texel tem cobertura (o `hrc_ladrilhos`
// escreve-o a cada gravação; a traça lê-o).
@group(3) @binding(16) var<storage, read_write> com_cobertura: array<u32>;
// ⭐ W5d: a c0 do entorno (só a saída a lê).
@group(2) @binding(14) var entorno_c0: texture_2d<f32>;
@group(2) @binding(15) var entorno_amostra: sampler;
// (o binding 11, a c0 de saída, vem do prelúdio `saida_*`: o formato segue o armazenamento.)

// O passo do traço curto, em texels.
override PASSO_HRC: f32 = 0.5;
// O filtro em cruz da Eq. 21 (sondas de v par e ímpar não conversam para n ≥ 1).
override BORRAO_HRC: bool = true;
// O «+1» da Fig. 12: a fluência do texel u lê-se na sonda u + 1 (evita as cruzes das diagonais
// partilhadas entre quadrantes) — salvo quando a sonda adiante cai DENTRO de um opaco (junto a uma
// face: lá lê-se no próprio u; com o «+1» cego o texel colado à face saía −43 %, medido W5c).
override MAIS_UM_HRC: bool = true;
// ⭐ W5r — T e R por COLUNAS: v é o índice mais rápido (`((k·nx + xi)·Y + v)`), e os fios seguem a mesma
// ordem — as leituras «em diagonal» (a linha v + 2k − 2ⁿ) ficam contíguas entre fios vizinhos. Só os
// endereços mudam: a conta de cada entrada é a mesma (handoff W5r). `false` = por linhas (o antes).
override COLUNAS_HRC: bool = true;
// ⭐ W5r — o arco lido da tabela (`arcos`), não recalculado por fio: a MESMA conta, feita uma vez na placa.
override ARCOS_TABELA: bool = true;
// ⭐ W5s — o traço curto que só lê ladrilhos sem cobertura é o VACUO exato: não marcha (handoff W5s).
override SALTA_O_VAZIO_HRC: bool = true;
// ⭐ W5s — um fio por SONDA de T0–T2: os 2ⁿ + 1 intervalos dela num laço, o meio em A lido uma vez e o
// salto do vazio decidido pelo LEQUE inteiro (o triângulo A, B₀, B_{2ⁿ}). `false` = um fio por intervalo.
override SONDA_HRC: bool = true;
// ⭐ W5t — o despacho em 3D (por colunas): `(v, xi, k)` são o índice do fio, sem as divisões do 1D — a subida e a
// descida pagam por FIO, não por byte (handoff W5t). `false` = 1D dobrado.
override GRADE3D_HRC: bool = true;
const LADO_DO_VAZIO: u32 = 8u;

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

// ⭐ W5d — `x` (texels da grade) cai no miolo cuja emissão não se recolhe.
fn no_miolo(x: vec2<f32>) -> bool {
    return all(x >= hrc.miolo.xy) && all(x < hrc.miolo.zw);
}

// Traço(A, B) na lei do meio, as entradas em (A, B]; a queda conta-se de A.
fn tracar(A: vec2<f32>, B: vec2<f32>) -> Par {
    return tracar_de(A, meio(A), B);
}

// [`tracar`] com o meio em A já lido (`m0`: partilhado pelos intervalos de uma sonda).
fn tracar_de(A: vec2<f32>, m0: Meio, B: vec2<f32>) -> Par {
    var par = VACUO;
    let seg = B - A;
    let len = length(seg);
    if len <= 0.0 {
        return par;
    }
    let dir = seg / len;
    var tau_ant = m0.tau;
    var luz_ant = m0.luz;
    var s = min(PASSO_HRC, len);
    loop {
        let x = A + dir * s;
        let m = meio(x);
        if m.luz && !luz_ant && !no_miolo(x) {
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

fn ladrilhos_por_linha() -> u32 {
    return (cas.tamanho.x + LADO_DO_VAZIO - 1u) / LADO_DO_VAZIO;
}

// ⭐ W5s — o traço A → B só lê texels SEM cobertura: cada amostra p lê pelo gather `floor(p − ½)` e o
// seguinte (presos à borda), e com os quatro a zero o meio é «fora» (τ 1, sem luz) — o VACUO exato. A
// folga de 0,001 cobre o arredondamento de `A + dir·s`.
fn so_vazio(A: vec2<f32>, B: vec2<f32>) -> bool {
    return caixa_vazia(min(A, B), max(A, B));
}

// Os ladrilhos que cobrem a caixa `[a, b]` (pontos da grade), com a folga do gather, estão a 0.
fn caixa_vazia(a: vec2<f32>, b: vec2<f32>) -> bool {
    let ultimo = vec2<i32>(cas.tamanho) - 1;
    let lo = vec2<u32>(clamp(vec2<i32>(floor(a - 0.501)), vec2<i32>(0), ultimo)) / LADO_DO_VAZIO;
    let hi = vec2<u32>(clamp(vec2<i32>(floor(b + 0.501)), vec2<i32>(0), ultimo)) / LADO_DO_VAZIO;
    let lx = ladrilhos_por_linha();
    for (var y = lo.y; y <= hi.y; y++) {
        for (var x = lo.x; x <= hi.x; x++) {
            if com_cobertura[y * lx + x] != 0u {
                return false;
            }
        }
    }
    return true;
}

// O arco do cone m da cascata n (Eq. 13): entre as arestas v_n(m) e v_n(m+1).
fn arco(n: u32, m: u32) -> f32 {
    if ARCOS_TABELA {
        return arcos[(1u << n) + m];
    }
    return arco_conta(n, m);
}

fn arco_conta(n: u32, m: u32) -> f32 {
    let lado = f32(1u << n);
    return atan((2.0 * f32(m + 1u) - lado) / lado) - atan((2.0 * f32(m) - lado) / lado);
}

fn indice_t(nx: u32, n: u32, xi: u32, v: u32, k: u32) -> u32 {
    if COLUNAS_HRC {
        return (k * nx + xi) * hrc.eixo.y + v;
    }
    return (v * nx + xi) * ((1u << n) + 1u) + k;
}

fn indice_r(nx: u32, n: u32, xi: u32, v: u32, m: u32) -> u32 {
    if COLUNAS_HRC {
        return (m * nx + xi) * hrc.eixo.y + v;
    }
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

// A entrada `e` de T ou R com `nk` intervalos (ou cones) por sonda → `(k, xi, v)`, na ordem do armazenamento
// (`t_saida[e]`/`r_saida[e]` é o índice dela).
fn decompor(e: u32, nk: u32) -> vec3<u32> {
    if COLUNAS_HRC {
        let resto = e / hrc.eixo.y;
        return vec3<u32>(resto / hrc.nx, resto % hrc.nx, e % hrc.eixo.y);
    }
    let resto = e / nk;
    return vec3<u32>(e % nk, resto % hrc.nx, resto / hrc.nx);
}

// ── ⭐ W5r: a tabela dos arcos, uma vez ao nascer ──
@compute @workgroup_size(64)
fn hrc_arcos(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i == 0u || i >= arrayLength(&arcos_escrita) {
        return;
    }
    let n = firstLeadingBit(i);
    arcos_escrita[i] = arco_conta(n, i - (1u << n));
}

// ── ⭐ W5s: os ladrilhos com cobertura, da própria captura — um grupo por ladrilho, um fio por texel ──
var<workgroup> aceso_local: atomic<u32>;

@compute @workgroup_size(8, 8)
fn hrc_ladrilhos(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let p = wid.xy * LADO_DO_VAZIO + lid.xy;
    if all(p < cas.tamanho) && textureLoad(cobertura, p, 0).r > 0.0 {
        atomicOr(&aceso_local, 1u);
    }
    workgroupBarrier();
    if all(lid.xy == vec2<u32>(0u)) {
        com_cobertura[wid.y * ladrilhos_por_linha() + wid.x] = atomicLoad(&aceso_local);
    }
}

// As pontas `(A, B)` do intervalo da entrada `e` de T_n (n ≤ 2).
fn pontas_da_traca(e: u32) -> array<vec2<f32>, 2> {
    let n = hrc.nivel;
    let d = decompor(e, (1u << n) + 1u);
    let u = f32(d.y << n);
    let lado = f32(1u << n);
    return array<vec2<f32>, 2>(
        na_grade(u, f32(d.z)),
        na_grade(u + lado, f32(d.z) + 2.0 * f32(d.x) - lado),
    );
}

// A sonda `s` de T_n → `(xi, v)`, na ordem do armazenamento (fios vizinhos, v vizinhos por colunas).
fn decompor_sonda(s: u32) -> vec2<u32> {
    if COLUNAS_HRC {
        return vec2<u32>(s / hrc.eixo.y, s % hrc.eixo.y);
    }
    return vec2<u32>(s % hrc.nx, s / hrc.nx);
}

// O leque da sonda `(u, v)` de T_n: as pontas extremas `[A, B₀, B_{2ⁿ}]` (o triângulo que os intervalos
// varrem).
fn leque(u: f32, v: f32, lado: f32) -> array<vec2<f32>, 3> {
    return array<vec2<f32>, 3>(
        na_grade(u, v),
        na_grade(u + lado, v - lado),
        na_grade(u + lado, v + lado),
    );
}

// A sonda `sv = (xi, v)` de T_n (n ≤ 2): os 2ⁿ + 1 intervalos dela.
fn tracar_sonda(sv: vec2<u32>) {
    let n = hrc.nivel;
    let u = f32(sv.x << n);
    let v = f32(sv.y);
    let lado = f32(1u << n);
    let l = leque(u, v, lado);
    if SALTA_O_VAZIO_HRC && caixa_vazia(min(l[0], min(l[1], l[2])), max(l[0], max(l[1], l[2]))) {
        for (var k = 0u; k <= (1u << n); k++) {
            t_saida[indice_t(hrc.nx, n, sv.x, sv.y, k)] = empacotar(VACUO);
        }
        return;
    }
    let m0 = meio(l[0]);
    for (var k = 0u; k <= (1u << n); k++) {
        let B = na_grade(u + lado, v + 2.0 * f32(k) - lado);
        t_saida[indice_t(hrc.nx, n, sv.x, sv.y, k)] = empacotar(tracar_de(l[0], m0, B));
    }
}

// ── T0, T1, T2: traçados ──
@compute @workgroup_size(64)
fn hrc_traca(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    if GRADE3D_HRC && SONDA_HRC && COLUNAS_HRC {
        if gid.x < hrc.eixo.y {
            tracar_sonda(vec2<u32>(gid.y, gid.x));
        }
        return;
    }
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    if SONDA_HRC {
        tracar_sonda(decompor_sonda(e));
        return;
    }
    let ab = pontas_da_traca(e);
    if SALTA_O_VAZIO_HRC && so_vazio(ab[0], ab[1]) {
        t_saida[e] = empacotar(VACUO);
        return;
    }
    t_saida[e] = empacotar(tracar(ab[0], ab[1]));
}

// A entrada (k, xi, v) de T_n pela fusão de T_{n−1}.
fn fusao_de_baixo(k: u32, xi: u32, v: i32) -> Par {
    let h = i32(1u << (hrc.nivel - 1u));
    if k % 2u == 0u {
        let c = k / 2u;
        return fundir(t_de_baixo(2u * xi, v, c), t_de_baixo(2u * xi + 1u, v + 2 * i32(c) - h, c));
    }
    let a = (k - 1u) / 2u;
    let b = (k + 1u) / 2u;
    let mais = fundir(t_de_baixo(2u * xi, v, b), t_de_baixo(2u * xi + 1u, v + 2 * i32(b) - h, a));
    let menos = fundir(t_de_baixo(2u * xi, v, a), t_de_baixo(2u * xi + 1u, v + 2 * i32(a) - h, b));
    return Par(0.5 * (mais.r + menos.r), 0.5 * (mais.t + menos.t));
}

// ── T3..TN: a fusão de baixo para cima (Eq. 18–20) ──
@compute @workgroup_size(64)
fn hrc_sobe(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    if GRADE3D_HRC && COLUNAS_HRC {
        if gid.x < hrc.eixo.y {
            t_saida[indice_t(hrc.nx, hrc.nivel, gid.y, gid.x, gid.z)] =
                empacotar(fusao_de_baixo(gid.z, gid.y, i32(gid.x)));
        }
        return;
    }
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let d = decompor(e, (1u << hrc.nivel) + 1u);
    t_saida[e] = empacotar(fusao_de_baixo(d.x, d.y, i32(d.z)));
}

// ── R_{N−1}..R0: a fusão de cima para baixo (Eq. 14–17) ──
@compute @workgroup_size(64)
fn hrc_desce(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    if GRADE3D_HRC && COLUNAS_HRC {
        if gid.x < hrc.eixo.y {
            r_saida[indice_r(hrc.nx, hrc.nivel, gid.y, gid.x, gid.z)] =
                empacotar(Par(soma_da_descida(gid.z, gid.y, gid.x), 0.0));
        }
        return;
    }
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let d = decompor(e, 1u << hrc.nivel);
    r_saida[e] = empacotar(Par(soma_da_descida(d.x, d.y, d.z), 0.0));
}

// R_n no cone `m` da sonda `(xi, v)`: as duas metades do cone.
fn soma_da_descida(m: u32, xi: u32, v: u32) -> vec3<f32> {
    let n = hrc.nivel;
    let cones = 1u << n;
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
    return soma;
}

// O R0 do cone 0 na sonda `(u, v)` (o céu × o arco além da última).
fn r0_na_sonda(u: u32, v: u32) -> vec3<f32> {
    if u < hrc.nx {
        return desempacotar(r_cima[indice_r(hrc.nx, 0u, u, v, 0u)]).r;
    }
    return cas.ceu.rgb * arco(0u, 0u);
}

// ── A fluência do quadrante: L(u, v) += R0((u + 1, v), 0) ──
@compute @workgroup_size(64)
fn hrc_soma(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    var u = e % hrc.eixo.x;
    var v = e / hrc.eixo.x;
    if COLUNAS_HRC {
        // Um grupo = um ladrilho 8×8 de (u, v): oito v seguidos do R0 (por colunas) e oito texels
        // seguidos do acumulado em qualquer quadrante. O `total` conta os ladrilhos inteiros.
        let ladrilho = e / 64u;
        let l = e % 64u;
        let lu = (hrc.eixo.x + 7u) / 8u;
        u = (ladrilho % lu) * 8u + l / 8u;
        v = (ladrilho / lu) * 8u + l % 8u;
        if u >= hrc.eixo.x || v >= hrc.eixo.y {
            return;
        }
    }
    // ⭐ W5m: o «+1» só no mesmo meio ou para um mais claro; para DENTRO de um corpo (translúcido
    // também) lê-se o próprio u — a sonda de dentro não conta a entrada, logo não vê o brilho da face
    // que u vê (misturá-la por τ perdia τ·brilho: a sala a 0,99, −1 %; handoff W5m §2).
    let tau = meio(na_grade(f32(u), f32(v))).tau;
    var f = r0_na_sonda(u, v);
    if MAIS_UM_HRC && meio(na_grade(f32(u + 1u), f32(v))).tau >= max(tau, 1e-30) {
        f = r0_na_sonda(u + 1u, v);
    }
    // Partir de DENTRO de um translúcido conta como entrada (a lei do gabarito): os traços da sonda
    // começam «já dentro» e não a contam — o τ do TEXEL (W5k: não o da sonda lida). A sonda dentro de
    // um OPACO fica com a luz que a rodeia: a máscara tira-a da bilinear, mas onde as quatro de um texel
    // caem dentro (um enxame denso) a bilinear crua lê-a — com zero, pintas pretas (medido na foto da
    // cena 2, W5c).
    if tau > 0.0 && tau < 1.0 {
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
    let tau_p = meio(P).tau;
    let fora = tau_p > 0.0;
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
            // O vizinho do OUTRO lado de uma borda não entra (a opacidade difere): é outro campo, também
            // num translúcido (⭐ W5m — pesá-lo por `min τ / max τ` puxava a borda para o interior).
            if meio(vec2<f32>(q) + 0.5).tau != tau_p {
                continue;
            }
            soma += acumulado[u32(q.y) * hrc.grade.x + u32(q.x)].rgb;
            peso += 1.0;
        }
        f = soma / peso;
    }
    // ⭐ W5d — a luz que vem de FORA desta grade (a c0 do entorno, que não recolhe a emissão daqui):
    // os refletores e os corpos que brilham fora da grade da vista (o rebote de fora da grade).
    if hrc.entorno.w > 0.5 {
        let q = (P * hrc.entorno.z + hrc.entorno.xy) / vec2<f32>(textureDimensions(entorno_c0));
        f += textureSampleLevel(entorno_c0, entorno_amostra, q, 0.0).rgb;
    }
    var alfa = 1.0;
    if MASCARA_C0 && !fora {
        alfa = 0.0;
    }
    textureStore(c0_saida, gid.xy, vec4<f32>(f, alfa));
}
