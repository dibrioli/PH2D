//! **As duas pontas de cada nome de sinal** (GEF02 onda 1): quantos campos o EMITEM e quantos o
//! OUVEM, e o sufixo que uma caixa de nome de sinal mostra ao artista.
//!
//! A contagem é por nome EXATO — o defeito que isto torna visível é a letra trocada que vira
//! silêncio (`coin_taken` × `coin_takn`). Um campo nunca é a sua própria outra ponta: quem emite só
//! soma em `emit`, e o sufixo de quem emite lê só `listen`.

use std::collections::BTreeMap;

/// O papel de um campo de nome de sinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// O campo publica o sinal (On Hit, On Death, …).
    Emit,
    /// O campo reage ao sinal (On signal, Signal Action `on`, …).
    Listen,
}

/// Quantos campos emitem e quantos ouvem um nome.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ends {
    pub emit: u32,
    pub listen: u32,
}

/// O que a caixa diz sobre a outra ponta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hint {
    /// Nome vazio: mudo de propósito, sem sufixo.
    Empty,
    /// `n ≥ 1` campos na outra ponta.
    Count(u32),
    /// Nenhum campo na outra ponta, mas um script Luau usa um nome não literal nessa direcção.
    ScriptMay,
    /// Ninguém na outra ponta.
    Nobody,
}

/// O censo de um quadro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignalCensus {
    pub names: BTreeMap<String, Ends>,
    /// Algum script emite com um nome que não é literal.
    pub dynamic_emit: bool,
    /// Algum script ouve com um nome que não é literal (ou ouve tudo, `on_signal`).
    pub dynamic_listen: bool,
}

impl SignalCensus {
    /// Soma um campo com o nome `name` no papel `role`; vazio não conta.
    pub fn add(&mut self, name: &str, role: Role) {
        if name.is_empty() {
            return;
        }
        let e = self.names.entry(name.to_owned()).or_default();
        match role {
            Role::Emit => e.emit += 1,
            Role::Listen => e.listen += 1,
        }
    }

    /// As pontas de `name` (zeros se ninguém o usa).
    #[must_use]
    pub fn ends(&self, name: &str) -> Ends {
        self.names.get(name).copied().unwrap_or_default()
    }

    /// O sufixo de uma caixa que tem `name` no papel `role`: conta a OUTRA ponta.
    #[must_use]
    pub fn hint(&self, name: &str, role: Role) -> Hint {
        if name.is_empty() {
            return Hint::Empty;
        }
        let e = self.ends(name);
        let (n, dynamic) = match role {
            Role::Emit => (e.listen, self.dynamic_listen),
            Role::Listen => (e.emit, self.dynamic_emit),
        };
        if n > 0 {
            Hint::Count(n)
        } else if dynamic {
            Hint::ScriptMay
        } else {
            Hint::Nobody
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_outra_ponta_e_contada_e_o_proprio_campo_nao() {
        let mut c = SignalCensus::default();
        c.add("coin_taken", Role::Emit);
        assert_eq!(c.hint("coin_taken", Role::Emit), Hint::Nobody);
        c.add("coin_taken", Role::Listen);
        assert_eq!(c.hint("coin_taken", Role::Emit), Hint::Count(1));
        assert_eq!(c.hint("coin_taken", Role::Listen), Hint::Count(1));
        assert_eq!(c.hint("coin_takn", Role::Emit), Hint::Nobody);
        assert_eq!(c.hint("", Role::Emit), Hint::Empty);
        c.dynamic_listen = true;
        assert_eq!(c.hint("coin_takn", Role::Emit), Hint::ScriptMay);
        assert_eq!(c.hint("coin_takn", Role::Listen), Hint::Nobody);
    }
}

/// QUEM ouve, numa linha de [`Listener`]: o componente (o nome do tipo), o documento de fora do mundo
/// (`MotionGraph`, `StateSets`) ou um script Luau.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Quem {
    Component(&'static str),
    Doc(&'static str),
    Script,
}

/// Um ouvinte de um nome: o objecto (o `Name` dele; `None` = sem nome, ou um documento) e o que ouve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listener {
    pub object: Option<String>,
    pub what: Quem,
}

/// Quem ouve um nome (doc Timeline 17 item 8): uma linha por (objecto, o que ouve), na ordem da tabela.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignalListeners {
    pub list: Vec<Listener>,
    /// Um script ouve com um nome não literal (ou ouve tudo): pode ouvir este também.
    pub script_may: bool,
}
