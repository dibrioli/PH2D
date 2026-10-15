//! `LuauScript` Component — the canonical way to attach gameplay
//! script behavior to a SimWorld entity (ADR-0025 §"Gameplay script
//! is a Component, not a class").
//!
//! # A forma v3 (ADR-0188 §2, 2026-10-15) — um SCRIPT DO PROJETO e os números que o artista PÔS
//!
//! O `source` aponta um [`ScriptId`] da [`ScriptLibrary`] do projeto (o texto viaja DENTRO do
//! `.ph2dproj`); vários objectos partilham um script. O v2 guardava o caminho de um ficheiro solto
//! (mover o ficheiro partia a ligação, e o projeto não viajava inteiro); o v1 (`bytecode` +
//! `lateral_key`) nunca foi gravado. O porquê de cada forma: `docs/Components/13_plano_script_properties.md`
//! §1 e o ADR-0188.
//!
//! ⚠️ **O `own` guarda só o que o artista PÔS** (a divergência D1 do oráculo): um valor igual ao
//! default continua próprio quando o default muda. A lei que o lê é a [`crate::props::resolve`].
//!
//! ⚠️ **CONFIG, nunca vivo.** A tabela `self` de cada objecto vive na VM
//! ([`crate::scene::SceneScripts`]) e **rebobinar é renascer** — um contador aqui dentro faria cada
//! quadro com entrada virar um passo de undo (a lei que o `Timer` pagou).

use std::collections::BTreeMap;

use bevy_ecs::component::Component;
use bevy_ecs::resource::Resource;
use ph2d_ecs::SimComponent;
use ph2d_script_doc::{ScriptId, ScriptLibrary};
use serde::{Deserialize, Serialize};

use crate::props::ScriptValue;

/// **Que script um objecto corre** (v3, ADR-0188 §2).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptSource {
    /// Sem script — não corre (um nada não é um script vazio).
    #[default]
    None,
    /// Um script da biblioteca do projeto.
    Library(ScriptId),
    /// O caminho de um ficheiro que o load não conseguiu importar — o cartão RECUSA-o em voz alta;
    /// nunca corre vazio.
    Unimported(String),
}

/// **Um script num objecto.**
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LuauScript {
    /// O script que ele corre.
    pub source: ScriptSource,
    /// Os números que o artista PÔS neste objecto, por nome.
    ///
    /// ⚠️ **`BTreeMap`** pela espinha do determinismo (a captura é a unidade do undo e do save); a
    /// ORDEM do painel vem das declarações do script, nunca daqui.
    pub own: BTreeMap<String, ScriptValue>,
}

impl LuauScript {
    /// Schema version. **3** (ADR-0188): `source` aponta a biblioteca do projeto; o v2 guardava o
    /// caminho do ficheiro ([`LuauScriptV2`], migrado pelo load do projeto).
    pub const VERSION: u32 = 3;

    /// Um script da biblioteca, sem números próprios.
    #[must_use]
    pub fn library(id: ScriptId) -> Self {
        Self {
            source: ScriptSource::Library(id),
            own: BTreeMap::new(),
        }
    }
}

impl SimComponent for LuauScript {}

/// ⛔ **Congelado** — a forma v2 (`source` = caminho; vazio = sem script), lida só pelo degrau
/// [`crate::project_scripts::migrate_v2_blobs`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LuauScriptV2 {
    pub source: String,
    pub own: BTreeMap<String, ScriptValue>,
}

impl From<LuauScriptV2> for LuauScript {
    fn from(v2: LuauScriptV2) -> Self {
        let source = if v2.source.trim().is_empty() {
            ScriptSource::None
        } else {
            ScriptSource::Unimported(v2.source)
        };
        Self {
            source,
            own: v2.own,
        }
    }
}

/// ⭐ **Os scripts do projeto, como recurso do mundo** — a ponte, o Inspector e o save leem-nos daqui
/// (ADR-0075: desacoplar por recursos). ⚠️ Não é componente: o `WorldSnapshot` (undo, save do
/// `ProjectState`) não o vê — o texto tem histórico próprio, e viaja no `ProjectFile.scripts`.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectScripts(pub ScriptLibrary);

/// ⭐ **Os scripts que o artista pediu para EDITAR** (o «Edit script» do cartão, CodeEditor E3): o
/// Inspector escreve, a família do editor (`ph2d-app-code`) drena-os por quadro e abre a aba.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptEditRequests(pub Vec<ScriptId>);

impl std::ops::Deref for ProjectScripts {
    type Target = ScriptLibrary;
    fn deref(&self) -> &ScriptLibrary {
        &self.0
    }
}

impl std::ops::DerefMut for ProjectScripts {
    fn deref_mut(&mut self) -> &mut ScriptLibrary {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_componente_viaja_no_fio_e_nao_carrega_bits_de_entidade() {
        let mut s = LuauScript::library(ScriptId(7));
        s.own.insert("speed".into(), ScriptValue::Number(9.0));
        s.own.insert("label".into(), ScriptValue::Text("oi".into()));
        let bytes = postcard::to_allocvec(&s).expect("serializa");
        let back: LuauScript = postcard::from_bytes(&bytes).expect("volta");
        assert_eq!(back, s);
        // ⛔ A cerca do §1 do plano, escrita como FORMA: dois objectos com o mesmo script e os mesmos
        // números têm os MESMOS bytes — nada neles depende de quem os carrega.
        let outro = s.clone();
        assert_eq!(postcard::to_allocvec(&outro).expect("serializa"), bytes);
    }

    #[test]
    fn o_default_e_sem_script() {
        let s = LuauScript::default();
        assert!(s.source == ScriptSource::None && s.own.is_empty());
    }

    /// O v2: caminho vazio = sem script; caminho = por importar. Os números ficam.
    #[test]
    fn um_v2_vira_v3_sem_perder_os_numeros() {
        let mut own = BTreeMap::new();
        own.insert("speed".to_owned(), ScriptValue::Number(2.0));
        let v3 = LuauScript::from(LuauScriptV2 {
            source: "/a/bob.luau".into(),
            own: own.clone(),
        });
        assert_eq!(v3.source, ScriptSource::Unimported("/a/bob.luau".into()));
        assert_eq!(v3.own, own);
        let vazio = LuauScript::from(LuauScriptV2 {
            source: "  ".into(),
            own: BTreeMap::new(),
        });
        assert_eq!(vazio.source, ScriptSource::None);
    }
}
