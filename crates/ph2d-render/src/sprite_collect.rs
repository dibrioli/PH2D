//! Sprite-pass instance collection + canonical sort.
//!
//! Extracted from [`crate::renderer`] (Motion Nodes M0.T11 kept `renderer.rs`
//! under its LOC cap): pulls every scene [`RenderInstance`] out of the
//! [`PresentWorld`], appends any external cooked-stream slice, and sorts the
//! combined buffer into the render order the draw loop expects. Pure over
//! `scratch` — no GPU.

use crate::sprite::RenderInstance;
use ph2d_ecs::PresentWorld;

/// Collect the scene instances from `present` plus the `extra` slice into
/// `scratch` (cleared first), then sort into canonical render order.
///
/// `extra` is an external instance slice injected into the sprite pass (Motion
/// Nodes M0.T11) — a cooked node-graph stream draws **without** being spawned
/// into the ECS `present` (stream ≠ ECS, ADR-0035). A motion instance has
/// `z_order = 0`, `texture_id = 0` (atlas), so it joins the base atlas run. Pass
/// `&[]` for the scene-only path.
///
/// ⭐⭐ **`rank_window` é a FAIXA de desenho** (ADR-0154 Fase 2): `Some((lo, hi))` colhe só as
/// instâncias cujo `z_order` cai em `[lo, hi)`. `None` = tudo, e é **byte-idêntico** ao mundo
/// pré-faixas.
///
/// ⚠️ **O filtro entra ANTES da ordenação, e é sobre o `z_order` — o rank canónico**, o mesmo
/// número que o `sort_key` carimbou. Filtrar depois de ordenar daria o mesmo conjunto mas obrigaria
/// o chamador a saber onde a faixa começa no vector ordenado, que é uma segunda resposta à mesma
/// pergunta.
///
/// ⚠️ **O `extra` NÃO é filtrado.** Ele é o fluxo cozido do Motion, que não passa pelo ECS e por
/// isso não tem rank (`z_order = 0` por construção). Ele pertence à faixa que corre o resto do
/// pipeline — a última de sprites —, e é lá que o chamador o passa; nas outras faixas ele chega
/// vazio.
///
/// ⭐⭐⭐ **`held_back` são as entidades que ESTA passagem não desenha porque OUTRA as desenha
/// depois** (o vidro jateado do *Edit Prefab*, 2026-09-07). `None` = ninguém, e é o caminho de
/// sempre.
///
/// ⚠️ **Ela é por ENTIDADE e não por faixa de rank, e a diferença é medida:** o rank canónico
/// **não** garante corrida contígua para uma sub-árvore — este ficheiro já o diz a propósito do
/// agrupamento de recorte (*«a foreign sprite whose rank lands between the clip-parent and a
/// descendant»*). Uma janela de rank levantaria essa sprite estranha junto com a receita, e o
/// artista veria um objecto alheio nítido por cima do vidro.
///
/// ⚠️ **E ela é uma RETENÇÃO, não uma ocultação:** quem a passa fica obrigado a desenhar essas
/// entidades noutra passagem. *A alternativa — desenhar duas vezes e deixar a nítida por cima —
/// deixa o borrão da peça a escapar por fora da silhueta dela, como um halo.*
///
/// ⭐⭐⭐ **`meshes` são as malhas desta chamada** (`crate::sprite_mesh`): uma instância cuja entidade
/// traz um [`crate::SpriteMesh`] recebe a marca da malha dela no `flip_uv`, e o desenho troca-lhe o
/// quad pela malha. ⛔ **Toda outra instância sai SEM marca** — as da cena sem malha e, sobretudo, o
/// `extra`, que chega de fora e indexaria a malha de outra chamada.
pub(crate) fn collect_sorted_instances(
    scratch: &mut Vec<RenderInstance>,
    meshes: &mut crate::sprite_mesh::MeshFrame,
    present: &mut PresentWorld,
    extra: &crate::LiftedInstances,
    rank_window: Option<(u32, u32)>,
    held_back: Option<&std::collections::BTreeSet<ph2d_ecs::Entity>>,
) {
    scratch.clear();
    meshes.clear();
    let mut q = present.world_mut().query::<(
        &RenderInstance,
        Option<&ph2d_ecs::SimRef>,
        Option<&crate::SpriteMesh>,
    )>();
    for (inst, sim_ref, malha) in q.iter(present.world()) {
        if let Some((lo, hi)) = rank_window
            && (inst.z_order < lo || inst.z_order >= hi)
        {
            continue;
        }
        if let (Some(held), Some(sim_ref)) = (held_back, sim_ref)
            && held.contains(&sim_ref.0)
        {
            continue;
        }
        let mut inst = *inst;
        crate::sprite_mesh::clear_mesh_tag(&mut inst);
        if let Some(malha) = malha {
            inst.flip_uv |=
                meshes.push(malha, inst.anchor, inst.size) << RenderInstance::MESH_SHIFT;
        }
        scratch.push(inst);
    }
    let de_fora = scratch.len();
    scratch.extend_from_slice(extra.instances());
    for inst in &mut scratch[de_fora..] {
        crate::sprite_mesh::clear_mesh_tag(inst);
    }
    // ⭐⭐⭐ **A fatia de fora TAMBÉM leva malhas** (plano `docs/Skeleton/03`, W7): os fantasmas do
    // onion de uma imagem presa ao esqueleto são a arte DEFORMADA em `t ± k`, e sem isto eles
    // desenhavam o quad de repouso — a cena a ensinar que a arte não dobra, exactamente onde o
    // animador olha para ver como ela dobrou.
    //
    // ⚠️ **Índices DESLOCADOS por `de_fora`, e as marcas entram no MESMO `MeshFrame` da cena** — as
    // duas metades do quadro partilham a costura e os intervalos, e uma marca de outro quadro ou de
    // outra chamada indexaria a malha errada. Por isso não há `frame.clear()` aqui: ele correu no
    // topo, antes da cena.
    //
    // ⛔ E é por isso que o `extra` deixou de ser uma fatia crua: um **vector paralelo** de malhas
    // ao lado dela é o padrão que o `corner_radius` proíbe por escrito. O par viaja junto.
    for (i, m) in extra.meshes() {
        if let Some(inst) = scratch.get_mut(de_fora + *i) {
            inst.flip_uv |= meshes.push(m, inst.anchor, inst.size) << RenderInstance::MESH_SHIFT;
        }
    }
    sort_render_order(scratch);
}

/// Sort an instance buffer into canonical render order, in place. Shared by
/// [`collect_sorted_instances`] (scene + extra) and
/// [`SpriteRenderer::render_instances_only`](crate::SpriteRenderer::render_instances_only)
/// (an isolated slice) so the two entry points key on the **same** order — the
/// draw loop downstream (`compute_runs`) requires contiguous `(texture_id,
/// sampling, clip, blend)` runs, and a second sort with different keys would
/// batch differently and silently mis-draw.
///
pub fn sort_render_order(scratch: &mut [RenderInstance]) {
    // Sort by (clip anchor, z_order, sub_order, texture_id, sampling).
    //
    // `sub_order` (ADR-0070-amendment-9) entra ENTRE o `z_order` e o
    // `texture_id`, e a cena inteira o deixa a `0` ⇒ a ordenação de toda
    // sprite extraída é byte-idêntica. Ele existe para o único produtor que
    // emite `n` linhas na MESMA fatia de z — um sink de Motion —, cuja ordem de
    // linhas era derrotada pelo desempate por textura assim que o grafo tinha
    // mídia mista. Ver o doc-comment do campo: a grandeza que faltava não era
    // «mais fundo», era «mais à frente dentro do mesmo fundo».
    //
    // z_order is the extract-time sequential counter from
    // `propagate_transforms`'s DFS so the render order matches the Hierarchy
    // panel — without this an image-tool bake that flipped a sprite from Atlas
    // (id=0) to Individual (id>0) silently jumped to the front because the old
    // `sort_by_key(i.texture_id)` grouped all Atlas before any Individual. The
    // secondary keys `texture_id` then `sampling` group instances sharing a bind
    // group + sampler into contiguous runs within an unchanged z slice
    // (W3.T3.11); z_order stays primary so render order matches the panel.
    //
    // PRIMARY key = clip anchor (W3 §8 audit fix): a clip-group's instances MUST
    // be contiguous because `clip_pass::encode_clip_groups` batches each group by
    // a consecutive-run scan. The canonical rank (`z_order`) does NOT guarantee
    // that — a clip member with a divergent `ZIndexOverride`/`YSort`, or a
    // foreign sprite whose rank lands between the clip-parent and a descendant,
    // would split the span and the clipped members would silently VANISH (drawn
    // against an unmarked stencil). Anchoring every clip instance on its
    // clip-parent's rank (`clip_group - 1`, the parent's own `z_order`) keeps the
    // whole subtree as one contiguous block. Non-clip / mask instances anchor on
    // their own `z_order` → byte-identical order (the mask pass batches by role,
    // not contiguity, so it's unaffected).
    scratch.sort_by_key(|i| {
        let clip_anchor = if i.clip_group != 0 {
            i.clip_group - 1
        } else {
            i.z_order
        };
        (
            clip_anchor,
            i.z_order,
            i.sub_order,
            i.texture_id,
            i.sampling,
        )
    });
}
