//! [`FlipDoc`] — a **cena** Flip: a coleção de objetos Flip do projeto.
//!
//! Espelha o `VecScene` (que é a coleção de `VecPath`): há UMA `FlipDoc` no
//! projeto (guardada no `ProjectState`, undo/save), e cada [`FlipObject`] dentro
//! dela é uma entidade ECS na Hierarquia (via `ph2d_ecs::FlipObjectRef`,
//! ADR-0110). O struct que a `02_referencia §1` esboçou como `FlipDoc {layers,…}`
//! virou o [`FlipObject`]; `FlipDoc` foi promovido ao invólucro de cena — porque
//! `FlipObjectRef(u64)` + a ponte spawn-por-objeto (T0.8/T0.9) exigem múltiplos
//! objetos, cada um uma entidade.

use crate::ids::{DrawingId, FlipObjectId, LayerId};
use crate::object::FlipObject;
use ph2d_core::Playhead;
use serde::{Deserialize, Serialize};

/// Uma amostra da cena inteira num instante: por objeto, o desenho ativo de cada
/// camada.
pub type SceneSample = Vec<(FlipObjectId, Vec<(LayerId, Option<DrawingId>)>)>;

/// A cena Flip. `PartialEq` para o diff de undo detectar mudança real (só vira
/// passo de histórico se a cena mudou de fato — mesma disciplina do `VecScene`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlipDoc {
    /// Os objetos Flip. A ordem de z na cena é projeção da Hierarquia (a shell
    /// re-sincroniza), como no `VecScene`.
    objects: Vec<FlipObject>,
    /// Próximo `FlipObjectId` livre (ids estáveis).
    next_object_id: u64,
}

impl FlipDoc {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Todos os objetos.
    #[must_use]
    pub fn objects(&self) -> &[FlipObject] {
        &self.objects
    }

    /// Sem objetos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Cria um objeto vazio (sem camadas) e devolve seu id estável.
    pub fn push_object(&mut self, name: impl Into<String>) -> FlipObjectId {
        let id = FlipObjectId(self.next_object_id);
        self.next_object_id += 1;
        self.objects.push(FlipObject::new(id, name));
        id
    }

    #[must_use]
    pub fn object(&self, id: FlipObjectId) -> Option<&FlipObject> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn object_mut(&mut self, id: FlipObjectId) -> Option<&mut FlipObject> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    /// Remove o objeto `id`; `true` se existia (Delete no canvas / Hierarquia).
    pub fn remove_object(&mut self, id: FlipObjectId) -> bool {
        if let Some(i) = self.objects.iter().position(|o| o.id == id) {
            self.objects.remove(i);
            true
        } else {
            false
        }
    }

    /// Amostra a cena inteira no tempo do playhead (cada objeto usa o próprio FPS).
    #[must_use]
    pub fn sample(&self, playhead: &Playhead) -> SceneSample {
        self.objects
            .iter()
            .map(|o| (o.id, o.sample_at(playhead)))
            .collect()
    }

    /// Serializa (postcard), prefixada pela versão de schema
    /// ([`crate::FLIP_SCHEMA_VERSION`]).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&(crate::FLIP_SCHEMA_VERSION, self)).map_err(|e| e.to_string())
    }

    /// Desserializa uma cena salva por [`Self::to_bytes`]; rejeita schema alheio.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let (ver, doc): (u32, FlipDoc) = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
        if ver != crate::FLIP_SCHEMA_VERSION {
            return Err(format!(
                "FlipDoc schema {ver} != {} — recusado",
                crate::FLIP_SCHEMA_VERSION
            ));
        }
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Hold, KeyKind};

    #[test]
    fn push_and_remove_objects_mint_stable_ids() {
        let mut doc = FlipDoc::new();
        assert!(doc.is_empty());
        let a = doc.push_object("A");
        let b = doc.push_object("B");
        assert_ne!(a, b);
        assert_eq!(doc.objects().len(), 2);
        assert!(doc.remove_object(a));
        // O id de B não mudou (estável) após remover A.
        assert!(doc.object(b).is_some());
        assert!(doc.object(a).is_none());
        // Um objeto novo não reusa o id de A.
        let c = doc.push_object("C");
        assert_ne!(c, a);
        assert_ne!(c, b);
    }

    #[test]
    fn scene_sample_covers_each_object() {
        let mut doc = FlipDoc::new();
        let oa = doc.push_object("A");
        let la = {
            let obj = doc.object_mut(oa).unwrap();
            obj.fps = 24.0;
            let l = obj.add_layer("L");
            obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe);
            l
        };
        let ph = Playhead::new(1.0 / 24.0);
        let sample = doc.sample(&ph);
        assert_eq!(sample.len(), 1);
        assert_eq!(sample[0].0, oa);
        assert_eq!(sample[0].1, vec![(la, Some(DrawingId(0)))]);
    }
}
