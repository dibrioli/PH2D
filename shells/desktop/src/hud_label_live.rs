//! **O texto que o JOGO muda** (TOP-20 #20) — o 10.º produtor de [`LiveGeometry`].
//!
//! Um [`ph2d_ecs::UiLabel`] tem uma FONTE (um contador, um relógio, uma etiqueta). O número dela é
//! derivado a cada quadro, os glyphs são re-cozidos, e o resultado entra no mapa de geometria viva
//! **no `id` do texto autorado** — que é como a casa já desenha um offset, uma estampa, um
//! contorno ou a simetria.
//!
//! # ⛔ O DOCUMENTO não é reescrito, e é a lei que decide isto
//!
//! O texto do artista fica onde está. O que muda é o que se DESENHA, e é a lei do ADR-0153 que a
//! `VecAnchors` já escreve para o layout: *o passe publica ONDE as coisas ficam; ele não escreve
//! ONDE elas estão*. Aqui: **o passe publica o que o rótulo MOSTRA; ele não escreve o que ele É.**
//!
//! Reescrever `VecShape::Text` por quadro seria escrever no `WorldSnapshot`: cada ponto marcado
//! viraria um passo de `Ctrl+Z` e entraria no ficheiro gravado.
//!
//! # ⭐ Os dois neutros são EXACTOS, e nenhum deles paga um cozimento
//!
//! 1. [`ph2d_ecs::LabelSource::Authored`] não deriva nada ⇒ o rótulo nem entra no mapa;
//! 2. uma fonte que não existe na cena devolve `None` ⇒ idem (e o artista continua a ver o que
//!    escreveu, em vez de um `0` inventado);
//! 3. e se o texto derivado for **igual** ao autorado, o mapa também fica sem ele — um documento
//!    cujo placar está em `0` e que diz `0` desenha pelo caminho de sempre, **ao bit**.

use ph2d_ecs::{Entity, SimWorld, UiLabel, VecShape};
use ph2d_tags::TagTree;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_glyph::{TextPlacement, text_to_compound_path};
use crate::vec_text_object::{axes_of_params, layout_of_params};

/// O que um rótulo pede a este quadro: onde desenhar, e com que texto.
struct Pedido {
    id: VecPathId,
    entity: Entity,
    params: ph2d_ecs::VecTextParams,
    texto: String,
}

/// ⭐ **Os glyphs de todos os rótulos cujo número mudou.** Vazio na cena típica.
///
/// ⚠️ **O estilo sai do caminho AUTORADO** (o `fill`/`stroke` do `VecPath` do documento), como no
/// `recook_text_object`: a cor de um placar é autoria, e re-derivá-la aqui seria uma segunda
/// resposta que divergiria no dia em que o artista mudasse a tinta.
pub(crate) fn cook(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    tags: &TagTree,
) -> LiveGeometry {
    // Fase de LEITURA: o mundo é emprestado enquanto se deriva, e larga-se antes do cozimento.
    let pedidos: Vec<Pedido> = {
        let candidatos: Vec<(VecPathId, Entity, UiLabel, ph2d_ecs::VecTextParams)> = map
            .iter()
            .filter_map(|(&id, &bits)| {
                let e = Entity::from_bits(bits);
                let l = sim.world().get::<UiLabel>(e)?.clone();
                match sim.world().get::<VecShape>(e) {
                    Some(VecShape::Text(p)) => Some((id, e, l, p.clone())),
                    _ => None,
                }
            })
            .collect();
        candidatos
            .into_iter()
            .filter_map(|(id, entity, label, params)| {
                let texto = ph2d_ecs::hud::texto(sim.world_mut(), tags, &label)?;
                // ⭐ O terceiro neutro: derivado igual ao autorado ⇒ o caminho de sempre, ao bit.
                (texto != params.text).then_some(Pedido {
                    id,
                    entity,
                    params,
                    texto,
                })
            })
            .collect()
    };
    let mut live = LiveGeometry::new();
    for p in pedidos {
        let font = crate::vec_font::resolve(p.params.family.as_deref());
        let (fill, stroke) = scene
            .paths()
            .iter()
            .find(|q| q.id == p.id)
            .map_or((None, None), |q| (q.fill.clone(), q.stroke.clone()));
        // ⚠️ **O mesmo enquadramento do `recook_text_object`**, e pela mesma porta: um texto que
        // monta um caminho-guia é colocado sobre ele; os outros são cozidos na baseline e
        // centrados. Duas respostas a *«onde este texto assenta?»* poriam o número do placar meio
        // caractere ao lado do sítio onde o artista o pôs.
        let guide = crate::vec_text_ride::guide_of(sim, scene, map, p.entity);
        let placement = guide.as_ref().map_or(
            TextPlacement::At([0.0, 0.0]),
            crate::vec_text_ride::Guide::placement,
        );
        let Some(mut compound) = text_to_compound_path(
            &font,
            &p.texto,
            &layout_of_params(&p.params),
            &axes_of_params(&p.params),
            &placement,
            &fill,
            &stroke,
        ) else {
            continue; // uma string que não produz glyph nenhum não substitui nada
        };
        if guide.is_none() {
            let ctr = crate::vec_glyph::path_center(&compound);
            crate::vec_glyph::offset_path(&mut compound, [-ctr[0], -ctr[1]]);
        }
        live.insert(p.id, vec![compound]);
    }
    live
}
