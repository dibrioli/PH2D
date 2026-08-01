//! **A LINHA DE CORTE** — o caminho que a tesoura usa como lâmina (plano 25 §7).
//!
//! Um marcador, e a ausência de campos é o desenho: a linha de corte **é um caminho como
//! qualquer outro** (a Pen a desenha, o Select a move, o Node a edita, a Hierarquia a nomeia,
//! o undo e o save a carregam). O que este componente acrescenta é só a RESPOSTA a uma
//! pergunta — *este caminho é lâmina ou é desenho?* — e é dela que caem as três consequências:
//!
//! 1. o painel oferece **Cut** / **Discard** quando ela existe;
//! 2. o overlay a desenha **hachurada com uma tesoura na ponta**, em vez de o render a
//!    desenhar como arte (uma lâmina não é obra: não herda cor nem espessura de traço, e não
//!    sai no export);
//! 3. o corte **nunca corta a si mesmo** — a lâmina é excluída dos alvos por este marcador, e
//!    não por uma lista de ids que alguém teria de manter.
//!
//! # Por que componente, e não um campo do `VecPath`
//!
//! Um campo no `VecPath` bumparia `VEC_SCENE_SCHEMA_VERSION` **e** `PROJECT_SCHEMA` (postcard é
//! posicional), e um bump **RECUSA todo projeto já salvo**. Um componente cunha blob-key própria
//! (`stable_type_id` do NOME) ⇒ **zero bump**, e documento antigo carrega inalterado. É o
//! precedente exato do [`crate::VecStrokeProfile`] (ADR-0148) e dos irmãos
//! [`crate::VecOffset`] / [`crate::VecTextPath`].
//!
//! ⚠️ **Sem o registro no `ComponentRegistry` este marcador é DESCARTADO pelo snapshot** — a
//! linha sobreviveria ao save como um caminho comum, sem fill nem stroke: invisível, inerte e
//! impossível de apagar pelo botão que existe para isso. O registro é o que o torna real.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Este caminho é uma linha de corte** (a lâmina), não desenho.
///
/// Marcador: a PRESENÇA é o booleano — o idioma do `Locked` e dos marcadores da física
/// (`Ccd`, `LockRotation`, `OneWayPlatform`). Um `bool` dentro de um componente teria dois
/// estados para a mesma coisa (ausente e `false`), e alguém acabaria a testar o errado.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VecCutPath;

impl SimComponent for VecCutPath {}
