//! **O `SignalAction` como o `PROJECT_SCHEMA` 128 o gravava** — CONGELADO, para a migração
//! `128 -> 129` (`docs/Components/08_plano_tags.md` §3.3).
//!
//! # Porque um tipo congelado, e não o vivo
//!
//! A W2 das Tags apendou `target_by` ao [`super::SignalAction`]. O postcard é POSICIONAL, e o blob de
//! um componente é opaco para o parse do ficheiro: um v128 lido com o tipo vivo pede a tag do alvo
//! onde já não há bytes e **falha longe da causa** — ou, com várias linhas, lê o comprimento do nome
//! da linha seguinte como a tag do alvo desta. ⇒ esta cópia lê os bytes antigos, e a lei do re-encode
//! vive aqui, ao lado do tipo que ela escreve; a shell só a chama no braço `128` do load (o precedente
//! exacto do `project_migrate_sprite`).
//!
//! ⚠️ **O [`SignalVerb`] é o VIVO, e isso é seguro por contrato**: a lista dele é APPEND-ONLY (a
//! posição é a tag), então todo verbo que um v128 guardou lê-se com o mesmo índice.
//!
//! ⛔ **Nunca a corra sobre um blob que não venha de um ficheiro v128.** Um blob vivo com UMA linha é
//! recusado (sobram os bytes do `target_by`), mas um com várias pode ler-se como v1 sem erro — quem
//! garante a origem é o número do ficheiro, não esta função.

use super::{SignalAction, SignalActions, SignalFrom, SignalTarget, SignalVerb};
use serde::Deserialize;

/// Uma linha, na forma v128. ⚠️ A ordem dos campos É o formato.
#[derive(Deserialize)]
struct SignalActionV1 {
    on: String,
    target: String,
    verb: SignalVerb,
    arg: String,
}

/// A tabela, na forma v128.
#[derive(Deserialize)]
struct SignalActionsV1(Vec<SignalActionV1>);

/// ⭐ **Os bytes de um `SignalActions` v128, reescritos no formato VIVO** — cada linha com
/// `target_by = Named` e `from = Anyone`, que é o que ela significava.
///
/// ⚠️⚠️ **Ela escreve o formato VIVO, não o v129** — e por isso ela cresce a cada campo apendado à
/// linha (o `from` do suplente #24 foi o segundo). *A escada do load tem TRÊS degraus vivos (`95`,
/// `128`, o corrente) e recusa tudo o que está no meio*, logo um v128 salta direito ao de hoje e
/// esta função é o salto inteiro. ⛔ Quem apendar o terceiro campo tem de o pôr aqui — senão um
/// ficheiro v128 deixa de compilar, que é o modo de falha bom.
///
/// `None` = os bytes não se leem como v128 (falha, ou sobra): quem chama deixa o blob como estava e
/// conta-o, porque reescrever com um palpite seria pior do que deixar.
#[must_use]
pub fn migrate_v1_blob(bytes: &[u8]) -> Option<Vec<u8>> {
    // ⚠️ `take_from_bytes` e **rejeitar o resto**, não `from_bytes`: o postcard consome um prefixo
    // válido e ignora o que sobra — a mesma defesa do `project_migrate_sprite`.
    let Ok((antigo, [])) = postcard::take_from_bytes::<SignalActionsV1>(bytes) else {
        return None;
    };
    let vivo = SignalActions(
        antigo
            .0
            .into_iter()
            .map(|a| SignalAction {
                on: a.on,
                target: a.target,
                verb: a.verb,
                arg: a.arg,
                target_by: SignalTarget::Named,
                from: SignalFrom::Anyone,
            })
            .collect(),
    );
    postcard::to_allocvec(&vivo).ok()
}
