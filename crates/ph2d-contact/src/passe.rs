//! ⭐⭐⭐ **O PASSE AUTOMÁTICO** — doc 115 W5: *«o app separa sozinho»* (ordem do dono, 2026-09-17,
//! escolhida por ele entre as três leituras possíveis de tirar o `motion.collide` do catálogo).
//!
//! # O que ele é, e o que ele NÃO é
//!
//! Ele é um **ACABAMENTO sobre o que vai ser desenhado**: recebe a corrente cozida de um sink,
//! afasta as peças que se sobrepõem, e devolve-a. Não há nó, não há fio, não há realimentação.
//!
//! ⚠️ **A morada foi MEDIDA e não escolhida** (doc 115 §3, a W0): as duas leituras — o passe como
//! acabamento e o passe realimentado no `rope.state` — são **indistinguíveis acima de 16
//! varreduras**, e a `32` as duas entregam zero pares sobrepostos. A barata é a que encosta na
//! barra, e ela não precisa de fechar anel nenhum.
//!
//! # ⛔ Porque ele é FINO, e isso é a prova de que não há segunda cópia da lei
//!
//! As portas partilhadas desta crate **já são** a lei: [`crate::colisores`] lê o que cada linha
//! declara, [`crate::inv_inercias`] diz quanto cada peça roda, e [`crate::separate`] resolve os
//! contactos na grelha espacial. O `motion.collide` chama exactamente as mesmas três e acrescenta
//! por cima os knobs do CARTÃO dele (o `Radius` de recuo, o `Strength`, o `falloff`) — que um
//! passe automático **não tem**, porque não tem cartão.
//!
//! ⇒ *a diferença entre os dois não é a lei, são os knobs*, e é por isso que este ficheiro pode
//! nascer sem duplicar uma linha de aritmética — e sobreviver ao dia em que a W6 apagar aquele nó.
//!
//! # As três ausências, cada uma uma decisão
//!
//! - **Sem recuo para DISCO.** O nó, ao encontrar uma peça que não declara nada, dá-lhe o disco do
//!   `Radius` do cartão — *sem isso uma corrente MISTA deixaria metade das peças inertes e caladas*.
//!   Aqui não há cartão de onde tirar um raio, e **inventar um seria pior**: uma peça que não
//!   declara forma nenhuma é uma peça que ninguém disse que colide.
//! - **Sem `Strength` e sem `falloff`.** Os dois são mistura no fim (a decisão 3 do nó), e os dois
//!   são autorados. Um passe sem cartão corre a lei inteira ou não corre.
//! - **Sem realimentação.** Ver a W0 acima.

use crate::{Pecas, Saida};
use ph2d_nodegraph::attr::{Column, Stream};

/// A coluna do peso inverso de cada peça (`0` = obstáculo, que não se move).
///
/// ⚠️ **O nome é uma dívida PRÉ-EXISTENTE e nomeada:** ele é escrito como literal em **49** sítios
/// por oito crates de nó, cada uma com a cópia privada dela (`INV_MASS_COL`, `INV_MASS`, ou o
/// literal cru), e uma delas exporta-o `pub`. Unificá-lo é uma varredura que atravessa linhas e não
/// é desta wave; o que esta const faz é não acrescentar a nona cópia **anónima**.
const INV_MASS_COLUMN: &str = "inv_mass";

/// A coluna da posição.
const P_COLUMN: &str = "P";

/// A coluna do ângulo, em graus — a mesma que o [`crate::colisores`] lê para orientar uma caixa.
const ROT_COLUMN: &str = "rot";

/// Os pesos de uma corrente, ou `1` para quem não os declara.
fn pesos(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COLUMN) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// **Separa as peças que a corrente DECLARA.** `None` quando ela não declara colisor nenhum — e é
/// isso que mantém toda cena sem colisão **byte-idêntica**: quem pergunta sai antes de tocar em
/// nada, sem clonar sequer a corrente.
///
/// `varreduras` é quantas passagens o solver faz; `0` devolve `None` (um passe que não varre nada
/// não é um passe, e devolver a corrente clonada seria trabalho por nada).
///
/// ⚠️ **O `rot` só se escreve quando alguma peça de facto RODOU** — a mesma lei estrutural do nó:
/// acrescentar a coluna a uma corrente que não a tinha muda a resposta de todo consumidor a jusante
/// que distinga *«sem rotação»* de *«rotação zero»*.
#[must_use]
pub fn separa_o_que_se_desenha(entrada: &Stream, varreduras: usize) -> Option<Stream> {
    if varreduras == 0 {
        return None;
    }
    let colisores = crate::colisores(entrada)?;
    let n = entrada.count();
    if colisores.len() != n || !colisores.iter().any(Option::is_some) {
        return None;
    }
    let Some(Column::Vec2(p)) = entrada.get(P_COLUMN) else {
        return None; // sem posição não há o que afastar
    };
    if p.len() != n {
        return None;
    }
    let w = pesos(entrada, n);
    let inv_i = crate::inv_inercias(entrada, &colisores, &w);
    let mut pos = p.clone();
    let mut giro = vec![0.0f32; n];
    crate::separate(
        &mut pos,
        &mut Saida { giro: &mut giro },
        &Pecas::novas(&colisores, &w, &inv_i),
        varreduras,
    );
    // ⭐ **Nada se mexeu ⇒ nada se escreve.** Uma cena cujas peças já estão separadas devolve a
    // corrente de entrada intacta, e o quadro é byte-idêntico ao de antes desta wave existir.
    if pos == *p && giro.iter().all(|g| *g == 0.0) {
        return None;
    }
    let mut out = Stream::new(n);
    for (name, col) in entrada.columns() {
        if name != P_COLUMN {
            out.set(name.clone(), col.clone());
        }
    }
    out.set(P_COLUMN, Column::Vec2(pos));
    if giro.iter().any(|g| *g != 0.0) {
        let antes = match entrada.get(ROT_COLUMN) {
            Some(Column::Scalar(v)) if v.len() == n => v.clone(),
            _ => vec![0.0; n],
        };
        out.set(
            ROT_COLUMN,
            Column::Scalar(antes.iter().zip(&giro).map(|(a, g)| a + g).collect()),
        );
    }
    Some(out)
}

#[cfg(test)]
#[path = "passe_tests.rs"]
mod tests;
