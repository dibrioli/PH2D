//! ⭐⭐⭐ **DE QUE LADO FICA O RETRATO DE UM CARTÃO — a lei, e ela tem DOIS leitores.**
//!
//! Report do dono (2026-09-20, com foto): *«neste caso o preview deveria ser colocado para cima
//! (já existe esse recurso). Todo o restante ficou OK.»* — sobre uma cena acabada de arrumar, em
//! que o cartão do topo de uma coluna reservava a moldura do retrato **em baixo**, e ela ficava
//! no corredor entre ele e o cartão seguinte.
//!
//! ⛔⛔ **A recusa que estava escrita em [`super::extensao_de`] MORREU aqui, e a morte é
//! deliberada:** ela dizia *«virá-la para cima é um gesto do artista e não uma posição que esta
//! função escolha»*, com o preço de reservar os dois lados. A ordem do dono desempatou, e o preço
//! não se paga — **reserva-se UM lado, o que a lei escolhe**.
//!
//! ⚠️⚠️ **A lei tem de ser a MESMA nos dois sítios, e é por isso que ela é uma função e não uma
//! decisão guardada:** quem ARRUMA (`ph2d-app-motion`) precisa dela para reservar o espaço do
//! lado certo, e quem PINTA (este painel) precisa dela para desenhar a moldura ali. Guardá-la
//! num mapa obrigaria a shell a escrever no estado do painel, que ela não alcança — e *duas
//! respostas à mesma pergunta divergem no dia em que uma delas mudar*.
//!
//! ⭐⭐ **E ela é um PONTO FIXO da arrumação, por construção.** Ela lê a topologia das COLUNAS
//! (quem está por cima de quem), e a topologia não depende do lado do retrato: o espaçamento
//! horizontal sai do `left`/`right` de cada caixa (que o lado não muda) e a ordem dentro de uma
//! coluna sai da lei das portas e dos baricentros (que não vê extensões). ⇒ arrumar DUAS vezes
//! chega ao fim: o 1.º passe entrega as colunas, o 2.º reserva os retratos, e um 3.º não mexe um
//! bit (gate `arrumar_duas_vezes_e_o_mesmo_que_arrumar_uma`).

use super::{CAPSULA_H, CARD_W, card_h, card_h_de, has_preview_slot};
use crate::snapshot::GraphNodeView;
use ph2d_nodegraph::layout::BAND_GAP;
use std::collections::BTreeSet;

/// **A caixa de um cartão POUSADO**, em unidades do grafo — as quatro coisas que a lei lê.
///
/// ⚠️ **A `altura` é a do CORPO e nunca a da extensão**: a extensão já inclui a moldura do
/// retrato de um dos lados, e perguntar *«há alguém por baixo?»* com ela dentro responderia
/// sobre o espaço que o retrato reservou em vez de sobre o cartão seguinte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaixaDoCartao {
    /// Canto esquerdo do CORPO (a pastilha cresce para os dois lados a partir do centro, e a
    /// moldura do retrato é do tamanho do corpo — ver [`super::preview_frame_rect`]).
    pub x: f32,
    /// Topo do corpo.
    pub y: f32,
    /// Altura do corpo.
    pub altura: f32,
    /// Este cartão tem moldura de retrato? Só a esses a lei responde.
    pub retrato: bool,
}

/// A caixa que este cartão desenha, onde ele está — irmã de [`super::extensao_desenhada`], e a
/// mesma aritmética: quem mede a extensão e quem mede a caixa nunca podem discordar sobre a
/// altura de um corpo.
#[must_use]
pub fn caixa_desenhada(n: &GraphNodeView) -> CaixaDoCartao {
    CaixaDoCartao {
        x: n.x,
        y: n.y,
        altura: card_h(n).max(CAPSULA_H),
        retrato: has_preview_slot(n),
    }
}

/// Ver [`caixa_desenhada`] — a MESMA aritmética, a partir das PEÇAS, para o cartão de GRUPO, que
/// é derivado pela shell e não passa pelo retrato do painel (a mesma razão de
/// [`super::extensao_de`] existir ao lado da irmã).
#[must_use]
pub fn caixa_de(
    x: f32,
    y: f32,
    fileiras_de_pino: f32,
    fileiras_de_param: f32,
    tem_readout: bool,
    tem_retrato: bool,
) -> CaixaDoCartao {
    CaixaDoCartao {
        x,
        y,
        altura: card_h_de(fileiras_de_pino, fileiras_de_param, tem_readout).max(CAPSULA_H),
        retrato: tem_retrato,
    }
}

/// ⭐⭐⭐ **A LEI: o retrato vai para o lado de FORA da coluna.**
///
/// Para cada caixa, `true` = a moldura fica **EM CIMA**. Um cartão que tem outro por baixo na
/// mesma coluna e **nenhum por cima** está no TOPO dela: ali o lado de baixo é um CORREDOR entre
/// duas fileiras — que é o que o dono fotografou — e o de cima é espaço aberto. Nos outros três
/// casos o retrato fica **EM BAIXO**, que é a omissão de sempre:
///
/// | por cima | por baixo | lado | porquê |
/// |---|---|---|---|
/// | não | **sim** | **EM CIMA** | é o topo da coluna: em baixo é corredor, em cima é fora |
/// | sim | não | em baixo | já está fora — é o fundo da coluna |
/// | sim | sim | em baixo | está no meio: os dois lados são corredor, e a omissão não surpreende |
/// | não | não | em baixo | está sozinho: os dois lados são fora, e a omissão não surpreende |
///
/// ⚠️ **«Na mesma coluna» é medido por SOBREPOSIÇÃO EM `x` das caixas**, e depois de uma
/// arrumação isso é exactamente *«na mesma coluna»*: o passo horizontal é
/// `max_right(c−1) − min_left(c) + GAP_X`, logo duas colunas vizinhas **nunca** se sobrepõem em
/// `x`. Numa disposição feita à mão a pergunta continua a ser a certa — *há alguém no corredor
/// para onde este retrato ia?*
///
/// ⭐⭐ **E o vizinho tem de estar na MESMA BANDA, com a régua DERIVADA da própria disposição:**
/// ela separa dois cartões de uma coluna por `GAP_Y` (`40`) de extensão a extensão e duas BANDAS
/// — dois pedaços do grafo que não se tocam — por [`BAND_GAP`] (`200`). Entre CORPOS isso dá, no
/// pior caso de uma coluna, `GAP_Y` mais a moldura que um dos dois reservou (`128`), ou seja
/// `168`; e entre bandas dá `BAND_GAP` ou mais. ⇒ **a régua é `BAND_GAP` e a comparação é
/// estrita**, e com ela um vizinho de coluna conta sempre e um de outra banda nunca.
///
/// ⛔ **Sem ela, o primeiro pedaço do grafo comportava-se diferente dos outros cinco** — só o
/// cartão do topo do ECRÃ tinha «nada por cima», e os outros liam o cartão da banda anterior como
/// se fosse do corredor deles. *O dono viu o primeiro e escreveu «todo o restante ficou OK»; sem
/// a régua, o restante deixaria de o estar.*
///
/// ⛔ **Os dois casos em que um vizinho de coluna fica a mais de `BAND_GAP`** (a lei dos
/// baricentros pode afastar um cartão para além do piso) **leem-se como «sozinho», e isso é
/// benigno:** o retrato vai para baixo, onde há mais espaço do que ele ocupa.
///
/// ⚠️ **Custo:** `O(cartões²)`, calculado **uma vez por quadro** e nunca por cartão — na cena
/// que o dono fotografou são `41` cartões, `1 681` comparações de `f32`.
#[must_use]
pub fn retratos_em_cima(caixas: &[CaixaDoCartao]) -> Vec<bool> {
    caixas
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if !c.retrato {
                return false;
            }
            let mut acima = false;
            let mut abaixo = false;
            for (j, o) in caixas.iter().enumerate() {
                // Mesma coluna: as duas caixas partilham alguma coluna de pixels.
                if i == j || c.x >= o.x + CARD_W || o.x >= c.x + CARD_W {
                    continue;
                }
                let vao_abaixo = o.y - (c.y + c.altura);
                let vao_acima = c.y - (o.y + o.altura);
                abaixo |= (0.0..BAND_GAP).contains(&vao_abaixo);
                acima |= (0.0..BAND_GAP).contains(&vao_acima);
            }
            abaixo && !acima
        })
        .collect()
}

/// Os ids desta tela cujo retrato a lei põe **em cima** — a porta que o painel usa, porque ele
/// pinta um cartão de cada vez e a lei responde sobre a tela toda.
pub(crate) fn retratos_em_cima_por_id(nos: &[GraphNodeView]) -> BTreeSet<u32> {
    let caixas: Vec<CaixaDoCartao> = nos.iter().map(caixa_desenhada).collect();
    retratos_em_cima(&caixas)
        .into_iter()
        .zip(nos.iter())
        .filter_map(|(em_cima, n)| em_cima.then_some(n.id))
        .collect()
}

#[cfg(test)]
#[path = "geom_retrato_tests.rs"]
mod tests;
