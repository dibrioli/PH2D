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
    // ⭐ W5u: 1 = os fios fazem mais de uma entrada neste passe (a decisão é da grade, no uniforme).
    por_fio: u32,
    _p1: u32,
    _p2: u32,
    // ⭐ W5d — o MIOLO (texels da grade, `[x0, y0, x1, y1]`) cuja emissão este solver NÃO recolhe: ele é
    // o ENTORNO de outro, e só a luz que vem de fora da grade daquele entra nas cascatas dele.
    miolo: vec4<f32>,
    // ⭐ W5d — a c0 do ENTORNO somada na saída: `(desloc, escala, ligado)` — o texel da grade daquele
    // é `P · escala + desloc`.
    entorno: vec4<f32>,
    // ⭐ bug 6 (handoff W6g §32) — `(base, arquiva, colunas do entorno, miolo vácuo)`: na 2.ª passagem de um
    // ENTORNO (o miolo vácuo) os R deste passe arquivam-se em `arquivo_dos_r[base + i]`; na vista, o
    // `colunas` diz o layout do arquivo que ela lê.
    arquivo: vec4<u32>,
    // ⭐ bug 6 — a vista lê o R do entorno PELA BORDA: `(X, Y, K, ligado)` do entorno no MESMO quadrante
    // (K = o nível mais alto arquivado) e a base de cada nível no arquivo.
    fora: vec4<u32>,
    // ⭐ bug 6 — a JANELA do arquivo (canónico do entorno, texels: `U0, U1, V0, V1`, os fins exclusivos): só as
    // sondas sob a grade da vista (mais uma linha de cada lado) se guardam — é só lá que ela lê.
    janela: vec4<u32>,
    // ⭐ bug 6 — o canónico desta grade → o do entorno no mesmo quadrante: `c_e = c · z + xy`.
    fora_af: vec4<f32>,
    fora_base: array<vec4<u32>, 5>,
}

@group(3) @binding(4) var<uniform> hrc: Hrc;
// ⭐ W5z: os T e os R (ligações 5–9) e os seus acessores (`ler_t_a`, `ler_t_b`, `ler_r_cima`,
// `escrever_t`, `escrever_r`) vêm do prelúdio: `hrc_niveis_buffer.wgsl` ou `hrc_niveis_textura.wgsl`.
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
// ⭐ bug 6: os R de todos os níveis do ENTORNO — ele escreve-os, a vista lê-os no lugar do céu.
@group(2) @binding(17) var<storage, read_write> arquivo_dos_r: array<vec2<u32>>;
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
// ⭐ W5u (só em 3D) — CONES_HRC cones vizinhos por fio na descida: o T da aresta comum lido uma vez (1 = um
// cone por fio). PARES_SOBE_HRC pares (k par, k ímpar) por fio na subida (0 = uma entrada por fio). A conta
// de cada entrada é a mesma (handoff W5u).
override CONES_HRC: u32 = 4u;
override PARES_SOBE_HRC: u32 = 4u;
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

// ⭐ bug 6 (handoff W6g §32) — o meio que os traços leem: na passagem do ARQUIVO (`arquivo.w = 1`) o miolo é
// VÁCUO (nem tapa nem brilha: é da vista, que o traça fino) — o R do entorno é então a luz de FORA que chega à
// borda dela, e a vista lê-o lá sem a versão grossa dos opacos dela pelo meio.
fn meio_do_traco(x: vec2<f32>) -> Meio {
    if hrc.arquivo.w == 1u && no_miolo(x) {
        return Meio(1.0, 0.0, vec2<i32>(0), false, vec4<f32>(0.0));
    }
    return meio(x);
}

// ⭐ W5d — `x` (texels da grade) cai no miolo cuja emissão não se recolhe.
fn no_miolo(x: vec2<f32>) -> bool {
    return all(x >= hrc.miolo.xy) && all(x < hrc.miolo.zw);
}

// Traço(A, B) na lei do meio, as entradas em (A, B]; a queda conta-se de A.
fn tracar(A: vec2<f32>, B: vec2<f32>) -> Par {
    return tracar_de(A, meio_do_traco(A), B);
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
        let m = meio_do_traco(x);
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
    return desempacotar(ler_t_a(indice_t(hrc.nx_baixo, hrc.nivel - 1u, xi, u32(v), k)));
}

// T_n (t_a) e T_{n+1} (t_b) na fusão de cima para baixo.
fn t_deste(xi: u32, v: u32, k: u32) -> Par {
    return desempacotar(ler_t_a(indice_t(hrc.nx, hrc.nivel, xi, v, k)));
}

fn t_de_cima(xi: u32, v: u32, k: u32) -> Par {
    if xi >= hrc.nx_cima {
        return VACUO;
    }
    return desempacotar(ler_t_b(indice_t(hrc.nx_cima, hrc.nivel + 1u, xi, v, k)));
}

// R_{n+1} em (xi, v, j) — no topo, ou fora do que se guarda, o céu × o arco (o que está além).
fn r_de_cima(xi: u32, v: i32, j: u32) -> vec3<f32> {
    let n = hrc.nivel + 1u;
    if hrc.topo == 1u || xi >= hrc.nx_cima || v < 0 || v >= i32(hrc.eixo.y) {
        return de_fora(xi, v, j, n);
    }
    return desempacotar(ler_r_cima(indice_r(hrc.nx_cima, n, xi, u32(v), j))).r;
}

// ⭐ bug 6 (handoff W6g §32) — a luz de FORA entra pela BORDA: no lugar do céu, o R do ENTORNO no mesmo cone,
// onde o raio sai desta grade (a sonda `(xi, v)` do nível `n` recua pelo centro do cone `j` até à borda `b`); os T
// desta grade até lá tapam-na. O arquivo do entorno vem da passagem com o miolo VÁCUO, logo o R numa sonda dele
// DENTRO desta grade, sobre o mesmo raio, é o que atravessa `b`: lê-se a coluna de sondas mais perto ATRÁS de `b`
// (nunca uma além, cujo R perderia o que fica entre as duas), no nível mais fino em ângulo cuja coluna ainda cai
// dentro desta grade; acima dele, o cone-pai pela fração do arco; fora do entorno, o céu.
fn de_fora(xi: u32, v: i32, j: u32, n: u32) -> vec3<f32> {
    if hrc.fora.w == 0u {
        return cas.ceu.rgb * arco(n, j);
    }
    let lado = f32(1u << n);
    let e = vec2<f32>(f32(xi) * lado + 0.5, f32(v) + 0.5);
    let d = vec2<f32>(lado, 2.0 * f32(j) + 1.0 - lado);
    let x = vec2<f32>(hrc.eixo);
    var t = 0.0;
    if e.x > x.x {
        t = max(t, (e.x - x.x) / d.x);
    }
    if e.y < 0.0 && d.y < 0.0 {
        t = max(t, e.y / d.y);
    }
    if e.y > x.y && d.y > 0.0 {
        t = max(t, (e.y - x.y) / d.y);
    }
    let b = clamp(e - t * d, vec2<f32>(0.0), x);
    // b no canónico do entorno (o mesmo quadrante: a escala e uma translação, a direção não muda).
    let xe = vec2<f32>(hrc.fora.xy);
    let ce = b * hrc.fora_af.z + hrc.fora_af.xy;
    let passo = d / d.x;
    // O nível: o mais fino em ângulo cuja coluna cabe nesta grade (o `fora.z`, da CPU); a coluna de sondas
    // mais perto ATRÁS de `b` e o recuo até ela, sobre o raio.
    let m = min(n, hrc.fora.z);
    let lado_e = f32(1u << m);
    let coluna = floor((ce.x - 0.5) / lado_e);
    let recuo = ce.x - (coluna * lado_e + 0.5);
    let pai = j >> (n - m);
    let nx = (hrc.fora.x + (1u << m) - 1u) >> m;
    // O arquivo guarda a radiância MÉDIA do cone (R / arco): o cone fino `j` é o pai × o arco dele.
    let ceu = cas.ceu.rgb;
    if coluna < 0.0 || coluna >= f32(nx) {
        return ceu * arco(n, j);
    }
    let vf = ce.y - recuo * passo.y - 0.5;
    let v0 = floor(vf);
    let fr = vf - v0;
    var soma = vec3<f32>(0.0);
    for (var k = 0; k < 2; k++) {
        let s = v0 + f32(k);
        var r = ceu;
        if s >= 0.0 && s < xe.y {
            let i = indice_na_janela(m, u32(coluna), u32(s), pai);
            if i != FORA_DA_JANELA {
                r = desempacotar(arquivo_dos_r[hrc.fora_base[m / 4u][m % 4u] + i]).r;
            }
        }
        soma += select(1.0 - fr, fr, k == 1) * r;
    }
    return soma * arco(n, j);
}

const FORA_DA_JANELA: u32 = 0xffffffffu;

// O R (nível `m`, sonda `(k, v)`, cone `c`) na janela do arquivo (a partir da base do nível), ou
// `FORA_DA_JANELA`. As colunas do nível são as `k` com `k·2ᵐ` em `[U0, U1)` arredondado para fora.
fn indice_na_janela(m: u32, k: u32, v: u32, c: u32) -> u32 {
    let j = hrc.janela;
    let k0 = j.x >> m;
    let colunas = ((j.y + (1u << m) - 1u) >> m) + 1u - k0;
    let linhas = j.w - j.z;
    if k < k0 || k >= k0 + colunas || v < j.z || v >= j.w {
        return FORA_DA_JANELA;
    }
    return (c * colunas + (k - k0)) * linhas + (v - j.z);
}

// ⭐ bug 6 — a 2.ª passagem de um ENTORNO guarda o R que acabou de escrever — a radiância MÉDIA do cone
// (R / arco) —, se a sonda cai na janela.
fn arquivar(xi: u32, v: u32, c: u32, e: vec2<u32>) {
    if hrc.arquivo.y == 1u {
        let i = indice_na_janela(hrc.nivel, xi, v, c);
        if i != FORA_DA_JANELA {
            let r = desempacotar(e).r / arco(hrc.nivel, c);
            arquivo_dos_r[hrc.arquivo.x + i] = empacotar(Par(r, 0.0));
        }
    }
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
            escrever_t(indice_t(hrc.nx, n, sv.x, sv.y, k), empacotar(VACUO));
        }
        return;
    }
    let m0 = meio_do_traco(l[0]);
    for (var k = 0u; k <= (1u << n); k++) {
        let B = na_grade(u + lado, v + 2.0 * f32(k) - lado);
        escrever_t(indice_t(hrc.nx, n, sv.x, sv.y, k), empacotar(tracar_de(l[0], m0, B)));
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
        escrever_t(e, empacotar(VACUO));
        return;
    }
    escrever_t(e, empacotar(tracar(ab[0], ab[1])));
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
    if GRADE3D_HRC && COLUNAS_HRC && PARES_SOBE_HRC > 0u && hrc.por_fio == 1u {
        if gid.x < hrc.eixo.y {
            subir_pares(gid);
        }
        return;
    }
    if GRADE3D_HRC && COLUNAS_HRC {
        if gid.x < hrc.eixo.y {
            escrever_t(indice_t(hrc.nx, hrc.nivel, gid.y, gid.x, gid.z),
                empacotar(fusao_de_baixo(gid.z, gid.y, i32(gid.x))));
        }
        return;
    }
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let d = decompor(e, (1u << hrc.nivel) + 1u);
    escrever_t(e, empacotar(fusao_de_baixo(d.x, d.y, i32(d.z))));
}

// ── R_{N−1}..R0: a fusão de cima para baixo (Eq. 14–17) ──
@compute @workgroup_size(64)
fn hrc_desce(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
    if GRADE3D_HRC && COLUNAS_HRC && CONES_HRC > 1u && hrc.por_fio == 1u {
        if gid.x < hrc.eixo.y {
            descer_cones(gid);
        }
        return;
    }
    if GRADE3D_HRC && COLUNAS_HRC {
        if gid.x < hrc.eixo.y {
            escrever_e_arquivar_r(indice_r(hrc.nx, hrc.nivel, gid.y, gid.x, gid.z), gid.y, gid.x, gid.z,
                empacotar(Par(soma_da_descida(gid.z, gid.y, gid.x), 0.0)));
        }
        return;
    }
    let e = entrada(gid, nw);
    if e >= hrc.total {
        return;
    }
    let d = decompor(e, 1u << hrc.nivel);
    escrever_e_arquivar_r(e, d.y, d.z, d.x, empacotar(Par(soma_da_descida(d.x, d.y, d.z), 0.0)));
}

fn escrever_e_arquivar_r(i: u32, xi: u32, v: u32, c: u32, e: vec2<u32>) {
    escrever_r(i, e);
    arquivar(xi, v, c, e);
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

// ⭐ W5u — os pares `c0..c0 + PARES_SOBE_HRC` da sonda `(xi, v)` de T_n: k = 2c (a fusão direta) e
// k = 2c + 1 (a média das duas), a MESMA conta do [`fusao_de_baixo`]; o T_{n−1} de baixo da aresta c + 1
// passa ao par seguinte.
fn subir_pares(gid: vec3<u32>) {
    let xi = gid.y;
    let v = i32(gid.x);
    let h = i32(1u << (hrc.nivel - 1u));
    let kmax = 1u << hrc.nivel;
    let c0 = gid.z * PARES_SOBE_HRC;
    var b_ant = t_de_baixo(2u * xi, v, c0);
    for (var p = 0u; p < PARES_SOBE_HRC; p++) {
        let c = c0 + p;
        if 2u * c > kmax {
            break;
        }
        let par = fundir(b_ant, t_de_baixo(2u * xi + 1u, v + 2 * i32(c) - h, c));
        escrever_t(indice_t(hrc.nx, hrc.nivel, xi, gid.x, 2u * c), empacotar(par));
        if 2u * c + 1u > kmax {
            break;
        }
        let b_prox = t_de_baixo(2u * xi, v, c + 1u);
        let mais = fundir(b_prox, t_de_baixo(2u * xi + 1u, v + 2 * i32(c + 1u) - h, c));
        let menos = fundir(b_ant, t_de_baixo(2u * xi + 1u, v + 2 * i32(c) - h, c + 1u));
        escrever_t(indice_t(hrc.nx, hrc.nivel, xi, gid.x, 2u * c + 1u),
            empacotar(Par(0.5 * (mais.r + menos.r), 0.5 * (mais.t + menos.t))));
        b_ant = b_prox;
    }
}

// ⭐ W5u — o T da aresta `aresta` que a descida da sonda `(xi, v)` lê: T_n se xi ímpar, T_{n+1} (aresta
// 2·aresta) se par.
fn t_da_aresta(xi: u32, v: u32, aresta: u32) -> Par {
    if xi % 2u == 1u {
        return t_deste(xi, v, aresta);
    }
    return t_de_cima(xi / 2u, v, 2u * aresta);
}

// ⭐ W5u — a metade do cone fino `j` na aresta `aresta`, com o T dela já lido: a MESMA conta do
// [`soma_da_descida`].
fn metade_da_descida(xi: u32, v: u32, aresta: u32, j: u32, tr: Par) -> vec3<f32> {
    let n = hrc.nivel;
    let lado = i32(1u << n);
    let a = arco(n + 1u, j);
    if xi % 2u == 1u {
        let q = i32(v) + 2 * i32(aresta) - lado;
        return a * tr.r + tr.t * r_de_cima((xi + 1u) / 2u, q, j);
    }
    let f0 = r_de_cima(xi / 2u, i32(v), j);
    let q = i32(v) + 2 * i32(2u * aresta) - 2 * lado;
    let f1 = a * tr.r + tr.t * r_de_cima(xi / 2u + 1u, q, j);
    return 0.5 * (f0 + f1);
}

// ⭐ W5u — os cones `m0..m0 + CONES_HRC` da sonda `(xi, v)` de R_n: o T da aresta m + 1 passa ao cone m + 1.
fn descer_cones(gid: vec3<u32>) {
    let xi = gid.y;
    let v = gid.x;
    let cones = 1u << hrc.nivel;
    let m0 = gid.z * CONES_HRC;
    var t_ant = t_da_aresta(xi, v, m0);
    for (var c = 0u; c < CONES_HRC; c++) {
        let m = m0 + c;
        if m >= cones {
            break;
        }
        let t_prox = t_da_aresta(xi, v, m + 1u);
        var soma = vec3<f32>(0.0);
        soma += metade_da_descida(xi, v, m, 2u * m, t_ant);
        soma += metade_da_descida(xi, v, m + 1u, 2u * m + 1u, t_prox);
        escrever_e_arquivar_r(indice_r(hrc.nx, hrc.nivel, xi, v, m), xi, v, m, empacotar(Par(soma, 0.0)));
        t_ant = t_prox;
    }
}

// O R0 do cone 0 na sonda `(u, v)` (além da última: o que vem de fora, [`de_fora`]).
fn r0_na_sonda(u: u32, v: u32) -> vec3<f32> {
    if u < hrc.nx {
        return desempacotar(ler_r_cima(indice_r(hrc.nx, 0u, u, v, 0u))).r;
    }
    return de_fora(u, i32(v), 0u, 0u);
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
