// ⭐ W5d — a BORDA EXATA da captura que a C3 lê, um texel de cada vez — a reconstrução do AAEDT
// (Gustavson & Strand, «Anti-aliased Euclidean distance transform», Pattern Recognition Letters
// 32(2), 2011): a cobertura é a ÁREA exata, logo um texel cheio está todo dentro, um vazio todo fora,
// e num parcial a borda é a reta de normal = o gradiente da cobertura (Sobel de pesos √2) que deixa
// dentro a área igual à cobertura. Saída: `(cobertura, normal para dentro, recuo)` — dentro ⟺
// `(p − centro)·normal ≥ recuo`; normal nula = sem gradiente (um fio de um texel: o iso-½).
//
// Concatenado depois de `c2_rc.wgsl` (o `dentro_exato` dele lê esta saída no lugar da cobertura).

@group(3) @binding(12) var borda_cobertura: texture_2d<f32>;
@group(3) @binding(13) var borda_saida: texture_storage_2d<rgba16float, write>;
@group(3) @binding(14) var borda_vizinhos: sampler;
// ⭐ W6f — as cunhas dos parciais de vértice (`BORDA_CANTOS`; o [`c3_cantos`] espalha-as).
@group(3) @binding(15) var borda_vertices: texture_storage_2d<rgba16float, write>;

// A cunha aceite pelo [`pela_cunha`] no referencial do texel: `(θ₁, s₁, θ₂, s₂)` — dentro ⟺ `y·u(θ) ≥ s`
// das duas (convexo) ou de uma (côncavo: `θ₂ + CONCAVO`); `SEM_CUNHA` no `x` = nenhuma.
const SEM_CUNHA: f32 = 100.0;
const CONCAVO: f32 = 10.0;
var<private> cunha_achada: vec4<f32> = vec4<f32>(SEM_CUNHA, 0.0, 0.0, 0.0);

// O `t ≥ 0` tal que a área do texel unitário (centrado) além da reta `u·x = t` vale `area ≤ ½` — a
// inversa da conta do trapézio (o `area_alem` da C3).
fn recuo_da_area(u: vec2<f32>, area: f32) -> f32 {
    let p = max(0.5 * min(abs(u.x), abs(u.y)), 1e-6);
    let q = 0.5 * max(abs(u.x), abs(u.y));
    if area <= p / (2.0 * q) {
        return p + q - sqrt(8.0 * p * q * area);
    }
    return q - 2.0 * q * area;
}

// A normal para dentro pelo Sobel (pesos √2) da 3×3 `k` (a cobertura presa a 1); zero sem gradiente.
fn normal_sobel(k: array<f32, 9>) -> vec2<f32> {
    var c: array<f32, 9>;
    for (var i = 0; i < 9; i++) {
        c[i] = min(k[i], 1.0);
    }
    let r2 = 1.4142135;
    let g = vec2<f32>(
        (c[2] + r2 * c[5] + c[8]) - (c[0] + r2 * c[3] + c[6]),
        (c[6] + r2 * c[7] + c[8]) - (c[0] + r2 * c[1] + c[2]),
    );
    let m = length(g);
    if m <= 1e-3 {
        return vec2<f32>(0.0);
    }
    return g / m;
}

// O recuo da reta de normal `u` que deixa dentro a área `a` de um texel.
fn recuo_de(u: vec2<f32>, a: f32) -> f32 {
    return select(-recuo_da_area(u, 1.0 - a), recuo_da_area(u, a), a <= 0.5);
}

// ⭐ W6g r8 — o texel cuja reta não explica a 3×3 (o candidato a vértice), no `c3_borda_adia`: a cunha
// sai para o [`c3_borda_vertices`] (a `pela_cunha` no kernel da captura inteira custava 55–61 registos
// contra 32; handoff W6g §21).
var<private> borda_adiada: bool = false;

// O que o `c3_borda` grava no texel `t` dada a sua 3×3 `k` (as coordenadas presas à textura, como a
// leitura direta): `(normal para dentro, recuo)` de um parcial; na contínua (`BORDA_CONTINUA`) o 3.º é a
// DISTÂNCIA com sinal no centro — o parcial −recuo, o fio cobertura − ½, o cheio/vazio da faixa pelas
// retas dos vizinhos ([`pelas_retas_proprias`]), o resto ±1. Com `adia`, o candidato a vértice leva a
// própria reta e o [`borda_adiada`].
fn linha_de(t: vec2<i32>, n: vec2<i32>, k: array<f32, 9>, adia: bool) -> vec3<f32> {
    let a = k[4];
    if a > 0.0 && a < 1.0 {
        let u = normal_sobel(k);
        if all(u == vec2<f32>(0.0)) {
            return vec3<f32>(0.0, 0.0, select(0.0, a - 0.5, BORDA_CONTINUA));
        }
        let r = recuo_de(u, a);
        if BORDA_CONTINUA && BORDA_VERTICE_AREA > 0.0 && !explica_a_3x3(u, r, k) {
            if adia {
                borda_adiada = true;
                return vec3<f32>(u, -r);
            }
            return vec3<f32>(u, pela_cunha(t, n, a, u, -r));
        }
        return vec3<f32>(u, select(r, -r, BORDA_CONTINUA));
    }
    if !BORDA_CONTINUA {
        return vec3<f32>(0.0);
    }
    return vec3<f32>(0.0, 0.0, pelas_retas_proprias(t, n, a >= 1.0));
}

// ⭐ W6d — a distância com sinal no centro de um CHEIO ou VAZIO da faixa pela reta PRÓPRIA de cada parcial
// vizinho (o Sobel da 3×3 dele, da 5×5 lida aqui; o recuo pela área dele): cheio = a menor ≥ 0 (num
// convexo o interior é a interseção dos semiplanos de apoio), vazio = a maior < 0; nenhuma ⇒ ±1. A reta
// com a normal do PRÓPRIO Sobel (diagonal no canto) chanfrava o canto convexo ~1,3 texel (handoff W6d §14).
fn pelas_retas_proprias(t: vec2<i32>, n: vec2<i32>, cheio: bool) -> f32 {
    var c: array<f32, 25>;
    for (var j = 0; j < 5; j++) {
        for (var i = 0; i < 5; i++) {
            let q = clamp(t + vec2<i32>(i - 2, j - 2), vec2<i32>(0), n - 1);
            c[j * 5 + i] = textureLoad(borda_cobertura, q, 0).r;
        }
    }
    var melhor = select(-1.0, 1.0, cheio);
    for (var j = 1; j < 4; j++) {
        for (var i = 1; i < 4; i++) {
            let a = c[j * 5 + i];
            let o = vec2<i32>(i - 2, j - 2);
            let q = t + o;
            if (i == 2 && j == 2) || any(q < vec2<i32>(0)) || any(q >= n) || a <= 0.0 || a >= 1.0 {
                continue;
            }
            let k = array<f32, 9>(
                c[(j - 1) * 5 + i - 1], c[(j - 1) * 5 + i], c[(j - 1) * 5 + i + 1],
                c[j * 5 + i - 1], a, c[j * 5 + i + 1],
                c[(j + 1) * 5 + i - 1], c[(j + 1) * 5 + i], c[(j + 1) * 5 + i + 1],
            );
            let u = normal_sobel(k);
            if all(u == vec2<f32>(0.0)) {
                continue;
            }
            // dentro ⟺ (x − c_q)·u ≥ r; no centro deste, x − c_q = −o.
            let d = -dot(vec2<f32>(o), u) - recuo_de(u, a);
            if cheio && d >= 0.0 {
                melhor = min(melhor, d);
            } else if !cheio && d < 0.0 {
                melhor = max(melhor, d);
            }
        }
    }
    return melhor;
}

// A fração do texel unitário (centrado) DENTRO de `y·u ≥ t` (`u` unitária, `t` de qualquer sinal).
fn area_dentro(u: vec2<f32>, t: f32) -> f32 {
    return select(1.0 - area_alem(u, -t, 1.0), area_alem(u, t, 1.0), t >= 0.0);
}

// ⭐ W6e — a reta `(u, r)` de um parcial explica a sua 3×3 `k`: a cobertura que ela prevê em cada
// vizinho difere da lida em no máximo `BORDA_VERTICE_PRE`. No vértice a reta sai diagonal e não explica.
fn explica_a_3x3(u: vec2<f32>, r: f32, k: array<f32, 9>) -> bool {
    for (var j = 0; j < 3; j++) {
        for (var i = 0; i < 3; i++) {
            if i == 1 && j == 1 {
                continue;
            }
            // no vizinho de deslocamento `o`: dentro ⟺ (y + o)·u ≥ r.
            let o = vec2<f32>(f32(i - 1), f32(j - 1));
            if abs(area_dentro(u, r - dot(o, u)) - min(k[j * 3 + i], 1.0)) > BORDA_VERTICE_PRE {
                return false;
            }
        }
    }
    return true;
}

// A fração do texel unitário (centrado) dentro de `y·u1 ≥ t1` E `y·u2 ≥ t2`, pela integral em
// [`COLUNAS`] colunas (o comprimento do intervalo de `y` no meio de cada uma).
const COLUNAS: i32 = 32;
fn area_da_intersecao(u1: vec2<f32>, t1: f32, u2: vec2<f32>, t2: f32) -> f32 {
    var s = 0.0;
    for (var k = 0; k < COLUNAS; k++) {
        let x = (f32(k) + 0.5) / f32(COLUNAS) - 0.5;
        var lo = -0.5;
        var hi = 0.5;
        for (var l = 0; l < 2; l++) {
            let u = select(u2, u1, l == 0);
            let r = select(t2, t1, l == 0) - x * u.x;
            if abs(u.y) < 1e-6 {
                hi = select(hi, -1.0, r > 0.0);
            } else if u.y > 0.0 {
                lo = max(lo, r / u.y);
            } else {
                hi = min(hi, r / u.y);
            }
        }
        s += max(hi - lo, 0.0);
    }
    return s / f32(COLUNAS);
}

// ⭐ W6e — a distância com sinal no centro do parcial `t` (cobertura `a`, normal própria `ut`) do VÉRTICE
// pela CUNHA das retas dos parciais a dois texels (o anel de Chebyshev 2: as 3×3 deles já não contêm o
// vértice; as adjacentes saem inclinadas por ele): a normal mais girada para cada lado de `ut`; convexo
// ⟺ cada um dentro da reta do outro (`d₂(c_q₁) + d₁(c_q₂) > 0`) — a interseção (o mínimo) —, côncavo a
// união (o máximo). Se a cunha não deixa dentro do texel a área `a` (a menos de `BORDA_VERTICE_AREA`: um
// arco, não um canto), `propria`.
fn pela_cunha(t: vec2<i32>, n: vec2<i32>, a: f32, ut: vec2<f32>, propria: f32) -> f32 {
    // As retas no referencial de `t`: dentro ⟺ y·u ≥ s, com o centro do parcial em `o`.
    var u1 = vec2<f32>(0.0);
    var s1 = 0.0;
    var o1 = vec2<f32>(0.0);
    var c1 = 0.0;
    var u2 = vec2<f32>(0.0);
    var s2 = 0.0;
    var o2 = vec2<f32>(0.0);
    var c2 = 0.0;
    for (var j = -2; j <= 2; j++) {
        for (var i = -2; i <= 2; i++) {
            let o = vec2<i32>(i, j);
            let q = t + o;
            if max(abs(i), abs(j)) != 2 || any(q < vec2<i32>(0)) || any(q >= n) {
                continue;
            }
            let aq = textureLoad(borda_cobertura, q, 0).r;
            // ⭐ W6f (`BORDA_CANTOS`): um CHEIO do anel com um vazio ao lado dá a reta da face alinhada à
            // grade (a aresta entre os dois; ela não tem parciais).
            if BORDA_CANTOS && aq >= 1.0 {
                for (var e = 0; e < 4; e++) {
                    let d = select(select(vec2<i32>(0, -1), vec2<i32>(0, 1), e == 3), select(vec2<i32>(-1, 0), vec2<i32>(1, 0), e == 1), e < 2);
                    let r = q + d;
                    if any(r < vec2<i32>(0)) || any(r >= n) || textureLoad(borda_cobertura, r, 0).r > 0.0 {
                        continue;
                    }
                    let u = -vec2<f32>(d);
                    let c = ut.x * u.y - ut.y * u.x;
                    let ov = vec2<f32>(o);
                    if c > c1 {
                        c1 = c;
                        u1 = u;
                        o1 = ov;
                        s1 = -0.5 + dot(ov, u);
                    }
                    if c < c2 {
                        c2 = c;
                        u2 = u;
                        o2 = ov;
                        s2 = -0.5 + dot(ov, u);
                    }
                }
                continue;
            }
            if aq <= 0.0 || aq >= 1.0 {
                continue;
            }
            var k: array<f32, 9>;
            for (var y = 0; y < 3; y++) {
                for (var x = 0; x < 3; x++) {
                    k[y * 3 + x] = textureLoad(borda_cobertura, clamp(q + vec2<i32>(x - 1, y - 1), vec2<i32>(0), n - 1), 0).r;
                }
            }
            let u = normal_sobel(k);
            if all(u == vec2<f32>(0.0)) {
                continue;
            }
            let c = ut.x * u.y - ut.y * u.x;
            let ov = vec2<f32>(o);
            if c > c1 {
                c1 = c;
                u1 = u;
                o1 = ov;
                s1 = recuo_de(u, aq) + dot(ov, u);
            }
            if c < c2 {
                c2 = c;
                u2 = u;
                o2 = ov;
                s2 = recuo_de(u, aq) + dot(ov, u);
            }
        }
    }
    if abs(u1.x * u2.y - u1.y * u2.x) < 1e-3 {
        return propria;
    }
    let convexo = (dot(o1, u2) - s2) + (dot(o2, u1) - s1) > 0.0;
    let i12 = area_da_intersecao(u1, s1, u2, s2);
    let modelo = select(area_dentro(u1, s1) + area_dentro(u2, s2) - i12, i12, convexo);
    if abs(modelo - a) > BORDA_VERTICE_AREA {
        return propria;
    }
    cunha_achada = vec4<f32>(atan2(u1.y, u1.x), s1, atan2(u2.y, u2.x) + select(CONCAVO, 0.0, convexo), s2);
    return select(max(-s1, -s2), min(-s1, -s2), convexo);
}

@compute @workgroup_size(8, 8)
fn c3_borda(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = vec2<i32>(textureDimensions(borda_cobertura));
    let t = vec2<i32>(gid.xy);
    if all(t < n) {
        borda_do_texel(t, n, false);
    }
}

// ⭐ W6g — a lista dos blocos 8×8 com candidato a vértice: o cabeçalho é o despacho INDIRETO do
// [`c3_cantos`] (`x` = quantos, `y` = 9: cada um espalha nos 3×3 blocos à volta), depois o bloco `x | y << 16`.
struct ListaDosCantos {
    n: atomic<u32>,
    // `9` e `1`, escritos pelo [`c3_cantos_zera`].
    y: u32,
    z: u32,
    blocos: array<u32>,
}
struct ListaLida {
    n: u32,
    y: u32,
    z: u32,
    blocos: array<u32>,
}
@group(3) @binding(19) var<storage, read_write> borda_lista: ListaDosCantos;
// ⭐ W6g r8/r9 — a marca de cada bloco listado neste quadro: `LISTADO` | o índice dele na lista + 1 (o slot
// da `V` no atlas); o [`c3_cantos_limpa`] repõe-na a 0. Reivindica-se por `atomicOr` (idempotente: uma troca
// apagava o índice que o vencedor já tinha escrito).
const LISTADO: u32 = 0x80000000u;
@group(3) @binding(21) var<storage, read_write> borda_marcas: array<atomic<u32>>;
// ⭐ W6g r9 — os blocos listados em que o [`c3_borda_vertices`] achou cunha: só eles espalham ([`c3_cantos`]).
@group(3) @binding(24) var<storage, read_write> lista_com_cunha: ListaDosCantos;
// ⭐ W6g r9 — `[slots da V2 usados, listados recusados (sem slot da V), blocos recusados (sem slot da V2)]`.
@group(3) @binding(25) var<storage, read_write> cantos_contadores: array<atomic<u32>, 4>;
var<workgroup> vertices_com_cunha: atomic<u32>;
// A entrada da tabela do vencedor de um bloco antes de saber o slot ([`c3_cantos`]).
const RESERVADO: u32 = 0xffffffffu;
// O atlas da `V` e da `V2`: blocos 8×8 em linhas de `ATLAS_LARGURA` (o slot `s` no bloco `(s % L, s / L)`).
const ATLAS_LARGURA: u32 = 128u;

fn no_atlas(s: u32, t: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(i32(s % ATLAS_LARGURA), i32(s / ATLAS_LARGURA)) * 8 + (t & vec2<i32>(7));
}

fn slots_do_atlas(d: vec2<u32>) -> u32 {
    return (d.x / 8u) * (d.y / 8u);
}

// O bloco 8×8 de `t` na fila dos blocos de uma captura de largura `w`.
fn bloco_de(t: vec2<i32>, w: i32) -> u32 {
    return u32(t.y / 8 * ((w + 7) / 8) + t.x / 8);
}

// A borda com os cantos (`BORDA_CANTOS`): a de todo texel sem a `pela_cunha`; o candidato a vértice
// lista o seu bloco (uma vez) para o [`c3_borda_vertices`].
@compute @workgroup_size(8, 8)
fn c3_borda_adia(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = vec2<i32>(textureDimensions(borda_cobertura));
    let t = vec2<i32>(gid.xy);
    if all(t < n) {
        borda_do_texel(t, n, true);
        let b = bloco_de(t, n.x);
        if borda_adiada && atomicOr(&borda_marcas[b], LISTADO) == 0u {
            let i = atomicAdd(&borda_lista.n, 1u);
            borda_lista.blocos[i] = u32(t.x / 8) | (u32(t.y / 8) << 16u);
            atomicOr(&borda_marcas[b], i + 1u);
        }
    }
}

// ⭐ W6g r8 — em despacho indireto sobre a lista deste quadro (só o `y = 0` dos 9 do cabeçalho): os 64 texels
// do bloco listado com a `pela_cunha` — a borda regravada (a mesma conta) e a `V` do bloco no slot dele do
// atlas (r9). Sem slot, o bloco fica na lei W6d (a borda do [`c3_borda_adia`]) e conta-se.
@compute @workgroup_size(8, 8)
fn c3_borda_vertices(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(workgroup_id) wid: vec3<u32>) {
    if wid.y != 0u {
        return;
    }
    if wid.x >= slots_do_atlas(textureDimensions(borda_vertices)) {
        if all(lid.xy == vec2<u32>(0u)) {
            atomicAdd(&cantos_contadores[1], 1u);
        }
        return;
    }
    let e = cantos_lista.blocos[wid.x];
    let n = vec2<i32>(textureDimensions(borda_cobertura));
    let t = vec2<i32>(i32(e & 0xffffu), i32(e >> 16u)) * 8 + vec2<i32>(lid.xy);
    if all(t < n) {
        borda_do_texel(t, n, false);
        textureStore(borda_vertices, no_atlas(wid.x, t), cunha_achada);
        if cunha_achada.x < SEM_CUNHA {
            atomicStore(&vertices_com_cunha, 1u);
        }
    }
    workgroupBarrier();
    if all(lid.xy == vec2<u32>(0u)) && atomicLoad(&vertices_com_cunha) != 0u {
        lista_com_cunha.blocos[atomicAdd(&lista_com_cunha.n, 1u)] = e;
    }
}

@compute @workgroup_size(1)
fn c3_cantos_zera() {
    atomicStore(&borda_lista.n, 0u);
    atomicStore(&lista_com_cunha.n, 0u);
    borda_lista.y = 9u;
    borda_lista.z = 1u;
    lista_com_cunha.y = 9u;
    lista_com_cunha.z = 1u;
    for (var k = 0; k < 4; k++) {
        atomicStore(&cantos_contadores[k], 0u);
    }
}


fn borda_do_texel(t: vec2<i32>, n: vec2<i32>, adia: bool) {
    let a = textureLoad(borda_cobertura, t, 0).r;
    var linha = vec3<f32>(0.0);
    if BORDA_CONTINUA {
        // ⭐ W6d: a 3×3 por quatro `textureGather` nos cantos de `t` (o amostrador prende à borda, como o
        // `clamp` da leitura direta); x = (u0, v1), y = (u1, v1), z = (u1, v0), w = (u0, v0).
        let nf = vec2<f32>(n);
        let g00 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t) / nf);
        let g11 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t + 1) / nf);
        let g10 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t + vec2<i32>(1, 0)) / nf);
        let g01 = textureGather(0, borda_cobertura, borda_vizinhos, vec2<f32>(t + vec2<i32>(0, 1)) / nf);
        // Longe de toda borda (a grande maioria) nenhum parcial na 3×3: ±1, sem listas.
        let parcial = (g00 > vec4<f32>(0.0) & g00 < vec4<f32>(1.0)) | (g11 > vec4<f32>(0.0) & g11 < vec4<f32>(1.0))
            | (g10 > vec4<f32>(0.0) & g10 < vec4<f32>(1.0)) | (g01 > vec4<f32>(0.0) & g01 < vec4<f32>(1.0));
        if any(parcial) {
            let k = array<f32, 9>(g00.w, g00.z, g10.z, g00.x, a, g10.y, g01.x, g01.y, g11.y);
            linha = linha_de(t, n, k, adia);
        } else {
            linha = vec3<f32>(0.0, 0.0, select(-1.0, 1.0, a >= 1.0));
        }
    } else if a > 0.0 && a < 1.0 {
        var k: array<f32, 9>;
        for (var j = 0; j < 3; j++) {
            for (var i = 0; i < 3; i++) {
                let q = clamp(t + vec2<i32>(i - 1, j - 1), vec2<i32>(0), n - 1);
                k[j * 3 + i] = textureLoad(borda_cobertura, q, 0).r;
            }
        }
        linha = linha_de(t, n, k, adia);
    }
    textureStore(borda_saida, t, vec4<f32>(a, linha));
}

// ⭐ W6f — a cada texel da captura, a cunha do parcial de VÉRTICE mais perto a Chebyshev ≤ 2 (o primeiro da
// varredura no empate: determinístico) no referencial dele: o `meio` decide ali pelas duas retas, sem
// interpolar (a bilinear de retas diferentes arredonda o vértice; handoff W6f §4).
@group(3) @binding(16) var cantos_vertices: texture_2d<f32>;
@group(3) @binding(17) var cantos_saida: texture_storage_2d<rgba16float, write>;
@group(3) @binding(20) var<storage, read> cantos_lista: ListaLida;
// ⭐ W6g r9 — a tabela por bloco da captura que o `meio` lê (`cantos_tabela`, `c2_rc.wgsl`): 0 = sem cunha,
// senão o slot da `V2` no atlas + 1.
@group(3) @binding(23) var<storage, read_write> cantos_tabela_escrita: array<atomic<u32>>;
var<workgroup> cantos_bloco: array<vec4<f32>, 144>;
var<workgroup> cantos_com_cunha: atomic<u32>;
var<workgroup> cantos_slot: u32;

// ⭐ W6g: em despacho INDIRETO — o grupo `(i, k)` cobre o bloco `k` dos 3×3 à volta do `i`-ésimo listado
// COM cunha (r9: a lista B do [`c3_borda_vertices`]; um texel a Chebyshev ≤ 2 de uma cunha está num bloco
// vizinho do dela). A apara lê a `V` pela marca do bloco (o slot); o bloco com alguma cunha reivindica a sua
// entrada na tabela (o primeiro dos vizinhos a chegar escreve-o, com os mesmos bits) e um slot da `V2`.
@compute @workgroup_size(8, 8)
fn c3_cantos(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(workgroup_id) wid: vec3<u32>) {
    let n = vec2<i32>(textureDimensions(borda_cobertura));
    let slots_v = slots_do_atlas(textureDimensions(cantos_vertices));
    let e = cantos_lista.blocos[wid.x];
    let k9 = i32(wid.y);
    let b0 = (vec2<i32>(i32(e & 0xffffu), i32(e >> 16u)) + vec2<i32>(k9 % 3 - 1, k9 / 3 - 1)) * 8;
    let canto = b0 - 2;
    for (var k = lid.y * 8u + lid.x; k < 144u; k += 64u) {
        let q = canto + vec2<i32>(i32(k % 12u), i32(k / 12u));
        var v = vec4<f32>(SEM_CUNHA, 0.0, 0.0, 0.0);
        if all(q >= vec2<i32>(0)) && all(q < n) {
            let m = atomicLoad(&borda_marcas[bloco_de(q, n.x)]) & ~LISTADO;
            if m != 0u && m - 1u < slots_v {
                v = textureLoad(cantos_vertices, no_atlas(m - 1u, q), 0);
            }
        }
        cantos_bloco[k] = v;
    }
    workgroupBarrier();
    let t = b0 + vec2<i32>(lid.xy);
    let no_bloco = all(b0 >= vec2<i32>(0)) && all(b0 < n);
    var melhor = vec4<f32>(SEM_CUNHA, 0.0, 0.0, 0.0);
    if all(t >= vec2<i32>(0)) && all(t < n) {
        var d_melhor = 99;
        for (var j = -2; j <= 2; j++) {
            for (var i = -2; i <= 2; i++) {
                let v = cantos_bloco[u32(i32(lid.y) + 2 + j) * 12u + u32(i32(lid.x) + 2 + i)];
                let d = i * i + j * j;
                if v.x >= SEM_CUNHA || d >= d_melhor {
                    continue;
                }
                d_melhor = d;
                // dentro ⟺ (y + t − q)·u ≥ s, com t − q = −(i, j).
                let o = vec2<f32>(f32(i), f32(j));
                let concavo = v.z > 0.5 * CONCAVO;
                let t2 = select(v.z, v.z - CONCAVO, concavo);
                melhor = vec4<f32>(
                    v.x,
                    v.y + dot(o, vec2<f32>(cos(v.x), sin(v.x))),
                    v.z,
                    v.w + dot(o, vec2<f32>(cos(t2), sin(t2))),
                );
            }
        }
        if melhor.x < SEM_CUNHA {
            atomicStore(&cantos_com_cunha, 1u);
        }
    }
    workgroupBarrier();
    if all(lid.xy == vec2<u32>(0u)) {
        cantos_slot = 0u;
        if no_bloco && atomicLoad(&cantos_com_cunha) != 0u {
            let b = bloco_de(b0, n.x);
            var r = atomicCompareExchangeWeak(&cantos_tabela_escrita[b], 0u, RESERVADO);
            while !r.exchanged && r.old_value == 0u {
                r = atomicCompareExchangeWeak(&cantos_tabela_escrita[b], 0u, RESERVADO);
            }
            if r.exchanged {
                let s = atomicAdd(&cantos_contadores[0], 1u);
                if s < slots_do_atlas(textureDimensions(cantos_saida)) {
                    cantos_slot = s + 1u;
                } else {
                    atomicStore(&cantos_tabela_escrita[b], 0u);
                    atomicAdd(&cantos_contadores[2], 1u);
                }
            }
        }
    }
    let slot = workgroupUniformLoad(&cantos_slot);
    if slot == 0u {
        return;
    }
    textureStore(cantos_saida, no_atlas(slot - 1u, t), melhor);
    if all(lid.xy == vec2<u32>(0u)) {
        atomicStore(&cantos_tabela_escrita[bloco_de(b0, n.x)], slot);
    }
}

// ⭐ W6g r4/r9 — em despacho indireto sobre a lista do quadro ANTERIOR, como o [`c3_cantos`]: a entrada da
// tabela dos 3×3 blocos à volta de cada listado volta a 0 e a marca dele também, antes do [`c3_borda_adia`]
// deste quadro (o atlas não se limpa: só se lê pelo slot que este quadro escreveu).
@compute @workgroup_size(8, 8)
fn c3_cantos_limpa(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(workgroup_id) wid: vec3<u32>) {
    if any(lid.xy != vec2<u32>(0u)) {
        return;
    }
    let e = cantos_lista.blocos[wid.x];
    let k9 = i32(wid.y);
    let bc = vec2<i32>(i32(e & 0xffffu), i32(e >> 16u));
    let b0 = (bc + vec2<i32>(k9 % 3 - 1, k9 / 3 - 1)) * 8;
    let n = vec2<i32>(textureDimensions(borda_cobertura));
    if all(b0 >= vec2<i32>(0)) && all(b0 < n) {
        atomicStore(&cantos_tabela_escrita[bloco_de(b0, n.x)], 0u);
    }
    if k9 == 4 {
        atomicStore(&borda_marcas[bloco_de(bc * 8, n.x)], 0u);
    }
}
