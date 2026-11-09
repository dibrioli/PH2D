//! ⭐ **O que a aba de código pede à cena** (plano 13 Onda B, item 10; dono 09/11: «espiar a cena» e
//! «mapa na Bancada», os dois ligados por omissão). A aba ocupa o lugar do canvas, por isso o pairar
//! num objecto não tem canvas onde acender: ou a aba CEDE a área à cena enquanto o rato pára no nome
//! (espiar), ou a Bancada desenha um mapa pequeno com o contorno de cada objecto. Recursos ECS: a
//! família escreve o pedido, a shell (a única que sabe o contorno de qualquer objecto,
//! `hover_highlight.rs` `hover_outline_of`) responde.

use bevy_ecs::resource::Resource;

/// O objecto que o rato espia: os bits e o índice da cor (`ColorToken::layer`). Com ele, a aba
/// cede a área à cena e a shell contorna-o nessa cor, por cima da selecção.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScenePeek(pub Option<(u64, usize)>);

/// O mapa da Bancada: os objectos cujo contorno de MUNDO a aba pede (`wanted`), e a resposta da
/// shell (`got`, em anéis achatados), um quadro depois.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct SceneOutlines {
    pub wanted: Vec<u64>,
    pub got: Vec<(u64, Vec<Vec<[f32; 2]>>)>,
}
