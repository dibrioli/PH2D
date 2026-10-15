//! ⭐ **A RECARGA** (CodeEditor Onda 1, E6): o texto novo de um script troca as funções e mantém
//! cada `self`; o que não compila é RECUSADO e **a última versão boa continua a correr**.
//!
//! - Um script que nunca compilou não tem versão boa — ninguém o corre.
//! - Cada recarga (de um script que já tinha sido lido) deixa UM registo na fila
//!   ([`SceneScripts::drain_reloads`]) — o selo «Reloaded · N objects» ou a recusa.

use mlua::Lua;
use ph2d_script_doc::{ScriptId, ScriptLibrary};

use super::{Deadline, SceneScripts, ScriptInfo, Slot, discard_queues};
use crate::diagnostic::ScriptDiagnostic;
use crate::module::load_module;

/// **Como acabou uma recarga.**
#[derive(Clone, Debug, PartialEq)]
pub enum ReloadOutcome {
    /// O texto novo corre.
    Reloaded,
    /// O texto novo não compilou — os objectos continuam na versão anterior, se havia.
    Refused(ScriptDiagnostic),
}

/// ⭐ **Uma recarga** — o que o selo diz.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptReload {
    pub script: ScriptId,
    pub outcome: ReloadOutcome,
    /// Quantos objectos apontam o script (no `sync` que recarregou).
    pub objects: usize,
}

impl SceneScripts {
    /// Recarrega `id` se o texto (ou o nome) mudou, ou se foi pedido (HR-16).
    pub(super) fn refresh(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        lib: &ScriptLibrary,
        id: ScriptId,
        users: usize,
    ) {
        let slot = self.slots.entry(id).or_insert_with(Slot::empty);
        slot.users = users;
        let Some(doc) = lib.get(id) else {
            slot.hash = None;
            slot.module = None;
            slot.info = ScriptInfo::Missing(format!("no script #{} in this project", id.0));
            return;
        };
        let hash = *blake3::hash(doc.text.as_bytes()).as_bytes();
        if !slot.forced && slot.hash == Some(hash) && slot.name == doc.name {
            return;
        }
        // Já lido antes (bom ou partido) ⇒ é uma RECARGA; a 1.ª leitura não leva selo.
        let reload = slot.hash.is_some();
        slot.forced = false;
        slot.hash = Some(hash);
        slot.name.clone_from(&doc.name);
        // ⚠️ O TOPO também corre sob o prazo: um laço sem saída fora de qualquer função congelaria
        // o app no instante em que o artista escolhe o script.
        let loaded = crate::console::speaking(lua, id, None, || {
            deadline.within(|| load_module(lua, &doc.name, &doc.text))
        });
        // ⚠️ O topo de um script pode ter escrito ou emitido — isso não é de objecto nenhum.
        discard_queues(lua);
        let outcome = match loaded {
            Ok(m) => {
                slot.info = ScriptInfo::Ready(m.decls.clone());
                slot.module = Some(m);
                slot.chunk.clone_from(&doc.name);
                // Recarregar não renasce — mas um script partido volta a tentar.
                for inst in self.instances.values_mut().filter(|i| i.script == id) {
                    inst.failed = None;
                    inst.chunk.clone_from(&doc.name);
                }
                ReloadOutcome::Reloaded
            }
            Err(d) => {
                // ⭐ O `module` NÃO sai: a última versão boa continua (catálogo #22).
                if let Some(c) = crate::console::of(lua) {
                    c.diagnostic(id, None, &d, true);
                }
                slot.info = ScriptInfo::Broken(d.clone());
                ReloadOutcome::Refused(d)
            }
        };
        if reload {
            // Um por script: a fila não cresce com gravações que ninguém drenou.
            self.reloads.insert(
                id,
                ScriptReload {
                    script: id,
                    outcome,
                    objects: users,
                },
            );
        }
    }

    /// ⭐ As recargas desde a última drenagem — uma por script (a mais recente), por `ScriptId`.
    pub fn drain_reloads(&mut self) -> Vec<ScriptReload> {
        std::mem::take(&mut self.reloads).into_values().collect()
    }

    /// Quantos objectos apontam `id` (no último `sync`).
    #[must_use]
    pub fn objects_using(&self, id: ScriptId) -> usize {
        self.slots.get(&id).map_or(0, |s| s.users)
    }

    /// ⭐ O texto actual está partido MAS os objectos correm a versão anterior — o que distingue
    /// «quebrado e a correr» de «quebrado e parado» (ambos [`ScriptInfo::Broken`]).
    #[must_use]
    pub fn runs_previous(&self, id: ScriptId) -> bool {
        self.slots
            .get(&id)
            .is_some_and(|s| matches!(s.info, ScriptInfo::Broken(_)) && s.module.is_some())
    }

    /// Recarrega `id` no próximo `sync` mesmo com o texto igual (o editor que grava sem mudar
    /// nada ainda mostra o selo). Sem efeito num script que nenhum objecto aponta.
    pub fn request_reload(&mut self, id: ScriptId) {
        if let Some(s) = self.slots.get_mut(&id) {
            s.forced = true;
        }
    }
}
