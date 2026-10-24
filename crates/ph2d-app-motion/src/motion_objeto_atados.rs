//! ⭐ **O passo do Motion preso à VIDA de um objecto Macro** (W18, aplicar uma macro,
//! `docs/Macros/16_W18_plano.md` §2.7): aplicar muda o mundo (o objecto Macro, o filho Motion) e o
//! documento do Motion, que vive fora do undo global. O undo global que apaga o objecto Macro desfaz
//! o passo do Motion no MESMO quadro, e o refazer que o devolve refá-lo — um Ctrl+Z, um Ctrl+Y.
//!
//! O molde do cemitério ([`ph2d_ecs::graveyard`]): só anda se o passo ainda for o topo da fila do
//! Motion (editou-se depois ⇒ nada anda, são dois Ctrl+Z como antes — nunca meio passo).

use ph2d_ecs::{Macro, StableId, With, World};
use ph2d_motion_doc::{MotionDoc, MotionHistory};
use std::collections::BTreeSet;

/// Quantos passos atados se guardam: a profundidade das filas.
const PROFUNDIDADE: usize = ph2d_ecs::GRAVEYARD_DEPTH;

#[derive(Debug, Clone)]
struct Atado {
    /// O `StableId` do objecto Macro.
    vida: u64,
    antes: MotionDoc,
    depois: MotionDoc,
    vivo: bool,
}

/// Os passos do Motion atados à vida de um objecto Macro, do mais antigo ao mais recente.
#[derive(Debug, Default)]
pub struct Atados(Vec<Atado>);

impl Atados {
    /// O documento passou de `antes` a `doc` por um aplicar do objecto Macro `vida`: um passo da fila,
    /// preso à vida dele. Os passos seguintes do MESMO aplicar fundem-se no primeiro.
    pub fn ata(
        &mut self,
        history: &mut MotionHistory,
        vida: u64,
        antes: MotionDoc,
        doc: &MotionDoc,
    ) {
        if let Some(a) = self.0.last_mut()
            && a.vida == vida
            && a.vivo
            && a.depois == antes
            && history.top() == Some(&a.antes)
        {
            a.depois = doc.clone();
            return;
        }
        history.push_undo(antes.clone());
        if self.0.len() >= PROFUNDIDADE {
            self.0.remove(0);
        }
        self.0.push(Atado {
            vida,
            antes,
            depois: doc.clone(),
            vivo: true,
        });
    }

    /// **Por quadro**: o objecto Macro que morreu desfaz o passo dele, o que voltou refá-lo (se cada um
    /// ainda está no topo da sua fila). Devolve se o documento mudou.
    pub fn acompanha(
        &mut self,
        history: &mut MotionHistory,
        doc: &mut MotionDoc,
        world: &mut World,
    ) -> bool {
        if self.0.is_empty() {
            return false;
        }
        let vivas: BTreeSet<u64> = world
            .query_filtered::<&StableId, With<Macro>>()
            .iter(world)
            .map(|s| s.0)
            .collect();
        let mut mudou = false;
        for a in self
            .0
            .iter_mut()
            .rev()
            .filter(|a| a.vivo && !vivas.contains(&a.vida))
        {
            a.vivo = false;
            if *doc == a.depois && history.top() == Some(&a.antes) {
                *doc = history.undo(doc).unwrap_or_else(|| a.antes.clone());
                mudou = true;
            }
        }
        for a in self
            .0
            .iter_mut()
            .filter(|a| !a.vivo && vivas.contains(&a.vida))
        {
            a.vivo = true;
            if *doc == a.antes && history.top_redo() == Some(&a.depois) {
                *doc = history.redo(doc).unwrap_or_else(|| a.depois.clone());
                mudou = true;
            }
        }
        mudou
    }

    /// O Ctrl+Z (`redo = false`) ou Ctrl+Y da ferramenta Motion encontra no topo o passo de um aplicar?
    /// Então é do undo GLOBAL: ele tira (ou devolve) o objecto Macro e este passo vai junto.
    #[must_use]
    pub fn topo_e_atado(&self, history: &MotionHistory, doc: &MotionDoc, redo: bool) -> bool {
        self.0.iter().any(|a| {
            if redo {
                !a.vivo && *doc == a.antes && history.top_redo() == Some(&a.depois)
            } else {
                a.vivo && *doc == a.depois && history.top() == Some(&a.antes)
            }
        })
    }
}
