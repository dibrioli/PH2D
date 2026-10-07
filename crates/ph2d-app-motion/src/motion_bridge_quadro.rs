//! **O COZIMENTO DE UM QUADRO** — o fim do [`super::dispatch`], partido para uma porta própria
//! (doc 110 §14.1 (5)): a sonda do relógio do ciclo 6 tem de medir o MESMO programa que o produto
//! corre, e duas cópias desta sequência (membranas, escopos, leques, preguiça, a placa ou a bomba)
//! divergiriam no dia em que uma ganhasse um passo.

use super::*;

/// Coze o quadro do `playhead`: as membranas, e então a placa (`cook_gpu`) ou a bomba da CPU.
pub(crate) fn coze_o_quadro(
    motion: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
    playhead: &ph2d_core::Playhead,
    fixed_dt: f64,
) {
    crate::motion_externals::publish_all(motion, playhead.time());
    // Time scopes (M2.N1): each `motion.time_remap` node rewrites the clock of
    // its upstream subtree. Rebuilt per frame — one pass over the node list, and
    // empty for a graph with no remapper (the common case), so the cook takes
    // its unscoped path unchanged.
    let scopes = ph2d_node_motion_time_remap::time_scopes(&motion.doc.graph, &motion.registry);
    // Os LEQUES de tempo: um `motion.trail` em `Resampled` tem a própria entrada
    // cozida em N instantes, em vez de lembrada num ring. Vazio para todo grafo
    // sem um (o caso comum), e então o cook toma o caminho de sempre.
    //
    // ⚠️ **Fica ao lado dos escopos de propósito** — os dois são a mesma pergunta
    // (*em que instante a sub-árvore de cima é lida?*) e um deles construído sem
    // o outro é um quadro a cozinhar com metade da resposta. O `fixed_dt` entra
    // aqui porque o `spacing` do rastro conta TIQUES, e a duração de um tique é
    // do shell, não do documento.
    //
    // ⚠️ **TRÊS produtores, um mapa** — o rastro re-cozido, a história da origem do emissor
    // e o atraso por cópia do `motion.clone`. Eles não colidem por construção (o mapa é
    // chaveado por `NodeId`, e um nó é de um tipo só), e a UNIÃO é montada aqui em vez de
    // dentro de um deles: quem sabe que existem três é o shell, e um `time_fans` que
    // chamasse o outro faria de duas crates-folha uma cadeia.
    let mut fans = ph2d_node_motion_trail::time_fans(&motion.doc.graph, &motion.registry, fixed_dt);
    fans.extend(ph2d_node_motion_emitter::time_fans(
        &motion.doc.graph,
        &motion.registry,
        fixed_dt,
    ));
    fans.extend(ph2d_node_motion_clone::fan::time_fans(
        &motion.doc.graph,
        &motion.registry,
        fixed_dt,
    ));
    motion.pump.set_time_fans(fans);
    // ⚠️ **O PLANO DE PREGUIÇA** (doc 89, folha 15) — quais roteadores podem saltar entradas
    // neste quadro. Ele vive ao lado dos leques pela mesma razão que eles vivem ao lado dos
    // escopos: os três são o que o cook precisa de saber e o documento não diz, e os três são
    // reconstruídos por quadro (um ramo que ganhou estado, ou um modo que o artista desligou,
    // tem de sair do plano no quadro em que isso acontece).
    //
    // ⚠️ **Reescreve, nunca acumula** — ver `Cook::set_lazy_branches`. Vazio (o caso comum, o
    // modo nasce desligado) é o caminho de sempre, ao bit.
    motion
        .pump
        .cook
        .set_lazy_branches(ph2d_node_value_switch::lazy::plan(
            &motion.doc.graph,
            &motion.registry,
        ));
    let target = motion_tick(playhead, fixed_dt);

    // ── GPU-resident cook (GPU/M5 Fase 1 + F1.2, ADR-0126) — opt-in preview ──
    // Unless `PH2D_GPU_COOK=0`, an unscoped document — one sink or several (doc 119) — cooks on the GPU
    // (fully, or hybrid from a CPU boundary). `Handled` = the GPU produced this
    // frame → skip the CPU pump; `FellThrough` = run the pump below. The whole
    // policy + dispatch lives in the `gpu` module (see there); this stays a seam.
    // ⚠️ A bypassed GROUP is short-circuited only on the CPU pump's graph (below),
    // so the GPU path is skipped while one exists — a muted preview must not cook
    // from the un-rewired graph (v1; the group-bypass module documents this).
    if motion.doc.bypassed_subgraphs.is_empty()
        && let gpu::GpuOutcome::Handled = gpu::cook_gpu(motion, gpu, target, fixed_dt, &scopes)
    {
        return;
    }

    // The graph the pump cooks: `doc.graph`, or a clone with every bypassed group
    // rewired to pass input[0] → output[0]. `None` (the common case) means the pump
    // reads the document graph unchanged. `output_nodes`/`time_scopes` above stay on
    // `doc.graph` — a bypass removes no node, so the sinks and scopes are the same.
    let cook = group_bypass::cook_graph(motion);
    // ⭐⭐⭐ **SÓ O ÚLTIMO TIQUE É DESENHADO** (report do dono, 18/09: *«189 objetos, Sweeps 1024 =
    // 3 FPS»*). Um quadro lento recupera vários tiques de simulação de uma vez, e cada um enche o
    // `instances`/`vector_instances` que o seguinte **sobrescreve** — só o último chega ao ecrã.
    //
    // ⇒ o passe de separação é um **ACABAMENTO SOBRE O QUE SE DESENHA** (doc 115 §3 W0: ele não
    // realimenta a simulação), logo pagá-lo nos tiques intermédios é trabalho para o lixo. ⚠️ E o
    // preço disso REALIMENTA: um quadro lento recupera mais tiques, que o tornam mais lento ainda.
    let tiques = ticks_owed(motion.pump.last_cooked_tick(), target);
    let ultimo = *tiques.end();
    let relogio = relogio::comeca(tiques.clone().count());
    for tick in tiques {
        motion.pump.set_separa_o_desenho(tick == ultimo);
        motion.pump.advance_or_scrub_scoped(
            cook.as_ref().unwrap_or(&motion.doc.graph),
            &motion.registry,
            &motion.sinks,
            tick,
            |t| t as f64 * fixed_dt,
            motion.default_uv_rect,
            motion.default_size,
            &scopes,
        );
    }

    relogio::regista(relogio, playhead.time());
    // LOD — the freeze fix (ADR-0154 follow-up). The cook just filled
    // `vector_instances` with one crisp `VectorInstance` per stamped live vector; a
    // grid of 160k is a per-frame freeze (~one Vello fill each). This moves any
    // geometry stamped past the knee onto `instances` as a GPU-instanced tile (which
    // scaled to millions), leaving the below-threshold shapes crisp. It runs ONLY on
    // the CPU pump. ⚠️ Since doc 121 W3 a live-vector graph MAY take the device route
    // (the shape pass draws the cook's copies sharp at any count, so no LOD photo is
    // needed there); the branch that returns early carries its shapes in
    // `gpu_cook.formas()`, never in `vector_instances`, so there is nothing to LOD.
    objects::apply_object_lod(
        &mut motion.pump.instances,
        &mut motion.pump.vector_instances,
        &motion.object_bake,
        objects::LOD_COUNT,
    );
    // A SONDA (`PH2D_PAN_DIAG=1`): o MUNDO de uma amostra de cada rota, depois do
    // cozimento e do LOD — o último sítio antes do desenho.
    if ph2d_pan_diag::on() {
        let vecs: Vec<[f32; 2]> = motion
            .pump
            .vector_instances
            .iter()
            .map(|v| v.world_pos)
            .collect();
        ph2d_pan_diag::note_instances(&motion.pump.instances, &vecs);
    }
}
