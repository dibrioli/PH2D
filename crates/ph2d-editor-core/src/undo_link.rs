//! ⭐ **O marcador do undo global** — um passo que mora em DUAS histórias e se desfaz como um só.
//!
//! A fila global do projecto (a da shell, por diff) e a história de uma família (o texto de uma aba
//! de script) são duas pilhas. Um gesto que muda as duas (renomear uma propriedade: o texto e as
//! chaves `own` de cada objecto) leva o MESMO [`UndoLink`] nas duas; desfazer por qualquer lado
//! desfaz os dois, ou nenhum. O passo global é o dono: todo desfazer ligado passa por ele, e a
//! família aplica a sua metade quando o lê em [`UndoLinks::done`].
//!
//! Recusa, nunca meio passo: se a outra metade já não está no topo da sua pilha (escreveu-se
//! depois na aba, ou mexeu-se na cena), nada anda e [`UndoLinks::refused`] diz porquê. O VS Code
//! desfaz só um lado e deixa o projecto partido (medição I10 §2, `docs/CodeEditor`).

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UndoLink(pub u64);

impl UndoLink {
    /// Um elo novo, único no processo.
    #[must_use]
    pub fn fresh() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// Porque um desfazer ligado não andou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A metade da família tem passos por cima do elo (o texto mudou depois).
    FamilyAhead,
    /// A fila global tem passos por cima do elo (a cena mudou depois).
    GlobalAhead,
}

/// A costura entre a shell (dona da fila global) e as famílias. Índice `[0]` = desfazer, `[1]` =
/// refazer.
#[derive(Debug, Default)]
pub struct UndoLinks {
    /// Família → shell: o próximo passo global leva este elo — e nasce mesmo sem o documento mudar
    /// (o texto está fora do diff).
    pub pending: Option<UndoLink>,
    /// Família → shell: o Ctrl+Z veio da aba; desfazer (`false`) / refazer o passo global com este elo.
    pub ask: Option<(UndoLink, bool)>,
    /// Família → shell: os elos cuja metade da família existe mas NÃO está no topo da pilha dela.
    pub held: [BTreeSet<UndoLink>; 2],
    /// Shell → família: os elos desfeitos (`false`) / refeitos na fila global desde a última leitura.
    pub done: Vec<(UndoLink, bool)>,
    /// Shell → família: os elos que a fila global ainda guarda (fora deles, a metade da família é
    /// um passo de texto como os outros).
    pub alive: BTreeSet<UndoLink>,
    /// Shell → família: o elo do passo do topo de cada lado.
    pub top: [Option<UndoLink>; 2],
    /// Quem recusou o último desfazer ligado (a família mostra a razão).
    pub refused: Option<Refusal>,
}

impl UndoLinks {
    /// O desfazer GLOBAL pode tirar o passo do topo, que leva `top`?
    ///
    /// # Errors
    /// [`Refusal::FamilyAhead`] se a metade da família desse elo não está no topo dela.
    pub fn global_may(&self, top: Option<UndoLink>, redo: bool) -> Result<(), Refusal> {
        match top {
            Some(l) if self.held[usize::from(redo)].contains(&l) => Err(Refusal::FamilyAhead),
            _ => Ok(()),
        }
    }

    /// A fila global vai tirar o degrau do topo (que leva `top`): `false` = não pode, e
    /// [`Self::refused`] diz porquê.
    pub fn begin_undo(&mut self, top: Option<UndoLink>, redo: bool) -> bool {
        match self.global_may(top, redo) {
            Ok(()) => true,
            Err(r) => {
                self.refused = Some(r);
                false
            }
        }
    }

    /// A fila global tirou o degrau que levava `top`: a família aplica a sua metade.
    pub fn end_undo(&mut self, top: Option<UndoLink>, redo: bool) {
        if let Some(l) = top {
            self.done.push((l, redo));
        }
    }

    /// O pedido da família, se o topo global é o elo dele: `Some(Ok(redo))` = desfazer/refazer
    /// agora; `Some(Err)` = recusado (fica em [`Self::refused`]).
    pub fn take_ask(&mut self) -> Option<Result<bool, Refusal>> {
        let (link, redo) = self.ask.take()?;
        if self.top[usize::from(redo)] == Some(link) {
            Some(Ok(redo))
        } else {
            self.refused = Some(Refusal::GlobalAhead);
            Some(Err(Refusal::GlobalAhead))
        }
    }

    /// A shell publica o estado da fila no fim do quadro.
    pub fn publish(
        &mut self,
        top: [Option<UndoLink>; 2],
        alive: impl IntoIterator<Item = UndoLink>,
    ) {
        self.top = top;
        self.alive.clear();
        self.alive.extend(alive);
    }

    /// O elo está vivo do lado global (ou ainda à espera de nascer lá)?
    #[must_use]
    pub fn is_alive(&self, link: UndoLink) -> bool {
        self.alive.contains(&link) || self.pending == Some(link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn um_elo_retido_pela_familia_recusa_o_desfazer_global_e_o_topo_errado_recusa_o_da_aba() {
        let mut l = UndoLinks::default();
        let a = UndoLink::fresh();
        assert_eq!(l.global_may(Some(a), false), Ok(()));
        l.held[0].insert(a);
        assert_eq!(l.global_may(Some(a), false), Err(Refusal::FamilyAhead));
        assert_eq!(
            l.global_may(Some(a), true),
            Ok(()),
            "cada lado tem a sua retenção"
        );
        assert_eq!(l.global_may(None, false), Ok(()));

        l.publish([Some(a), None], [a]);
        l.ask = Some((a, false));
        assert_eq!(l.take_ask(), Some(Ok(false)));
        l.ask = Some((a, true));
        assert_eq!(l.take_ask(), Some(Err(Refusal::GlobalAhead)));
        assert_eq!(l.refused, Some(Refusal::GlobalAhead));
        assert_eq!(l.take_ask(), None);
    }
}
