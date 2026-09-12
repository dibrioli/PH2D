//! A re-exportação de [`ph2d_timeline_preview`] — o módulo mudou de crate, o nome não.
//!
//! ⭐ A **lei** (o censo `O(bindings)` do estado de antes do `apply`, a lista dos quatro factos que
//! uma curva conduz, e a declaração no ledger) é pura sobre o `World`/`TimelineDoc` e saiu para a
//! folha [`ph2d_timeline_preview`] na Fase C da W2, porque apareceu um **segundo consumidor numa
//! crate diferente**: os Smart Bones ([`ph2d_app_skeleton::smart`]) largam exactamente os mesmos
//! quatro factos ao desligar-se. *Duas famílias que partilham código partilham uma FOLHA, nunca uma
//! delas à outra* (HOWTO §1.2).
//!
//! ⛔ **Os GATES ficaram aqui, e isso é o desenho** (HOWTO §2.6 — *o teste segue o SUJEITO*): o que
//! eles medem é se o estado de pré-visualização atravessa a `crate::undo::ProjectState::capture`,
//! e a captura é da shell. A folha não sabe o que é uma captura.
//!
//! ⛔ **Este ficheiro FICA** para que os quatro sítios que escrevem `crate::timeline_preview::…`
//! continuem byte a byte iguais (HOWTO §1.4).

pub(crate) use ph2d_timeline_preview::*;

#[cfg(test)]
#[path = "timeline_preview_tests.rs"]
mod timeline_preview_tests;
