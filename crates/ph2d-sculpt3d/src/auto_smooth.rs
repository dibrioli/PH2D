//! **O ORÇAMENTO DE ALISAMENTO** — quantas vezes o laplaciano é aplicado, e com
//! que força cada vez.
//!
//! ⚠️ **O `autosmooth_factor` do Blender NÃO é a força de um lerp.** Esta é a
//! coisa que faltava, e ela não é um detalhe de afinação: um único
//! `lerp(p, média, fator)` reproduz a referência em `0,24` e diverge de forma
//! crescente acima disso. Em `1,0` o Blender aplica o operador **quatro vezes**
//! — suporte de ~4 anéis — contra uma vez, suporte de 1 anel.
//!
//! A lei é `brushes/smooth.cc:34-48`, `iteration_strengths`:
//!
//! ```text
//! c     = min(força, 1)
//! count = trunc(c · 4)
//! last  = 4 · (c − count/4)
//! passadas = [1,0] × count  ++  [last]
//! ```
//!
//! ⚠️ **Cada elemento é uma passada laplaciana COMPLETA**, não uma fração de
//! uma. Uma passada de força `1,0` com peso cheio faz `p ← centroide(vizinhos)`
//! — **substituição total**, não mistura. É por isso que a tabela abaixo não é
//! uma reescala do que tínhamos: ela muda o SUPORTE do alisamento.
//!
//! | `auto_smooth` | passadas | forças | orçamento |
//! |---|---|---|---|
//! | 0,05 | 1 | `[0,20]` | 0,20 |
//! | 0,10 | 1 | `[0,40]` | 0,40 |
//! | 0,24 | 1 | `[0,96]` | 0,96 |
//! | **0,25** | **2** | **`[1,00 · 0,00]`** | 1,00 |
//! | 0,50 | 3 | `[1 · 1 · 0]` | 2,00 |
//! | 0,75 | 4 | `[1 · 1 · 1 · 0]` | 3,00 |
//! | **1,00** | **5** | **`[1 · 1 · 1 · 1 · 0]`** | 4,00 |
//!
//! ⚠️ **O orçamento total é LINEAR no fator** (`Σ = 4c`); o que é quantizado em
//! quartos é como ele se PARTE entre substituições cheias e um resto. A razão de
//! projeto: cada passada é uma combinação convexa de peso `≤ 1`, logo uma
//! contração incondicionalmente estável — o laço é como o Blender exprime um
//! orçamento de até `4,0` sem nunca usar peso `> 1` num lerp, o que extrapolaria
//! além do centroide e oscilaria.
//!
//! ⚠️ **A última passada é anexada SEMPRE, mesmo valendo zero**, e nos quatro
//! valores exactos (`0,25 · 0,50 · 0,75 · 1,00`) ela é um no-op que ainda
//! percorre o conjunto. Mantê-la é fidelidade barata: o resultado é idêntico com
//! ou sem ela, e removê-la seria uma optimização que eu teria de justificar
//! contra a referência em vez de a herdar.

use crate::Pass;

/// O tecto de iterações CHEIAS — o `max_iterations` de `smooth.cc:36`.
///
/// ⚠️ **Ele não é um recurso, é a definição do orçamento.** Quatro é o
/// denominador que torna o fator do artista uma contagem: `0,25` é *uma*
/// substituição, `1,0` são *quatro*. Mudá-lo mudaria o que o slider significa,
/// não o quanto ele custa.
pub const MAX_ITERATIONS: usize = 4;

/// Quantas passadas o laço pode produzir — as cheias **mais o resto**.
pub const MAX_PASSES: usize = MAX_ITERATIONS + 1;

/// As passadas de um alisamento, sem alocar.
///
/// ⚠️ **Um `Vec` seria uma alocação por dab, por cópia de simetria** — e o
/// autosmooth roda uma vez por cópia (ver [`crate::SculptStroke::dab`]). O tecto
/// é conhecido e pequeno, então o buffer é do tamanho dele.
pub struct Iterations {
    buf: [Pass; MAX_PASSES],
    len: usize,
}

impl Iterations {
    /// As passadas, na ordem em que correm.
    #[must_use]
    pub fn as_slice(&self) -> &[Pass] {
        &self.buf[..self.len]
    }
}

/// **A LEI** — `brushes/smooth.cc:34-48`, verbatim na ordem das operações.
///
/// ⚠️ **A ordem aritmética é a da referência e não a álgebra equivalente:**
/// `4 · (c − count/4)` e `4c − count` são a mesma matemática e **não** os mesmos
/// bits. Escrever a segunda seria trocar a fonte por uma simplificação que
/// ninguém pediu.
///
/// ⚠️ **`as usize` trunca em direcção a zero, como o `int()` do C++** — e
/// satura, então um `NaN` daria `0` em vez de estourar. Quem garante que ele não
/// chega aqui é [`crate::Brush::auto_smooth_brush`], que peneira o não-finito
/// **antes**; esta função não repete a guarda porque duas respostas à mesma
/// pergunta divergem no dia em que uma delas mudar.
#[must_use]
pub fn iteration_strengths(strength: f32) -> Iterations {
    let clamped = strength.min(1.0);
    let count = (clamped * MAX_ITERATIONS as f32) as usize;
    let last = MAX_ITERATIONS as f32 * (clamped - count as f32 / MAX_ITERATIONS as f32);
    let mut buf = [Pass { weight: 0.0 }; MAX_PASSES];
    for slot in buf.iter_mut().take(count) {
        *slot = Pass { weight: 1.0 };
    }
    buf[count] = Pass { weight: last };
    Iterations {
        buf,
        len: count + 1,
    }
}

#[cfg(test)]
#[path = "auto_smooth_tests.rs"]
mod tests;
