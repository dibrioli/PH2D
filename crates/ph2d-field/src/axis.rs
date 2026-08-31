//! ⭐⭐⭐ **O EIXO DE UM MODIFICADOR** — qual das três direcções locais ele usa.
//!
//! # ⛔⛔ O report que o obrigou (Enio, 2026-08-31)
//!
//! *«Melhor colocar os 3 eixos como opções para cada modificador. Acho que é isso que está criando
//! o problema: o efeito está atuando num eixo diferente do desejado.»*
//!
//! Ele tinha razão, e a foto prova-o: uma chapa **alta** (`1,005` em Y) e **fina** (`0,072` em Z)
//! com uma dobra que só sabia agir em **Z** — havia `0,072` de matéria para dobrar, e o que ele
//! queria era curvar a altura. ⚠️ **Rodar o nó não é a saída**, e a razão já estava escrita para o
//! espelho: um modificador age no espaço **local**, *antes* da pose, então usar a rotação exigiria
//! um nó intermédio só para rodar, aplicar e desrodar. *Uma equivalência que precisa de uma
//! terceira entidade não é uma equivalência.*
//!
//! # ⭐ A lei: CONJUGAÇÃO, e não um operador por eixo
//!
//! Cada modificador continua a ter **uma** lei, escrita no eixo canónico dele. O eixo escolhido
//! entra por fora: leva-se a peça ao referencial canónico, aplica-se a lei, e desfaz-se. Isso é
//! `f_A = P⁻¹ ∘ f_canónico ∘ P` — ver `ph2d_field_eval::stack`.
//!
//! ⛔⛔ **E `P` tem de ser uma permutação CÍCLICA, nunca uma troca de dois eixos.** Uma troca tem
//! determinante `−1`: ela **espelha** a peça, e uma torção espelhada gira ao contrário. As três
//! permutações cíclicas (identidade e as duas rotações) têm determinante `+1`, e para qualquer par
//! `(de, para)` existe exactamente uma delas que leva um ao outro. *A escolha do eixo não pode
//! mudar a quiralidade do efeito.*

use serde::{Deserialize, Serialize};

/// Um dos três eixos **locais** do nó.
///
/// ⚠️ **Locais, e é a convenção da casa** — a mesma do espelho e das dimensões da forma. O que o
/// artista roda é a peça; o que este eixo escolhe é a direcção em que o modificador age *dentro*
/// dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    /// Os três, na ordem em que o painel os mostra. ⚠️ **É a fonte da contagem** — um eixo novo não
    /// existe, mas quem contar `3` à mão fica com um número escrito em dois sítios.
    pub const ALL: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    /// As chaves i18n dos três, na ordem de [`Self::ALL`]. ⚠️ **Chaves, nunca rótulos** (HR-15).
    pub const KEYS: [&'static str; 3] = ["field.axis.x", "field.axis.y", "field.axis.z"];

    /// O índice `0..2`, que é como ele viaja pelo painel (um número numa linha).
    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }

    /// ⭐ **COAGE, nunca recusa** — a lei da porta deste módulo. Um índice fora da faixa vira o mais
    /// próximo, porque a alternativa seria o `FieldDoc::new` recusar a peça inteira ao revalidar.
    #[must_use]
    pub fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            i if i <= 0 => Axis::X,
            1 => Axis::Y,
            _ => Axis::Z,
        }
    }

    /// ⭐⭐⭐ **A rotação cíclica que leva ESTE eixo ao eixo `para`** — `0`, `1` ou `2`.
    ///
    /// ⚠️ **Cíclica, e nunca uma troca**: ver a nota do módulo. `s = (para − de) mod 3` é a única
    /// rotação que resolve o par, e ela existe sempre.
    #[must_use]
    pub fn shift_to(self, para: Axis) -> usize {
        (3 + usize::from(para.index()) - usize::from(self.index())) % 3
    }

    /// Um vector do referencial do mundo levado ao **canónico**, para uma rotação de `s` passos.
    ///
    /// ⚠️ A bola de bordo é uma esfera, então levar um modificador a outro eixo é exactamente
    /// permutar as coordenadas do **centro** dela — não há lei nova a escrever.
    #[must_use]
    pub fn to_canonical(v: [f32; 3], s: usize) -> [f32; 3] {
        [v[(3 - s) % 3], v[(4 - s) % 3], v[(5 - s) % 3]]
    }

    /// O caminho de volta da [`Self::to_canonical`].
    #[must_use]
    pub fn from_canonical(v: [f32; 3], s: usize) -> [f32; 3] {
        [v[s % 3], v[(1 + s) % 3], v[(2 + s) % 3]]
    }
}

#[cfg(test)]
#[path = "axis_tests.rs"]
mod tests;
