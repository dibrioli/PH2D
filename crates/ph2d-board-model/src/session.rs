//! **A sessão ao vivo do documento** (ADR-0181 §5): os ids sorteados com o bit alto, a cauda das
//! chaves de z e o caderno do que mudou. Nada disto é conteúdo nem vai para o ficheiro.

use std::collections::BTreeMap;

use crate::{BoardDoc, Element, ElementId, FracKey};

/// O bit alto de um id nascido numa sessão ao vivo (ADR-0181 §5): os sorteados nunca colidem com os
/// do contador local, que os ficheiros de hoje usam.
pub const SESSION_ID_BIT: u64 = 1 << 63;

/// O passo do splitmix64 (Steele, Lea & Flood, 2014).
pub(crate) fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// O sorteio de uma sessão ao vivo (splitmix64 sobre um contador atómico: sorteia com `&self`).
/// `None` = fora de sessão. Não é conteúdo: o `PartialEq` ignora-o, e nunca vai para o ficheiro —
/// o do documento ([`Session`]) e o da equipa ([`crate::Team::set_session`]).
#[derive(Debug, Default)]
pub(crate) struct Lottery(Option<std::sync::atomic::AtomicU64>);

impl Lottery {
    pub(crate) fn new(seed: Option<u64>) -> Self {
        Self(seed.map(std::sync::atomic::AtomicU64::new))
    }

    /// O próximo número sorteado; `None` fora de sessão.
    pub(crate) fn draw(&self) -> Option<u64> {
        use std::sync::atomic::Ordering::Relaxed;
        let r = self.0.as_ref()?;
        Some(splitmix64(r.fetch_add(0x9e37_79b9_7f4a_7c15, Relaxed)))
    }
}

impl Clone for Lottery {
    fn clone(&self) -> Self {
        use std::sync::atomic::Ordering::Relaxed;
        Self::new(self.0.as_ref().map(|r| r.load(Relaxed)))
    }
}

impl PartialEq for Lottery {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// A sessão ao vivo de um documento: o gerador dos ids sorteados e das caudas das chaves de z, e o
/// caderno do que mudou desde a última leitura ([`BoardDoc::take_changes`]). Não é conteúdo (o
/// `PartialEq` do documento ignora-o) nem vai para o ficheiro.
#[derive(Clone, Debug, Default)]
pub(crate) struct Session(Option<Box<SessionState>>);

#[derive(Clone, Debug)]
pub(crate) struct SessionState {
    rng: Lottery,
    /// O estado de cada elemento tocado ANTES da 1.ª mudança desde a última leitura (`None` = não
    /// existia).
    changes: BTreeMap<ElementId, Option<Element>>,
}

impl Session {
    fn next(&self) -> Option<u64> {
        self.0.as_ref()?.rng.draw()
    }

    /// Anota o estado de `id` antes de uma mudança (só a 1.ª desde a última leitura conta).
    pub(crate) fn note(&mut self, id: ElementId, before: Option<&Element>) {
        if let Some(s) = &mut self.0 {
            s.changes.entry(id).or_insert_with(|| before.cloned());
        }
    }
}

impl PartialEq for Session {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl BoardDoc {
    /// Um id ainda não usado neste quadro: o contador local; numa sessão ao vivo, sorteado com o
    /// [`SESSION_ID_BIT`] (duas máquinas nunca fabricam o mesmo).
    pub fn mint_id(&mut self) -> ElementId {
        while let Some(r) = self.session.next() {
            let id = ElementId(r | SESSION_ID_BIT);
            if !self.elements.contains_key(&id) {
                return id;
            }
        }
        self.next_id += 1;
        ElementId(self.next_id)
    }

    /// Entra (`Some(semente)`, do sistema) ou sai (`None`) de uma sessão ao vivo (ADR-0181 §5).
    pub fn set_session(&mut self, seed: Option<u64>) {
        self.session = Session(seed.map(|seed| {
            Box::new(SessionState {
                rng: Lottery::new(Some(seed)),
                changes: BTreeMap::new(),
            })
        }));
    }

    /// O que mudou desde a última leitura, em sessão: cada id tocado com o estado ANTES da 1.ª
    /// mudança (`None` = nasceu). O estado de agora é [`Self::get_any`]. Fora de sessão, nada.
    pub fn take_changes(&mut self) -> Vec<(ElementId, Option<Element>)> {
        self.session
            .0
            .as_mut()
            .map(|s| std::mem::take(&mut s.changes).into_iter().collect())
            .unwrap_or_default()
    }

    /// A `version`/`nonce` que a sessão ao vivo deriva dos carimbos (ADR-0181 §5) — não é uma
    /// mudança (não passa pelas operações nem pelo caderno).
    pub fn set_mark(&mut self, id: ElementId, version: u64, nonce: u32) {
        if let Some(el) = self.elements.get_mut(&id) {
            (el.version, el.nonce) = (version, nonce);
        }
    }
    /// Está numa sessão ao vivo?
    #[must_use]
    pub fn in_session(&self) -> bool {
        self.session.0.is_some()
    }

    /// ⭐ A porta ÚNICA das chaves novas de z e de ordem entre irmãos: a chave entre `lo` e `hi`, com
    /// a cauda desta sessão se está ao vivo (duas pessoas a criar no mesmo sítio ao mesmo tempo não
    /// empatam — `colab_oracle` c05, c06).
    #[must_use]
    pub fn key_between(&self, lo: Option<&FracKey>, hi: Option<&FracKey>) -> FracKey {
        let k = FracKey::between(lo, hi);
        match self.session.next() {
            Some(salt) => k.salted(salt, hi),
            None => k,
        }
    }
}
