//! **O mundo da macro num instante** — a porta ÚNICA que o aplicar (o fim) e o palco (cada quadro) leem:
//! o inicial mais os passos até `t`, em ids locais. A física não está aqui: o [`Simula`] diz só QUANDO
//! correu; quem a recoze é o palco.

use super::formato::{ErroDeLeitura, Evento, Inicial, Mudanca, Objeto, Quem, Recurso, Simula};
use ph2d_macro::{MacroTime, Track};
use std::collections::BTreeMap;

/// Um objecto local em `t`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjetoEm {
    pub quem: Quem,
    /// O `StableId` (na gravação; depois do aplicar, no projecto de destino).
    pub original: u64,
    /// `None` = o objecto Macro.
    pub pai: Option<u32>,
    /// Pelo nome canónico.
    pub componentes: BTreeMap<String, Vec<u8>>,
    /// Já nasceu e ainda não morreu.
    pub vivo: bool,
}

/// A trilha do mundo lida (uma vez; o palco guarda-a).
#[derive(Clone, Debug, PartialEq)]
pub struct Lida {
    pub inicial: Inicial,
    pub eventos: Vec<(MacroTime, Evento)>,
}

impl Lida {
    /// # Errors
    /// Os bytes do inicial ou de um passo não se leem.
    pub fn de(t: &Track) -> Result<Self, ErroDeLeitura> {
        Ok(Self {
            inicial: Inicial::le(&t.initial)?,
            eventos: t
                .events
                .iter()
                .map(|e| Evento::le(&e.payload).map(|v| (e.t, v)))
                .collect::<Result<_, _>>()?,
        })
    }

    /// As corridas da física, com o instante em que cada uma começa.
    pub fn simulas(&self) -> impl Iterator<Item = (MacroTime, &Simula)> {
        self.eventos.iter().filter_map(|(t, e)| match e {
            Evento::Simula(s) => Some((*t, s)),
            Evento::Passo { .. } => None,
        })
    }

    /// Todos os objectos que a trilha nomeia (o inicial e os que aparecem nos passos).
    pub fn objetos(&self) -> impl Iterator<Item = &Objeto> {
        self.inicial
            .objetos
            .iter()
            .chain(self.eventos.iter().flat_map(|(_, e)| match e {
                Evento::Passo { novos, .. } => novos.as_slice(),
                Evento::Simula(_) => &[],
            }))
    }

    /// Todos os recursos embutidos.
    pub fn recursos(&self) -> impl Iterator<Item = &Recurso> {
        self.inicial
            .recursos
            .iter()
            .chain(self.eventos.iter().flat_map(|(_, e)| match e {
                Evento::Passo { recursos, .. } => recursos.as_slice(),
                Evento::Simula(_) => &[],
            }))
    }

    /// **O mundo da macro em `t`** (os passos com instante ≤ `t`; `MacroTime(u64::MAX)` = o fim).
    #[must_use]
    pub fn em(&self, t: MacroTime) -> BTreeMap<u32, ObjetoEm> {
        let quem: BTreeMap<u32, (Quem, u64)> = self
            .objetos()
            .map(|o| (o.id, (o.quem, o.original)))
            .collect();
        let mut m: BTreeMap<u32, ObjetoEm> = BTreeMap::new();
        let novo = |id: u32| {
            let (q, original) = quem.get(&id).copied().unwrap_or((Quem::Mundo, 0));
            ObjetoEm {
                quem: q,
                original,
                pai: None,
                componentes: BTreeMap::new(),
                vivo: false,
            }
        };
        for l in &self.inicial.linhas {
            let o = m.entry(l.id).or_insert_with(|| novo(l.id));
            o.pai = l.pai;
            o.vivo = true;
            o.componentes = l
                .componentes
                .iter()
                .map(|c| (c.nome.clone(), c.bytes.clone()))
                .collect();
        }
        for (te, e) in &self.eventos {
            if *te > t {
                break;
            }
            let Evento::Passo { mudancas, .. } = e else {
                continue;
            };
            for mu in mudancas {
                match mu {
                    Mudanca::Nasce(l) => {
                        let o = m.entry(l.id).or_insert_with(|| novo(l.id));
                        o.pai = l.pai;
                        o.vivo = true;
                        o.componentes = l
                            .componentes
                            .iter()
                            .map(|c| (c.nome.clone(), c.bytes.clone()))
                            .collect();
                    }
                    Mudanca::Morre(id) => {
                        if let Some(o) = m.get_mut(id) {
                            o.vivo = false;
                        }
                    }
                    Mudanca::Muda {
                        id,
                        pai,
                        componentes,
                    } => {
                        let o = m.entry(*id).or_insert_with(|| novo(*id));
                        o.vivo = true;
                        if let Some(p) = pai {
                            o.pai = *p;
                        }
                        for (n, b) in componentes {
                            match b {
                                Some(b) => {
                                    o.componentes.insert(n.clone(), b.clone());
                                }
                                None => {
                                    o.componentes.remove(n);
                                }
                            }
                        }
                    }
                }
            }
        }
        m
    }

    /// O fim da gravação.
    #[must_use]
    pub fn no_fim(&self) -> BTreeMap<u32, ObjetoEm> {
        self.em(MacroTime(u64::MAX))
    }
}

#[cfg(test)]
#[path = "estado_tests.rs"]
mod tests;
