//! **O DRENO DO PAINEL AUTORADO — um braço por variante, e nenhum curinga.**
//!
//! # O defeito que este arquivo existe para não ter
//!
//! Até 2026-08-30 o dreno morava no `mod.rs` e era um `if let`:
//!
//! ```ignore
//! for intent in ph2d_panel_authored::drain_intents() {
//!     if let ph2d_panel_authored::AuthoredIntent::Fired { key } = intent {
//!         self.signals.publish(ph2d_runtime::Signal::from_control(&key));
//!     }
//! }
//! ```
//!
//! As outras QUATRO variantes caíam fora e morriam no fim do quadro. O preço não é *"um intent
//! perdido"* — é **seis famílias de widget mortas de uma vez**: `Tabs`, `SegmentedAdaptive`,
//! `RadioGroup` e `Dropdown` publicam [`AuthoredIntent::Choice`]; `TextInput` e `NumberInput`
//! publicam [`AuthoredIntent::Text`]. O sintoma do artista é o pior que há — *o chip ACENDE*
//! (quem marca a opção é o `rows::select_in`, no `apply_event`, e ele funciona) *e nada muda*.
//!
//! ⚠️ **Esta acusação sobrevive a TODO gate de registo do repo.** O
//! `architecture_panel_wiring_parity` mede **focalizabilidade** — pintado sem `InteractiveState`
//! ⇒ morto sob o dedo —, e aqui o widget está registado, focável, vivo sob o rato e com o
//! handler a existir. Os `seam_*` provam que o clique **chega à ferramenta**, nunca que a escrita
//! dela chega a um consumidor. *A pergunta que faltava é a terceira: o leitor DECIDE, ou entrega
//! a alguém que descarta?*
//!
//! # ⛔ Um `_ => {}` aqui reproduz o defeito na primeira variante nova
//!
//! É por isso que [`signal_name`] é um `match` com **braço NOMEADO para cada variante**,
//! incluindo as três que ficam deliberadamente caladas — cada uma com o motivo escrito ao lado.
//! O gate `the_drain_names_every_authored_intent_variant` lê ESTE arquivo e reprova se uma
//! variante do enum não tiver braço, ou se alguém escrever um curinga.
//!
//! # A escolha de desenho: o NOME carrega a opção (b), e a origem não ganha carga (a)
//!
//! Uma escolha traz um dado que o `Fired` não tem — *qual* opção. Havia duas saídas:
//!
//! **(a) dar carga ao [`ph2d_runtime::SignalOrigin::Control`].** ⛔ **Medida e recusada**, por
//! duas razões que apontam para o mesmo lado:
//!
//! 1. **Contradiz o contrato declarado da crate.** O doc do `SignalOrigin` diz, com todas as
//!    letras: *"o CONTRATO é o nome (ADR-0143) — um consumidor casa numa string e nunca precisa
//!    perguntar a origem"*. Um consumidor obrigado a ler `origin` para saber QUAL aba foi
//!    escolhida é um consumidor que passou a perguntar a origem.
//! 2. **`SignalOrigin` é `Copy`**, e o texto de um campo não é. Para o `Choice` o `index` até
//!    caberia (`usize` é `Copy`); para o `Text` a carga é uma `String`, e ela mataria o `Copy`
//!    do enum inteiro — ou seja, a saída (a) nem sequer resolve as duas variantes que sobram.
//!
//! ⚠️ **O gate `the_event_core_is_a_leaf` NÃO era o obstáculo** — foi lido: ele mede
//! `[dependencies]` do manifesto (a crate tem de ser folha), e um campo a mais num enum não
//! acrescenta dependência nenhuma. *A cerca que decidiu foi outra, e é a do próprio ADR-0143.*
//!
//! **(b) o NOME compõe a row com a opção** — adoptada. É literalmente a lei que o `Contact` e a
//! `Animation` desta casa já seguem: *chegada e saída não se distinguem por um campo,
//! distinguem-se por serem NOMES diferentes*; *fim e volta idem*. Escolher a aba **Multiply** da
//! row **Blend** grita `blend/multiply`, e escolher **Screen** grita `blend/screen` — duas
//! distinções, dois nomes, zero campos novos.
//!
//! ⚠️ **A opção é slugificada pela MESMA porta que cunha a chave da row**
//! ([`crate::ui_panel_spec::key_of`]). Uma segunda derivação daria ao artista um nome que ele não
//! consegue prever — e o `key_of` mapeia todo carácter não-alfanumérico para `_`, então o
//! separador `/` **não pode colidir** com nada que ele escreva.
//!
//! ⚠️ **(b) resolve o `Choice` e NÃO resolve o `Text`**, e isso está nomeado no braço dele: o
//! conteúdo de um campo de texto é ARBITRÁRIO, e um nome de sinal que o artista digita não é um
//! contrato — é um espaço de nomes ilimitado em que cada tecla inventa um sinal novo.

use ph2d_panel_authored::AuthoredIntent;
use ph2d_runtime::{Signal, SignalOutbox};

/// O separador entre a chave da row e a da opção.
///
/// ⚠️ **Ele é seguro por CONSTRUÇÃO, não por convenção:** [`crate::ui_panel_spec::key_of`] troca
/// todo carácter que não seja `[a-z0-9]` por `_`, então nenhuma chave — nem a da row, nem a da
/// opção — pode conter uma barra. Um `.` ou um `:` teriam a mesma propriedade; o que não a teria
/// é o `_`, que colidiria com a chave de uma row chamada *"Blend Multiply"*.
const OPTION_SEP: char = '/';

/// **O nome do sinal que este intent grita** — `None` = deliberadamente calado.
///
/// A porta única, e é ela que o gate mede: um `match` sem curinga sobre o enum inteiro.
#[must_use]
pub(crate) fn signal_name(intent: &AuthoredIntent) -> Option<String> {
    match intent {
        // Um APERTO não tem estado — ele não é um valor que se leia depois —, então este canal é
        // o único que o carrega. (Era o único braço que existia.)
        AuthoredIntent::Fired { key } => Some(key.clone()),
        // **UMA DE N.** A distinção viaja no NOME, não numa carga: ver o § (b) do cabeçalho.
        AuthoredIntent::Choice { key, index } => Some(choice_name(key, *index)),
        // ⛔ **CALADO por ter OUTRO FIO, e ele existe:** o valor de um slider vive no
        // `WidgetStore`, e quem dirige a arte a partir dele é o `crate::vec_widget_drive`
        // (`Drive::Opacity`). Publicá-lo também aqui poria o mesmo facto em dois fios — a
        // divergência que este repo passa a vida a curar.
        AuthoredIntent::Value { .. } => None,
        // ⛔ **CALADO pela mesma razão do `Value`**, e o fio é o mesmo arquivo
        // (`Drive::Visible`, alimentado por `Toggle`/`Checkbox`).
        AuthoredIntent::Flag { .. } => None,
        // ⛔ **CALADO, e este é o único dos três cujo fio NÃO está completo — a dívida fica
        // nomeada em vez de disfarçada.** O conteúdo chega ao `WidgetStore` (é o `buffer` de um
        // `NumberInput` e o `text` de um `TextInput`, ambos lidos por `WidgetStore::text`), mas o
        // `vec_widget_drive::bindable` ainda não aceita estes dois tipos, então ninguém a jusante
        // o lê.
        //
        // ⛔ **E compor o conteúdo no NOME está REFUTADO, não adiado:** o nome é um contrato que
        // um consumidor casa literalmente, e um contrato que o artista digita não é um contrato —
        // é um espaço de nomes ilimitado onde cada tecla inventa um sinal novo. A cura é um
        // CONSUMIDOR do valor, do lado de lá da fronteira, e não mais um produtor deste lado.
        AuthoredIntent::Text { .. } => None,
    }
}

/// O nome composto de uma escolha: `<row>/<opção>`.
///
/// ⚠️ **O recuo é o ÍNDICE, e ele nunca colapsa no nome do `Fired`.** A row é encontrada na tabela
/// VIVA (a mesma que o `apply_event` acabou de percorrer, no mesmo quadro e na mesma thread), então
/// a opção está lá; se um dia não estiver, o sinal sai `blend/2` em vez de `blend` — um nome
/// distinto do que um botão chamado *Blend* gritaria. *Recuar para o nome da row faria uma escolha
/// disparar, em silêncio, o contrato de outro controle.*
#[must_use]
pub(crate) fn choice_name(key: &str, index: usize) -> String {
    let label = ph2d_panel_authored::rows::with_rows(|rows| {
        rows.iter()
            .find(|r| r.key == key)
            .and_then(|r| r.options.get(index).cloned())
    });
    match label {
        Some(l) => format!("{key}{OPTION_SEP}{}", crate::ui_panel_spec::key_of(&l)),
        None => format!("{key}{OPTION_SEP}{index}"),
    }
}

/// **Publica o que um intent tem a dizer**, se tiver.
///
/// ⚠️ **A CHAMADA a `drain_intents` fica no `render_loop/mod.rs`, de propósito** — e não aqui,
/// que seria o sítio arrumado. O gate
/// `the_authored_intent_queue_has_a_drain_and_it_runs_before_the_signal_drain`
/// (`shells/desktop/tests/the_two_halves_read_the_glyph_through_one_door.rs`) mede a ORDEM do
/// quadro **por posição de texto dentro do `mod.rs`**: virar o quadro < drenar o painel < ler os
/// sinais. Levar a chamada para um irmão apaga a única lente que essa lei tem — *e foi medido: o
/// gate reprovou com «a fila de intents não tem dreno», que é a acusação certa lida do lugar
/// errado.*
///
/// O que vem para cá é a DECISÃO (o `match` sem curinga), que é o que tem gate próprio.
pub(crate) fn publish(signals: &mut SignalOutbox, intent: &AuthoredIntent) {
    if let Some(name) = signal_name(intent) {
        signals.publish(Signal::from_control(&name));
    }
}

#[cfg(test)]
#[path = "authored_intents_tests.rs"]
mod tests;
