//! **O COZIMENTO DE UM QUADRO** — o fim do [`super::dispatch`], partido para uma porta própria
//! (doc 110 §14.1 (5)): a sonda do relógio do ciclo 6 tem de medir o MESMO programa que o produto
//! corre, e duas cópias desta sequência (membranas, escopos, leques, preguiça, a placa ou a bomba)
//! divergiriam no dia em que uma ganhasse um passo.

use super::*;

/// **A mão do artista no painel do editor**, que o quadro lê para as tomadas dos gizmos: o nó
/// seleccionado (o dos deformadores) e o param em arrasto (o do pivô). O palco de uma macro não tem
/// mão — `MaoDoEditor::default()` —, e assim nunca coze pelo que o artista faz no editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MaoDoEditor {
    pub(crate) selecionado: Option<u32>,
    pub(crate) arrasto: Option<(u32, &'static str)>,
}

impl MaoDoEditor {
    /// O que o painel publicou neste quadro.
    pub(crate) fn do_painel() -> Self {
        Self {
            selecionado: params::selected_motion_node(),
            arrasto: ph2d_panel_motion_graph::current_graph_param_scrub(),
        }
    }
}

/// **O quadro inteiro** — os sinks e as tomadas que o grafo implica, e o cozimento. É a porta do
/// [`super::dispatch`] e a do tocador de uma macro ([`crate::macro_track::tocar`]): uma sequência,
/// dois chamadores. A gravação regista o relógio do quadro aqui (ADR-0186).
pub(crate) fn o_quadro(
    motion: &mut MotionState,
    gpu: Option<&ph2d_gpu::GpuContext>,
    playhead: &ph2d_core::Playhead,
    fixed_dt: f64,
    mao: MaoDoEditor,
) {
    // The render output IS the Output node — the sink follows the graph (wire a
    // chain into an Output node and it draws). `None` (no Output node) → the pump
    // renders nothing. Recomputed each frame so add/delete/rewire just works.
    motion.sinks = crate::motion_objeto::sinks_visiveis(motion, output_nodes(&motion.doc.graph));
    // As TOMADAS, recomputadas pelo mesmo motivo dos sinks: elas se curam sozinhas depois
    // de um load, um undo ou uma edição do grafo.
    motion.signal_taps = signal_nodes(&motion.doc.graph);
    // ⚠️ **ARMADAS na bomba, não passadas na chamada** — a marcha tem mais de uma porta (a
    // rota da GPU híbrida marcha por `advance_or_scrub_to_nodes_scoped`), e enquanto a tomada
    // era argumento da porta de sinks um documento híbrido cozinhava, desenhava e **não
    // gritava nada**, com a suíte verde. Medido: a cena `=26` planeja HÍBRIDA.
    // ⚠️ **As tomadas são a UNIÃO de dois pedidos, e cada um mantém o seu nome.** Os
    // sinais precisam das suas; o gizmo de canvas dos deformadores de quadrilátero precisa
    // do stream que ENTRA no nó seleccionado (a caixa a que os offsets dele se referem).
    // Juntar aqui, e não alargar o `signal_taps`, é o que impede o dreno de sinais de um
    // dia ver um nó que não é um `pulse.signal` — *o canal é partilhado, o significado não*.
    let mut taps = motion.signal_taps.clone();
    for n in crate::warp_gizmo::taps_for(motion, mao.selecionado)
        .into_iter()
        .chain(crate::collider_gizmo::taps_for(motion))
        // ⭐ E o gizmo de uma corrente de POSIÇÕES (ordem do dono, 2026-09-19): ele pede os sinks
        // cujo desenho a PLACA ainda não resolveu — *«esta corrente tem aparência?»* é pergunta
        // do cozido, e quando o dispositivo desenhou o quadro anterior a resposta já está no
        // registo dele (ver o `ponto_gizmo::taps_for`: sem isto a tomada recozinhava a simulação
        // inteira na CPU, ciclo 12).
        .chain(crate::ponto_gizmo::taps_for(motion, motion.pump.a_lei()))
        // ⭐ E o gizmo do PIVÔ (ordem do dono, 2026-09-19): a saída da forma cujo `Pivot X`/`Y`
        // está a ser arrastado AGORA — a forma e o sink dela, e só durante o gesto. ⚠️ Ele pede o
        // sink ELE PRÓPRIO: o irmão acima deixou de pedir os que a placa já desenhou.
        .chain(crate::pivot_gizmo::taps_for(motion, mao.arrasto))
    {
        if !taps.contains(&n) {
            taps.push(n);
        }
    }
    motion.pump.set_taps(&taps);
    // Só as dos sinais cozem em todo tique; as dos gizmos, só no tique desenhado.
    motion.pump.set_taps_por_tique(motion.signal_taps.len());
    // O que este quadro gritou, e o livro-razão de onde isso sai. Limpos aqui, não no dreno:
    // um quadro em que o shell não drena (a ferramenta saiu de foco) não pode deixar o grito
    // de ontem para amanhã.
    motion.signals_out.clear();
    motion.pump.clear_tap_fires();
    // As MEMBRANAS: post-drain, pre-cook. O instante e' propriedade do GRUPO, entao
    // as tres moram numa porta so (`motion_externals`).
    motion.macro_quadro(playhead, fixed_dt);
    coze(motion, gpu, playhead, fixed_dt);
    motion.macro_cozeu();
}

/// Coze o quadro do `playhead`: as membranas, e então a placa (`cook_gpu`) ou a bomba da CPU — sem os
/// sinks e as tomadas do [`o_quadro`], para os arneses que os armam à mão; e diz o que a fita
/// guardou (a faixa da Timeline e o cartão das entradas, doc 132 §2).
#[cfg(test)]
pub(crate) fn coze_o_quadro(
    motion: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
    playhead: &ph2d_core::Playhead,
    fixed_dt: f64,
) {
    coze(motion, Some(gpu), playhead, fixed_dt);
    crate::motion_historia::publica(motion, fixed_dt);
}

fn coze(
    motion: &mut MotionState,
    gpu: Option<&ph2d_gpu::GpuContext>,
    playhead: &ph2d_core::Playhead,
    fixed_dt: f64,
) {
    crate::motion_externals::publish_all(motion, playhead.time());
    motion.pump.define_a_tocar(playhead.is_playing());
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
    // A lente do quadro (ciclo 20): ANTES do atalho da placa, que retorna cedo.
    motion.lente.prepara(&motion.doc.graph, target);
    motion.filtro.prepara(&motion.doc.graph, &motion.sinks);
    motion
        .pump
        .isola_na_camada(!motion.filtro.pilha().is_empty());

    // ── GPU-resident cook (GPU/M5 Fase 1 + F1.2, ADR-0126) — opt-in preview ──
    // Unless `PH2D_GPU_COOK=0`, an unscoped document — one sink or several (doc 119) — cooks on the GPU
    // (fully, or hybrid from a CPU boundary). `Handled` = the GPU produced this
    // frame → skip the CPU pump; `FellThrough` = run the pump below. The whole
    // policy + dispatch lives in the `gpu` module (see there); this stays a seam.
    // ⚠️ A bypassed GROUP is short-circuited only on the CPU pump's graph (below),
    // so the GPU path is skipped while one exists — a muted preview must not cook
    // from the un-rewired graph (v1; the group-bypass module documents this).
    if let Some(gpu) = gpu
        && motion.doc.bypassed_subgraphs.is_empty()
        && let gpu::GpuOutcome::Handled = gpu::cook_gpu(motion, gpu, target, fixed_dt, &scopes)
    {
        // ⭐ A origem de cada objecto Motion, na placa (doc ObjectsAndComponents 01 §13).
        crate::motion_objeto::origem::aplica_gpu(motion, gpu);
        if ph2d_pan_diag::on() {
            let c = &motion.gpu_cook;
            ph2d_pan_diag::note_device(
                c.instances().map_or(0, |i| i.len() as usize),
                c.formas().map_or(0, |f| f.len() as usize),
            );
            // doc 123 §8: o mundo da 1.ª cópia pela leitura dos cartões (zero download novo).
            ph2d_pan_diag::note_device_world(crate::pan_diag_tap::mundo_lido(motion));
        }
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
    // ⇒ um tique intermédio não compõe nem separa: coze só o ESTADO (`ph2d_eval_motion::so_o_estado`,
    // 11/10 — a cena `=8` sem placa cozinhava o `collide` em cada um). ⚠️ E o preço disso
    // REALIMENTA: um quadro lento recupera mais tiques, que o tornam mais lento ainda.
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
    // ⭐ A origem de cada objecto Motion, ANTES do LOD (doc ObjectsAndComponents 01 §13).
    crate::motion_objeto::origem::aplica_cpu(motion);
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
        &mut motion.pump.faixas,
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
