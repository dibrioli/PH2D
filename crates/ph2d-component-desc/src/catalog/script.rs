//! **O script do utilizador** — um tipo só, e uma armadilha medida ao lado dele.
//!
//! ⛔ **`register_script_components` NÃO é chamado no boot.** O `init.rs` chama quatro dos
//! cinco registradores (ecs · render · physics · field) e **não** o do script — medido em
//! 2026-08-24, e já era assim quando a auditoria de 21/08 o registou como a ambiguidade §8.1
//! (*"não determinado se é decisão ou esquecimento"*).
//!
//! **A consequência é dura:** um componente não registado é **descartado em silêncio** pelo
//! `WorldSnapshot` — logo o `LuauScript` de uma entidade **não é salvo nem desfeito**. Pôr um
//! `LuauScript` num objeto hoje é escrever num campo que desaparece ao fechar o projeto.
//!
//! ⚠️ **Por isso ele é [`crate::Attach::Machinery`] AQUI, e a declaração é uma cerca, não uma
//! opinião:** oferecê-lo na paleta do `+` daria ao artista um componente que se anexa, se vê,
//! e evapora — que é precisamente o *"pior que um botão ausente: o artista conclui que
//! gravou"* que esta linha existe para não repetir.
//!
//! ⚠️ **A cura não é desta fase e não é uma linha:** chamar o registador põe o `LuauScript` no
//! snapshot, o que MOVE o formato do projeto (mais um blob por entidade com script) — é um
//! degrau de `PROJECT_SCHEMA` com a migração ao lado, e uma decisão sobre um subsistema
//! (`ScriptHost`) que hoje corre um script *placeholder* e nunca recebe `provide_read`.
//! Quando isso for resolvido, esta linha vira `Authored` com `applies_to: ANY` — e é o censo
//! da shell que vai reclamar se o registador aparecer sem a declaração mudar.

use crate::{ComponentCategory as C, ComponentDesc as D};

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[D::machinery(
    "ph2d::script::LuauScript",
    "Script",
    C::Scripting,
)];
