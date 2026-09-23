// ⭐⭐⭐⭐ **A TINTA FINA LIDA NA PLACA** — o gémeo em WGSL da lei que vive na
// `ph2d-mesh-colors`.
//
// ⛔⛔ **Ele NÃO é uma segunda redacção: ele é CONFERIDO contra a lei**, ponto
// a ponto, por um gate de paridade que corre os dois lados sobre a mesma malha
// e o mesmo plano (`tinta_paridade_tests`). O doc da `Tinta::cor_tri` já
// escrevia esta cláusula antes de este ficheiro existir: *«ela existe para que
// a lei da interpolação tenha UM dono; o gémeo em WGSL é conferido contra esta
// função, e não contra uma segunda redacção da mesma aritmética»*.
//
// ⚠️ **Este bloco é CONCATENADO ao `mesh.wgsl` e só quando a placa anuncia o
// `PRIMITIVE_INDEX`** — sem ele o `@builtin(primitive_index)` faz a VALIDAÇÃO
// do módulo inteiro falhar, e a peça deixaria de desenhar de todo. Ver
// `fonte::mesh_wgsl`.

struct TintaCfg {
    // ⛔⛔ **O `lado` GLOBAL SAIU daqui em 2026-09-23, e a ausência é a P2.**
    // Enquanto a retícula era uniforme ele descrevia a peça inteira; hoje o
    // lado é da FACE e mora no registo dela (palavra `10`). *Repô-lo aqui é
    // pintar a tinta de umas faces no sítio das outras.*
    //
    // Onde cada bloco começa. Eles são da MALHA e não da face, logo não cabem
    // no registo por face.
    verts: u32,
    // ⭐⭐ **Quantas amostras o bloco das ARESTAS tem ao todo** — e não quantas
    // arestas a malha tem. Com um nível POR FACE (a P2) elas deixaram de ter o
    // mesmo comprimento, logo `arestas × (lado − 1)` não descreve nada: o
    // início do bloco de cada aresta é um PREFIXO, e ele viaja no registo da
    // face (`11 + s`). *O que sobra de global é a SOMA, que é isto.*
    arestas_amostras: u32,
    // ⚠️ `0` quer dizer *nenhum plano ligado* — e aí o `fs_main_tinta` devolve
    // o `in.vcolor` de sempre. Sem esta palavra, uma peça sem plano leria um
    // buffer de um elemento e pintaria a peça inteira com ele.
    armado: u32,
    // ⚠️ **RESERVA declarada, e o `cfg_de` deixa-a a ZERO com gate.** Um
    // `uniform` alinha a `16` bytes, logo a 4.ª palavra existe quer alguém a
    // queira quer não — *uma posição sem dono e sem régua é onde o campo
    // seguinte aterra por engano*.
    reserva: u32,
};

// ⚠️⚠️ **GRUPO 1 e não um grupo próprio, e a razão é a FREQUÊNCIA** — a mesma
// que o `mesh.wgsl` já escreve para separar os quatro grupos que ele tem: o
// grupo 1 é o bind POR OBJECTO, e um plano de tinta é por objecto como a
// `obj.model` é. ⛔ E há um limite por baixo disto: o `max_bind_groups` de
// omissão é **4**, e os quatro já estavam tomados — um `@group(4)` não
// compilaria sem subir um limite que nenhuma outra peça desta casa precisa.
@group(1) @binding(1) var<storage, read> tinta_amostras: array<f32>;
@group(1) @binding(2) var<storage, read> tinta_topo: array<u32>;
@group(1) @binding(3) var<storage, read> tinta_origem: array<u32>;
@group(1) @binding(4) var<storage, read> tinta_idx: array<u32>;
@group(1) @binding(5) var<storage, read> tinta_pos: array<f32>;
@group(1) @binding(6) var<uniform> tinta_cfg: TintaCfg;

const TINTA_STRIDE: u32 = 19u;
// O sentinela do 4.º índice de um triângulo — o MESMO valor das duas crates,
// e isso é gateado (`o_sentinela_do_triangulo_e_o_mesmo_nas_duas_crates`).
const TINTA_TRI: u32 = 0xffffffffu;

fn tinta_amostra(i: u32) -> vec3<f32> {
    let b = 3u * i;
    return vec3<f32>(tinta_amostras[b], tinta_amostras[b + 1u], tinta_amostras[b + 2u]);
}

fn tinta_vert(v: u32) -> vec3<f32> {
    let b = 3u * v;
    return vec3<f32>(tinta_pos[b], tinta_pos[b + 1u], tinta_pos[b + 2u]);
}

// ⭐ **O endereço global de uma amostra** — o gémeo do `enderecos::indice`.
//
// `sitio` é `0` canto, `1` aresta, `2` interior; `a` é o canto/lado/índice e
// `t` o passo ao longo do lado.
fn tinta_indice(base: u32, sitio: u32, a: u32, t: u32) -> u32 {
    if (sitio == 0u) {
        return tinta_topo[base + a];
    }
    if (sitio == 1u) {
        let w = tinta_topo[base + 4u + a];
        // ⭐⭐⭐ **O PASSO DO SUBCONJUNTO, que é a P2 inteira num produto.** A
        // aresta leva o MÁXIMO dos dois vizinhos, logo a face GROSSA lê um
        // subconjunto EXACTO das amostras da fina: a amostra `t` dela mora na
        // `t × (le / lf)` da aresta, e a divisão é inteira porque os dois
        // lados são potências de dois. *Sem esta multiplicação a fronteira
        // parte-se: `t = 1` de um lado cai num oitavo do caminho do outro.*
        let lf = tinta_topo[base + 10u];
        let la = tinta_topo[base + 15u + a];
        var tt = t * (la / lf);
        // ⚠️ **A VIRADA**: a face que percorre a aresta do vértice MAIOR para o
        // menor conta `t` ao contrário. Sem esta linha a tinta fica ESPELHADA
        // ao longo de metade das arestas da peça, e nenhuma contagem o vê.
        // ⛔ E ela vem DEPOIS do passo e conta contra o lado da ARESTA.
        if ((w & 1u) == 1u) { tt = la - tt; }
        return tinta_cfg.verts + tinta_topo[base + 11u + a] + (tt - 1u);
    }
    return tinta_cfg.verts + tinta_cfg.arestas_amostras + tinta_topo[base + 8u] + a;
}

// O gémeo do `enderecos::sitio_tri`, devolvido como `(sitio, a, t)`.
fn tinta_sitio_tri(l: u32, i: u32, j: u32, k: u32) -> vec3<u32> {
    let i0 = i == 0u; let j0 = j == 0u; let k0 = k == 0u;
    if (!i0 && j0 && k0) { return vec3<u32>(0u, 0u, 0u); }
    if (i0 && !j0 && k0) { return vec3<u32>(0u, 1u, 0u); }
    if (i0 && j0 && !k0) { return vec3<u32>(0u, 2u, 0u); }
    // lado 0 = a→b, `t` conta de `a`: em `k = 0` andar é subir `j`.
    if (k0) { return vec3<u32>(1u, 0u, j); }
    // lado 1 = b→c, `t` conta de `b`: em `i = 0` andar é subir `k`.
    if (i0) { return vec3<u32>(1u, 1u, k); }
    // lado 2 = c→a, `t` conta de `c`: em `j = 0` andar é subir `i`.
    if (j0) { return vec3<u32>(1u, 2u, i); }
    let ll = l - 1u;
    let antes = (j - 1u) * ll - (j - 1u) * j / 2u;
    return vec3<u32>(2u, antes + (i - 1u), 0u);
}

// O gémeo do `enderecos::sitio_quad`.
fn tinta_sitio_quad(l: u32, i: u32, j: u32) -> vec3<u32> {
    let i0 = i == 0u; let im = i == l; let j0 = j == 0u; let jm = j == l;
    if (i0 && j0) { return vec3<u32>(0u, 0u, 0u); }
    if (im && j0) { return vec3<u32>(0u, 1u, 0u); }
    if (im && jm) { return vec3<u32>(0u, 2u, 0u); }
    if (i0 && jm) { return vec3<u32>(0u, 3u, 0u); }
    if (j0) { return vec3<u32>(1u, 0u, i); }
    if (im) { return vec3<u32>(1u, 1u, j); }
    // ⚠️ c→d ANDA PARA TRÁS no eixo `i`.
    if (jm) { return vec3<u32>(1u, 2u, l - i); }
    // ⚠️ d→a ANDA PARA TRÁS no eixo `j`.
    if (i0) { return vec3<u32>(1u, 3u, l - j); }
    return vec3<u32>(2u, (j - 1u) * (l - 1u) + (i - 1u), 0u);
}

// ⭐⭐ **A leitura de um TRIÂNGULO** — o gémeo do `amostragem::leitura_tri`,
// com os dois sub-triângulos (o direito e o INVERTIDO).
fn tinta_cor_tri(base: u32, bar: vec3<f32>) -> vec3<f32> {
    let l = tinta_topo[base + 10u];
    let lf = f32(l);
    let b0 = max(bar, vec3<f32>(0.0));
    let s = b0.x + b0.y + b0.z;
    var b = vec3<f32>(1.0, 0.0, 0.0);
    if (s > 0.0) { b = b0 / s; }
    let sc = b * lf;
    let p = clamp(floor(sc), vec3<f32>(0.0), vec3<f32>(lf));
    let f = sc - p;
    let i = u32(p.x); let j = u32(p.y); let k = u32(p.z);
    let soma = min(i + j + k, l);
    var ijk0: vec3<u32>; var ijk1: vec3<u32>; var ijk2: vec3<u32>;
    var w = vec3<f32>(0.0);
    if (l - soma == 0u) {
        ijk0 = vec3<u32>(i, j, k); ijk1 = ijk0; ijk2 = ijk0;
        w = vec3<f32>(1.0, 0.0, 0.0);
    } else if (l - soma == 1u) {
        ijk0 = vec3<u32>(i + 1u, j, k);
        ijk1 = vec3<u32>(i, j + 1u, k);
        ijk2 = vec3<u32>(i, j, k + 1u);
        w = f;
    } else {
        ijk0 = vec3<u32>(i, j + 1u, k + 1u);
        ijk1 = vec3<u32>(i + 1u, j, k + 1u);
        ijk2 = vec3<u32>(i + 1u, j + 1u, k);
        w = vec3<f32>(1.0) - f;
    }
    var out = vec3<f32>(0.0);
    let s0 = tinta_sitio_tri(l, ijk0.x, ijk0.y, ijk0.z);
    out += tinta_amostra(tinta_indice(base, s0.x, s0.y, s0.z)) * w.x;
    let s1 = tinta_sitio_tri(l, ijk1.x, ijk1.y, ijk1.z);
    out += tinta_amostra(tinta_indice(base, s1.x, s1.y, s1.z)) * w.y;
    let s2 = tinta_sitio_tri(l, ijk2.x, ijk2.y, ijk2.z);
    out += tinta_amostra(tinta_indice(base, s2.x, s2.y, s2.z)) * w.z;
    return out;
}

// ⭐⭐ **A leitura de um QUAD** — o gémeo do `amostragem::leitura_quad`. Aqui
// não há sub-triângulos invertidos: uma célula tem sempre quatro cantos, e a
// leitura é a BILINEAR deles.
fn tinta_cor_quad(base: u32, uv: vec2<f32>) -> vec3<f32> {
    let l = tinta_topo[base + 10u];
    let lf = f32(l);
    let c = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)) * lf;
    // ⚠️⚠️ **O corte do piso em `L−1` guarda o ENDEREÇO e não a COR, e isso
    // está MEDIDO:** em `u = 1` exacto o `floor` dá `L`, a célula seria
    // `[L, L+1]` — um endereço fora da face —, e o `fu` sai **exactamente
    // zero**, logo as duas amostras de fora entram com peso `0` e a cor é a
    // MESMA. ⇒ a mutação que apaga este corte **sobrevive ao gate de paridade
    // de COR, e sobreviverá sempre**: ela não é uma régua em falta.
    //
    // ⭐ O que ela muda é uma leitura fora do intervalo — inofensiva em WGSL
    // (a especificação torna-a segura) e um **pânico** do lado da `ph2d-mesh-colors`,
    // onde a mesma mutação SANGRA (`M12`). O corte fica porque os dois lados
    // têm de se ler como a mesma lei: *duas redacções que concordam na saída e
    // divergem no texto são o que faz a próxima emenda viajar só para um.*
    let i = min(u32(floor(c.x)), l - 1u);
    let j = min(u32(floor(c.y)), l - 1u);
    let fu = c.x - f32(i);
    let fv = c.y - f32(j);
    var out = vec3<f32>(0.0);
    let a = tinta_sitio_quad(l, i, j);
    out += tinta_amostra(tinta_indice(base, a.x, a.y, a.z)) * ((1.0 - fu) * (1.0 - fv));
    let b = tinta_sitio_quad(l, i + 1u, j);
    out += tinta_amostra(tinta_indice(base, b.x, b.y, b.z)) * (fu * (1.0 - fv));
    let d = tinta_sitio_quad(l, i + 1u, j + 1u);
    out += tinta_amostra(tinta_indice(base, d.x, d.y, d.z)) * (fu * fv);
    let e = tinta_sitio_quad(l, i, j + 1u);
    out += tinta_amostra(tinta_indice(base, e.x, e.y, e.z)) * ((1.0 - fu) * fv);
    return out;
}

// ⭐⭐⭐ **A COR DE UM FRAGMENTO** — de `(triângulo, ponto no objecto)` à tinta.
//
// ⚠️ **As baricêntricas saem da POSIÇÃO e não de um atributo**, e a razão é o
// desenho: a malha é desenhada INDEXADA, logo um vértice é partilhado por
// várias faces e não pode carregar um canto. A posição interpolada é, por
// construção, a combinação baricêntrica dos três vértices — resolvê-la de
// volta é exacto e não custa um buffer novo.
//
// ⚠️⚠️ **E a conversão para o `(u, v)` de um QUAD precisa de saber qual das
// metades da diagonal `a–c` este triângulo é** (o bit `sub` da origem): em
// `sub = 0` os cantos são `(a,b,c)` e em `sub = 1` são `(a,c,d)`. *Sem ele,
// metade de cada quad lê o `(u, v)` da outra metade.*
fn tinta_no_ponto(pi: u32, p: vec3<f32>) -> vec3<f32> {
    let o = tinta_origem[pi];
    let face = o >> 1u;
    let sub = o & 1u;
    let base = face * TINTA_STRIDE;

    let va = tinta_idx[3u * pi];
    let vb = tinta_idx[3u * pi + 1u];
    let vc = tinta_idx[3u * pi + 2u];
    let pa = tinta_vert(va);
    let pb = tinta_vert(vb);
    let pc = tinta_vert(vc);

    // Baricêntricas por áreas (produto vectorial), com a normal do triângulo
    // como eixo — a forma que não depende de qual plano o triângulo ocupa.
    let n = cross(pb - pa, pc - pa);
    let dd = dot(n, n);
    var bar = vec3<f32>(1.0, 0.0, 0.0);
    if (dd > 0.0) {
        let wa = dot(cross(pc - pb, p - pb), n) / dd;
        let wb = dot(cross(pa - pc, p - pc), n) / dd;
        bar = vec3<f32>(wa, wb, 1.0 - wa - wb);
    }

    if (tinta_topo[base + 9u] == 3u) {
        return tinta_cor_tri(base, bar);
    }
    // `sub = 0`: (a,b,c) ⇒ u = βb + βc, v = βc.
    // `sub = 1`: (a,c,d) ⇒ u = βc,      v = βc + βd.
    var uv = vec2<f32>(bar.y + bar.z, bar.z);
    if (sub == 1u) { uv = vec2<f32>(bar.y, bar.y + bar.z); }
    return tinta_cor_quad(base, uv);
}
