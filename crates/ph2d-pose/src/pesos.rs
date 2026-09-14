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
