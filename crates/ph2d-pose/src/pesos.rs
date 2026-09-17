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
/// `d(v)` é a distância, **andando pelas arestas da malha**, do vértice `v` à
/// fronteira do conjunto — negativa dentro, positiva fora. O peso é
/// `suave(0,5 − d/banda)`, com `suave(x) = x²(3 − 2x)` cortado a `[0,1]`:
/// a `d = −banda/2` vale `1`, a `d = +banda/2` vale `0`, e a transição mede
/// **exactamente `banda`**.
///
/// # ⚠️ Porque é que ela é DIJKSTRA e não mais difusão
///
/// A difusão espalha **uma aresta por passagem**, logo uma faixa de `k` arestas
/// custa `k²` passagens e o preço é `O(V · k²)` — a faixa de **um raio** do
/// pincel pedia `~1 200` passagens na peça de fábrica. A distância custa
/// `O(V log V)` **uma vez**, e não depende da largura pedida.
///
/// ⚠️ **A fronteira fica a MEIO da aresta que a atravessa**, que é o que faz a
/// transição ficar centrada onde o anel binário acabava: sem isso ela
/// deslocar-se-ia meia aresta para fora, e a saída voltaria a depender da
/// densidade — *que é o defeito inteiro que esta função existe para curar*.
///
/// ⛔ **Um vértice sem vizinhos MANTÉM o peso** — a mesma cláusula da
/// [`suavizar`], pela mesma razão.
pub fn por_distancia(viz: &Vizinhanca, posicoes: &[[f32; 3]], pesos: &mut [f32], banda: f32) {
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

    // (1) As SEMENTES: cada aresta que atravessa a fronteira põe os dois
    // extremos a meia aresta dela, com o sinal do lado em que estão.
    let mut d = vec![f32::INFINITY; n];
    let mut fila: std::collections::BinaryHeap<(std::cmp::Reverse<Ordenavel>, u32)> =
        std::collections::BinaryHeap::new();
    for v in 0..n {
        for &u in viz.de(u32::try_from(v).unwrap_or(u32::MAX)) {
            let u = u as usize;
            if dentro[v] == dentro[u] {
                continue;
            }
            let meio = comp(v, u) * 0.5;
            if meio < d[v] {
                d[v] = meio;
                fila.push((
                    std::cmp::Reverse(Ordenavel(meio)),
                    u32::try_from(v).unwrap_or(u32::MAX),
                ));
            }
        }
    }
    // ⚠️ Sem fronteira nenhuma o campo é constante e não há o que esbater — e
    // devolver aqui é o que mantém um segmento vazio ou cheio **ao bit**.
    if fila.is_empty() {
        return;
    }

    // (2) DIJKSTRA sobre os comprimentos das arestas.
    while let Some((std::cmp::Reverse(Ordenavel(dv)), v)) = fila.pop() {
        let v = v as usize;
        if dv > d[v] {
            continue;
        }
        for &u in viz.de(u32::try_from(v).unwrap_or(u32::MAX)) {
            let u = u as usize;
            let alt = dv + comp(v, u);
            if alt < d[u] {
                d[u] = alt;
                fila.push((
                    std::cmp::Reverse(Ordenavel(alt)),
                    u32::try_from(u).unwrap_or(u32::MAX),
                ));
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
