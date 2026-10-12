//! ⭐⭐⭐ **AS CERCAS DO COLISOR** — as perguntas que decidem se um documento com contacto entre
//! peças pode cozer no DISPOSITIVO (docs 109 W2 e 115 W5/§33).
//!
//! # Porque elas são um ASSUNTO e vivem num ficheiro só
//!
//! As duas respondem à MESMA razão de motor — *o contacto e o passe do fim só existem na CPU*, logo
//! a mesma cena daria uma pilha num lado e um borrão no outro, sem erro nenhum — por **rotas
//! diferentes**: o nome escrito num text param, e o botão `Collide` de uma peça (a `source.shape` e,
//! desde a §33, o `source.object`).
//!
//! ⭐ Desde o doc 130 o passe corre também na placa, e desde o doc 131 D.14 a escrita do raio pelo
//! nome também: o que sobra é o nome que a placa não resolve e a cerca do fim.
//!
//! ⚠️ **O DESPACHO fica no [`super`]**, de propósito: o que este ficheiro tem são as PERGUNTAS, e
//! quem decide a ordem do cozimento é o `cook_gpu`. O gate
//! `a_cerca_do_passe_esta_de_facto_ligada_ao_cozimento` lê aquele ficheiro por `include_str!`
//! justamente porque *um gate que chama a função em vez de percorrer a rota afirma que a lei
//! existe, nunca que o produto a usa*.

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{KernelResolver, StreamOp};
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};

/// A frase da recusa do colisor nomeado num param que a placa não resolve
/// ([`graph_declares_collider`]) — uma constante, para o gate que prova a LIGAÇÃO ler a frase do
/// produto e não uma cópia dela.
pub(super) const RECUSA_COLISOR: &str =
    "CPU: um nome de colisor num param que a placa nao resolve (doc 109, doc 131 D.14)";

/// A frase da recusa do PASSE automático (doc 115 §33, doc 130 §5) — desde o doc 130 só quando
/// alguma peça declara e NENHUMA corrente de sink chegou à placa com a coluna do colisor: a
/// divergência seria entre *haver um passe* e *não haver nenhum*.
pub(super) const RECUSA_PASSE: &str =
    "CPU: uma peca declara colisor (Collide) e nenhuma saida chegou a placa com ele (doc 130)";

/// As varreduras do passe de cada sink, na ordem de `motion.sinks` (a mesma dos estilos) — a
/// MESMA porta que a CPU lê ([`ph2d_eval_motion::sink_collide_sweeps`]).
pub(super) fn varreduras_do_passe(motion: &MotionState) -> Vec<u32> {
    motion
        .sinks
        .iter()
        .map(|&s| ph2d_eval_motion::sink_collide_sweeps(&motion.doc.graph, s) as u32)
        .collect()
}

/// ⭐ **O passe na placa é um estágio que DESPACHA** (doc 130): uma cena cujo único trabalho de
/// placa é separar o que o sink desenha (a pilha `=114`: tudo a montante é `sim.step`, na CPU) vai
/// à rota híbrida — a separação de centenas de peças é o custo, não o lowering. Uma porta só, que o
/// `cook_gpu` e o censo de rota chamam.
pub(crate) fn estagios_que_despacham(
    plan: &ph2d_gpu_cook::GpuPlan,
    kernels: &dyn ph2d_nodegraph::gpu::KernelResolver,
    declara: bool,
) -> usize {
    plan.dispatching_stages(kernels) + usize::from(declara)
}

/// ⭐ **O `Relatorio` da rota da placa** (doc 130 §5): o perfilador do quadro lê o da bomba, e
/// sob a placa quem separou foi o dispositivo. Sem passe neste cozimento, o vazio — como a CPU.
pub(super) fn publica_o_relatorio_da_placa(motion: &mut MotionState, gpu: &ph2d_gpu::GpuContext) {
    let r = if motion.gpu_cook.passe_correu() == 0 {
        Some(ph2d_contact::passe::Relatorio::default())
    } else {
        motion.gpu_cook.relatorio_do_passe(gpu)
    };
    if let Some(r) = r {
        motion.pump.publica_relatorio(r);
    }
}

/// ⭐ **A cerca do fim** (doc 130 §5): a CPU disse que alguma peça declara, o cozimento na placa
/// correu, e nenhum sink chegou lá com a coluna — a placa desenharia sobreposto o que a CPU separa.
pub(super) fn cerca_do_passe(motion: &MotionState, declara: bool) -> bool {
    declara && motion.gpu_live && motion.gpu_cook.passe_correu() == 0
}

/// ⭐⭐⭐ **ALGUMA PEÇA DECLARA O COLISOR NESTE QUADRO?** (doc 115 §33) — o `Collide` RESOLVIDO (um
/// fio conta) de todo nó que o registo diz declarar por interruptor
/// ([`ph2d_node_registry::NodeRegistry::collider_switch`]: a `source.shape` e o `source.object`).
///
/// # Porque a pergunta é só «alguém declara?»
///
/// Desde a §33 o passe do fim do cozimento arma-se SOZINHO: o sink lê toda declaração que lhe
/// chega. ⇒ uma declaração tem sempre um leitor, e a metade «e alguém a lê?» que as cercas antigas
/// perguntavam (`graph_reads_declared_collider`) passou a ser sempre verdade. O passe corre **no
/// fim do cozimento da CPU**; na rota do dispositivo não existe corrente de CPU para separar ⇒ a
/// MESMA cena sairia separada num lado e sobreposta no outro.
///
/// ⚠️ **E um objecto SEM o botão continua a cozer no dispositivo** — a membrana já não declara
/// nada, e é o cartão que decide. A recusa é barata por MEDIÇÃO: a `8` varreduras a separação de
/// centenas de peças custa `12,4 %` de um quadro e o cozimento inteiro na CPU `0,02 %` (doc 115 §11).
///
/// ⭐ **E a escrita do raio PELO NOME também declara** (doc 131 D.14, [`escreve_colisor_pelo_nome`]):
/// ela vai à placa, logo arma a mesma cerca do fim.
pub(crate) fn peca_declara_colisor(motion: &mut MotionState, seconds: f64) -> bool {
    let declarantes: Vec<(
        ph2d_nodegraph::graph::NodeId,
        &'static NodeManifest,
        &'static str,
    )> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter_map(|n| {
            let ty = NodeTypeId::of(n.type_name.as_str());
            let chave = motion.registry.collider_switch(ty)?;
            let manifest = motion.registry.resolve(ty)?.manifest();
            Some((n.id, manifest, chave))
        })
        .collect();
    let botao = declarantes.into_iter().any(|(id, manifest, chave)| {
        crate::motion_externals::resolved_params(motion, id, seconds, manifest)
            .get(chave)
            .is_some_and(|&v| v >= 0.5)
    });
    botao || escreve_colisor_pelo_nome(motion, seconds)
}

/// ⭐⭐ **Um nome de colisor que a placa NÃO resolve manda o documento para a CPU** (doc 109 W2,
/// doc 131 D.14).
///
/// Desde o D.14 o sequenciador resolve ele próprio o nome de um text param que o TIPO do nó declara
/// a um [`StreamOp`] — a escrita por nome (o `Custom` do `motion.drive`, [`StreamOp::WriteNamed`])
/// e a leitura (o `value.attribute`, [`StreamOp::Project`]): os dois lados escrevem e lêem a MESMA
/// coluna, e o passe da placa separa-a. Recusa-se só o que sobra — um text param de colisor que
/// nenhum `StreamOp` do nó nomeia (outra chave, outro nó): quem o lê e o que faz com ele, a placa
/// não sabe.
///
/// ⚠️ **A pergunta continua a ser sobre o NOME**, aparado: um falso positivo é cozinhar na CPU,
/// nunca uma cena errada.
pub(super) fn graph_declares_collider(graph: &Graph, kernels: &dyn KernelResolver) -> bool {
    use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN};
    graph.node_text_params().iter().any(|(id, params)| {
        let resolvida = graph
            .node(*id)
            .and_then(|n| kernels.stream_op(NodeTypeId::of(n.type_name.as_str())))
            .and_then(|op| match op {
                StreamOp::WriteNamed { text_param, .. } | StreamOp::Project { text_param, .. } => {
                    Some(*text_param)
                }
                _ => None,
            });
        params.iter().any(|(chave, v)| {
            resolvida != Some(chave.as_str())
                && matches!(
                    v.trim(),
                    COLLIDER_COLUMN | COLLIDER_BOX_COLUMN | COLLIDER_OFFSET_COLUMN
                )
        })
    })
}

/// ⭐ **Alguma escrita por nome ACTIVA escreve o raio do colisor neste quadro?** (doc 131 D.14) —
/// o irmão do botão para a cerca do fim ([`cerca_do_passe`]): o `when` resolvido (um fio conta) e
/// o nome EXACTO, como a CPU o escreve. Só o raio: a escrita por nome é escalar, e a caixa e o
/// centro são `Vec2` — um escalar com esses nomes não declara em nenhum dos lados.
fn escreve_colisor_pelo_nome(motion: &mut MotionState, seconds: f64) -> bool {
    let escritas: Vec<_> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter_map(|n| {
            let ty = NodeTypeId::of(n.type_name.as_str());
            let Some(StreamOp::WriteNamed {
                text_param, when, ..
            }) = motion.registry.stream_op(ty)
            else {
                return None;
            };
            let nome = motion
                .doc
                .graph
                .node_text_param_overrides(n.id)?
                .get(*text_param)?;
            (nome == ph2d_nodegraph::attr::COLLIDER_COLUMN)
                .then(|| Some((n.id, motion.registry.resolve(ty)?.manifest(), *when)))
                .flatten()
        })
        .collect();
    escritas.into_iter().any(|(id, manifest, when)| {
        let p = crate::motion_externals::resolved_params(motion, id, seconds, manifest);
        when(&|nome: &str| p.get(nome).copied().unwrap_or(0.0))
    })
}

#[cfg(test)]
#[path = "motion_bridge_gpu_collider_tests.rs"]
mod collider_tests;

#[cfg(test)]
#[path = "motion_bridge_gpu_colisor_nome_tests.rs"]
mod colisor_nome_tests;
