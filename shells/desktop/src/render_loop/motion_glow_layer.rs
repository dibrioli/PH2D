//! **A CAMADA MOTION que o glow bright-passa** — a lista de instâncias que o passe
//! de isolamento re-renderiza no RT `Rgba16Float` (`present.rs`, Pass 1c).
//!
//! ## O bug que este módulo existe para curar
//!
//! Enio, 2026-08-20: *"Glow não funciona com shape"* — e depois, decidindo a cura:
//! *"tudo deve brilhar, não só shape, mas os objetos Sprite, Vector, Flip e mais
//! tarde os objetos 3d"*.
//!
//! O Motion desenha em **duas metades**, e depois do cook cada elemento cai numa:
//!
//! | metade | lista | quem desenha | quando |
//! |---|---|---|---|
//! | sprites | `pump.instances` | `SpriteRenderer` → `game_rt` | **antes** do tonemap (HDR) |
//! | vetor vivo | `pump.vector_instances` | `motion_shape_gen::encode` → cena Vello | **depois** (LDR) |
//!
//! O passe do glow lia só a primeira. Um `source.shape` — e um `source.object` de
//! VETOR ou de FLIP abaixo do limiar de LOD — emite `geometry_id` e cai na segunda,
//! então o bright-pass lia um RT em que aquele elemento nunca foi desenhado.
//!
//! ⚠️ **E o defeito era INTERMITENTE POR CONTAGEM:** a partição de LOD
//! (`motion_bridge_objects::apply_object_lod`, `LOD_COUNT = 16_000`) MOVE
//! para `instances` toda geometria carimbada acima do limiar que tenha tile — e
//! essas brilhavam. *A mesma forma não brilhava com 16 000 cópias e brilhava com
//! 16 001.* Este módulo mata o degrau: o tile entra na lista do glow **em qualquer
//! contagem**.
//!
//! ## Por que o TILE serve, e não é um remendo
//!
//! A rota «óbvia» seria rasterizar a metade vetorial em alta fidelidade num alvo
//! HDR. ⛔ **Medido: o Vello 0.8 não pode.** O `render_to_texture` dele escreve
//! numa *storage texture* `Rgba8Unorm` (`vello_pass.rs`: *"Vello requires Rgba8Unorm
//! + STORAGE_BINDING"*), então o RT `Rgba16Float` do glow está fora do alcance
//! dele — e passar por um intermediário LDR **perderia o HDR**, que é precisamente
//! onde o bloom vive.
//!
//! ⚠️ **A observação que torna isto barato: o halo NÃO PRECISA DE NITIDEZ.** A
//! primeira coisa que o passe faz com este RT é um bright-pass em meia resolução,
//! seguido de **seis** reduções de mip. Um tile de DPI fixo é indistinguível de uma
//! curva perfeita depois de duas dessas. O caminho CRISPO fica intocado para o
//! quadro visível; o que muda é só de onde o *bloom* tira a silhueta.
//!
//! ⚠️ **E o HDR sobrevive**, que é o que uma ponte por Vello não conseguiria:
//! [`vector_instance_as_tile`]
//! mantém o `tint` verbatim, o tile é branco, e o shader de sprite multiplica os
//! dois — um `tint` de `40` chega ao bright-pass como `40`.
//!
//! ## O que ainda não brilha, e onde está escrito
//!
//! Uma geometria viva **sem tile assado** não pode contribuir — não há de onde tirar
//! a silhueta. Hoje isso é o `source.shape`, cujo caminho paramétrico vive no
//! `shape_store` e nunca foi assado (os `source.object` de vetor e de Flip **são**
//! assados, por [`crate::motion_object_bake`]). É a segunda metade desta ordem, e
//! está registada no [`BUGS_motion_nodes.md`](../../../../docs/Motion%20Nodes/BUGS_motion_nodes.md)
//! como Bug #2.

use super::motion_bridge::vector_instance_as_tile;
use crate::motion_object_bake::ObjectBake;
use ph2d_eval_motion::VectorInstance;
use ph2d_render::RenderInstance;

/// A lista que o passe de isolamento do glow desenha: os sprites, mais toda
/// geometria viva que TENHA um tile assado, convertida em quad.
///
/// ⚠️ **Não é um `apply_*` — não move nada.** A partição de LOD move (ela decide o
/// que o artista VÊ); esta função só **deriva** uma segunda vista para o
/// bright-pass. As duas listas de origem ficam intactas, e é isso que mantém o
/// caminho crispo do quadro visível byte-a-byte como estava.
///
/// ⚠️ **Ordem: sprites primeiro, tiles depois.** O passe de isolamento reordena por
/// z (`sort_render_order`), então a ordem daqui não decide a aparência — mas ela
/// decide a de um `RenderInstance` empatado, e um append determinístico é o que
/// mantém o RT reproduzível entre quadros.
#[must_use]
pub(crate) fn layer_instances(
    sprites: &[RenderInstance],
    vectors: &[VectorInstance],
    bake: &ObjectBake,
) -> Vec<RenderInstance> {
    let mut out = Vec::with_capacity(sprites.len() + vectors.len());
    out.extend_from_slice(sprites);
    for vi in vectors {
        if let Some(texture_id) = bake.tile_texture_for_gid(vi.geometry_id) {
            out.push(vector_instance_as_tile(vi, texture_id));
        }
    }
    out
}

/// Quantas geometrias vivas do quadro NÃO têm tile — as que o glow não alcança.
///
/// ⚠️ **Uma sonda, não um portão.** Ela existe para o gate afirmar o buraco em vez
/// de o descrever em prosa: enquanto o `source.shape` não for assado este número é
/// maior que zero, e quando alguém o assar o gate que o fixa fica vermelho e obriga
/// a reconferir esta nota — que é exactamente o que impede a nota de envelhecer.
#[must_use]
#[cfg(test)]
pub(crate) fn unreachable_geometries(vectors: &[VectorInstance], bake: &ObjectBake) -> usize {
    vectors
        .iter()
        .filter(|vi| bake.tile_texture_for_gid(vi.geometry_id).is_none())
        .count()
}

#[cfg(test)]
#[path = "motion_glow_layer_tests.rs"]
mod tests;
