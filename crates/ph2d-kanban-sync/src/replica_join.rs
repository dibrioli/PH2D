//! Abrir, entrar, voltar e partilhar a meio — e os ids sorteados da sessão.

use ph2d_board_sync::PeerId;
use ph2d_kanban_model::{Kanban, KanbanId, KanbanSet, SESSION_ID_BIT, splitmix64};

use super::KanbanReplica;
use crate::msg::{KSnapshot, KWrite};

/// Os bytes de `k` para a sessão: sem os registos pessoais (W9 — a Inbox é só de quem a escreveu).
fn bytes_of(k: &Kanban, wire: KanbanId) -> Vec<u8> {
    let mut k = k.without_personal();
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
        self.clock.n = self.clock.n.max(snap.clock.min(ph2d_board_sync::MAX_STAMP));
        crate::snapshot::unpack(
            &snap.stamps,
            &mut self.stamps,
            &mut self.elems,
            &mut self.tops,
        );
        Ok(ids)
    }

    /// Põe o Kanban `wire` no meu projecto (no id que já lhe dei, ou num novo).
    pub(crate) fn adopt_kanban(
        &mut self,
        set: &mut KanbanSet,
        wire: KanbanId,
        mut k: Kanban,
    ) -> KanbanId {
        // W9 — a agenda da sessão é a MINHA agenda (uma por projecto); o que é pessoal fica o meu.
        let local = match self.map.get(&wire) {
            Some(l) => *l,
            None if wire == KanbanId::AGENDA => KanbanId::AGENDA,
            None => set.create(&k.header.name),
        };
        k.id = local;
        self.map.insert(wire, local);
        self.shadow.insert(wire, k.without_personal());
        // Os meus eventos juntam-se aos da sessão (o próximo `local` leva-os aos outros: o calendário
        // da sessão é a união); os pessoais ficam os meus.
        if local == KanbanId::AGENDA
            && let Some(mine) = set.get(local)
        {
            k.keep_personal_of(mine);
            for (id, e) in &mine.events {
                k.events.entry(*id).or_insert_with(|| e.clone());
            }
        }
        set.insert(k);
        self.arm(set);
        local
    }

    /// ⭐ Liga o sorteio dos ids em cada Kanban da sessão que ainda não o tem (entrar, hospedar, um
    /// Kanban reposto inteiro): todo `Kanban::mint` passa a dar ids que nenhum outro par fabrica.
    /// [`Self::local`] chama-o a cada imagem.
    pub fn arm(&self, set: &mut KanbanSet) {
        for (wire, local) in &self.map {
            if let Some(k) = set.doc_mut(*local)
                && !k.in_session()
            {
                k.set_session(Some(splitmix64(self.seed ^ splitmix64(wire.0))));
            }
        }
    }

    /// Sai da sessão: os Kanbans voltam ao contador local (os ids sorteados ficam, com o bit alto).
    pub fn disarm(&self, set: &mut KanbanSet) {
        for local in self.map.values() {
            if let Some(k) = set.doc_mut(*local) {
                k.set_session(None);
            }
        }
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
            stamps: crate::snapshot::pack(&self.stamps, &self.elems, &self.tops),
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

    /// Um id da sessão para um Kanban partilhado a meio (o bit alto ligado: nunca colide com o
    /// contador nem com o de outro par). Os REGISTOS usam o `Kanban::mint` de sempre: em sessão ele
    /// já sorteia ([`Self::arm`]).
    pub fn mint(&mut self) -> u64 {
        self.minted += 1;
        SESSION_ID_BIT | (splitmix64(self.seed ^ splitmix64(self.minted)) >> 1)
    }
}
