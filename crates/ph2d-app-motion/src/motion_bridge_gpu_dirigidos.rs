//! **Os valores dos params dirigidos e o PLANO DO PRODUTO** — a porta que o `cook_gpu`, o censo de
//! rota e a sonda do relógio chamam (doc 110 §14). Irmão do [`super`] pelo tecto de LOC; o corte
//! é por responsabilidade: ali a rota e o cozimento, aqui *o que o planeador recebe*.

use crate::motion_state::MotionState;

/// Um nó e os fios que chegam aos params dele: `(nó, [(param, (condutor, porta))])`.
type FioDeParam = (
    ph2d_nodegraph::graph::NodeId,
    Vec<(String, (ph2d_nodegraph::graph::NodeId, u16))>,
);

/// ⭐⭐⭐ **OS VALORES DOS PARAMS DIRIGIDOS, neste instante** (doc 110 §3 · doc 102 W1).
///
/// A razão de existir de um `value.*` é dirigir um param, e até esta wave isso derrubava a cadeia
/// inteira para a CPU — **medido, 6 de 6, e a primeira era uma constante**. O planeador deixou de
/// recusar; o que ele passou a exigir é o NÚMERO, e quem o sabe é quem coze o condutor.
///
/// ⚠️ **O condutor é cozido na CPU, de propósito, e isso NÃO é a metade lenta:** ele é uma
/// sub-árvore de um elemento (o `driven_value` lê o elemento `0`), e a CPU já o cozia hoje — só
/// que arrastava consigo o consumidor e os `4,19 M` objectos dele. *O que esta wave tira do
/// caminho lento é o consumidor, não o condutor.* ⭐ E desde 07/10 o condutor que a PLACA coze
/// fica de fora daqui (doc 110 §14.1 (2)): o plano encena-o e o número vai direto ao uniform.
///
/// ⚠️⚠️ **E ele é derivado POR INSTANTE.** Um `value.lfo` dá um número diferente a cada tique;
/// derivar uma vez e reutilizar em toda a marcha congelaria a animação no primeiro sub-passo —
/// com a cena a mexer-se, que é a forma mais cara de estar errado.
pub(crate) fn valores_dirigidos(
    motion: &mut MotionState,
    playhead: f64,
) -> ph2d_gpu_cook::DrivenParams {
    let mut fora = ph2d_gpu_cook::DrivenParams::new();
    // ⭐⭐ **O INTERRUPTOR DESTA WAVE** (`PH2D_MOTION_DRIVEN_GPU=0`) — devolver o mapa VAZIO é
    // exactamente a lei de antes dela: sem valores, nenhum nó com fio é encenado
    // (`plan::DrivenParams`). É por isso que ele é uma bissecção honesta e não um segundo
    // caminho — *o caso vazio já era o produto.*
    if !motion.driven_gpu {
        return fora;
    }
    if motion.doc.graph.all_param_sources().is_empty() {
        return fora;
    }
    // ⚠️ **Fotografados ANTES de cozer:** o `cook` toma o `motion` emprestado mutavelmente, e o
    // `all_param_sources` é uma leitura dele — os dois empréstimos não vivem juntos.
    // ⭐ Os que a placa coze ficam de fora (doc 110 §14.1 (2)) — o plano encena-os; cozê-los
    // aqui seria a volta pela CPU que a cura existe para tirar.
    let na_placa = if motion.condutores_na_placa {
        ph2d_gpu_cook::device_driven_params(&motion.doc.graph, &motion.registry, &motion.registry)
    } else {
        Default::default()
    };
    let fios: Vec<FioDeParam> = motion
        .doc
        .graph
        .all_param_sources()
        .iter()
        .map(|(n, m)| {
            let fora_da_placa = m
                .iter()
                .filter(|(p, _)| !na_placa.contains(&(*n, (*p).clone())));
            (*n, fora_da_placa.map(|(p, s)| (p.clone(), *s)).collect())
        })
        .collect();
    for (node, params) in fios {
        for (param, (src, port)) in params {
            let Ok(saida) =
                motion
                    .pump
                    .cook
                    .cook(&motion.doc.graph, &motion.registry, src, playhead)
            else {
                continue;
            };
            // ⚠️ **A MESMA porta que o `EvalCtx::param` usa** — um segundo leitor do mesmo valor
            // seria a forma clássica de as duas rotas discordarem no elemento que leem.
            // ⚠️ **Um condutor VAZIO entra na mesma, com `None`.** Ele foi consultado, e é isso
            // que a chave diz; o número ausente faz o param cair no override/default — a lei do
            // `driven_value`, a MESMA dos dois lados. Deixá-lo de fora faria o nó recuar para a
            // CPU só nos tiques em que um `pulse.*` não dispara, e a cena engasgava.
            let v = saida
                .get(port as usize)
                .and_then(ph2d_nodegraph::param_source::driven_value);
            fora.entry(node).or_default().insert(param, v);
        }
    }
    fora
}

/// ⭐⭐ **O PLANO QUE A PONTE PEDE, neste instante** — a porta ÚNICA do [`cook_gpu`] e do censo de
/// rota (`motion_route_census`): um censo que planeia com o mapa VAZIO conta como CPU as cenas
/// cujo fio de valor o dispositivo toma (medido em 07/10: `=116`, `=117`). As CHAVES dirigidas
/// decidem quem é encenado; o instante vive aqui dentro, longe dos valores por tique do laço.
pub(crate) fn plano_do_produto(motion: &mut MotionState, playhead: f64) -> ph2d_gpu_cook::GpuPlan {
    let sinks = motion.sinks.clone();
    plano_do_produto_para(motion, &sinks, playhead)
}

/// [`plano_do_produto`] para saídas à escolha — a porta dos gates e das sondas que montam uma
/// cadeia com o sink deles. ⭐ Com os fios ligados é a porta dos CONDUTORES NA PLACA (doc 110
/// §14.1 (2)): o `valores_dirigidos` já não coze o que a placa coze, e o plano encena-o.
pub(crate) fn plano_do_produto_para(
    motion: &mut MotionState,
    sinks: &[ph2d_nodegraph::graph::NodeId],
    playhead: f64,
) -> ph2d_gpu_cook::GpuPlan {
    let dirigidos = valores_dirigidos(motion, playhead);
    let porta = if motion.driven_gpu && motion.condutores_na_placa {
        ph2d_gpu_cook::plan_with_device_drivers
    } else {
        ph2d_gpu_cook::plan_driven_many
    };
    porta(
        &motion.doc.graph,
        &motion.registry,
        &motion.registry,
        sinks,
        &dirigidos,
    )
}
