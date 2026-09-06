//! ⭐⭐⭐ **A ORDEM EM QUE A ONDA CORRE PELA FILA** (ciclo 2, W2 — doc 105).
//!
//! **O problema, medido:** o stagger tinha `reverse` — um booleano —, e era só isso. A onda
//! corria da esquerda para a direita, ou ao contrário. *«Do meio para fora»* e *«cada um na sua
//! vez, sorteado»* — os dois gestos que toda referência oferece e que um artista pede no
//! primeiro dia — eram **inexprimíveis**.
//!
//! ## O que a lista de candidatos perdeu ao ser MEDIDA
//!
//! O [plano do ciclo](../../../docs/Motion%20Nodes/105_ciclo_2_animadores.md) pedia sete
//! entradas (`Index` · `Reverse` · `From Center` · `From Edges` · `Random` · `By X` · `By Y`).
//! **Ficaram três**, e as quatro que caíram caíram por razões diferentes:
//!
//! - ⛔ **`Reverse` e `From Edges` são a COMPOSIÇÃO que já existe.** O `reverse` espelha o
//!   `raw`, e espelhar o *«do meio para fora»* **é** o *«das pontas para o meio»* — exactamente
//!   a mesma curva. Duas entradas de enum para o que um toggle já faz é a definição de duas
//!   portas para uma pergunta. ⭐ E o desenho que fica é melhor que o das referências (o
//!   Cavalry lista as duas separadas): **três ordens × dois sentidos = seis comportamentos, com
//!   três entradas e um toggle que já existia.**
//! - ⛔ **`By X` e `By Y` pedem um POSTO, e um posto é uma ordenação** — saber que lugar o
//!   elemento `i` ocupa em `x` exige olhar todos os outros. Isso não é um mapa por-elemento, e
//!   este nó tem kernel de GPU: pô-lo aqui derrubava o nó inteiro para a CPU, que é o erro que a
//!   lei nº 1 do ciclo proíbe. ⚠️ **A saída existe e tem nome:** o `motion.sort(key = X)` a
//!   montante, com a `Index` aqui — e ele tem um efeito que esta porta não tem (ele **reordena**
//!   as linhas, então tudo a jusante vê a lista noutra ordem). *Duas ferramentas, dois efeitos,
//!   e o artista escolhe qual quer.*
//!
//! ## As três que ficam são todas FUNÇÃO DE `i` E `n`
//!
//! É o que as mantém no dispositivo: cada elemento calcula a sua posição na onda sem olhar para
//! nenhum vizinho.

/// A escada do param `order` — **append-only**, é o wire format.
pub const ORDER_INDEX: i32 = 0;
pub const ORDER_FROM_CENTER: i32 = 1;
pub const ORDER_RANDOM: i32 = 2;

/// ⭐ **A POSIÇÃO DE `i` NA ONDA, `0..1`** — a porta única, lida pela CPU e pelo WGSL.
///
/// - `Index` — `i/(n−1)`, a fila da esquerda para a direita. **O default, byte a byte.**
/// - `From Center` — `|2i − (n−1)| / (n−1)`: o meio arranca em `0` e as duas pontas em `1`.
///   ⚠️ Numa fila PAR os dois elementos do meio partilham o mínimo, que é o que o olho espera.
/// - `Random` — um hash de `(i, seed)` em `0..1`. ⚠️ **Não é uma permutação, e é de propósito:**
///   uma permutação exigiria ordenar (ver o doc do módulo). O que o artista quer de um stagger
///   aleatório é *um atraso próprio para cada um*, e é isso que um hash dá — por elemento, sem
///   olhar para ninguém, e igual em toda corrida com a mesma semente.
#[must_use]
pub fn raw_at(order: i32, i: u32, n: u32, seed: u32) -> f32 {
    if n <= 1 {
        return 0.0;
    }
    let last = (n - 1) as f32;
    match order {
        ORDER_FROM_CENTER => {
            // `|2i − (n−1)|` vai de `0` (meio) a `n−1` (pontas) — a divisão devolve `0..1`.
            let d = (2.0 * i as f32 - last).abs();
            d / last
        }
        ORDER_RANDOM => hash01(i, seed),
        // `Index` e qualquer valor fora de alcance: o de sempre.
        _ => i as f32 / last,
    }
}

/// Um `0..1` determinista a partir de `(i, seed)` — o finalizador do splitmix32, a mesma
/// família que o `motion.scatter` já usa. ⚠️ **Escrito em `u32` com `wrapping`**, para o WGSL
/// poder fazer a MESMA aritmética: em `f32` as duas rotas divergiriam.
#[must_use]
pub fn hash01(i: u32, seed: u32) -> f32 {
    let mut x = i
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    // `/ 2^32` — a divisão exacta que o WGSL escreve igual.
    x as f32 / 4_294_967_296.0
}

#[cfg(test)]
#[path = "order_tests.rs"]
mod tests;
