//! **O canal da aparência DESLOCADA** (doc 89, folha 14) — o que um `source.object`
//! com `time_offset` não-nulo lê, e quais offsets o bake precisa assar.
//!
//! FILHO de `objects` via `#[path]`, então `use super::*` alcança o `appearance_tile`
//! privado e o `is_reserved` do publicador. Cortado por ASSUNTO: o pai responde *o que
//! a cena tem*, este responde *QUANDO olhar para isso*.

use super::*;

/// **O canal da aparência DESLOCADA** (doc 89, folha 14) — o que cada
/// `source.object` com `time_offset` não-nulo lê.
///
/// Duas escritas, nesta ordem, e as duas são necessárias:
///
/// **1. O padrão TRANSPARENTE.** Um sprite, uma forma vetorial, um grupo — nenhum tem
/// animação própria, então "com que cara ele está meio segundo à frente" tem a mesma
/// resposta que "agora". Copiar o canal cru é o que torna o param **inofensivo** onde
/// ele não tem o que fazer; sem isso o nó leria um external que ninguém publicou e o
/// objeto desapareceria da cena.
///
/// **2. O Flip VENCE.** Um objeto Flip é o meio que de facto tem um desenho por
/// quadro — é o P0 que esta wave fecha —, então a tile assada no quadro deslocado
/// sobrescreve a cópia. Se o bake não produziu geometria naquele quadro (a animação
/// não tem desenho ali), a cópia transparente FICA: pulado, nunca adivinhado, que é a
/// mesma cerca que o resto desta membrana honra.
///
/// ⚠️ **`&mut MotionState` + `seconds`, e não `(cook, graph)`, desde 2026-08-28:** o `off` de
/// cada pedido é um param que pode vir de um FIO, e resolver um param conduzido é cozinhar o
/// driver ([`super::super::motion_externals::resolved_params`]). Com a assinatura antiga esta
/// membrana lia `override → default` enquanto o `eval` do nó monta a MESMA chave por
/// `ctx.param` — que resolve o conduzido — e as duas chaves DIVERGIAM: o objeto sumia.
pub(crate) fn publish_shifted(motion: &mut MotionState, seconds: f64) {
    let requests = shifted_requests(motion, seconds);
    // Campos disjuntos do `MotionState`: o cook muta, os bakes só se leem.
    let cook = &mut motion.pump.cook;
    let flip_bakes = &motion.flip_object_bake;
    for (name, off) in requests {
        if crate::render_loop::motion_bridge::shapes::is_reserved(&name) {
            continue; // the editor's namespace
        }
        let key = ph2d_nodegraph::external::appearance_of(&name, off);
        if let Some(e) = cook.externals().get(&name) {
            let unshifted = e.value.clone();
            cook.set_external(key.clone(), unshifted);
        }
        if let Some(tile) = flip_bakes.tile_named_shifted(&name, off) {
            cook.set_external(
                key,
                appearance_tile(
                    tile.size,
                    [1.0, 1.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    tile.texture_id,
                ),
            );
        }
    }
}

/// **Os deslocamentos de tempo que o DOCUMENTO pede** — os `time_offset` não-nulos
/// dos `source.object`, mais o `0.0` implícito (o nome cru, que todo objeto publica).
///
/// ⚠️ **É o grafo que decide, e é isso que limita o recurso.** Uma tile deslocada custa
/// VRAM, e a pergunta *"quantas?"* tem uma resposta que o artista põe na tela com a
/// mão: o número de nós `source.object`. Sem esta varredura o bake teria de adivinhar
/// uma faixa de offsets — e adivinhar é assar tiles que ninguém pediu.
pub(crate) fn wanted_shifts(motion: &mut MotionState, seconds: f64) -> Vec<f32> {
    let mut out = vec![0.0_f32];
    for (_, off) in shifted_requests(motion, seconds) {
        if !out.iter().any(|x| x.to_bits() == off.to_bits()) {
            out.push(off);
        }
    }
    out
}

/// **`(nome do objeto, offset)` de cada `source.object` DESLOCADO** — o que o canal
/// deslocado tem de publicar.
///
/// ⚠️ O offset zero fica de fora de propósito: ele já é o nome cru, e publicá-lo de
/// novo seria uma segunda escrita no canal que todo o resto já usa.
pub(crate) fn shifted_requests(motion: &mut MotionState, seconds: f64) -> Vec<(String, f32)> {
    // O id vem do MANIFEST do próprio nó, nunca de um literal repetido aqui.
    let ty = ph2d_node_source_object::MANIFEST.id;
    // Os nós primeiro: resolver a escada COZINHA o driver, e isso precisa do `motion` inteiro.
    let ids: Vec<(ph2d_nodegraph::graph::NodeId, String)> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_id() == ty)
        .filter_map(|n| {
            motion
                .doc
                .graph
                .node_text_params()
                .get(&n.id)
                .and_then(|p| p.get("object"))
                .map(|name| (n.id, name.clone()))
        })
        .collect();
    let mut out = Vec::new();
    for (id, name) in ids {
        // ⚠️ **A ESCADA INTEIRA** — `conduzido → override → default`, a mesma do `ctx.param`
        // com que o `eval` do nó monta a chave que ele vai LER. Ver o doc de [`publish_shifted`].
        let off = crate::render_loop::motion_externals::resolved_params(
            motion,
            id,
            seconds,
            &ph2d_node_source_object::MANIFEST,
        )
        .get(ph2d_node_source_object::TIME_OFFSET_PARAM)
        .copied()
        .unwrap_or(0.0);
        if off == 0.0 || name.trim().is_empty() {
            continue;
        }
        out.push((name, off));
    }
    out
}
