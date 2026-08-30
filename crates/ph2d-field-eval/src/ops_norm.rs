//! ⭐⭐ **A RAIZ COM PISO** — e o defeito de três camadas abaixo que ela cura.
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::ops`] responde a *«que forma cada primitiva é»*. Isto é outra coisa: é a aritmética que
//! **toda** norma desta crate atravessa, e a razão de ela existir é uma medição — não uma
//! conveniência. O arquivo passou as `700` linhas do gate de LOC quando o chanfro entrou (Enio,
//! 2026-08-30). ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ O `pub(crate) use` no [`crate::ops`] mantém `ops::safe_sqrt` e `ops::length2` — cortar um
//! arquivo não pode custar uma reescrita em cada sítio que o chamava.

use fidget::context::Tree;

/// Piso do argumento do `sqrt`, para que o **gradiente** exista em zero.
///
/// ⭐ **`sqrt` tem derivada INFINITA em zero**, e as normas abaixo são somas de `max(q, 0)`: dentro
/// de uma caixa ou de um cilindro **todos** os termos são exatamente zero, então o argumento é zero
/// no interior INTEIRO da peça, e não num ponto isolado. A diferenciação automática devolve ali
/// `0/0`, e quem consome esse `NaN` é a extração da malha: sem normal não há QEF, a célula cai no
/// baricentro das travessias, e a **quina viva** sai serrilhada.
///
/// ⚠️ **Este é o mecanismo por trás do achado da W0** (*"o desvio é igual à fração de célula em que
/// a face cai"*), e ele estava atribuído ao extrator. Duas hipóteses foram medidas e **refutadas**
/// antes desta: o leque da `fidget` e a interpolação linear da travessia. O que a medição fechou foi
/// a aritmética: o desvio era `0,72 × fração`, que é literalmente o baricentro. §21 do doc de
/// resultados.
///
/// ⚠️ **O número é de REPRESENTAÇÃO.** `sqrt(1e-30) = 1e-15`, que é 8 ordens de grandeza abaixo do
/// ULP de um `f32` de ordem 1 (1,19e-7) — logo o VALOR não muda em nenhum bit —, e `1e-30` é normal
/// em `f32` (o mínimo é 1,18e-38), então o piso não vira zero na fita. Abaixo do piso a derivada é a
/// de uma constante, que é **zero e finita**; é isso que se queria.
///
/// ⛔ Não vale trocar por `sqrt(s + ε)`: isso muda o valor em `√ε` em **toda parte**, e um raio de
/// filete deixaria de ser o pedido.
const LENGTH_FLOOR: f64 = 1.0e-30;

/// A raiz com o piso acima. ⚠️ **Toda raiz de uma soma de quadrados desta crate passa por aqui** —
/// uma que não passe reintroduz o `NaN` no gradiente, e o sintoma aparece na malha, três camadas
/// abaixo, como uma quina serrilhada.
pub(crate) fn safe_sqrt(s: Tree) -> Tree {
    s.max(LENGTH_FLOOR).sqrt()
}

pub(crate) fn length2(x: &Tree, y: &Tree) -> Tree {
    safe_sqrt(x.square() + y.square())
}

pub(crate) fn length3(x: &Tree, y: &Tree, z: &Tree) -> Tree {
    safe_sqrt(x.square() + y.square() + z.square())
}
