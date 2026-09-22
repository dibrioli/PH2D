//! ⭐⭐⭐⭐ **AS AMOSTRAS DO PLANO, COMO O ARQUIVO AS GUARDA** — filho
//! (`#[path]`) de [`super`] (`doc.rs`), cortado dele pelo ASSUNTO: lá *o que a
//! cena É como bytes*, aqui *como um plano de milhões de amostras cabe num
//! ficheiro*.
//!
//! # ⛔⛔ O que um plano custa CRU
//!
//! MEDIDO em 2026-09-21 na peça de fábrica (`98 306` vértices):
//!
//! | degrau | amostras | cruas |
//! |---|---:|---:|
//! | `2x` | `393 218` | `4,7 MB` |
//! | `8x` | `6 291 458` | **`75,5 MB`** |
//! | `16x` | `25 165 826` | **`302 MB`** |
//!
//! ⇒ *armar o `8x`, dar UM traço e gravar custaria `75 MB`.*
//!
//! # ⛔⛔⛔ E as CORRIDAS só ganham numa das três formas — MEDIDO
//!
//! A 1.ª redacção deste módulo guardava **só** corridas, com uma tabela que
//! saíra de uma sonda sobre [`ph2d_mesh_colors::Tinta::nova`]. A medição sobre
//! a [`ph2d_mesh_colors::Tinta::semeada`] — que é **como um plano nasce numa
//! peça já pintada** — derrubou-a:
//!
//! | plano ao `8x` | corridas | em corridas | contra cru |
//! |---|---:|---:|---:|
//! | `nova` (peça nunca pintada) | `1` | `~0` | **`0,000×`** |
//! | `semeada` de cor CHAPADA | `3 145 729` | `40,9 MB` | `0,542×` |
//! | `semeada` de cor VARIADA | `6 291 458` | `81,8 MB` | **`1,083×`** |
//!
//! ⚠️ **A semente INTERPOLA**, e interpolar entre três cores iguais não devolve
//! a cor em `f32`: os pesos baricêntricos somam `1` com erro de último bit. ⇒
//! *um plano «chapado» não é chapado nos BITS*, e um semeado de cor variada tem
//! todas as amostras distintas — onde cada corrida custa `1` byte de varint
//! **mais** os `12` da cor, ou seja `+8,3 %`.
//!
//! ⭐ ⇒ **duas formas, e o escritor escolhe a MENOR**, por uma conta exacta e
//! barata (uma passagem sobre as corridas). O pior caso passa a ser
//! `+1 byte`, e o melhor continua a ser `75 MB → 0`.
//!
//! ⛔ **A decisão de ter DUAS formas é medida, e a de não as ter também era:**
//! a 1.ª redacção dizia por escrito que *«`8 %` num caso que a pintura real não
//! produz não paga uma segunda forma»* — e a pintura real produz-o, porque toda
//! peça com cor por vértice semeia o plano dela.
//!
//! # ⛔⛔⛔ E a igualdade é por BITS, nunca por `==`
//!
//! `-0.0 == 0.0` é **verdade** em `f32` e os bits são diferentes. Uma corrida
//! fechada por `==` juntaria os dois e devolveria os bits errados na leitura —
//! *uma compressão que se diz sem perda e que troca um sinal de zero é pior do
//! que uma que se diz com perda*. Com `to_bits` o `NaN` também se comporta: ele
//! nunca é igual a si próprio por `==` (o que partiria toda corrida de `NaN`) e
//! é igual a si próprio nos bits.

/// Uma corrida de amostras iguais: quantas, e a cor.
pub(super) type Corrida = (u32, [f32; 3]);

fn bits(c: [f32; 3]) -> [u32; 3] {
    [c[0].to_bits(), c[1].to_bits(), c[2].to_bits()]
}

/// **As amostras em corridas.**
pub(super) fn em_corridas(amostras: &[[f32; 3]]) -> Vec<Corrida> {
    let mut out: Vec<Corrida> = Vec::new();
    for &a in amostras {
        match out.last_mut() {
            // ⚠️ `u32::MAX` parte a corrida em vez de estourar o contador — um
            // plano com mais de `4 · 10⁹` amostras iguais é inalcançável hoje
            // (o `16x` da peça de fábrica tem `25 · 10⁶`), e *um contador que
            // dá a volta escreve a cor no sítio errado em silêncio*.
            Some((n, cor)) if bits(*cor) == bits(a) && *n < u32::MAX => *n += 1,
            _ => out.push((1, a)),
        }
    }
    out
}

/// Quantos bytes um `u32` custa em varint — a conta que decide entre as duas
/// formas, e ela é EXACTA (o postcard usa LEB128 de 7 bits por byte).
fn varint(n: u32) -> usize {
    match n {
        0..=0x7f => 1,
        0x80..=0x3fff => 2,
        0x4000..=0x1f_ffff => 3,
        0x20_0000..=0xfff_ffff => 4,
        _ => 5,
    }
}

/// ⭐⭐⭐⭐ **A ESCOLHA: corridas ou cruas, a MENOR das duas.**
///
/// ⚠️ **A conta é exacta e custa uma passagem**, e é por isso que ela não
/// serializa as duas para comparar: aos `16x` cada serialização são `300 MB`.
/// Os dois lados omitem o prefixo de comprimento, que é o mesmo nos dois.
pub(super) fn a_menor_forma(amostras: &[[f32; 3]]) -> AmostrasDoc {
    let corridas = em_corridas(amostras);
    let custo_corridas: usize = corridas.iter().map(|&(n, _)| varint(n) + 12).sum();
    if custo_corridas < amostras.len() * 12 {
        AmostrasDoc::Corridas(corridas)
    } else {
        AmostrasDoc::Cruas(amostras.to_vec())
    }
}

/// As amostras de um plano, numa das duas formas.
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum AmostrasDoc {
    /// Uma por uma — a forma que ganha quando todas são distintas.
    Cruas(Vec<[f32; 3]>),
    /// Em corridas — a forma que leva um plano por pintar a `~0`.
    Corridas(Vec<Corrida>),
}

impl AmostrasDoc {
    /// **As amostras de volta** — `None` quando não somam o que a topologia
    /// desta malha pede. Ver [`das_corridas`].
    pub(super) fn amostras(&self, esperadas: usize) -> Option<Vec<[f32; 3]>> {
        match self {
            Self::Cruas(v) if v.len() == esperadas => Some(v.clone()),
            Self::Cruas(_) => None,
            Self::Corridas(c) => das_corridas(c, esperadas),
        }
    }
}

/// **As amostras de volta** — `None` quando as corridas não somam o que a
/// topologia desta malha pede.
///
/// ⚠️ **A contagem ESPERADA vem de fora**, e é a metade que faz disto uma
/// leitura segura: as corridas são entrada de terceiro, e um plano com o
/// tamanho errado instalado numa malha é tinta no sítio errado — o defeito que
/// o pânico de 21/09 (§14) já custou uma sessão do dono.
pub(super) fn das_corridas(corridas: &[Corrida], esperadas: usize) -> Option<Vec<[f32; 3]>> {
    // ⛔ A soma é feita ANTES de alocar: um documento forjado com corridas que
    // somam biliões faria o `with_capacity` pedir a memória toda.
    let total: u64 = corridas.iter().map(|&(n, _)| u64::from(n)).sum();
    if total != esperadas as u64 {
        return None;
    }
    let mut out = Vec::with_capacity(esperadas);
    for &(n, cor) in corridas {
        out.extend(std::iter::repeat_n(cor, n as usize));
    }
    Some(out)
}

#[cfg(test)]
#[path = "doc_tinta_tests.rs"]
mod tests;
