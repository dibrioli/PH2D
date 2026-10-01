//! ⭐⭐⭐⭐ **O WGSL da tabela do mundo** — os kernels de [`crate::ceu_tempo`], num ficheiro à parte
//! porque o módulo passou do tecto de linhas quando a tabela passou a guardar também as LÂMPADAS.

/// O WGSL dos dois kernels — acrescenta-se ao molde da marcha, só para estas duas entradas.
pub(crate) const WGSL: &str = r"
struct Tabela {
    entradas: u32, fatias: u32, quadro: u32, velha: u32,
    unidade: f32, concordancia: f32, vagas: u32, palavras: u32,
    // Quantas fatias uma célula recebe NESTE quadro — ver `FATIAS_POR_QUADRO`.
    por_quadro: u32, _a: u32, _b: u32, _c: u32,
};
@group(1) @binding(0) var<uniform> tab: Tabela;
@group(1) @binding(1) var<storage, read_write> t: array<atomic<u32>>;
// A LISTA de trabalho do movimento: `[0]` o contador, `[1..4]` os argumentos do despacho indirecto,
// e depois pares `(pixel, célula << 8 | fatia)`.
@group(1) @binding(2) var<storage, read_write> w: array<atomic<u32>>;

// O ponto fixo das somas: `Σ c ≤ 48`, logo `48 · 65 536` cabe à vontade num `u32`.
const FIXO: f32 = 65536.0;

fn mistura(x: u32) -> u32 {
    var h = x * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

// ⭐ A CÉLULA do ponto: o nível sai do tamanho do pixel NESTE ponto (uma célula de `1`–`2` píxeis,
// logo o zoom escolhe outro nível e nunca lê células de outro tamanho), e a normal cai num de `16`
// baldes — as duas faces de uma parede fina nunca partilham uma célula.
fn nivel_da_celula(p: vec3<f32>, n: vec3<f32>) -> i32 {
    return nivel_de(pixel_no_mundo(p));
}

fn nivel_de(tamanho: f32) -> i32 {
    return i32(ceil(log2(max(tamanho / tab.unidade, 1.0))));
}

// ⭐⭐⭐⭐ O nível de uma célula do CHÃO sai da PEGADA do pixel no plano, não da secção do raio: ao
// longe o chão é visto de RASPÃO e um pixel cobre `1/cos` de chão na direcção da profundidade — com
// células do tamanho da secção cada pixel caía numa célula que nenhum outro tocara, e um giro pedia
// o chão inteiro de novo (medido no nó: a lista transbordava e ficavam píxeis sem sombra).
// ⚠️ O `cos` pára em `1/16` (quatro níveis). ⛔ Na PEÇA a mesma regra foi medida e RECUSADA: a
// oclusão dos tubos muda depressa na borda e a célula maior pintava-a grossa (`2 → 11 752` canais
// acima de `8` níveis no nó).
fn nivel_do_chao(p: vec3<f32>) -> i32 {
    return nivel_de(pegada_no_chao(p));
}

fn chave_da_celula(p: vec3<f32>, n: vec3<f32>) -> vec2<u32> {
    return chave_no_nivel(p, n, nivel_da_celula(p, n));
}

fn chave_no_nivel(p: vec3<f32>, n: vec3<f32>, nivel: i32) -> vec2<u32> {
    let lado = tab.unidade * exp2(f32(nivel));
    let g = vec3<i32>(floor(p / lado));
    var e = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
    if (n.z < 0.0) {
        e = (vec2<f32>(1.0) - abs(e.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), e >= vec2<f32>(0.0));
    }
    let balde = vec2<u32>(clamp((e + vec2<f32>(1.0)) * 2.0, vec2<f32>(0.0), vec2<f32>(3.0)));
    let resto = bitcast<u32>(nivel) * 16u + balde.x * 4u + balde.y;
    var h = mistura(bitcast<u32>(g.x));
    h = mistura(h ^ bitcast<u32>(g.y));
    h = mistura(h ^ bitcast<u32>(g.z));
    h = mistura(h ^ resto);
    // ⛔⛔⛔ **A IMPRESSÃO é um SEGUNDO hash, independente do que escolhe o lugar** (2026-09-30). Ela
    // era `mistura(h)`, uma função do MESMO `h` de `32` bits: duas células com o mesmo `h` caíam no
    // mesmo lugar com a mesma impressão e FUNDIAM-SE — e com milhões de células vivas o aniversário
    // dá milhares de pares (`N²/2³³`). No céu não se via (as oclusões vizinhas são parecidas); com a
    // SOMBRA das lâmpadas na tabela, uma célula da peça na sombra fundida com uma do chão ao sol
    // pintava um ponto claro (medido no nó: `77` níveis).
    var f = mistura(bitcast<u32>(g.z) ^ 0x68e31da4u);
    f = mistura(f ^ bitcast<u32>(g.x));
    f = mistura(f ^ resto);
    f = mistura(f ^ bitcast<u32>(g.y));
    return vec2<u32>(h, f | 1u);
}

// ⭐ A troca da IMPRESSÃO de uma vaga, FORTE: a `Weak` do WGSL pode falhar sem razão (o valor velho
// é o esperado e não houve troca), e uma falha espúria deixava a vaga VAZIA para trás enquanto a
// chave ia viver na seguinte — a procura pára na primeira vazia e nunca mais a achava.
struct Troca { exchanged: bool, old_value: u32 };
fn troca_forte(b: u32, velho: u32, novo: u32) -> Troca {
    for (var n: u32 = 0u; n < 4u; n = n + 1u) {
        let r = atomicCompareExchangeWeak(&t[b], velho, novo);
        if (r.exchanged || r.old_value != velho) { return Troca(r.exchanged, r.old_value); }
    }
    return Troca(false, velho);
}

// ⭐ A ENTRADA da célula: sondagem linear de `8`, reclamando uma vazia ou ROUBANDO uma velha.
// `0xffffffff` quando não há lugar — o pixel marcha então uma fatia só para si.
fn entrada_de(k: vec2<u32>) -> u32 {
    for (var i: u32 = 0u; i < 8u; i = i + 1u) {
        let e = (k.x + i) % tab.entradas;
        let b = e * tab.palavras;
        let r = troca_forte(b, 0u, k.y);
        if (r.exchanged || r.old_value == k.y) { return e; }
        let lida = atomicLoad(&t[b + 4u]);
        if (tab.quadro - lida > tab.velha) {
            let s = troca_forte(b, r.old_value, k.y);
            // ⛔⛔ **Quem perde o roubo para a MESMA chave fica com a mesma célula** (report do dono
            // de 2026-10-01, *«o ruído persiste se a view é rotacionada várias vezes»*): os píxeis de
            // uma célula roubam-na todos ao mesmo tempo, e o perdedor seguia para a vaga SEGUINTE e
            // reclamava uma SEGUNDA célula com a mesma chave. No quadro assente o `zera` dava-a por
            // cheia e o `grava` nunca a achava (a procura pára na primeira) ⇒ uma célula «cheia» de
            // soma zero, que muito depois — quando a primeira era roubada por outra chave — a
            // procura passava a ler como CÉU ABERTO: o ponto claro na face escura do tubo.
            if (!s.exchanged && s.old_value == k.y) { return e; }
            if (s.exchanged) {
                atomicStore(&t[b + 1u], 0u);
                atomicStore(&t[b + 2u], 0u);
                atomicStore(&t[b + 3u], 0u);
                for (var q: u32 = 5u; q < tab.palavras; q = q + 1u) { atomicStore(&t[b + q], 0u); }
                return e;
            }
        }
    }
    return 0xffffffffu;
}

// Só PROCURA a célula (sem a reclamar): a entrada, ou `0xffffffff` se ela não existe.
fn procura(k: vec2<u32>) -> u32 {
    for (var i: u32 = 0u; i < 8u; i = i + 1u) {
        let e = (k.x + i) % tab.entradas;
        let v = atomicLoad(&t[e * tab.palavras]);
        if (v == k.y) { return e; }
        if (v == 0u) { return 0xffffffffu; }
    }
    return 0xffffffffu;
}

// ⭐ A MESMA soma do `ceu_por_cones`, só com os cones `j ≡ fatia (mod fatias)`.
fn ceu_fatia(erguido: vec3<f32>, n: vec3<f32>, fatia: u32, fatias: u32) -> vec2<f32> {
    var soma = 0.0;
    var peso = 0.0;
    for (var j: u32 = fatia; j < s.ao_rays; j = j + fatias) {
        let dd = direccao_do_cone(j, s.ao_rays);
        let c = dot(n, dd);
        if (c <= 0.0) { continue; }
        peso = peso + c;
        let ate = min(s.ao_reach, cerca_da_bola(erguido, dd, s.ao_reach));
        soma = soma + c * visivel_ceu(erguido, dd, ate, 1.0 / c);
    }
    return vec2<f32>(soma, peso);
}

// O peso de TODOS os cones — é o denominador do `ceu_por_cones`, sem marchar nada.
fn peso_total(n: vec3<f32>) -> f32 {
    var peso = 0.0;
    for (var j: u32 = 0u; j < s.ao_rays; j = j + 1u) {
        peso = peso + max(dot(n, direccao_do_cone(j, s.ao_rays)), 0.0);
    }
    return peso;
}


// ⭐⭐⭐⭐ O que o pixel `i` MOSTRA — a peça (o `centro`) ou o CHÃO que só recebe (normal `+y`). Os
// passes da tabela leem-no; ⚠️ a aritmética é a do `escreve_a_luz`, logo o ponto é o mesmo ao bit.
struct Alvo { p: vec3<f32>, n: vec3<f32>, chao: bool, ok: bool };

fn alvo_do_pixel(i: u32) -> Alvo {
    var a: Alvo;
    let c = centro[i];
    let r = ray_at_plane(raio(f32(i % s.w) + 0.5, f32(i / s.w) + 0.5));
    if (c.x >= 0.0) {
        a.p = r.o + r.d * c.x;
        a.n = s.right * c.y + s.up * c.z + s.fwd * c.w;
        a.ok = true;
        return a;
    }
    let q = chao_em(r);
    a.p = q.xyz;
    a.n = vec3<f32>(0.0, 1.0, 0.0);
    a.chao = true;
    a.ok = q.w != 0.0;
    return a;
}

// O nível da célula de um alvo — a peça pela secção do raio, o chão pela pegada.
fn nivel_do_alvo(a: Alvo) -> i32 {
    if (a.chao) { return nivel_do_chao(a.p); }
    return nivel_da_celula(a.p, a.n);
}

// ⭐ A chave da célula do alvo num nível. ⚠️ O CHÃO tem células À PARTE: uma peça pousada nele tem
// pontos com a mesma posição e a mesma normal, e as duas perguntas não se misturam.
fn chave_do_alvo(a: Alvo, nivel: i32) -> vec2<u32> {
    let k = chave_no_nivel(a.p, a.n, nivel);
    if (!a.chao) { return k; }
    return vec2<u32>(mistura(k.x ^ 0x2545f491u), mistura(k.y ^ 0x2545f491u) | 1u);
}

// ⭐⭐⭐⭐ **AS LÂMPADAS DO CHÃO NA TABELA** — a sombra de cada lâmpada num ponto não depende da câmara,
// e um giro não a tem de marchar outra vez. ⛔⛔ **Só no CHÃO, e a PEÇA foi medida e RECUSADA**
// (2026-09-30, nó a aproximar): uma célula tem `1`–`2` píxeis e guarda UM valor, logo a PENUMBRA
// suave de um tubo sobre outro saía em DEGRAUS do tamanho da célula (`2 273` canais acima de `8`
// níveis, máx. `123`, contra `175`/`39` sem ela) — a oclusão do céu aguenta a célula porque é
// suave; a sombra de uma lâmpada não. No chão a sombra é larga e a célula é a PEGADA do pixel, e
// era lá que estava o custo (`~80 %` da sombra do passe da luz). Palavra `5`: `LUZ_RECLAMADA` mais quantas AMOSTRAS a célula
// somou; depois duas lâmpadas por palavra, `16` bits cada, em ponto fixo. `tab.palavras == 5` é a
// tabela sem lâmpadas (ver `crate::ceu_tempo::palavras_para`).
const LUZ_RECLAMADA: u32 = 0x80000000u;
// ⚠️ `32` amostras de `1024` cabem nos `16` bits (`32 768`), e é o tecto da soma: uma célula de
// raspão com mais píxeis ignora os de sobra — a média dos primeiros `32` já é a média.
const LUZ_ESCALA: f32 = 1024.0;
const LUZ_AMOSTRAS: u32 = 32u;
// O item da lista de trabalho que marcha as lâmpadas de uma célula (as fatias do céu são `0..8`).
const LAMPADAS: u32 = 0xfeu;
// O item de um pixel SEM LUGAR na tabela: o céu inteiro só para ele (ver `ceu_tempo_pede`).
const TODAS: u32 = 0xfdu;

// ⭐⭐⭐⭐ **Quantas fatias uma célula recebe POR QUADRO de movimento** — metade das direcções (as
// primeiras na ordem invertida em bits, logo espalhadas pela esfera), e a outra metade no quadro
// seguinte. ⚠️ É o PICO que isto corta, não a média: o 1.º quadro de um gesto encontra de uma vez as
// células que o quadro assente não tocou, e era ele que fazia o quadro SEGUINTE baixar de resolução.
// ⛔⛔ **O TECTO (a célula ficar com metade até o quadro assente) foi medido e RECUSADO**: `~2 ms`
// mais barato num giro e um VIÉS largo nos sulcos da rosca (`9 024` canais acima de `8` níveis
// contra a barra de `260`) — e esse viés saltava ao parar.
const FATIAS_POR_QUADRO: u32 = {FATIAS_POR_QUADRO}u;
// ⭐⭐⭐⭐ **E só no(s) primeiro(s) quadro(s) depois do ASSENTE** (`tab.por_quadro`, escolhido na CPU por
// `crate::ceu_tempo::QUADROS_COM_TECTO`) — é ali que o pico mora; no resto do gesto uma célula nova
// leva as `8` de uma vez. Medido (rosca a girar, canais acima de `8` no último quadro): `151` com as
// `8` sempre, `516` com a metade sempre — a faixa nova que o giro revela em cada quadro ficava com
// meio céu.

// ⭐⭐⭐⭐ **Uma célula COPIADA de um nível ao lado não serve de FONTE a outra cópia** — num zoom
// contínuo cada nível novo copiava o anterior, que já era cópia, e a resolução do primeiro nível
// ficava pintada na peça para sempre (escamas num tubo, medidas no nó a aproximar). A marca vive na
// palavra das fatias (céu) e na das amostras (lâmpadas); a herança das VIZINHAS propaga-a.
const COPIA_CEU: u32 = 0x100u;
const COPIA_LUZ: u32 = 0x40000000u;

fn fatias_de(t3: u32) -> u32 {
    return t3 & 0xffu;
}

// O que uma célula tem de ter para servir de FONTE a uma herança.
fn cheia() -> u32 {
    return min(FATIAS_POR_QUADRO, tab.fatias);
}

// ⏱️ Os contadores do relógio das sondas, depois da lista: `0` céu, `1` lâmpadas da peça, `2` do
// chão, `3` transbordo.
fn conta_item(k: u32) {
    atomicAdd(&w[4u + tab.vagas * 2u + k], 1u);
}

fn com_lampadas() -> bool {
    return tab.palavras > 5u;
}

// Um pixel do chão usa a tabela? — com lâmpadas nela, e LONGE da peça (`chao_perto_da_peca`).
fn chao_na_tabela(a: Alvo) -> bool {
    return com_lampadas() && !chao_perto_da_peca(a.p);
}

// Reclama uma AMOSTRA de lâmpadas na célula `b` — `false` quando ela já somou as que cabem.
fn amostra_de_luz(b: u32) -> bool {
    return (atomicAdd(&t[b + 5u], 1u) & 0xffffu) < LUZ_AMOSTRAS;
}

fn junta_lampada(b: u32, l: u32, v: f32) {
    let q = u32(clamp(v, 0.0, 1.0) * LUZ_ESCALA + 0.5);
    atomicAdd(&t[b + 6u + l / 2u], q << (16u * (l % 2u)));
}

// ⭐⭐⭐⭐ O CÉU DO CHÃO numa célula do chão — as palavras do céu (`Σ ceu`, `Σ 1`, em ponto fixo), que
// uma célula do chão não usa para os cones: o `ceu_do_chao` também não depende da câmara.
fn junta_ceu_do_chao(b: u32, v: f32) {
    atomicAdd(&t[b + 1u], u32(clamp(v, 0.0, 1.0) * FIXO));
    atomicAdd(&t[b + 2u], u32(FIXO));
}

fn ceu_do_chao_da_celula(b: u32) -> f32 {
    let peso = f32(atomicLoad(&t[b + 2u]));
    if (peso <= 0.0) { return -1.0; }
    return f32(atomicLoad(&t[b + 1u])) / peso;
}

// Escreve no pixel `i` a média das lâmpadas da célula `b` — `false` se ela ainda não tem nenhuma.
fn le_lampadas(b: u32, i: u32) -> bool {
    let n = min(atomicLoad(&t[b + 5u]) & 0xffffu, LUZ_AMOSTRAS);
    if (n == 0u) { return false; }
    let base = i * passo_da_luz();
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let w = atomicLoad(&t[b + 6u + l / 2u]);
        luz[base + 1u + l] = f32((w >> (16u * (l % 2u))) & 0xffffu) / (LUZ_ESCALA * f32(n));
    }
    return true;
}

// ⭐⭐⭐ O QUADRO ASSENTE, passe 1 — ZERAR: cada célula que um pixel toca é reclamada, esvaziada e
// dada por CHEIA. ⚠️ É um passe à parte porque o seguinte SOMA: zerar e somar no mesmo passe
// deixaria a ordem dos pixels decidir o que se apaga.
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_zera(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let a = alvo_do_pixel(g.y * s.w + g.x);
    if (!a.ok || (a.chao && !chao_na_tabela(a))) { return; }
    let e = entrada_de(chave_do_alvo(a, nivel_do_alvo(a)));
    if (e == 0xffffffffu) { return; }
    let b = e * tab.palavras;
    atomicStore(&t[b + 1u], 0u);
    atomicStore(&t[b + 2u], 0u);
    // ⚠️ Uma célula do CHÃO nasce com o céu CHEIO: o céu do chão é o `ceu_do_chao`, na luz.
    atomicStore(&t[b + 3u], tab.fatias);
    atomicMax(&t[b + 4u], tab.quadro);
    for (var q: u32 = 5u; q < tab.palavras; q = q + 1u) { atomicStore(&t[b + q], 0u); }
}

// ⭐⭐⭐ O QUADRO ASSENTE: a oclusão já foi marchada inteira pela luz; aqui só se guarda na célula.
// ⛔ Escrever também nos quatro CANTOS da pegada do pixel (levados ao plano tangente), para cobrir as
// células que nenhum centro de pixel atravessa, foi medido e RECUSADO: o nó a `3°` não ficou mais
// barato e as curvas passaram de `358` para `1 023` canais acima de `8` níveis — o plano tangente
// erra numa superfície curva. Quem cobre essas células é a herança das vizinhas (`ceu_tempo_pede`).
//
// ⭐⭐⭐ Passe 2 — SOMAR: a célula guarda a MÉDIA dos píxeis que caem nela, pesada como os cones a
// pesam. ⛔ Guardar o céu do ÚLTIMO pixel a escrever (a 1.ª redacção) punha cada pixel parado a ler o
// céu de um vizinho sorteado pela ordem dos grupos — medido no nó: `203`–`253` canais acima de `8`
// níveis contra a exacta, pior que o quadro a passo `2` que a cache substitui (`31`).
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_grava(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let a = alvo_do_pixel(i);
    if (!a.ok || (a.chao && !chao_na_tabela(a))) { return; }
    let e = procura(chave_do_alvo(a, nivel_do_alvo(a)));
    if (e == 0xffffffffu) { return; }
    let b = e * tab.palavras;
    // ⭐ As lâmpadas deste pixel já foram marchadas pela luz do quadro assente — somam-se aqui.
    if (com_lampadas() && a.chao && amostra_de_luz(b)) {
        for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
            junta_lampada(b, l, luz[i * passo_da_luz() + 1u + l]);
        }
        if (a.chao) { junta_ceu_do_chao(b, luz[i * passo_da_luz()]); }
    }
    if (a.chao) { return; }
    let peso = peso_total(a.n);
    atomicAdd(&t[b + 1u], u32(luz[i * passo_da_luz()] * peso * FIXO));
    atomicAdd(&t[b + 2u], u32(peso * FIXO));
}

// O ponto e a normal do pixel `i`, reconstruídos do `centro` — os três passes do movimento leem-no.
fn ponto_do_pixel(i: u32) -> array<vec3<f32>, 2> {
    let c = centro[i];
    let r = ray_at_plane(raio(f32(i % s.w) + 0.5, f32(i / s.w) + 0.5));
    return array<vec3<f32>, 2>(r.o + r.d * c.x, s.right * c.y + s.up * c.z + s.fwd * c.w);
}

// ⭐⭐⭐⭐ O QUADRO DE MOVIMENTO, passe 1 — PEDIR: cada pixel acha a célula do ponto e, se lhe faltam
// fatias, reclama-as e ACRESCENTA o trabalho a uma lista. ⚠️ Marchar aqui dentro seria pagar a
// divergência: uma célula a encher no meio de um grupo de `64` píxeis prende o grupo inteiro, e as
// células novas vêm espalhadas pela imagem — medido, o quadro ficava mais caro do que o de antes.
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_pede(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let a = alvo_do_pixel(i);
    if (!a.ok || (a.chao && !chao_na_tabela(a))) { return; }
    let nivel = nivel_do_alvo(a);
    let k = chave_do_alvo(a, nivel);
    // ⭐ PROCURAR antes de RECLAMAR: num giro quase todo pixel cai numa célula que já existe, e a
    // procura só LÊ — a reclamação é um `compareExchange` por vaga sondada.
    var e = procura(k);
    if (e == 0xffffffffu) { e = entrada_de(k); }
    if (e != 0xffffffffu) {
        // ⭐ E a marca de leitura só se ESCREVE quando muda — os `1`–`4` píxeis de uma célula
        // escreviam todos o mesmo número, em atómicos, a cada quadro.
        let lida = e * tab.palavras + 4u;
        if (atomicLoad(&t[lida]) != tab.quadro) { atomicMax(&t[lida], tab.quadro); }
    }
    if (com_lampadas() && a.chao) { pede_lampadas(e, i, a, nivel); }
    // O céu do CHÃO vive nas palavras do céu da célula dele (ver `junta_ceu_do_chao`).
    if (a.chao) { return; }
    let pn = array<vec3<f32>, 2>(a.p, a.n);
    var cel = e;
    if (e != 0xffffffffu) {
        let tem = fatias_de(atomicLoad(&t[e * tab.palavras + 3u]));
        if (tem >= tab.fatias) { return; }
        if (tem == 0u) {
            let h = herda_das_vizinhas(e, pn[0], pn[1]);
            conta_item(4u + h);
            if (h == HERDOU) { return; }
            if (h == SEM_VIZINHAS && herda_do_nivel(e, pn[0], pn[1])) { conta_item(7u); return; }
        }
    }
    if (e == 0xffffffffu) {
        // A tabela estava cheia à volta da chave: o céu INTEIRO só para este pixel, e nada se guarda.
        // ⛔⛔ Era UMA fatia (`6` dos `48` cones) — um ponto claro do tamanho da célula na face escura
        // de um tubo (report do dono de 2026-10-01). Com o despejo de [`VELHA`] estes píxeis são
        // raros, e um pixel raro paga as `8` fatias em vez de pintar ruído.
        conta_item(8u);
        let vaga = atomicAdd(&w[0], 1u);
        if (vaga >= tab.vagas) { conta_item(3u); return; }
        conta_item(0u);
        atomicStore(&w[4u + vaga * 2u], i);
        atomicStore(&w[5u + vaga * 2u], (cel << 8u) | TODAS);
        return;
    }
    // ⭐⭐⭐⭐ Até `FATIAS_POR_QUADRO` fatias por célula e por quadro — a reclamação é um
    // `compareExchange`, para uma fatia reclamada ser sempre uma fatia marchada.
    let b3 = e * tab.palavras + 3u;
    var w3 = atomicLoad(&t[b3]);
    let limite = min(fatias_de(w3) + tab.por_quadro, tab.fatias);
    for (var q: u32 = 0u; q < tab.fatias; q = q + 1u) {
        if (fatias_de(w3) >= limite) { return; }
        // A vaga na lista vem ANTES da fatia: uma fatia reclamada sem vaga ficaria contada e nunca somada.
        let vaga = atomicAdd(&w[0], 1u);
        if (vaga >= tab.vagas) { conta_item(3u); return; }
        var k = 0xffu;
        let r = atomicCompareExchangeWeak(&t[b3], w3, w3 + 1u);
        if (r.exchanged) {
            k = fatias_de(w3);
            w3 = w3 + 1u;
            conta_item(0u);
        } else {
            // Outro pixel da célula reclamou primeiro: a vaga fica vazia e lê-se outra vez.
            w3 = r.old_value;
        }
        atomicStore(&w[4u + vaga * 2u], i);
        atomicStore(&w[5u + vaga * 2u], (cel << 8u) | k);
    }
}

// ⭐⭐⭐⭐ As LÂMPADAS de uma célula que não as tem: herdadas do mesmo ponto um nível ao lado, ou
// marchadas por UM item da lista. ⚠️ A vaga vem antes da reclamação, pela razão das fatias: uma célula
// reclamada sem vaga ficava sem sombra até o quadro assente.
fn pede_lampadas(e: u32, i: u32, a: Alvo, nivel: i32) {
    if (e != 0xffffffffu) {
        let b = e * tab.palavras;
        if (atomicLoad(&t[b + 5u]) != 0u) { return; }
        if (lampadas_das_vizinhas(b, a, nivel) || lampadas_do_nivel(b, a, nivel)) { return; }
    }
    let vaga = atomicAdd(&w[0], 1u);
    if (vaga >= tab.vagas) { conta_item(3u); return; }
    conta_item(select(1u, 2u, a.chao));
    var k = LAMPADAS;
    if (e != 0xffffffffu) {
        // Outro pixel da mesma célula ganhou a reclamação: a vaga fica vazia.
        if ((atomicOr(&t[e * tab.palavras + 5u], LUZ_RECLAMADA) & LUZ_RECLAMADA) != 0u) { k = 0xffu; }
    }
    atomicStore(&w[4u + vaga * 2u], i);
    atomicStore(&w[5u + vaga * 2u], (e << 8u) | k);
}

// ⭐⭐⭐⭐ As lâmpadas herdadas das seis vizinhas do mesmo nível — a cerca do céu, lâmpada a lâmpada:
// pelo menos DUAS vizinhas, e todas a CONCORDAR em todas as lâmpadas. ⚠️ Uma célula vazia a meio de
// uma face já vista (o «sal» do `herda_das_vizinhas`) é a maioria das que um giro encontra, e a
// sombra de uma lâmpada só muda depressa na BORDA da sombra — é ali que a cerca recusa e a célula
// marcha.
fn lampadas_das_vizinhas(b: u32, a: Alvo, nivel: i32) -> bool {
    let lado = tab.unidade * exp2(f32(nivel));
    var soma = vec4<f32>(0.0);
    var menor = vec4<f32>(1.0);
    var maior = vec4<f32>(0.0);
    var quantas = 0.0;
    var copia = 0u;
    var soma_ceu = 0.0;
    var menor_ceu = 1.0;
    var maior_ceu = 0.0;
    for (var k: u32 = 0u; k < 6u; k = k + 1u) {
        var d = vec3<f32>(0.0);
        d[k / 2u] = select(-lado, lado, (k & 1u) != 0u);
        let v = procura(chave_do_alvo(Alvo(a.p + d, a.n, a.chao, true), nivel));
        if (v == 0xffffffffu) { continue; }
        let vb = v * tab.palavras;
        let w5 = atomicLoad(&t[vb + 5u]);
        let n = min(w5 & 0xffffu, LUZ_AMOSTRAS);
        if (n == 0u) { continue; }
        copia = copia | (w5 & COPIA_LUZ);
        if (a.chao) {
            let c = ceu_do_chao_da_celula(vb);
            if (c < 0.0) { continue; }
            soma_ceu = soma_ceu + c;
            menor_ceu = min(menor_ceu, c);
            maior_ceu = max(maior_ceu, c);
        }
        var x = vec4<f32>(0.0);
        for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
            let w = atomicLoad(&t[vb + 6u + l / 2u]);
            x[l] = f32((w >> (16u * (l % 2u))) & 0xffffu) / (LUZ_ESCALA * f32(n));
        }
        soma = soma + x;
        menor = min(menor, x);
        maior = max(maior, x);
        quantas = quantas + 1.0;
    }
    if (quantas < 2.0 || any(maior - menor > vec4<f32>(tab.concordancia))) { return false; }
    if (a.chao && maior_ceu - menor_ceu > tab.concordancia) { return false; }
    let r = atomicCompareExchangeWeak(&t[b + 5u], 0u, LUZ_RECLAMADA);
    if (r.exchanged) {
        for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
            junta_lampada(b, l, soma[l] / quantas);
        }
        if (a.chao) { junta_ceu_do_chao(b, soma_ceu / quantas); }
        atomicStore(&t[b + 5u], LUZ_RECLAMADA | copia | 1u);
    }
    return true;
}

// ⭐ As lâmpadas do MESMO ponto um nível ao lado — a mais fina primeiro. A sombra de uma lâmpada é a
// mesma nos dois tamanhos de célula; o erro é o de ler uma célula `2×` maior ou menor.
fn lampadas_do_nivel(b: u32, a: Alvo, nivel: i32) -> bool {
    for (var d: i32 = -1; d <= 1; d = d + 2) {
        let v = procura(chave_do_alvo(a, nivel + d));
        if (v == 0xffffffffu) { continue; }
        let vb = v * tab.palavras;
        let w5 = atomicLoad(&t[vb + 5u]);
        let n = min(w5 & 0xffffu, LUZ_AMOSTRAS);
        if (n == 0u || (w5 & COPIA_LUZ) != 0u) { continue; }
        let r = atomicCompareExchangeWeak(&t[b + 5u], 0u, LUZ_RECLAMADA);
        if (r.exchanged) {
            for (var q: u32 = 6u; q < tab.palavras; q = q + 1u) {
                atomicStore(&t[b + q], atomicLoad(&t[vb + q]));
            }
            if (a.chao) {
                atomicStore(&t[b + 1u], atomicLoad(&t[vb + 1u]));
                atomicStore(&t[b + 2u], atomicLoad(&t[vb + 2u]));
            }
            atomicStore(&t[b + 5u], LUZ_RECLAMADA | COPIA_LUZ | n);
        }
        return true;
    }
    return false;
}

// ⭐⭐⭐⭐ Uma célula VAZIA herda a média das seis vizinhas cheias do mesmo nível e da mesma normal.
// ⚠️ A célula vazia a meio de uma superfície já vista não é superfície NOVA: é uma célula que nenhum
// centro de pixel do quadro assente atravessou (uma célula tem `1`–`2` píxeis e a superfície corta-a
// de raspão). Medido: um único quadro de `3°` encontrava `~10 %` dos píxeis da peça nestas células,
// espalhados como sal pela peça inteira, e cada um marchava os `48` cones. O erro de herdar é a
// distância de UMA célula.
// ⛔ **Medir o céu do território NOVO na célula do nível de CIMA (2× maior) foi medido e RECUSADO**
// (2026-09-30): o nó a girar `16,1 → 15,3 ms` e o erro a explodir (`36 → 588` canais acima de `8`
// níveis, pior `18 → 129`) — a célula maior atravessa as fendas que a oclusão existe para mostrar.
// ⛔ **As 26 vizinhas (o cubo `3×3×3`) foram medidas e RECUSADAS** (2026-09-30, nó num giro): o 1.º
// quadro marchava menos céu (`54 846 → 39 559` fatias) e a imagem espalhava o erro (`252 → 1 573`
// canais acima de `8` níveis contra a exacta) — com mais vizinhas a cerca de concordância vê uma
// vizinhança maior do que a célula, e a herança atravessa a curvatura.
const HERDOU: u32 = 0u;
// Menos de duas vizinhas cheias no nível: a célula está numa zona que o nível ainda não viu.
const SEM_VIZINHAS: u32 = 1u;
// As vizinhas discordam: o céu muda depressa aqui (uma sombra de contacto) e a célula MARCHA.
const DISCORDAM: u32 = 2u;

fn herda_das_vizinhas(e: u32, p: vec3<f32>, n: vec3<f32>) -> u32 {
    let nivel = nivel_da_celula(p, n);
    let lado = tab.unidade * exp2(f32(nivel));
    var soma = 0.0;
    var quantas = 0.0;
    var copia = 0u;
    var menor = 1.0;
    var maior = 0.0;
    for (var a: u32 = 0u; a < 6u; a = a + 1u) {
        var d = vec3<f32>(0.0);
        d[a / 2u] = select(-lado, lado, (a & 1u) != 0u);
        let v = procura(chave_no_nivel(p + d, n, nivel));
        if (v == 0xffffffffu) { continue; }
        let t3 = atomicLoad(&t[v * tab.palavras + 3u]);
        if (fatias_de(t3) < cheia()) { continue; }
        copia = copia | (t3 & COPIA_CEU);
        let pv = f32(atomicLoad(&t[v * tab.palavras + 2u]));
        if (pv <= 0.0) { continue; }
        let ceu = f32(atomicLoad(&t[v * tab.palavras + 1u])) / pv;
        soma = soma + ceu;
        quantas = quantas + 1.0;
        menor = min(menor, ceu);
        maior = max(maior, ceu);
    }
    // Só onde as vizinhas CONCORDAM: numa sombra de contacto o céu muda depressa de célula para
    // célula, e ali herdar é errar — a célula marcha os cones como qualquer outra.
    // ⚠️ E pelo menos DUAS: com uma vizinha só não há concordância a medir, e era dali que vinham os
    // pontos claros isolados no fundo das fendas.
    if (quantas < 2.0) { return SEM_VIZINHAS; }
    if (maior - menor > tab.concordancia) { return DISCORDAM; }
    let r = atomicCompareExchangeWeak(&t[e * tab.palavras + 3u], 0u, tab.fatias | copia);
    if (r.exchanged) {
        let peso = peso_total(n);
        atomicStore(&t[e * tab.palavras + 1u], u32(soma / quantas * peso * FIXO));
        atomicStore(&t[e * tab.palavras + 2u], u32(peso * FIXO));
    }
    return HERDOU;
}

// ⭐⭐⭐⭐ Uma célula VAZIA herda a do MESMO ponto um nível ao lado — a mais fina primeiro.
// ⚠️ O nível sai do tamanho do pixel no ponto, logo um ZOOM (e a profundidade a mudar num giro) passa
// píxeis inteiros para o nível vizinho a cada quadro: sem esta porta cada um reclamava as `48`
// direcções de uma célula nova para medir o céu que a célula do lado já tem. O erro é o de ler uma
// célula `2×` maior ou menor — e o quadro ASSENTE regrava a tabela no nível certo.
fn herda_do_nivel(e: u32, p: vec3<f32>, n: vec3<f32>) -> bool {
    let nivel = nivel_da_celula(p, n);
    for (var a: i32 = -1; a <= 1; a = a + 2) {
        let v = procura(chave_no_nivel(p, n, nivel + a));
        if (v == 0xffffffffu) { continue; }
        let t3 = atomicLoad(&t[v * tab.palavras + 3u]);
        if ((t3 & COPIA_CEU) != 0u || fatias_de(t3) < cheia()) { continue; }
        let pv = f32(atomicLoad(&t[v * tab.palavras + 2u]));
        if (pv <= 0.0) { continue; }
        let ceu = f32(atomicLoad(&t[v * tab.palavras + 1u])) / pv;
        let r = atomicCompareExchangeWeak(&t[e * tab.palavras + 3u], 0u, tab.fatias | COPIA_CEU);
        if (r.exchanged) {
            let peso = peso_total(n);
            atomicStore(&t[e * tab.palavras + 1u], u32(ceu * peso * FIXO));
            atomicStore(&t[e * tab.palavras + 2u], u32(peso * FIXO));
        }
        return true;
    }
    return false;
}

// Os argumentos do despacho indirecto do passe 2: quantos grupos de `64` a lista pede.
@compute @workgroup_size(1, 1, 1)
fn ceu_tempo_args() {
    let n = min(atomicLoad(&w[0]), tab.vagas);
    atomicStore(&w[1], (n + 63u) / 64u);
    atomicStore(&w[2], 1u);
    atomicStore(&w[3], 1u);
}

// ⭐⭐⭐⭐ Passe 2 — MARCHAR: uma fatia por item da lista, lado a lado, sem divergência de célula.
@compute @workgroup_size(64, 1, 1)
fn ceu_tempo_marcha(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= min(atomicLoad(&w[0]), tab.vagas)) { return; }
    let i = atomicLoad(&w[4u + g.x * 2u]);
    let ck = atomicLoad(&w[5u + g.x * 2u]);
    let k = ck & 0xffu;
    if (k == 0xffu) { return; }
    let cel = ck >> 8u;
    if (k == LAMPADAS) {
        marcha_lampadas(i, cel);
        return;
    }
    let pn = ponto_do_pixel(i);
    if (k == TODAS) {
        // Sem lugar na tabela: os `48` cones, só para este pixel — a mesma lei do quadro exacto.
        luz[i * passo_da_luz()] = ceu_por_cones(pn[0] + pn[1] * (s.hit_eps * 4.0), pn[1]);
        return;
    }
    // ⭐ A fatia reclamada em `k`-ésimo lugar é a de índice INVERTIDO em bits: as primeiras que uma
    // célula recebe ficam espalhadas pela esfera, e uma célula ainda a encher lê um conjunto de cones
    // uniforme em vez de metade do céu.
    let ordem = array<u32, 8>(0u, 4u, 2u, 6u, 1u, 5u, 3u, 7u);
    var fatia = k;
    if (tab.fatias == 8u) { fatia = ordem[k]; }
    let sp = ceu_fatia(pn[0] + pn[1] * (s.hit_eps * 4.0), pn[1], fatia, tab.fatias);
    if (cel == 0xffffffu) { return; }
    atomicAdd(&t[cel * tab.palavras + 1u], u32(sp.x * FIXO));
    atomicAdd(&t[cel * tab.palavras + 2u], u32(sp.y * FIXO));
}

// As lâmpadas de uma célula, marchadas a partir do ponto deste pixel — ou só para ele, sem célula.
fn marcha_lampadas(i: u32, cel: u32) {
    let a = alvo_do_pixel(i);
    if (cel == 0xffffffu) {
        let base = i * passo_da_luz();
        for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
            luz[base + 1u + l] = sombra_da_lampada(a.p, a.n, a.chao, l);
        }
        if (a.chao) { luz[base] = ceu_do_chao(a.p); }
        return;
    }
    let b = cel * tab.palavras;
    if (!amostra_de_luz(b)) { return; }
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        junta_lampada(b, l, sombra_da_lampada(a.p, a.n, a.chao, l));
    }
    if (a.chao) { junta_ceu_do_chao(b, ceu_do_chao(a.p)); }
}

// ⭐⭐⭐⭐ Passe 3 — LER: cada pixel lê a célula, já com as fatias deste quadro somadas.
@compute @workgroup_size(8, 8, 1)
fn ceu_tempo_le(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let a = alvo_do_pixel(i);
    if (!a.ok || (a.chao && !chao_na_tabela(a))) { return; }
    let e = procura(chave_do_alvo(a, nivel_do_alvo(a)));
    // Sem entrada, quem escreveu este pixel foi o passe 2.
    if (e == 0xffffffffu) { conta_item(9u); return; }
    let b = e * tab.palavras;
    if (com_lampadas() && a.chao) { le_lampadas(b, i); }
    if (a.chao) {
        let c = ceu_do_chao_da_celula(b);
        if (c >= 0.0) { luz[i * passo_da_luz()] = c; }
        return;
    }
    // ⛔⛔ **Aqui vivia um recurso ao nível VIZINHO para a célula «a encher», e saiu por MEDIÇÃO**
    // (2026-09-30): o `pede` reclama todas as fatias em falta no MESMO quadro, antes deste passe, logo
    // a contagem já está cheia quando se lê — o ramo só corria quando a lista transbordava. A prova
    // de mutação deixou-o viver (`M5`), e sem ele os gestos de câmara medem o mesmo dentro da
    // variância das corridas. *Uma linha que a mutação não consegue matar não é lei.*
    let soma = f32(atomicLoad(&t[b + 1u]));
    let peso = f32(atomicLoad(&t[b + 2u]));
    // ⛔⛔ **Uma célula «cheia» de soma ZERO não é céu aberto** — era `1,0`, e cada uma pintava um
    // ponto claro (report do dono de 2026-10-01). As cheias-e-vazias nasciam quase todas de uma
    // corrida na reclamação (ver `entrada_de`, `991 → 5` por quadro); as que sobram pagam aqui os
    // cones inteiros, e a célula volta a VAZIA para o pedido do quadro seguinte a encher.
    if (peso <= 0.0) {
        conta_item(10u);
        let pn = ponto_do_pixel(i);
        luz[i * passo_da_luz()] = ceu_por_cones(pn[0] + pn[1] * (s.hit_eps * 4.0), pn[1]);
        atomicStore(&t[b + 3u], 0u);
        return;
    }
    luz[i * passo_da_luz()] = soma / peso;
}
";
