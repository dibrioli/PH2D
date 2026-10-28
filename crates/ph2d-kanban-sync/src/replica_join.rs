//! Abrir, entrar, voltar e partilhar a meio — e os ids sorteados da sessão.

use ph2d_board_sync::PeerId;
use ph2d_kanban_model::{Kanban, KanbanId, KanbanSet};

use super::KanbanReplica;
use crate::msg::{KSnapshot, KWrite};

/// O bit dos ids sorteados numa sessão (o mesmo do modelo): nunca colidem com os do contador local.
const SESSION_ID_BIT: u64 = 1 << 63;

/// SplitMix64 — sorteia sem estado partilhado (a colisão entre dois pares é ~1e-8 a um milhão de
/// registos, ADR-0181 §5).
fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn bytes_of(k: &Kanban, wire: KanbanId) -> Vec<u8> {
    let mut k = k.clone();
    k.id = wire;
    postcard::to_allocvec(&k).expect("postcard escreve o Kanban")
}

impl KanbanReplica {
    /// Abre uma sessão sobre os Kanbans `shared` do meu projecto (o anfitrião: os ids da sessão são
    /// os meus). `seed` (do sistema) sorteia os ids novos.
    #[must_use]
    pub fn host(peer: PeerId, seed: u64, set: &KanbanSet, shared: &[KanbanId]) -> Self {
        let mut r = Self::blank(peer, seed, true);
        for id in shared {
            if let Some(k) = set.get(*id) {
                r.map.insert(*id, *id);
                r.shadow.insert(*id, k.clone());
                // Os de antes vão a quem entra com o retrato (`all_files`), não a cada mudança.
                r.files.extend(crate::table::files_of(k));
            }
        }
        r
    }

    /// Entra a partir do retrato: cada Kanban da sessão entra no MEU projecto com um id meu (o meu
    /// Kanban 1 não é o 1 do anfitrião). Devolve os ids locais, pela ordem do retrato.
    ///
    /// # Errors
    /// Um retrato corrompido.
    pub fn join(
        peer: PeerId,
        seed: u64,
        snap: &KSnapshot,
        set: &mut KanbanSet,
    ) -> Result<(Self, Vec<KanbanId>), String> {
        let mut r = Self::blank(peer, seed, false);
        let ids = r.adopt(snap, set)?;
        Ok((r, ids))
    }

    /// Volta depois de uma queda: os Kanbans passam a ser os do retrato e as minhas escritas
    /// reaplicam-se por cima e reenviam-se (as que lá chegaram são idempotentes).
    ///
    /// # Errors
    /// Um retrato corrompido.
    pub fn rejoin(&mut self, set: &mut KanbanSet, snap: &KSnapshot) -> Result<(), String> {
        let _ = self.local(set);
        let mine = self.mine();
        let fresh = Self {
            outbox: std::mem::take(&mut self.outbox),
            clock: self.clock,
            minted: self.minted,
            map: std::mem::take(&mut self.map),
            files: std::mem::take(&mut self.files),
            ..Self::blank(self.peer, self.seed, self.host)
        };
        *self = fresh;
        self.adopt(snap, set)?;
        self.receive(set, &mine);
        self.outgoing.extend(mine);
        Ok(())
    }

    fn adopt(&mut self, snap: &KSnapshot, set: &mut KanbanSet) -> Result<Vec<KanbanId>, String> {
        let mut ids = Vec::new();
        for (wire, bytes) in &snap.kanbans {
            let k: Kanban = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
            ids.push(self.adopt_kanban(set, *wire, k));
        }
        self.clock.n = self.clock.n.max(snap.clock);
        self.stamps.extend(snap.stamps.iter().cloned());
        self.elems.extend(snap.elems.iter().cloned());
        self.tops.extend(snap.tops.iter().cloned());
        Ok(ids)
    }

    /// Põe o Kanban `wire` no meu projecto (no id que já lhe dei, ou num novo).
    pub(crate) fn adopt_kanban(
        &mut self,
        set: &mut KanbanSet,
        wire: KanbanId,
        mut k: Kanban,
    ) -> KanbanId {
        let local = match self.map.get(&wire) {
            Some(l) => *l,
            None => set.create(&k.header.name),
        };
        k.id = local;
        self.map.insert(wire, local);
        self.shadow.insert(wire, k.clone());
        set.insert(k);
        local
    }

    /// O retrato para quem entra (chame [`Self::local`] antes: o que mudei vai aos outros, e o
    /// retrato leva-o já).
    #[must_use]
    pub fn snapshot(&self, set: &KanbanSet) -> KSnapshot {
        KSnapshot {
            kanbans: self
                .map
                .iter()
                .filter_map(|(w, l)| set.get(*l).map(|k| (*w, bytes_of(k, *w))))
                .collect(),
            stamps: self.stamps.iter().map(|(k, v)| (*k, *v)).collect(),
            elems: self
                .elems
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            tops: self.tops.iter().map(|(k, v)| (*k, *v)).collect(),
            clock: self.clock.n,
        }
    }

    /// Leva o meu Kanban `local` para a sessão a meio (um elemento novo no Quadro que o referencia).
    /// Devolve o id da sessão; a escrita sai no próximo [`Self::local`].
    pub fn share(&mut self, set: &KanbanSet, local: KanbanId) -> Option<KanbanId> {
        if let Some(w) = self.wire_id(local) {
            return Some(w);
        }
        let k = set.get(local)?;
        let wire = if self.host {
            local
        } else {
            KanbanId(self.mint())
        };
        self.map.insert(wire, local);
        self.shadow.insert(wire, k.clone());
        self.outgoing.push(KWrite::Shared {
            kanban: wire,
            bytes: bytes_of(k, wire),
        });
        let files = self.new_files(set, local);
        self.outgoing.extend(files);
        Some(wire)
    }

    /// ⭐ Um id NOVO para um registo criado durante a sessão (o bit alto ligado: nunca colide com o
    /// contador local nem com o de outro par). ⛔ Numa sessão, os registos novos usam ESTE, nunca o
    /// `Kanban::mint` (dois pares a criar ao mesmo tempo dariam o mesmo id).
    pub fn mint(&mut self) -> u64 {
        self.minted += 1;
        SESSION_ID_BIT | (splitmix(self.seed ^ splitmix(self.minted)) >> 1)
    }
}
