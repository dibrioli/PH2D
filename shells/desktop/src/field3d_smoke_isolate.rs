//! ⭐⭐ **O ISOLAMENTO do modelador** — *mostra-me só este* — em duas leis, e elas são **duas
//! perguntas**.
//!
//! ⚠️ **Arquivo irmão por LOC** (HR-18, W97): o `field3d_smoke.rs` estava a **606 linhas** contra o
//! tecto de 600 — e estava **antes desta wave**, escondido por outros ✗ que cancelavam a suíte antes
//! de lá chegar. ⛔ *Split, nunca allowlist.*
//!
//! ⭐ O corte é por ASSUNTO e não por tamanho: as duas leis puras aqui dentro são as que os gates
//! dirigem sem armar o módulo, e as duas casca finas em cima delas são as únicas que tocam o estado.

use super::with_smoke;

/// ⭐ **ISOLA o nó dado — ou devolve a peça inteira à vista.** Devolve o estado NOVO
/// (`true` = isolado).
///
/// ⚠️ **A lei é a do módulo irmão, LIDA e não re-decidida** (`sculpt3d_objects::toggle_isolate`):
///
/// - **Toggle**, não um modo com saída própria — *"um «sair do isolamento» separado seria uma
///   segunda porta para o mesmo fato, e a que o artista não acha quando a cena some"*.
/// - **Nada entra na história**: isolar não move um vértice.
/// - **Isolar «nada» é recusado** — apagaria a cena da tela sem nada para devolver.
///
/// ⚠️ Isolar OUTRO nó com um já isolado **troca**, não sai: o gesto é *"mostra-me este"*, e obrigar
/// a sair primeiro seria dois gestos para uma intenção.
pub(crate) fn toggle_isolate(node: Option<u64>) -> bool {
    with_smoke(|s| {
        s.isolated = next_isolation(s.isolated, node);
        s.isolated.is_some()
    })
    .unwrap_or(false)
}

/// ⭐ **A LEI do isolamento, sem estado nenhum** — e é ela que os gates dirigem.
///
/// ⚠️ Separada de propósito: o [`Smoke`] só nasce com o módulo **armado** (env var ou pill), então um
/// gate que quisesse exercer o toggle teria de o armar — e o estado do smoke é `thread_local`, que
/// com `--test-threads=1` é partilhado. *Armar o módulo para provar uma regra de três linhas
/// contaminaria todos os gates que correm depois.*
///
/// | atual | pedido | fica | porquê |
/// |---|---|---|---|
/// | qualquer | `None` | como estava | isolar «nada» apagaria a cena sem nada para devolver |
/// | `Some(x)` | `Some(x)` | `None` | **toggle**: a mesma porta sai |
/// | `Some(x)` | `Some(y)` | `Some(y)` | **troca**: o gesto é *"mostra-me este"* |
/// | `None` | `Some(y)` | `Some(y)` | entra |
pub(crate) fn next_isolation(current: Option<u64>, asked: Option<u64>) -> Option<u64> {
    match asked {
        None => current,
        Some(a) if current == Some(a) => None,
        Some(a) => Some(a),
    }
}

/// ⭐⭐ **A LEI DA TECLA** (W44) — e ela é **outra pergunta**, não um caso da de cima.
///
/// | atual | seleção | fica | porquê |
/// |---|---|---|---|
/// | `Some(_)` | qualquer | **`None`** | há isolamento ⇒ a tecla SAI, seja o que for que esteja escolhido |
/// | `None` | `Some(s)` | `Some(s)` | entra no escolhido |
/// | `None` | `None` | `None` | não há o que isolar |
///
/// # ⚠️ Por que não é a [`next_isolation`]
///
/// As duas mexem no mesmo campo e respondem a perguntas diferentes:
///
/// - o **chip numa linha** diz *"mostra-me ESTE"* — com `A` isolado e `B` escolhido, ele **troca**,
///   porque foi no `B` que se clicou;
/// - a **tecla** é global e diz *"dentro ou fora"* — com `A` isolado e `B` escolhido, ela **sai**.
///
/// ⛔ Unificá-las com uma bandeira faria uma das duas mentir sobre o gesto que a chamou. É a mesma
/// disciplina que separa [`toggle_isolate`] de [`forget_isolation`] (*sair* e *o alvo morreu* são
/// fatos diferentes) — e a lei da tecla é a que o módulo irmão já tem escrita
/// (`sculpt3d_objects::toggle_isolate`: se está isolado, larga; senão isola o que está em mãos).
///
/// # ⭐ E é ela que garante que a PORTA DE SAÍDA existe sempre
///
/// A razão escrita para o isolamento ser um toggle era *"um «sair» separado seria a porta que o
/// artista não acha quando a cena some"*. ⚠️ **O chip não a cumpria:** ele só é pintado quando o
/// escolhido se destaca da peça, então com a **raiz** escolhida — ou com nada — a fileira inteira
/// desaparece e não há gesto nenhum que devolva a peça. A tecla responde de qualquer sítio.
pub(crate) fn key_isolation(current: Option<u64>, selected: Option<u64>) -> Option<u64> {
    if current.is_some() {
        return None;
    }
    selected
}

/// **A tecla pediu o toggle** — aplicado pela ponte, que é quem tem a seleção.
pub(crate) fn toggle_isolate_by_key(selected: Option<u64>) -> Option<u64> {
    with_smoke(|s| {
        s.isolated = key_isolation(s.isolated, selected);
        s.isolated
    })
    .flatten()
}
