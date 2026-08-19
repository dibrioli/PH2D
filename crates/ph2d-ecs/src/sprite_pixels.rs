//! `SpritePixels` — a entidade tem **pixels próprios**, e este é o nome durável deles.
//!
//! ## Por que ela existe
//!
//! `SpriteSource::Individual { texture_id }` guarda um **id de alocação da GPU** dentro de um
//! componente **persistido**. O `IndividualTextureStore` recomeça a numerar em `1` a cada
//! processo, então noutra sessão aquele id ou não existe (o `bind_group` devolve `None` e o
//! sprite **some**) ou pertence a outra textura (o sprite exibe **os pixels de outro**). É a
//! mesma doença que `Entity::to_bits()` tinha no undo: *referência durável entre objetos é o
//! NOME, nunca os bits*.
//!
//! Esta componente é o nome, e o nome é o **conteúdo**: um [`AssetId`] (blake3 dos pixels, HR-6),
//! o mesmo que o `AssetDb` cunha. Dois sprites com os mesmos pixels partilham uma entrada no
//! arquivo de graça, e não há contador de ids para manter. Como toda componente, ela viaja no
//! `WorldSnapshot`, logo o **undo** também a preserva sem custo.
//!
//! ⚠️ **Hash de conteúdo porque estes pixels são um SNAPSHOT imutável.** Uma folha *autorada*
//! (o hand-packed) muda a cada arrasto do artista, e um id de conteúdo obrigaria a re-carimbar
//! todo sprite a cada gesto — por isso ela virá com um id estável de **documento**, no espírito do
//! [`crate::PaintedDoc`]. São dois tempos de vida, não uma inconsistência.
//!
//! ## Por que NÃO é um terceiro remédio ad-hoc
//!
//! O [`crate::PaintedDoc`] nomeia um documento em **camadas** (não achatável) e o
//! [`crate::BakedForm`] nomeia **canais de G-buffer + rig de luz**. Nenhum dos dois é *pixels
//! chapados* — o caso base é que nunca existiu, e é ele que faltava debaixo dos dois. Toda
//! ferramenta de imagem (trim · bgremoval · make-square · padding · upscale · rasterize ·
//! equalize · painter) sai por um funil só (`commit_edited_texture`), e até aqui o que ela
//! produzia não sobrevivia a fechar o app.
//!
//! ## O que ela NÃO faz
//!
//! Não põe pixels no ECS. Os bytes vivem no `AssetDb` durante a sessão e no documento
//! `ph2d-sprite-sheet` dentro do arquivo; aqui viaja só a identidade, e a shell mantém o ciclo de
//! vida (pixels ⇒ id; sprite carregado ⇒ textura re-materializada ⇒ `source` re-apontado).

use bevy_ecs::component::Component;
use ph2d_asset::AssetId;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Identidade durável dos pixels próprios desta entidade — o blake3 do conteúdo.
///
/// Guarda um [`AssetId`] (e não um tipo do `ph2d-sprite-sheet`) para manter `ph2d-ecs` sem
/// dependência do formato de ARQUIVO: a seta aponta para a identidade do asset, que é fundação,
/// nunca para o documento que a grava. A shell conhece os dois; nenhum dos dois conhece o outro.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpritePixels(pub AssetId);

impl SimComponent for SpritePixels {}
