//! **O aviso de imagem trocada** ([ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)): a
//! imagem nova não trouxe corte, e o corte `grid` e as animações foram feitos para a de antes.
//!
//! Componente, para o aviso entrar e sair pelo desfazer como a troca que o causou; irmão do tocador pelo postcard
//! posicional. Refazer o corte à mão também o cala: o aviso só vale enquanto a grelha é esta.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// O corte que ficou da imagem de antes: `[colunas, linhas]`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteCutNotice {
    pub grid: [u32; 2],
}

impl SimComponent for SpriteCutNotice {}
