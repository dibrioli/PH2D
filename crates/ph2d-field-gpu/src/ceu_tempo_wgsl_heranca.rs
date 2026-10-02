//! ⭐⭐⭐⭐ **A HERANÇA da tabela do mundo** — as funções WGSL com que uma célula VAZIA herda o céu
//! e as lâmpadas das vizinhas (ou do mesmo ponto um nível ao lado), e a que pede as lâmpadas de uma
//! célula que não as tem. Corte por RESPONSABILIDADE do [`crate::ceu_tempo_wgsl`], que passou do
//! tecto de linhas (`708` de `700`): lá ficam a célula, a procura, os passes e a leitura; aqui o
//! que se faz quando a célula está vazia. ⚠️ O WGSL não pede declaração antes do uso, logo a ordem
//! das duas metades no texto é livre — quem as junta é o [`crate::ceu_tempo`].
pub(crate) const WGSL: &str = r"
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
";
