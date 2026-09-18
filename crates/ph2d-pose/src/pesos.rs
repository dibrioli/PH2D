//! §4 — suavizar os pesos.
//!
//! Cada segmento leva, **independentemente**, `N` iterações, e cada iteração
//! substitui o peso de cada vértice pela **média simples dos pesos dos
//! vizinhos**.
//!
//! Três cláusulas, e cada uma é um defeito se for esquecida:
//! - ⚠️ **o próprio vértice NÃO entra na média** (é média dos vizinhos, não do
//!   anel-1 fechado);
//! - ⚠️ **as ligações artificiais entre peças (§2.4) NÃO entram** aqui — elas
//!   servem a travessia e o crescimento, e mais nada;
//! - ⭐ **Jacobi limpo**: cada iteração lê **só** o estado da iteração anterior.

use crate::vetor::V3;
/// Quantas passagens de média alisam a pertença antes de dela se tirar a
/// fronteira. ⚠️ **Duas, e não é um número escolhido:** a correcção que elas
/// compram é sub-aresta por construção (o corte fica dentro da aresta que já o
/// atravessava), logo mais passagens não alargam nada — e o preço é `O(V)` por
/// passagem contra o `O(V log V)` da marcha.
const PASSAGENS_DA_INTERFACE: usize = 2;
use crate::vizinhanca::Vizinhanca;

/// Suaviza, em cima do lugar, os pesos de **um** segmento.
///
/// ⭐⭐ **É aqui que somos melhores que o alvo, e a afirmação vem com reserva.**
/// A saída desta fase no alvo **pode** não ser reprodutível quando a região
/// excede uma partição da estrutura espacial de aceleração — um **risco do
/// mecanismo**, nomeado na espec §4. ⚠️ **Nunca foi observado**: `4×3` corridas
/// do oráculo em duas sessões, até `66 049` vértices, voltaram idênticas ao bit.
/// ⇒ a frase honesta **não** é «o alvo é não-determinístico»; é *«se ele
/// divergir, nós não divergimos»* — o nosso Jacobi é determinístico em **toda**
/// a malha, incluindo o regime em que ele poderia não ser.
///
/// ⚠️ E a escolha não é uma cópia: acima desse limiar as duas saídas podem
/// divergir **sem que nenhuma esteja errada**, e um gate de paridade escrito
/// nesse regime mediria a partição do oráculo, não a nossa lei.
pub fn suavizar(viz: &Vizinhanca, pesos: &mut [f32], iteracoes: u32) {
    if iteracoes == 0 {
        return;
    }
    let n = viz.n_vertices();
    let mut anterior = vec![0.0f32; n];
    for _ in 0..iteracoes {
        anterior.copy_from_slice(pesos);
        for (v, peso) in pesos.iter_mut().enumerate().take(n) {
            let vizinhos = viz.de(v as u32);
            if vizinhos.is_empty() {
                // ⛔ **DIVERGÊNCIA DELIBERADA (§11.4, item 7 da lista de
                // verificação).** O alvo calcula aqui a média de um conjunto
                // vazio e deixa o peso **indefinido**; nós **mantemos o peso
                // que o vértice tinha**. Sem fixtura — o corpus não tem malha
                // com vértice solto —, e por isso ela está declarada em vez de
                // medida. *Uma divergência escrita é uma decisão; uma não
                // escrita é um bug à espera de ser lido como paridade.*
                continue;
            }
            let mut soma = 0.0;
            for &u in vizinhos {
                soma += anterior[u as usize];
            }
            *peso = soma / vizinhos.len() as f32;
        }
    }
}

/// ⭐⭐⭐ **A TRANSIÇÃO COMO DISTÂNCIA NO BARRO** — a lei alternativa à difusão.
///
/// Recebe o campo **cumulativo** de pertença (`0` ou `1` por vértice, o anel
/// que o §3.3 cresceu) e devolve-o esbatido por uma transição de largura
/// `banda`, medida em **unidades de objecto**.
///
/// # A lei
///
/// `d(v)` é a distância **sobre a superfície** do vértice `v` à fronteira do
/// conjunto — negativa dentro, positiva fora. O peso é `suave(0,5 − d/banda)`,
/// com `suave(x) = x²(3 − 2x)` cortado a `[0,1]`: a `d = −banda/2` vale `1`, a
/// `d = +banda/2` vale `0`, e a transição mede **exactamente `banda`**.
///
/// # ⛔⛔⛔ Porque é que ela NÃO é Dijkstra — e a medição que o decidiu
///
/// A 1.ª redacção desta função somava **comprimentos de aresta** (Dijkstra), e
/// o report do dono de 2026-09-17 (*«está quase bom! mas surgem estrias»*)
/// mediu-a: numa esfera de `97 922` vértices, a razão entre a distância que a
/// lei usava e a **exacta** (que numa esfera se calcula à mão — a geodésica até
/// à fronteira de uma calota é `θ_v − θ_r`) variava de **`0,89` a `1,37`**
/// conforme a DIRECÇÃO.
///
/// ⚠️ **O mecanismo é que um caminho por arestas só pode tomar as direcções que
/// a malha tem.** Numa malha estruturada — a esfera UV do módulo — isso torna
/// `d` quase constante em cada *anel* do grafo: as curvas de nível deixam de
/// ser círculos e passam a ser os **losangos** do grafo, o peso fica em
/// patamares, e cada degrau entre patamares é uma dobra da superfície. *As
/// estrias da foto são as fronteiras desses patamares.*
///
/// ⭐⭐ **A cura é a frente atravessar TRIÂNGULOS** (marcha rápida de
/// Kimmel–Sethian): um vértice é actualizado a partir de uma FACE com os dois
/// outros cantos já resolvidos, e o valor sai de uma quadrática que interpola
/// a frente **dentro** do triângulo em vez de a fazer dobrar num vértice. Com
/// ela a razão fecha em `1,00`–`1,03` e a quebra de normal na faixa (p90) cai
/// de `13,5°` para `1,0°` — ver [`atravessa`] e os gates da `ph2d-sculpt3d`.
///
/// ⚠️ **A aresta fica como TECTO, não como lei:** quando a direcção
/// característica cai fora do triângulo (ângulo obtuso no vértice que se
/// actualiza), a resposta certa é mesmo a aresta, e é o que a [`atravessa`]
/// devolve ao recusar. *Tomar sempre o mínimo mantém a monotonia de que a
/// marcha depende.*
///
/// # ⭐ O corte na meia-banda
///
/// Fora de `|d| ≤ banda/2` o peso está **cortado** em `0` ou `1`, logo a marcha
/// PÁRA aí — e um vértice nunca alcançado fica em `+∞`, que o passo (3) lê como
/// o mesmo `0`/`1` pelo lado em que está.
///
/// ⭐ **E ele é BYTE-NEUTRO, medido e não argumentado:** a impressão digital do
/// campo na peça do report lê `ee632d84f7ae65c9` com o corte e sem ele (sonda
/// `diag_a_impressao_do_campo` da `ph2d-sculpt3d`).
///
/// ⚠️ **Preço:** à largura de fábrica ele torna a lei **mais barata** que a
/// Dijkstra que substitui (`2,9`–`3,6 ms` contra `3,4`–`4,7` a `97 922`
/// vértices) apesar de a actualização ser mais cara, porque aquela marchava a
/// malha **inteira**; ⛔ e **mais cara no tecto** (`8,7 ms`), porque o preço
/// passou a seguir a ÁREA da faixa. *A lei antiga era plana na largura pedida;
/// esta não é, e a nota do [`crate::Controlos::banda_do_peso`] diz o mesmo.*
///
/// ⛔ **Uma semente nunca é actualizada** — ela é a condição de fronteira, e
/// deixá-la descer poria a lei a re-derivar o dado que lhe foi entregue.
///
/// ⛔ **Um vértice sem vizinhos MANTÉM o peso** — a mesma cláusula da
/// [`suavizar`], pela mesma razão.
pub fn por_distancia(viz: &Vizinhanca, posicoes: &[V3], pesos: &mut [f32], banda: f32) {
    let n = viz.n_vertices();
    if !(banda.is_finite() && banda > 0.0) || n == 0 {
        return;
    }
    // ⚠️ A pertença é **fotografada** antes de qualquer escrita: o passo (3)
    // reescreve `pesos`, e ler o lado de um vértice já reescrito daria a
    // fronteira num sítio que depende da ORDEM da varredura.
    let dentro: Vec<bool> = pesos.iter().map(|&w| w >= 0.5).collect();
    let comp = |a: usize, b: usize| -> f32 {
        let (p, q) = (posicoes[a], posicoes[b]);
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
    };

    // (0) ONDE A FRONTEIRA DE FACTO ESTÁ, **dentro** da aresta que a atravessa.
    //
    // ⛔⛔ A pertença é BINÁRIA, logo cravar o corte a meia aresta erra até
    // meia aresta **por vértice e ao acaso** — e a fronteira sai serrilhada.
    // Medido no report das estrias: com o corte a meio, a rugosidade do campo
    // nas 4 primeiras arestas lê `p90 0,266` degraus contra `0,03`–`0,07` no
    // resto. ⇒ duas passagens de média (com peso próprio, senão um campo
    // binário oscila em xadrez) dão um indicador contínuo cujo nível `0,5` é
    // uma curva lisa, e é o cruzamento DELE que semeia.
    //
    // ⚠️ **O LADO continua a vir do campo binário**, nunca do indicador: quem
    // pertence à região é o que o crescimento do §3.3 disse, e alisá-lo
    // apagaria uma região de um vértice só numa peça grossa.
    let mut suave_da_pertenca: Vec<f32> =
        dentro.iter().map(|&d| if d { 1.0 } else { 0.0 }).collect();
    let mut anterior = suave_da_pertenca.clone();
    for _ in 0..PASSAGENS_DA_INTERFACE {
        anterior.copy_from_slice(&suave_da_pertenca);
        for v in 0..n {
            let anel = viz.de(u32::try_from(v).unwrap_or(u32::MAX));
            if anel.is_empty() {
                continue;
            }
            let soma: f32 = anel.iter().map(|&u| anterior[u as usize]).sum();
            suave_da_pertenca[v] = 0.5 * anterior[v] + 0.5 * soma / anel.len() as f32;
        }
    }

    // (1) As SEMENTES: cada aresta que atravessa a fronteira põe cada extremo à
    // distância dele ao corte.
    let mut d = vec![f32::INFINITY; n];
    let mut semente = vec![false; n];
    let mut fila: std::collections::BinaryHeap<(std::cmp::Reverse<Ordenavel>, u32)> =
        std::collections::BinaryHeap::new();
    for v in 0..n {
        for &u in viz.de(u32::try_from(v).unwrap_or(u32::MAX)) {
            let u = u as usize;
            if dentro[v] == dentro[u] {
                continue;
            }
            let salto = suave_da_pertenca[u] - suave_da_pertenca[v];
            let fraccao = if salto.abs() > 1e-6 {
                ((0.5 - suave_da_pertenca[v]) / salto).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let ate_ao_corte = comp(v, u) * fraccao;
            if ate_ao_corte < d[v] {
                d[v] = ate_ao_corte;
                semente[v] = true;
            }
        }
        if semente[v] {
            fila.push((
                std::cmp::Reverse(Ordenavel(d[v])),
                u32::try_from(v).unwrap_or(u32::MAX),
            ));
        }
    }
    // ⚠️ Sem fronteira nenhuma o campo é constante e não há o que esbater — e
    // devolver aqui é o que mantém um segmento vazio ou cheio **ao bit**.
    if fila.is_empty() {
        return;
    }

    // (2) A MARCHA, e ela atravessa FACES.
    let meia_banda = banda * 0.5;
    let mut fixo = vec![false; n];
    while let Some((std::cmp::Reverse(Ordenavel(dv)), v)) = fila.pop() {
        let v = v as usize;
        if dv > d[v] {
            continue;
        }
        // ⭐ O corte: o resto da malha está cortado em `0`/`1` por construção.
        if dv > meia_banda {
            break;
        }
        fixo[v] = true;
        for &vizinho in viz.de(u32::try_from(v).unwrap_or(u32::MAX)) {
            let u = vizinho as usize;
            if fixo[u] || semente[u] {
                continue;
            }
            // (a) A aresta — sempre disponível, e é o tecto.
            let mut melhor = dv + comp(v, u);
            // (b) As FACES que têm `u` e `v` lado a lado: o outro braço do
            //     canto é o terceiro ponto, e a frente atravessa esse
            //     triângulo. ⛔ Procurar o terceiro ponto na ADJACÊNCIA não
            //     serve — ver [`Vizinhanca::cantos`].
            for &(a, b) in viz.cantos(vizinho) {
                let (a, b) = (a as usize, b as usize);
                let w = if a == v {
                    b
                } else if b == v {
                    a
                } else {
                    continue;
                };
                if !fixo[w] {
                    continue;
                }
                if let Some(t) = atravessa(posicoes[u], posicoes[v], posicoes[w], d[v], d[w]) {
                    melhor = melhor.min(t);
                }
            }
            if melhor < d[u] {
                d[u] = melhor;
                fila.push((std::cmp::Reverse(Ordenavel(melhor)), vizinho));
            }
        }
    }

    // (3) O PESO, com o sinal a vir do lado.
    for v in 0..n {
        if viz.de(u32::try_from(v).unwrap_or(u32::MAX)).is_empty() {
            continue;
        }
        let assinada = if dentro[v] { -d[v] } else { d[v] };
        let x = (0.5 - assinada / banda).clamp(0.0, 1.0);
        pesos[v] = x * x * (3.0 - 2.0 * x);
    }
}

/// A frente a atravessar **um triângulo** — a actualização que torna o círculo
/// redondo (Kimmel–Sethian).
///
/// `c` é o vértice a resolver; `p` e `q` são os outros dois cantos da face, com
/// os tempos `tp` e `tq` já fixos. Devolve `None` quando a direcção
/// característica cai **fora** do triângulo — ali a resposta certa é a aresta,
/// e quem chama já a tem.
///
/// ⚠️ **As duas cercas escrevem-se MULTIPLICADAS e nunca divididas** pelo
/// cosseno: a forma clássica `a·cosθ < h < a/cosθ` inverte de sentido com um
/// ângulo obtuso em `c`, e `h·cosθ < a` é a mesma condição sem esse ramo.
pub(crate) fn atravessa(c: V3, p: V3, q: V3, tp: f32, tq: f32) -> Option<f32> {
    // `a` é o canto de tempo MENOR — a quadrática é escrita a partir dele.
    let (pa, pb, ta, tb) = if tp <= tq {
        (p, q, tp, tq)
    } else {
        (q, p, tq, tp)
    };
    if !(ta.is_finite() && tb.is_finite()) {
        return None;
    }
    let u = tb - ta;
    let ea = [pa[0] - c[0], pa[1] - c[1], pa[2] - c[2]];
    let eb = [pb[0] - c[0], pb[1] - c[1], pb[2] - c[2]];
    let lb = (ea[0] * ea[0] + ea[1] * ea[1] + ea[2] * ea[2]).sqrt();
    let la = (eb[0] * eb[0] + eb[1] * eb[1] + eb[2] * eb[2]).sqrt();
    if !(la > 0.0 && lb > 0.0) {
        return None;
    }
    let cos_t = (ea[0] * eb[0] + ea[1] * eb[1] + ea[2] * eb[2]) / (la * lb);
    let sin2_t = (1.0 - cos_t * cos_t).max(0.0);
    let quad = la * la + lb * lb - 2.0 * la * lb * cos_t;
    if quad <= 0.0 {
        return None;
    }
    let lin = 2.0 * lb * u * (la * cos_t - lb);
    let cons = lb * lb * (u * u - la * la * sin2_t);
    let disc = lin * lin - 4.0 * quad * cons;
    if disc < 0.0 {
        return None;
    }
    let t = (-lin + disc.sqrt()) / (2.0 * quad);
    if t.partial_cmp(&u) != Some(std::cmp::Ordering::Greater) {
        return None;
    }
    let h = lb * (t - u) / t;
    if !(la * cos_t < h && h * cos_t < la) {
        return None;
    }
    Some(ta + t)
}

/// `f32` ordenável para a fila — a distância nunca é `NaN` aqui (as arestas têm
/// comprimento finito e as sementes são metades delas).
#[derive(PartialEq)]
struct Ordenavel(f32);
impl Eq for Ordenavel {}
impl PartialOrd for Ordenavel {
    fn partial_cmp(&self, outro: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(outro))
    }
}
impl Ord for Ordenavel {
    fn cmp(&self, outro: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&outro.0)
    }
}
