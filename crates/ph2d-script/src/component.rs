//! `LuauScript` Component — the canonical way to attach gameplay
//! script behavior to a SimWorld entity (ADR-0025 §"Gameplay script
//! is a Component, not a class").
//!
//! # ⭐⭐⭐ A forma de 2026-09-16 (TOP-20 #16) — um CAMINHO e os números que o artista PÔS
//!
//! Até aqui o componente guardava `bytecode: AssetId` e `lateral_key: u64`, e as duas escolhas
//! foram **medidas e retiradas** (`docs/Components/13_plano_script_properties.md` §1):
//!
//! - ⛔⛔ **`lateral_key = entity.to_bits() ^ hash`** punha **bits de alocação dentro dos bytes de um
//!   componente** — exactamente o que o `CLAUDE.md` §5 proíbe (*o undo respawna tudo com bits
//!   novos, e bits dentro dos bytes envenenam o próprio undo*). Só não mordeu porque o componente
//!   **nunca foi gravado**: o registador não era chamado no boot.
//! - ⛔ **`bytecode: AssetId`** nomeava um asset que **nenhum sítio do app produz** — não há
//!   compilador de Luau para o índice, nem sítio onde o artista escolha um.
//!
//! ⇒ **o mesmo contrato do `AudioSource2D`** (TOP-20 #4): o componente nomeia o FICHEIRO, e a
//! recarga por hash de conteúdo do `ScriptHost` (HR-16) é o que o torna vivo. As duas consequências
//! são as do som e estão declaradas lá: mover o ficheiro parte a ligação, e o projecto não embute o
//! script — a cura das duas é a mesma, pôr o tipo no índice de assets.
//!
//! ⚠️ **O `own` guarda só o que o artista PÔS** (a divergência D1 do oráculo): um valor igual ao
//! default continua próprio quando o default muda. A lei que o lê é a [`crate::props::resolve`].
//!
//! ⚠️ **CONFIG, nunca vivo.** A tabela `self` de cada objecto vive na VM
//! ([`crate::scene::SceneScripts`]) e **rebobinar é renascer** — um contador aqui dentro faria cada
//! quadro com entrada virar um passo de undo (a lei que o `Timer` pagou).

use std::collections::BTreeMap;

use bevy_ecs::component::Component;
use ph2d_ecs::SimComponent;
use serde::{Deserialize, Serialize};

use crate::props::ScriptValue;

/// **Um script num objecto.**
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LuauScript {
    /// O ficheiro `.luau`, como o artista o escolheu. **Vazio = sem script** (a lei do produtor:
    /// um componente sem nome não corre, em vez de correr um nada).
    pub source: String,
    /// Os números que o artista PÔS neste objecto, por nome.
    ///
    /// ⚠️ **`BTreeMap`** pela espinha do determinismo (a captura é a unidade do undo e do save); a
    /// ORDEM do painel vem das declarações do script, nunca daqui.
    pub own: BTreeMap<String, ScriptValue>,
}

impl LuauScript {
    /// Schema version. ⚠️ **2** desde o TOP-20 #16 — a forma `{ bytecode, lateral_key }` (v1)
    /// **nunca foi gravada** (o registador não corria no boot), então não há degrau a migrar dela.
    pub const VERSION: u32 = 2;

    /// Um script no ficheiro `source`, sem números próprios.
    #[must_use]
    pub fn at(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            own: BTreeMap::new(),
        }
    }
}

impl SimComponent for LuauScript {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_componente_viaja_no_fio_e_nao_carrega_bits_de_entidade() {
        let mut s = LuauScript::at("/tmp/bob.luau");
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
    fn o_default_e_um_script_vazio() {
        let s = LuauScript::default();
        assert!(s.source.is_empty() && s.own.is_empty());
    }
}
