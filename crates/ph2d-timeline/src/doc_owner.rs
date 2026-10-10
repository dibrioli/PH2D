//! ⭐⭐ **O DONO de cada clip** (v19, doc ObjectsAndComponents 01 §6, escolha 9.1 = A).
//!
//! O componente Animação de um objecto **não é um componente do mundo**: é o conjunto dos clips
//! deste documento que o nomeiam. Uma verdade só — um marcador no ECS ao lado dos clips teria
//! duas pilhas de undo a discordar (o Ctrl+Z do canvas tirava o marcador e deixava os clips).
//!
//! A chave de VISTA é um `u64`: `0` = a Cena (o `0` nunca é uma entidade bevy, a mesma razão por
//! que é o sentinela das bindings), outro valor = os bits do objecto dono. Um dono `missing`
//! (apagado, ou à espera da cura do load) não pertence a vista nenhuma — o clip fica dormente e
//! volta com o objecto, exactamente como a track de uma binding.

use super::*;
use crate::binding::WireId;

/// O dono de um clip: a identidade gravada e os bits desta sessão — o mesmo par do
/// [`TargetBinding`], com a mesma vida (carimbo no save, cura por `StableId`, purga).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipOwner {
    /// O `StableId` do objecto (gravado). NULL até ao primeiro carimbo.
    pub wire_id: WireId,
    /// Os bits vivos (sessão; `0` = por resolver).
    #[serde(skip)]
    pub entity: u64,
    /// `true` quando o objecto não resolve nesta sessão.
    #[serde(skip)]
    pub missing: bool,
}

impl ClipOwner {
    /// Um dono vivo, ainda sem identidade gravada.
    #[must_use]
    pub const fn live(entity: u64) -> Self {
        Self {
            wire_id: WireId::NULL,
            entity,
            missing: false,
        }
    }
}

/// A vista da Cena.
pub const SCENE: u64 = 0;

/// O dono gravável da chave de vista `owner` (`SCENE` ⇒ `None`).
pub(super) fn scope_of(owner: u64) -> Option<ClipOwner> {
    (owner != SCENE).then(|| ClipOwner::live(owner))
}

/// `a` e `b` são a MESMA animação — o escopo dos nomes de clip. Pela identidade gravada quando as
/// duas a têm (um dono à espera da cura do load ainda não tem bits), senão pelos bits.
pub(super) fn same_animation(a: Option<ClipOwner>, b: Option<ClipOwner>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) if x.wire_id != WireId::NULL && y.wire_id != WireId::NULL => {
            x.wire_id == y.wire_id
        }
        (Some(x), Some(y)) => x.entity == y.entity,
        _ => false,
    }
}

impl TimelineDoc {
    /// O clip `index` pertence à animação de `owner` (`SCENE` = a da Cena)?
    #[must_use]
    pub fn clip_belongs_to(&self, index: usize, owner: u64) -> bool {
        self.clips.get(index).is_some_and(|c| match c.owner {
            None => owner == SCENE,
            Some(o) => owner != SCENE && !o.missing && o.entity == owner,
        })
    }

    /// Os índices dos clips da animação de `owner`, por ordem.
    pub fn clips_of(&self, owner: u64) -> impl Iterator<Item = usize> + '_ {
        (0..self.clips.len()).filter(move |&i| self.clip_belongs_to(i, owner))
    }

    /// ⭐ **O clip chamado `name` na animação de `owner`** — a porta de quem guarda um clip pelo
    /// NOME (o osso inteligente). O nome é único por animação, nunca no documento: um dono sem
    /// esse clip responde `None`, e NUNCA o homónimo de outro dono (onda 4, item 2).
    #[must_use]
    pub fn clip_named(&self, owner: u64, name: &str) -> Option<usize> {
        self.clips_of(owner).find(|&i| self.clips[i].name == name)
    }

    /// `true` se o objecto `entity` tem o componente Animação (é dono de ≥ 1 clip vivo).
    #[must_use]
    pub fn has_animation(&self, entity: u64) -> bool {
        entity != SCENE && self.clips_of(entity).next().is_some()
    }

    /// ⭐ **A animação onde `e` vive**: a do primeiro da cadeia `ChildOf` (ele incluído) que tenha
    /// uma, ou a da Cena. A porta de quem resolve um clip pelo NOME a partir de um objecto (o osso
    /// inteligente, a guarda do reset). Pára num ciclo em vez de num número escolhido.
    #[must_use]
    pub fn animation_of(&self, world: &bevy_ecs::world::World, e: ph2d_ecs::Entity) -> u64 {
        let mut visto: Vec<ph2d_ecs::Entity> = Vec::new();
        let mut cur = Some(e);
        while let Some(e) = cur.filter(|e| !visto.contains(e)) {
            if self.has_animation(e.to_bits()) {
                return e.to_bits();
            }
            visto.push(e);
            cur = world
                .get::<ph2d_ecs::ChildOf>(e)
                .map(ph2d_ecs::ChildOf::parent);
        }
        SCENE
    }

    /// Os objectos vivos que têm Animação (com repetições — um por clip).
    pub fn animation_owners(&self) -> impl Iterator<Item = u64> + '_ {
        self.clips
            .iter()
            .filter_map(|c| c.owner.filter(|o| !o.missing && o.entity != SCENE))
            .map(|o| o.entity)
    }

    /// Os donos de todos os clips, para a vida da identidade (`persist.rs`).
    pub fn clip_owners_mut(&mut self) -> impl Iterator<Item = &mut ClipOwner> {
        self.clips.iter_mut().filter_map(|c| c.owner.as_mut())
    }

    /// **Acrescenta um clip à animação de `owner`** e devolve o índice — `None` quando a
    /// animação já tem [`MAX_CLIPS`] (o recurso é o seletor, que mostra UMA animação) ou o
    /// documento chegou ao índice de `u16` que um strip guarda.
    pub fn add_clip_for(&mut self, name: String, owner: u64) -> Option<usize> {
        self.clip_room(owner).ok()?;
        Some(self.push_clip(name, scope_of(owner)))
    }

    /// ⭐ **Cabe mais um clip na animação de `owner`?** — a porta única do tecto, que criar e
    /// copiar consultam e que quem aplica as intenções lê para DIZER porque não (onda 4, item 4).
    pub fn clip_room(&self, owner: u64) -> Result<(), crate::ClipRefusal> {
        if self.clips_of(owner).count() >= MAX_CLIPS {
            Err(crate::ClipRefusal::AnimationFull)
        } else if self.clips.len() > usize::from(u16::MAX) {
            Err(crate::ClipRefusal::DocumentFull)
        } else {
            Ok(())
        }
    }

    /// **Tira o componente Animação de `owner`**: todos os clips dele saem (e os strips que os
    /// tocavam). Devolve quantos. A Cena não se tira.
    pub fn remove_animation(&mut self, owner: u64) -> usize {
        if owner == SCENE {
            return 0;
        }
        let mine: Vec<usize> = self.clips_of(owner).collect();
        mine.iter().rev().filter(|&&i| self.remove_clip(i)).count()
    }

    /// Purga os clips cujo dono `dead` responde `true` — a metade de clips da purga das
    /// bindings (`ph2d-timeline-persist::upkeep`). Devolve quantos saíram.
    pub fn purge_dead_owners(&mut self, dead: impl Fn(&ClipOwner) -> bool) -> usize {
        let doomed: Vec<usize> = (0..self.clips.len())
            .filter(|&i| self.clips[i].owner.as_ref().is_some_and(&dead))
            .collect();
        doomed
            .iter()
            .rev()
            .filter(|&&i| self.remove_clip(i))
            .count()
    }

    /// **A cópia de um objecto leva a animação dele** (escolha do dono, 08/10: cópia PRÓPRIA):
    /// cada clip de `from` volta como clip de `to`, e cada track que anima um objecto da cópia
    /// (`remap` responde os bits novos) passa a animar o novo, por uma binding nova com os
    /// mesmos ajustes. Track de objecto de fora da cópia continua no alvo original. Devolve
    /// quantos clips se copiaram.
    pub fn copy_animation(
        &mut self,
        from: u64,
        to: u64,
        remap: impl Fn(u64) -> Option<u64>,
    ) -> usize {
        let src: Vec<usize> = self.clips_of(from).collect();
        let mut alvo: BTreeMap<AnimTarget, AnimTarget> = BTreeMap::new();
        let mut copied = 0;
        for i in src {
            let Some(j) = self.duplicate_clip(i) else {
                break;
            };
            // O dono primeiro, o nome depois: a unicidade é a da animação de DESTINO, logo a
            // cópia de «andar» continua «andar» (onda 4, item 2).
            self.clips[j].owner = Some(ClipOwner::live(to));
            self.clips[j].name =
                self.unique_clip_name(&self.clips[i].name.clone(), Some(j), self.clips[j].owner);
            let targets: Vec<AnimTarget> = self.clips[j]
                .clip
                .tracks()
                .iter()
                .map(|(t, _)| *t)
                .collect();
            for t in targets {
                let novo = match alvo.get(&t) {
                    Some(n) => *n,
                    None => {
                        let Some(n) = self.rebind_for_copy(t, &remap) else {
                            continue;
                        };
                        alvo.insert(t, n);
                        n
                    }
                };
                let c = &mut self.clips[j];
                if let Some(track) = c.clip.track(t).cloned() {
                    c.clip.remove_track(t);
                    c.clip.push(novo, track);
                }
                if let Some(e) = c.expr.remove(&t) {
                    c.expr.insert(novo, e);
                }
                if let Some(p) = c.paths.remove(&t) {
                    c.paths.insert(novo, p);
                }
            }
            copied += 1;
        }
        copied
    }

    /// Uma binding nova, igual à de `target`, sobre o objecto que `remap` dá — `None` quando o
    /// objecto da binding não está na cópia.
    fn rebind_for_copy(
        &mut self,
        target: AnimTarget,
        remap: &impl Fn(u64) -> Option<u64>,
    ) -> Option<AnimTarget> {
        let b = self.binding(target)?;
        let entity = remap(b.entity)?;
        let mut nova = b.clone();
        nova.target = AnimTarget::new(self.next_target);
        self.next_target += 1;
        nova.entity = entity;
        nova.wire_id = WireId::NULL;
        nova.missing = false;
        let t = nova.target;
        self.bindings.push(nova);
        Some(t)
    }
}

impl TimelineDoc {
    /// A chave de vista do clip `index`: os bits do dono, ou [`SCENE`].
    pub(crate) fn view_key(&self, index: usize) -> u64 {
        self.clips
            .get(index)
            .and_then(|c| c.owner)
            .map_or(SCENE, |o| o.entity)
    }
}
