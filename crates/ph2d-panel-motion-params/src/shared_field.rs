//! **QUEM É DONO DO CAMPO DE TEXTO PARTILHADO** de um slot — as duas metades da mesma
//! pergunta, lado a lado de propósito.
//!
//! Há **um** `TextInput` por slot ([`crate::snapshot::param_text_id`]) e **quatro** rows que o
//! usam: a `Text` (a fórmula), o *Custom…* da `Channels`, o campo da `Source` e o caminho da
//! `File`. O `snapshot_ids` declara essa partilha como desenho — *«um campo de texto por slot,
//! seja a row quem for»*.
//!
//! ⚠️ **Uma partilha assim tem DUAS perguntas, não uma:**
//!
//! - *de quem é o buffer quando ele VOLTA?* → [`shared_text_param`], que o `on_text_commit` lê;
//! - *o que é que o buffer MOSTRA?* → [`shared_text_value`], que o `seed_rows` lê.
//!
//! ⛔⛔ **Até 2026-08-30 só a primeira existia.** O `on_text_commit` tinha as quatro armas e o
//! `seed_rows` semeava **só a `Text`** ⇒ as outras três abriam sempre em branco (com o
//! placeholder no lugar do valor) e, porque o `Blur` comita, tocar no campo **gravava o vazio**:
//! clicar no caminho de um ficheiro e clicar noutro sítio APAGAVA-O. Report do Enio sobre a cena
//! `=109`: *"O painel não imprime o caminho do CSV em lugar nenhum"* — a metade visível.
//!
//! ⚠️ **Estar no mesmo ficheiro é o ponto.** As duas listas divergiram porque viviam em módulos
//! diferentes (`events.rs` e `row_state.rs`), cada uma a parecer completa onde estava. Aqui uma
//! variante nova é **erro de compilação nas duas** (nenhuma tem braço curinga), e o gate
//! `every_row_that_can_commit_the_shared_field_also_fills_it` mede que continuam a concordar.

use crate::snapshot::ParamRow;

/// Quantas variantes o [`ParamRow`] tem — a metade **contada** de que um censo precisa.
///
/// ⚠️ Um `match` exaustivo obriga quem acrescenta uma variante a responder às duas perguntas
/// acima, mas **não** obriga a amostra de um teste a incluí-la: *um match exaustivo não guarda
/// a lista que um laço percorre*. É este número que a obriga.
///
/// ⚠️ **Instrumento, não produto** (`cfg(test)`): as duas portas abaixo é que shipam. Ele existe
/// para o censo, e o censo corre no portão de fecho, que compila os testes.
#[cfg(test)]
pub(crate) const PARAM_ROW_KINDS: usize = 13;

/// Um índice distinto por variante — só para o censo saber o que já viu.
///
/// ⚠️ Exaustivo **sem curinga**: uma variante nova obriga a numerá-la aqui, e aí o
/// [`PARAM_ROW_KINDS`] fica a dever um degrau, que é exactamente o alarme que se quer.
#[cfg(test)]
pub(crate) fn param_row_kind(row: &ParamRow) -> usize {
    match row {
        ParamRow::Scalar(_) => 0,
        ParamRow::Color(_) => 1,
        ParamRow::Toggle(_) => 2,
        ParamRow::Enum(_) => 3,
        ParamRow::Angle(_) => 4,
        ParamRow::Seed(_) => 5,
        ParamRow::Text(_) => 6,
        ParamRow::Curve(_) => 7,
        ParamRow::Gradient(_) => 8,
        ParamRow::Palette(_) => 9,
        ParamRow::Channels(_) => 10,
        ParamRow::Source(_) => 11,
        ParamRow::File(_) => 12,
    }
}

/// **De quem é o buffer quando ele volta** — o text param que uma edição do campo partilhado
/// escreve, ou `None` para uma row que não usa o campo.
///
/// ⚠️ A `Curve`, a `Gradient` e a `Palette` **também** carregam uma `String` num text param, e
/// mesmo assim respondem `None`: elas têm editor próprio e o artista nunca vê a string. *Ter um
/// valor de texto e usar o CAMPO de texto são perguntas diferentes.*
pub(crate) fn shared_text_param(row: &ParamRow) -> Option<&'static str> {
    match row {
        ParamRow::Text(r) => Some(r.name),
        ParamRow::Channels(r) => Some(r.text_param),
        ParamRow::Source(r) => Some(r.param),
        ParamRow::File(r) => Some(r.name),
        ParamRow::Scalar(_)
        | ParamRow::Color(_)
        | ParamRow::Toggle(_)
        | ParamRow::Enum(_)
        | ParamRow::Angle(_)
        | ParamRow::Seed(_)
        | ParamRow::Curve(_)
        | ParamRow::Gradient(_)
        | ParamRow::Palette(_) => None,
    }
}

/// **O que o buffer mostra** — o valor vivo do documento que o campo partilhado espelha, ou
/// `None` para uma row que não usa o campo.
///
/// ⚠️ Os três doc-comments do [`crate::snapshot`] já prometiam isto — *"shown in the Custom
/// field"*, *"fills the field"*, *"the current path"* — e nenhum tinha código por baixo. *Uma
/// promessa escrita na struct não é um leitor.*
pub(crate) fn shared_text_value(row: &ParamRow) -> Option<&str> {
    match row {
        ParamRow::Text(r) => Some(&r.value),
        ParamRow::Channels(r) => Some(&r.custom),
        ParamRow::Source(r) => Some(&r.current),
        ParamRow::File(r) => Some(&r.value),
        ParamRow::Scalar(_)
        | ParamRow::Color(_)
        | ParamRow::Toggle(_)
        | ParamRow::Enum(_)
        | ParamRow::Angle(_)
        | ParamRow::Seed(_)
        | ParamRow::Curve(_)
        | ParamRow::Gradient(_)
        | ParamRow::Palette(_) => None,
    }
}
