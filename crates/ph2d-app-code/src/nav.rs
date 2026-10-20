//! ⭐ **UM histórico da família** (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.5): os sítios por onde
//! o Ctrl+clique/F12 e os links levaram, com o ‹ › (Alt+← / Alt+→ e os botões do cabeçalho da
//! página). Medido no Godot 4.7.2 (`ferramentas/oraculo_ajuda/saida/amostras.txt`): empilha o
//! Ctrl+clique que abre uma página e o link para OUTRA página; o link na mesma página, o índice e o
//! Ctrl+F não. Fechar uma aba tira as entradas dela.

use ph2d_script::ScriptId;

/// Um sítio da família.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Place {
    Script {
        id: ScriptId,
        caret: usize,
    },
    /// `page` = o índice no catálogo; `scroll` = a rolagem da vista quando se saiu dela.
    Help {
        page: usize,
        scroll: f32,
    },
}

impl Place {
    /// A aba do sítio (o id do hero).
    pub fn tab(self) -> u64 {
        match self {
            Place::Script { id, .. } => id.0,
            Place::Help { page, .. } => crate::help::tab_id(page),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Nav {
    list: Vec<Place>,
    /// O sítio corrente em `list` (sem sentido com ela vazia).
    at: usize,
}

impl Nav {
    /// Onde se está agora: a entrada corrente segue-o (o cursor, a rolagem); outra aba (escolhida na
    /// fila de abas) entra como sítio novo depois da corrente.
    pub fn record(&mut self, here: Place) {
        match self.list.get_mut(self.at) {
            Some(p) if p.tab() == here.tab() => *p = here,
            Some(_) => {
                self.list.truncate(self.at + 1);
                self.list.push(here);
                self.at += 1;
            }
            None => {
                self.list = vec![here];
                self.at = 0;
            }
        }
    }

    /// Ir de `from` para `to` (o Ctrl+clique que abre uma página, o link para outra página).
    pub fn go(&mut self, from: Place, to: Place) {
        self.record(from);
        self.list.truncate(self.at + 1);
        self.list.push(to);
        self.at = self.list.len() - 1;
    }

    /// ‹ a partir de `here`.
    pub fn back(&mut self, here: Place) -> Option<Place> {
        if self.list.is_empty() {
            return None;
        }
        self.record(here);
        let i = self.at.checked_sub(1)?;
        self.at = i;
        Some(self.list[i])
    }

    /// › a partir de `here`.
    pub fn forward(&mut self, here: Place) -> Option<Place> {
        if self.list.get(self.at).is_none_or(|p| p.tab() != here.tab()) {
            return None;
        }
        self.record(here);
        let p = *self.list.get(self.at + 1)?;
        self.at += 1;
        Some(p)
    }

    /// Há para onde voltar / avançar a partir da aba `tab`.
    pub fn can(&self, tab: u64) -> (bool, bool) {
        let Some(cur) = self.list.get(self.at) else {
            return (false, false);
        };
        if cur.tab() != tab {
            return (true, false);
        }
        (self.at > 0, self.at + 1 < self.list.len())
    }

    /// A aba `tab` fechou: as entradas dela saem (e duas iguais seguidas fundem-se). Devolve o sítio
    /// corrente que fica (o ANTERIOR, se era o fechado).
    pub fn remove_tab(&mut self, tab: u64) -> Option<Place> {
        let mut kept = Vec::with_capacity(self.list.len());
        let mut at = None;
        for (i, p) in self.list.iter().enumerate() {
            if p.tab() == tab {
                continue;
            }
            if kept.last().is_some_and(|q: &Place| q.tab() == p.tab()) {
                kept.pop();
            }
            kept.push(*p);
            if i <= self.at {
                at = Some(kept.len() - 1);
            }
        }
        self.list = kept;
        self.at = at.unwrap_or(0);
        self.list.get(self.at).copied()
    }

    /// O sítio corrente.
    pub fn current(&self) -> Option<Place> {
        self.list.get(self.at).copied()
    }

    /// ⚠️ SONDA: as abas do histórico e a corrente.
    #[cfg(test)]
    pub fn trail(&self) -> (Vec<u64>, usize) {
        (self.list.iter().map(|p| p.tab()).collect(), self.at)
    }
}

#[cfg(test)]
mod tests {
    use super::{Nav, Place};
    use ph2d_script::ScriptId;

    fn s(caret: usize) -> Place {
        Place::Script {
            id: ScriptId(1),
            caret,
        }
    }

    fn h(page: usize, scroll: f32) -> Place {
        Place::Help { page, scroll }
    }

    /// O roteiro do oráculo: Ctrl+clique, ‹ volta ao script; de novo, link para outra página, ‹ › .
    #[test]
    fn o_roteiro_do_godot() {
        let mut n = Nav::default();
        n.go(s(5), h(0, 0.0));
        // o link na mesma página não passa por aqui; ‹ volta ao script, no cursor do clique
        assert_eq!(n.back(h(0, 40.0)), Some(s(5)));
        n.go(s(5), h(0, 0.0));
        n.go(h(0, 12.0), h(3, 0.0));
        assert_eq!(n.back(h(3, 7.0)), Some(h(0, 12.0)));
        assert_eq!(n.forward(h(0, 12.0)), Some(h(3, 7.0)));
        assert_eq!(n.forward(h(3, 7.0)), None);
        assert_eq!(n.can(h(3, 0.0).tab()), (true, false));
    }

    #[test]
    fn fechar_uma_aba_tira_as_entradas_e_volta_a_anterior() {
        let mut n = Nav::default();
        n.go(s(5), h(0, 0.0));
        n.go(h(0, 1.0), h(3, 0.0));
        assert_eq!(n.remove_tab(h(3, 0.0).tab()), Some(h(0, 1.0)));
        assert_eq!(n.remove_tab(h(0, 0.0).tab()), Some(s(5)));
        assert_eq!(n.trail().0.len(), 1);
    }
}
