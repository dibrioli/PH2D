//! GPU-resident cook routing (GPU/M5 Fase 1 F1.1 + F1.2, ADR-0126).
//!
//! The per-frame decision — does this document's chain cook on the GPU, and if
//! so fully or from a CPU boundary — is a **pure function** of the plan and a
//! few flags, extracted here so it is unit-testable without a device (the bridge
//! tests are headless; the ε-parity of the actual dispatch is gated in the
//! motor, `ph2d-gpu-cook`'s `gpu_cpu_parity`). The bridge's `dispatch` reads the
//! route and drives the pump / `GpuCook` accordingly.

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// Whether the GPU produced this frame (the caller skips the CPU pump) or the
/// frame fell through to it (GPU off / no useful GPU work / a fully-GPU cook that
/// errored). A hybrid frame is always `Handled` — the pump was already marched
/// to the boundary, so re-running the sink loop would corrupt its clock.
#[derive(Debug)]
pub(super) enum GpuOutcome {
    Handled,
    FellThrough,
}

/// Which cook path this frame takes.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GpuRoute {
    /// The CPU pump renders the sinks (no GPU, or the GPU can't claim useful work).
    Cpu,
    /// The whole chain is kernel-covered: cook it 100% on the GPU, no CPU pump,
    /// no readback (F1.1).
    FullyGpu,
    /// A CPU prefix cooks up to the plan's boundary nodes; the GPU runs the
    /// suffix. **The nodes are not carried here**: `plan.boundaries` already
    /// names them, and copying them into the route would be a second list to keep
    /// in step with the first ([[feedback_two_doors_to_the_same_question_diverge]]).
    Hybrid,
}

/// ⭐⭐ **A ROTA, DITA EM VOZ ALTA** — `PH2D_MOTION_ROUTE_LOG=1`.
///
/// ⛔⛔ **Por que ela existe** (auditoria de performance, [doc 98 §2.3](../../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)):
/// o device faz **4,19 M objectos em 3,85 ms** contra **195,9 ms da CPU** — `50,9×` — e
/// **69,7% das cenas que este produto expõe caem para a CPU**. Até esta linha existir,
/// [`GpuOutcome::FellThrough`] era consumido pela ponte e **não acendia nada**: um grafo no
/// device e o mesmo grafo num núcleo só têm exactamente a mesma aparência na UI. *Um custo de
/// 50× que nenhuma superfície nomeia não é uma escolha de ninguém — é um acidente que se repete.*
///
/// ⚠️ **Disparo por BORDA, e a borda regista-se mesmo com o log desligado** — senão ligar a
/// variável a meio de uma sessão ficaria calado até à próxima mudança de rota. O que a variável
/// governa é a IMPRESSÃO, não o registo. (Um aviso chaveado por conteúdo imprimiria a 60 Hz;
/// esta linha pagou-se noutra wave deste módulo.)
pub(super) fn say_route(motion: &mut MotionState, reason: &'static str) {
    if motion.route_said == Some(reason) {
        return;
    }
    motion.route_said = Some(reason);
    if std::env::var_os("PH2D_MOTION_ROUTE_LOG").is_some_and(|v| v != "0") {
        eprintln!("[motion-route] {reason}");
    }
}

/// **Toda saída para a CPU passa por AQUI e nomeia-se.** ⛔ Um `return
/// GpuOutcome::FellThrough` cru é a forma que tornou o §2.3 possível: cinco recusas, cada uma
/// com o seu motivo, e nenhuma delas alcançável de fora.
fn fell(motion: &mut MotionState, reason: &'static str) -> GpuOutcome {
    say_route(motion, reason);
    GpuOutcome::FellThrough
}

/// Choose the cook route from the plan and this frame's flags — the one place
/// the "fully vs hybrid vs CPU" policy lives.
///
/// - GPU is on by default (`gpu_enabled`; `PH2D_GPU_COOK=0` opts out) and for a
///   document with **at least one** sink and **no time scopes** — `motion.time_remap`
///   recuses to the CPU whole. ⭐ **Several sinks go to the device since doc 119 W3**
///   (the union plan + `cook_many`); it used to be *«F2+ territory»*, a note about a
///   price nobody had measured.
/// - Fully-GPU when the plan claims the whole chain (no boundaries).
/// - Hybrid when the plan leaves **any** CPU boundaries **and** the GPU suffix
///   has at least one dispatching stage — a boundary whose only GPU stage is the
///   pass-through `output` would upload the sink stream just to lower it (no
///   compute win), so that recuses to the CPU.
///
/// **N boundaries used to recuse here**, on the reasoning that the pump handed
/// over one cooked node per tick and *"marching it twice would advance its clock
/// twice"*. That described the CALLER: the march and the `pre` feedback are per
/// call, so the pump now takes the whole set and marches once
/// (`advance_or_scrub_to_nodes_scoped`). The reasoning was also unreachable when
/// it was written — no kernel had two stream inputs — and it silently swallowed
/// the two-seam case the day `motion.look_at` made it reachable.
pub fn gpu_route(
    gpu_enabled: bool,
    n_sinks: usize,
    scopes_empty: bool,
    boundaries: &[(NodeId, usize)],
    dispatching_stages: usize,
) -> GpuRoute {
    if !gpu_enabled || n_sinks == 0 || !scopes_empty {
        return GpuRoute::Cpu;
    }
    match boundaries {
        [] => GpuRoute::FullyGpu,
        _ if dispatching_stages >= 1 => GpuRoute::Hybrid,
        _ => GpuRoute::Cpu,
    }
}

/// ⭐ **As CERCAS DE FONTE** (o vector vivo e o objecto) vivem num irmão — ver o cabeçalho dele.
#[path = "motion_bridge_gpu_objecto.rs"]
mod objecto;
pub(super) use objecto::{
    cook_publishes_live_geometry, cook_publishes_only_atlas_objects, graph_has_object_source,
};

/// ⭐⭐⭐ **As três CERCAS DO COLISOR** vivem num irmão (docs 109 W2 · 115 W1/W4) — ver o cabeçalho
/// dele. O que fica AQUI é o despacho, que é o que decide a ordem do cozimento.
#[path = "motion_bridge_gpu_colisor.rs"]
mod colisor;
#[path = "motion_bridge_gpu_forma.rs"]
pub(crate) mod forma;
#[path = "motion_bridge_gpu_mistura.rs"]
pub(crate) mod mistura;
use colisor::{RECUSA_COLISOR, RECUSA_PASSE, graph_declares_collider};
pub(crate) use colisor::{estagios_que_despacham, peca_declara_colisor};

#[path = "motion_bridge_gpu_tomadas.rs"]
mod tomadas;
#[cfg(test)]
pub(crate) use tomadas::espera_as_tomadas;

/// O motivo de um documento de várias saídas não ir à placa quando uma delas não pode ser encenada
/// (doc 119 W3): ela ficaria fronteira, e o estilo de uma saída iria para as linhas de outra.
pub(crate) const RECUSA_SAIDA_FORA_DA_PLACA: &str =
    "CPU: uma das saidas nao vai a' placa (o plano deixou-a fronteira)";

/// O motivo de a ordem por linha (`Draw Order: Stream`) em mais de uma saída ficar na CPU (doc 119
/// W4): a CPU entrelaça as linhas das saídas por índice, e a placa só sabe ordenar faixas.
pub(crate) const RECUSA_ORDEM_ENTRE_SAIDAS: &str =
    "CPU: ordem por linha em mais de uma saida -- a placa so' ordena faixas (doc 119 W4)";

/// **Este sink mistura EM GRUPO?** — a pergunta da cerca da rota da placa ([`mistura`]), PURA para uma cena a poder
/// perguntar sem montar o quadro (a `=13` pergunta-o). `isolada` = a camada filtrada do Motion
/// está ligada: o `Scene` vai à pousa dela e não agrupa ([`ph2d_eval_motion::MisturaDoSink::pousa_na_camada`]).
pub(crate) fn sink_mistura_em_grupo(graph: &Graph, sink: NodeId, isolada: bool) -> bool {
    let m = mistura_do_sink(graph, sink);
    m.tem_camada() && !m.pousa_na_camada(isolada)
}

fn mistura_do_sink(graph: &Graph, sink: NodeId) -> ph2d_eval_motion::MisturaDoSink {
    // ⚠️ O tag numa variável, e não no campo: o gate `every_gpu_cook_call_receives_the_style` conta
    // as ocorrências de `blend` seguido de vírgula neste ficheiro, uma por chamada de cook.
    let tag = ph2d_eval_motion::sink_style(graph, sink).blend;
    ph2d_eval_motion::MisturaDoSink {
        blend: tag,
        com: ph2d_eval_motion::sink_blend_with(graph, sink),
        sink: 0,
    }
}

/// Os relógios que o device marcha: um tique vira `sub` sub-passadas.
///
/// ⚠️ **O TIQUE não se subdivide, só o PLAYHEAD** — e as duas metades disso são load-bearing.
/// O device avança o ping-pong do `pre` a cada CHAMADA de `cook` (o `self.prev` é reatribuído no
/// fim dela), então `sub` chamadas são `sub` sub-tiques sem renumerar nada; e o ring de scrub
/// chaveia pelo tique, com `should_record` a deduplicar — então a 1ª sub-passada grava o estado
/// de ENTRADA do quadro e as seguintes não o sobrescrevem com um estado do meio.
///
/// O `dt` que as leis de contagem leem sai de `playhead − last_playhead` no próprio device, então
/// ele subdivide sozinho e os nascimentos telescopam, exactamente como na CPU.
///
/// `sub <= 1` devolve o que este código sempre devolveu, termo a termo.
fn substep_clocks(
    ticks: &[u64],
    sub: u32,
    fixed_dt: f64,
    drives_loop: bool,
) -> Vec<(f64, Option<u64>)> {
    ticks
        .iter()
        .flat_map(|&t| {
            let clock = drives_loop.then_some(t);
            (1..=sub.max(1)).map(move |k| {
                let frac = f64::from(k) / f64::from(sub.max(1));
                // O quadro `t` cobre `((t-1)·dt, t·dt]`; a última sub-passada cai em `t·dt`.
                ((t as f64 - 1.0 + frac) * fixed_dt, clock)
            })
        })
        .collect()
}

#[path = "motion_bridge_gpu_marcha.rs"]
mod marcha;
// Os valores dos params dirigidos e o plano do produto — irmão pelo tecto de LOC.
#[path = "motion_bridge_gpu_dirigidos.rs"]
mod dirigidos;
#[cfg(test)]
pub(crate) use dirigidos::plano_do_produto_para;
pub(crate) use dirigidos::{plano_do_produto, valores_dirigidos};

/// O estilo de cada saída para a rota da placa, com a lei do dono da BOMBA (`a_lei`) — a MESMA
/// fonte que a rota da CPU usa (`MotionCookPump::pump`). Lida do ambiente aqui, as rotas
/// discordavam sempre que alguém mudava a lei da bomba (doc 123 §4).
pub(crate) fn estilos_das_saidas(motion: &MotionState) -> Vec<ph2d_render::SinkStyle> {
    let lei = motion.pump.a_lei();
    let isolada = motion.pump.camada_isolada();
    motion
        .sinks
        .iter()
        .map(|&s| {
            let mut estilo = ph2d_render::SinkStyle {
                so_com_forma: lei,
                ..ph2d_eval_motion::sink_style(&motion.doc.graph, s)
            };
            // O modo vai à pousa da camada — a MESMA regra da bomba.
            if mistura_do_sink(&motion.doc.graph, s).pousa_na_camada(isolada) {
                estilo.blend = 0;
            }
            estilo
        })
        .collect()
}

/// The GPU-resident cook for this frame (GPU/M5 Fase 1 + F1.2, ADR-0126).
///
/// Unless `PH2D_GPU_COOK=0`, an unscoped document cooks on the GPU — ONE sink or
/// several, planned as their UNION and lowered one after another (doc 119 W2–W4):
/// compute passes in one submit, the lowering writes the renderer's instance
/// buffer directly, zero readback. **Fully-GPU** when the plan claims the whole
/// chain; **hybrid** when a node has no kernel — the CPU prefix cooks up to that
/// boundary on the persistent pump (memo + `pre` feedback + the tick march, so a
/// sequential prefix sims correctly and a scrub is bit-exact), and only its
/// output stream crosses to the GPU, which runs the covered suffix. Anything the
/// plan can't usefully claim returns [`GpuOutcome::FellThrough`] to the CPU pump.
///
/// The graph panel reads a GPU frame through the bounded **tap** (Fase 4,
/// `readout::take_tap` — readouts, digest, probe), so a fully-GPU document is
/// no longer blind in the editor; the tap is one frame behind the cook it
/// samples (the documented ordering asymmetry vs the CPU memo).
pub(super) fn cook_gpu(
    motion: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
    target: u64,
    fixed_dt: f64,
    scopes: &TimeScopes,
) -> GpuOutcome {
    let t = std::time::Instant::now();
    motion.gpu_live = false;
    // Fast-path guard so a GPU-off or sinkless document never plans.
    // ⚠️ **Os dois motivos separam-se aqui de propósito:** eles leem-se iguais numa recusa
    // («a CPU desenhou») e um deles é uma ESCOLHA do artista (`PH2D_GPU_COOK=0`) enquanto o
    // outro é uma escada que ele não pediu e cujo preço é `50,9×`.
    if !motion.gpu_enabled {
        return fell(motion, "CPU: o device esta desligado (PH2D_GPU_COOK=0)");
    }
    // ⭐⭐⭐ **A cerca do MULTI-SINK caiu** (doc 119 W3): o plano é a UNIÃO das saídas (cada nó
    // cozido uma vez) e o `cook_many` baixa-as uma a seguir à outra no mesmo buffer. Fica só o
    // documento SEM saída, que não tem o que desenhar.
    if motion.sinks.is_empty() {
        return fell(motion, "CPU: o documento nao tem saida");
    }
    // A cena de demo que ensina um modo SÓ da CPU pede-a, e a leitura da rota diz porquê.
    if let Some(porque) = motion.cpu_pedida {
        return fell(motion, porque);
    }
    // ⭐⭐⭐ **AS FORMAS VIVAS VÃO À PLACA** (doc 121 W3). Até esta wave um grafo com uma forma
    // viva recusava aqui pelo TIPO do nó (`graph_has_live_vector_source`) e pela instância
    // condicional (o L-System em `Branches`): a placa não tinha rota para um `geometry_id` e
    // desenhava quadrados de átlas em branco. Hoje o cozimento escreve as linhas de forma para o
    // passe de formas, e a pergunta é de CONTEÚDO — cada forma publicada neste quadro, a placa a
    // desenha como o Vello? (a MESMA porta que a rota da CPU pergunta cópia a cópia). ⚠️ Antes do
    // plano, pela razão de sempre: a bomba da CPU tem de ser dona do tique desde o início.
    motion.formas_no_dispositivo.clear();
    let vivas = forma::handles_publicados(&motion.pump.cook);
    if !vivas.is_empty() && forma::saida_com_mistura_em_formas(motion) {
        return fell(motion, forma::RECUSA_FORMA_COM_MISTURA);
    }
    if let Err(porque) = forma::formas_para_a_placa(
        crate::motion_shape_placa::por_ordem(),
        &vivas,
        &motion.shape_store,
        &mut motion.placa_geometrias,
    ) {
        return fell(motion, porque);
    }
    // Doc 109 · 131 D.14: um nome de colisor que a placa não resolve — ver [`graph_declares_collider`].
    if graph_declares_collider(&motion.doc.graph, &motion.registry) {
        return fell(motion, RECUSA_COLISOR);
    }
    // ⭐ Doc 130: o passe automático (doc 115 §33) corre TAMBÉM na placa, antes do lowering de
    // cada sink — a pergunta deixou de recusar e passou a ARMAR a cerca do fim. Ver
    // [`peca_declara_colisor`] e [`colisor::cerca_do_passe`].
    let declara = peca_declara_colisor(motion, target as f64 * fixed_dt);
    motion
        .gpu_cook
        .set_passe(colisor::varreduras_do_passe(motion));
    // A tomada de um sink (o gizmo do colisor) vê a corrente separada em qualquer rota.
    motion.pump.set_sinks_do_desenho(&motion.sinks);
    // ⭐⭐⭐ **Um sink que mistura EM GRUPO** (doc 118 W3/W4 · B6 · P4): formas E imagens vão à placa
    // de formas, que compõe o alcance no tom das formas; cai para a CPU o que a placa não compõe.
    // Ver [`mistura`].
    let em_grupo: Vec<bool> = motion
        .sinks
        .iter()
        .map(|&s| sink_mistura_em_grupo(&motion.doc.graph, s, motion.pump.camada_isolada()))
        .collect();
    let misturas =
        crate::motion_shape_placa::Misturas::do_quadro(gpu, motion.pump.camada_isolada());
    if let Some(porque) = mistura::recusa_da_mistura_em_grupo(motion, misturas, &em_grupo) {
        return fell(motion, porque);
    }
    motion.gpu_cook.set_saidas_em_grupo(em_grupo);
    // A `source.object` that resolves to a live VECTOR publishes a `geometry_id`
    // external (ADR-0154 reused for objects, so a stamped vector stays crisp). The
    // GPU cook has no `geometry_id` route — it would draw a blank atlas quad — so
    // recuse to the CPU render, which draws it. CONTENT-aware, with the externals in
    // hand: a pure-sprite object graph publishes only `texture_id` and stays on the
    // GPU stamp (this wave's point). A node-type flag would recuse EVERY object graph.
    if graph_has_object_source(&motion.doc.graph, &motion.registry)
        && cook_publishes_live_geometry(&motion.pump.cook)
    {
        return fell(
            motion,
            "CPU: um source.object resolveu para geometria vectorial viva",
        );
    }
    // **O ritmo deste grafo** (doc 89, folha 13): de quantas sub-passadas o plano marcha. ⚠️ NÃO
    // há recusa aqui, e é por construção: o ritmo é do GRAFO — a mesma porta que o pump da CPU
    // pergunta —, então marchar o plano inteiro `n` vezes dá a cada ilha exactamente os `n`
    // sub-tiques que o bracket da CPU lhe daria. Os dois produtores concordam sem ninguém ter de
    // escolher entre acelerar e estar certo.
    let sub = ph2d_nodegraph::cook::graph_substeps(&motion.doc.graph, &motion.registry);
    // ⭐⭐⭐ **OS PARAMS DIRIGIDOS** (doc 110 §3): sem isto, qualquer fio de valor no grafo derruba
    // a cadeia inteira para a CPU. ⚠️ **As CHAVES é que decidem o plano** (quem é encenado) e os
    // VALORES é que entram no uniform — e o plano deriva-se uma vez, porque quem tem fio não muda
    // dentro do quadro; os valores, esses, re-derivam-se por tique no laço abaixo.
    // ⚠️⚠️ **O instante do PLANO vive dentro de [`plano_do_produto`], e isso é a cerca.** Os
    // valores do laço abaixo são de OUTRO instante (um por tique), e passar-lhes este congelava a
    // animação no primeiro sub-passo — com a cena a mexer-se, que é a forma mais cara de estar
    // errado. Fora de escopo, ela deixa de ser escrevível: *o erro que não compila não precisa de
    // gate.*
    motion.fases.cercas = t.elapsed();
    let t = std::time::Instant::now();
    let plan = plano_do_produto(motion, target as f64 * fixed_dt);
    motion.fases.plano = t.elapsed();
    // Como este sink DESENHA (doc 89, folha 17): blend · pivô · filtro · ordem. Lido da
    // porta ÚNICA — a MESMA que o pump da CPU pergunta no laço de sinks —, e resolvido
    // AQUI, ao lado do sink que o plano escolheu: um segundo leitor teria liberdade de
    // arredondar diferente, e as duas rotas desenhariam o mesmo documento de maneiras
    // diferentes, que nenhum gate que olha para uma rota consegue ver.
    //
    // ⭐⭐ **UM ESTILO POR SAÍDA**, na ordem do plano (doc 119 W3). ⚠️ Com várias saídas elas têm de
    // TODAS ter ido à placa — pedir uma saída não é garantir que ela chega lá (uma que o plano não
    // encena fica fronteira), e cozinhar assim poria o estilo de uma nas linhas de outra.
    if motion.sinks.len() > 1 && plan.sinks != motion.sinks {
        return fell(motion, RECUSA_SAIDA_FORA_DA_PLACA);
    }
    let estilos = estilos_das_saidas(motion);
    // ⭐ **A ORDEM DE DESENHO** (doc 119 W4): a placa só reproduz a da CPU ordenando FAIXAS, logo a
    // ordem por linha (`stream_order`) em mais de uma saída fica na CPU. A MESMA porta que o cook
    // repete como recusa.
    if !ph2d_gpu_cook::ordem_reproduzivel(&estilos) {
        return fell(motion, RECUSA_ORDEM_ENTRE_SAIDAS);
    }
    // The count-changing cerca (this wave): an OBJECT graph whose GPU suffix
    // reorders / changes count would mis-bind the texture-run partition — the
    // boundary `texture_id` column aligns with the sink ONLY when the suffix is
    // per-element. Recuse it to the CPU render (which draws it correctly). A
    // non-object graph is unaffected: no `texture_id`, no partition to mis-bind.
    //
    // ⭐⭐ **Ciclo 12 (doc 120 §8.2): a cerca pergunta pelo CONTEÚDO, como a irmã de cima.** A
    // partição por textura só pode desalinhar quando HÁ textura própria a partir: um objecto todo
    // no átlas partilhado (`texture_id = 0`, o sprite de átlas) dá a partição VAZIA por construção
    // (`texture_runs_from_boundary` — o ramo *«all-atlas»*), qualquer que seja a contagem. Recusá-lo
    // mandava para a CPU a cadeia inteira de uma escada de partículas por uma partição que não existe.
    if graph_has_object_source(&motion.doc.graph, &motion.registry)
        && !cook_publishes_only_atlas_objects(&motion.pump.cook)
        && plan.suffix_changes_count(&motion.registry)
    {
        return fell(
            motion,
            "CPU: sob um source.object, o sufixo de GPU muda a contagem",
        );
    }
    let route = gpu_route(
        motion.gpu_enabled,
        motion.sinks.len(),
        scopes.is_empty(),
        &plan.boundaries,
        estagios_que_despacham(&plan, &motion.registry, declara),
    );
    // ⛔ Um condutor com estado é prefixo da CPU: só a marcha do ramo híbrido o avança (doc 120 §8.3.3).
    let route = match route {
        GpuRoute::FullyGpu if !plan.condutores_na_marcha.is_empty() => GpuRoute::Hybrid,
        r => r,
    };
    match route {
        GpuRoute::Cpu if !scopes.is_empty() => fell(
            motion,
            "CPU: escopo de tempo (motion.time_remap) -- a escada do doc 98 §2",
        ),
        GpuRoute::Cpu => fell(
            motion,
            "CPU: fronteira sem estagio de GPU que despache (so o output passa-through)",
        ),
        GpuRoute::FullyGpu => {
            // A stateless plan is `f(params, playhead)` — one cook at the target
            // tick's time, as F1.1 always did. A plan that drives a `pre` loop
            // (ADR-0127) is SEQUENTIAL: its trajectory is the sum of its steps,
            // so it owes one cook per fixed tick, under the same law as the CPU
            // pump (`ticks_owed`) and for the same reason — one big jump would
            // make the motion depend on the frame rate.
            //
            // `rewind_for` is the scrub (D5): forward it just answers
            // `last + 1`; backwards it restores the newest checkpoint at or
            // before the target, on the device, and says which tick to re-sim
            // from. It replaces `ticks_owed` here rather than wrapping it —
            // `ticks_owed` answers `target..=target` for a backwards jump, which
            // for a sim means "cook the past against the future's state".
            // ⚠️ Lido ANTES do `rewind_for`, que o apaga ao rebobinar. Um plano sem estado não guarda
            // o último tique: vale o das entradas (a MESMA regra da CPU — um tique para trás repete).
            let ja_cozido = motion
                .gpu_cook
                .last_cooked_tick()
                .or_else(|| motion.pump.tique_das_entradas());
            let ticks: Vec<u64> = match plan.drives_a_loop() {
                true => (motion.gpu_cook.rewind_for(target)..=target).collect(),
                false => vec![target],
            };
            let ticks = substep_clocks(&ticks, sub, fixed_dt, plan.drives_a_loop());
            let gizmos = tomadas::pede(motion, gpu);
            // ⭐ AS ENTRADAS DE CADA TIQUE (doc 128 §3.3): o que já foi cozido repete o que a fita
            // gravou; o novo grava-se, e um ramo esquece também o futuro do anel da placa.
            let mut replay = motion
                .pump
                .comeca_o_replay(ticks.first().and_then(|r| r.1).unwrap_or(target));
            motion.gpu_live = ticks.iter().all(|&(playhead, tick)| {
                let t = tick.unwrap_or(target);
                if motion.pump.entradas_do_tique(&mut replay, t, ja_cozido)
                    == Some(ph2d_eval_motion::Gravacao::Ramo)
                {
                    motion.gpu_cook.forget_after(t);
                }
                // ⚠️ **POR TIQUE**: um `value.lfo` dá outro número a cada sub-passo, e reutilizar
                // o do plano congelaria a animação no primeiro deles.
                let t = std::time::Instant::now();
                let dirigidos = valores_dirigidos(motion, playhead);
                motion.fases.dirigidos += t.elapsed();
                motion.gpu_cook.set_driven(dirigidos);
                let t = std::time::Instant::now();
                let feito = motion.gpu_cook.cook_many(
                    gpu,
                    &motion.doc.graph,
                    &motion.registry,
                    &motion.registry,
                    &plan,
                    &[],
                    ph2d_gpu_cook::CookClock { playhead, tick },
                    motion.default_uv_rect,
                    motion.default_size,
                    &estilos,
                );
                conta_o_cozimento(&mut motion.fases, &motion.gpu_cook, t);
                forma::anota_a_mistura(&mut motion.formas_pedem_o_vello, feito)
            });
            if colisor::cerca_do_passe(motion, declara) {
                return fell(motion, RECUSA_PASSE);
            }
            // A cook that errored leaves `gpu_live` false → let the CPU pump draw.
            if motion.gpu_live {
                // As tomadas, que esta rota (sem marcha) deixava cair — ver [`tomadas::recolhe`].
                let ultimo = ticks.last().map_or(0.0, |(playhead, _)| *playhead);
                tomadas::recolhe(motion, &gizmos, ultimo, target, fixed_dt, scopes);
                say_route(motion, "device: o plano inteiro (fully-GPU)");
                colisor::publica_o_relatorio_da_placa(motion, gpu);
                forma::anota_as_formas(motion, vivas);
                GpuOutcome::Handled
            } else {
                fell(motion, "CPU: o cook no device ERROU -- o pump desenha")
            }
        }
        GpuRoute::Hybrid => {
            // Cook the CPU prefix up to EVERY boundary — and every stateful CPU driver — on the
            // pump, ONE call per tick for the whole set (the march and the `pre` advance are per
            // CALL), then upload the boundary streams to the GPU suffix ([`marcha`]).
            let marcha::Entrega {
                relogios,
                dirigidos,
            } = marcha::marcha(motion, &plan, target, sub, fixed_dt, scopes);
            // Borrow-splitting: the cook takes `&motion.gpu_cook` mutably while the streams live
            // in `motion.pump`, so the hand-off is materialised first — only the plan's boundaries
            // (the march also cooked the drivers).
            let handed: Vec<(NodeId, &ph2d_nodegraph::attr::Stream)> = motion
                .pump
                .boundary_streams()
                .iter()
                .filter(|(n, _)| plan.boundaries.iter().any(|(b, _)| b == n))
                .map(|(n, s)| (*n, s))
                .collect();
            // A boundary that failed to cook is simply missing, and the sequencer validates the
            // set against `plan.boundaries` — an incomplete hand-off is REFUSED
            // (`BoundaryMismatch`) rather than dispatched against an empty stream.
            if !relogios.is_empty() {
                motion.gpu_live = relogios
                    .iter()
                    .zip(dirigidos)
                    .all(|(&(playhead, tick), d)| {
                        motion.gpu_cook.set_driven(d);
                        let t = std::time::Instant::now();
                        let feito = motion.gpu_cook.cook_many(
                            gpu,
                            &motion.doc.graph,
                            &motion.registry,
                            &motion.registry,
                            &plan,
                            &handed,
                            ph2d_gpu_cook::CookClock { playhead, tick },
                            motion.default_uv_rect,
                            motion.default_size,
                            &estilos,
                        );
                        conta_o_cozimento(&mut motion.fases, &motion.gpu_cook, t);
                        forma::anota_a_mistura(&mut motion.formas_pedem_o_vello, feito)
                    });
            }
            if colisor::cerca_do_passe(motion, declara) {
                return fell(motion, RECUSA_PASSE);
            }
            // The pump was marched to the boundary this frame regardless of the
            // GPU result — do NOT also run the sink loop (it would early-return on
            // the same tick and leave `instances` holding the boundary lowering).
            // A GPU failure here renders nothing this frame rather than corrupt
            // the pump's clock.
            say_route(
                motion,
                "device: HIBRIDO -- prefixo na CPU, sufixo no device",
            );
            if motion.gpu_live {
                forma::anota_as_formas(motion, vivas);
                colisor::publica_o_relatorio_da_placa(motion, gpu);
            }
            GpuOutcome::Handled
        }
    }
}

/// Soma um `cook_many` (começado em `t`) e o que o `GpuCook` mediu dentro dele ao relógio do quadro.
fn conta_o_cozimento(
    fases: &mut super::fases::FasesDoQuadro,
    cook: &ph2d_gpu_cook::GpuCook,
    t: std::time::Instant,
) {
    fases.cook_many += t.elapsed();
    fases.cozimento = fases.cozimento.soma(cook.fases_do_ultimo_cozimento());
}

/// Does editing `param` on a node of `type_name` **re-number** the emitter's ids
/// — ADR-0130 D7, the trigger for restarting the GPU sim?
///
/// **Only `rate` does**, and the distinction is the whole point. Particle `k` is
/// born at `k / rate` (`emit`), so `rate` is the ONLY param in the emitter's
/// count law that re-defines what id `k` *names*: the same id becomes a different
/// particle, and the paired state (`gather_row = current_id − prev_first`) would
/// hand the new ids the old window's rows — a silent mispair as the window jumps.
/// There is no state to carry, because "the state of id `k`" no longer refers to
/// the same thing; so the sim restarts at the tick on screen
/// ([`GpuCook::reseed_from_next_tick`]).
///
/// `life` and `max` were in this list and **should not have been** (the smoke
/// question *"o que impede que as atualizações sejam feitas em tempo real?"*).
/// They do not touch `birth(k) = k/rate` at all — they only move the window's
/// LEFT edge (`first = newest + 1 − n`), so id `k` keeps its identity and every
/// survivor keeps its row. Shrinking is therefore live and **exact**: the
/// survivors' trajectories are bit-identical to a run that never changed. Growing
/// reveals ids the previous frame did not carry, and those are seeded by the
/// per-element `gather_paired` bounds check that already exists for newborns
/// (ADR-0130 D4) — no restart needed, and no read of a row that isn't there.
///
/// The launch params (`speed`/`angle`/`spread`/`seed`) and the geometry
/// (`x`/`y`/`size`) do not re-number either — the gather pairs id-for-id, the
/// running sim KEEPS, and only newborns take the new launch.
///
/// It is a tiny allowlist tied to ONE node's numbering law, not a general rule
/// that could rot ([[feedback_a_condition_that_enumerates_its_readers_rots]]): a
/// new numbering param on the emitter is a deliberate change to `emit` that
/// revisits this line.
pub(super) fn edit_renumbers_emitter(type_name: &str, param: &str) -> bool {
    type_name == "motion.emitter" && param == "rate"
}

#[cfg(test)]
#[path = "motion_bridge_gpu_tests.rs"]
mod tests;

/// Ciclo 19: a fita das entradas pela rota da placa.
#[cfg(test)]
#[path = "motion_bridge_gpu_entradas_tests.rs"]
mod entradas_tests;

/// ⭐ **O PORTÃO DA COSTURA** — a rota `FullyGpu` cozinha as TOMADAS que nunca marchou.
/// Vive aqui, e não ao lado do gizmo, porque o sujeito é esta ponte: `GpuOutcome` é
/// `pub(super)` do `motion_bridge`, logo um irmão do `render_loop` não o alcança.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_gpu_taps_tests.rs"]
mod taps_tests;

/// ⭐⭐⭐ **As cenas de VÁRIAS saídas, pela placa, contra a CPU** (doc 119 W3/W4) — a prova no
/// produto de que levantar a cerca do multi-sink não muda o que se vê.
#[cfg(test)]
#[path = "motion_bridge_gpu_varias_saidas_tests.rs"]
mod varias_saidas_tests;

/// A fila C do doc 131 §4 — a sonda única de velocidade.
#[cfg(test)]
#[path = "motion_bridge_gpu_fila_c_probe.rs"]
mod fila_c_probe;

/// A deriva da `=129` por tique e por nó (doc 131 §2) — sonda.
#[cfg(test)]
#[path = "motion_bridge_gpu_deriva_129_tests.rs"]
mod deriva_129_tests;

/// ⭐⭐⭐ **As FORMAS pela rota do DISPOSITIVO desenham o que a rota da CPU desenha** (doc 121 W3).
#[cfg(test)]
#[path = "motion_bridge_gpu_formas_tests.rs"]
mod formas_tests;

/// ⭐⭐⭐ **A mistura em grupo pela rota do DISPOSITIVO** (B6): paridade com a CPU e as pousas.
#[cfg(test)]
#[path = "motion_bridge_gpu_mistura_tests.rs"]
mod mistura_tests;

/// ⭐⭐⭐ **As IMAGENS de um grupo na placa** (P4, doc 132 §4): paridade com a cena vectorial da CPU,
/// nenhum quadro pelas sprites, e o preço.
#[cfg(test)]
#[path = "motion_bridge_gpu_mistura_imagens_tests.rs"]
mod mistura_imagens_tests;

/// ⭐⭐⭐ **O `Collide` na placa** (doc 130): as cenas do passe aceites e iguais à CPU.
#[cfg(test)]
#[path = "motion_bridge_gpu_passe_tests.rs"]
mod passe_tests;

/// A sonda da W4 do doc 121: por onde vai cada cena de demo com formas, e porquê.
#[cfg(test)]
#[path = "motion_bridge_gpu_rota_das_formas_probe.rs"]
mod rota_das_formas_probe;
