//! **A porta ÚNICA das MEMBRANAS** — tudo o que o shell publica no canal externo
//! do grafo, no instante em que tem de ser publicado.
//!
//! Um nó recebe params, entradas e o playhead — nada mais. Tudo o que ele precisa
//! e não pode alcançar (a biblioteca de vetor, a fonte tipográfica, um arquivo de
//! som) chega por aqui, sob uma **chave de conteúdo** que os dois lados derivam da
//! mesma função.
//!
//! ⚠️ **O INSTANTE é a metade que não se pode mover, e cada membrana já pagou por
//! ele:** publicar **antes** do dreno de intents mintaria a chave PRÉ-edição — a
//! forma escolhida no quadro anterior, o texto sem a tecla que o artista acabou de
//! digitar, o espectro de um quadro que o cook não vai cozinhar. É por isso que as
//! três moram numa chamada só: *post-drain, pre-cook* é uma propriedade do GRUPO, e
//! três sítios espalhados são três oportunidades de a quarta membrana nascer no
//! lugar errado.
//!
//! ⚠️ **E o mesmo *pré-cook* que protege a chave é o que a faz perder um param
//! CONDUZIDO** (doc 58) — ver [`driven_params`], que é a resposta, e mora aqui pela
//! mesma razão que o instante: é uma propriedade do GRUPO, e a membrana que nascer
//! amanhã tem de a herdar sem a redescobrir.

use std::collections::BTreeMap;

use crate::motion_state::MotionState;

/// **A SOAK do quadro** — nada acumula ao longo de horas de laço. Vive ao lado da porta das
/// membranas porque é aqui que a varredura mora.
#[cfg(test)]
#[path = "motion_frame_soak_tests.rs"]
mod soak_tests;

/// Publica as três fontes externas do quadro. `seconds` é o relógio do playhead —
/// as três o consomem: a de áudio porque as bandas são função dele, e as de forma e
/// texto porque um param conduzido por fio só tem valor num INSTANTE ([`driven_params`]).
pub(crate) fn publish_all(motion: &mut MotionState, seconds: f64) {
    // ADR-0154: a forma vetorial VIVA.
    super::motion_shape_gen::publish(motion, seconds);
    // O texto, uma instância por GLIFO.
    super::motion_text_gen::publish(motion, seconds);
    // As bandas de áudio — função do ARQUIVO e do PLAYHEAD (doc 63 §6), e a única
    // das três que muda com o relógio.
    super::motion_audio_gen::publish(motion, seconds);
    // A TABELA — função do FICHEIRO e de mais nada, então não recebe o relógio (doc 63 §6).
    super::motion_table_gen::publish(motion);
    // ⭐ As FITAS do L-System (doc 95) — recebe o relógio pela mesma razão que a forma e o
    // texto: um param conduzido por fio só tem valor num INSTANTE, e a chave de conteúdo é
    // cunhada dos params.
    super::motion_lsystem_gen::publish(motion, seconds);
    // ⚠️ **E VARRE o que ninguém pediu neste quadro** — depois das três, nunca no meio:
    // a forma e o texto internam no MESMO store, e varrer entre elas apagaria as
    // geometrias que a seguinte ainda ia pedir. Um param de forma conduzido por um
    // relógio muda a chave de conteúdo a 60 Hz, e sem esta linha o store cresce uma
    // entrada por quadro (ver `VecPathStore::sweep`, e o OOM que a escreveu).
    let _dropped = motion.shape_store.sweep();
}

/// **Os params de `node` que um FIO conduz, já resolvidos** (doc 58) — vazio no caso comum.
///
/// ⚠️ **Sem isto, uma forma (ou um texto) com um param conduzido desenha NADA, em silêncio.**
/// A chave de conteúdo é derivada dos params, o shell publica **antes** do cook, e o valor de
/// um param conduzido só existe **durante** o cook — então o shell cunhava a chave do valor
/// ESTÁTICO, o nó lia a do valor conduzido, as duas não se encontravam e o `eval` clonava o
/// external vazio. Medido em 2026-08-21: `drive_param(shape, trim_end, …)` ⇒ o cook devolve
/// **contagem 0**. É o modo de falha mais caro que esta casa conhece — a arte desaparece com o
/// nó certo selecionado e nada vermelho em lado nenhum — e ele estava aqui desde que os params
/// conduzidos existem; o Trim só o tornou provável, porque *keyar o `end` de 0 a 1 desenha a
/// forma* é literalmente para o que o Trim Paths serve.
///
/// ⚠️ **Resolve pela PORTA DO COOK, nunca por uma segunda avaliação.** Cozinhar o nó-motor
/// aqui não é trabalho extra: o `Cook` memoiza por revisão, então o cook do quadro encontra
/// exactamente este resultado. Uma leitura própria do driver seria a segunda porta que diverge
/// no primeiro ajuste — e divergir aqui é a forma a voltar a desaparecer.
///
/// ⚠️ **A escada que o chamador monta é a do `EvalCtx::param`: conduzido → override →
/// default.** Trocar a ordem faria um param conduzido perder para um override antigo, que é o
/// mesmo defeito com outra cara.
/// **A ESCADA INTEIRA, resolvida para UM nó — a porta única de toda membrana** (doc 58).
///
/// `conduzido → override → default`, que é **exactamente** a ordem do
/// [`EvalCtx::param`](ph2d_nodegraph::cook::EvalCtx::param). Uma membrana que cunha uma chave
/// de conteúdo tem de a derivar dos mesmos números que o nó vai ler; qualquer degrau em falta
/// faz as duas chaves divergirem, o nó lê um external que ninguém publicou, e **o que ele
/// desenha desaparece em silêncio**.
///
/// ⚠️⚠️ **Esta função existe porque a lei estava ESCRITA e não era HERDADA.** O cabeçalho deste
/// arquivo já dizia *"a membrana que nascer amanhã tem de a herdar sem a redescobrir"* — e o
/// censo de 2026-08-28 mediu que das quatro membranas que derivam uma chave de params, **duas
/// nunca a herdaram**: o `motion_audio_gen` (as OITO bandas) e o canal deslocado do
/// `source.object` (o `time_offset`). As duas curadas tinham a escada **copiada**, e uma lei
/// copiada duas vezes não é uma lei — só uma PORTA é.
///
/// ⚠️ Devolve **todo** param declarado, nunca só os conduzidos: um `get` que caísse fora do
/// mapa teria de reimplementar o degrau do default, que é o degrau que se esquece.
pub(super) fn resolved_params(
    motion: &mut MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    seconds: f64,
    manifest: &'static ph2d_nodegraph::node::NodeManifest,
) -> BTreeMap<&'static str, f32> {
    let driven = driven_params(motion, node, seconds);
    let ov = motion.doc.graph.node_param_overrides(node);
    manifest
        .params
        .iter()
        .map(|p| {
            let v = driven
                .get(p.name)
                .copied()
                .or_else(|| ov.and_then(|m| m.get(p.name).copied()))
                .unwrap_or(p.default);
            (p.name, v)
        })
        .collect()
}

pub(super) fn driven_params(
    motion: &mut MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    seconds: f64,
) -> BTreeMap<String, f32> {
    let Some(sources) = motion.doc.graph.param_sources(node) else {
        return BTreeMap::new();
    };
    let wanted: Vec<(String, ph2d_nodegraph::graph::NodeId, u16)> = sources
        .iter()
        .map(|(name, (src, port))| (name.clone(), *src, *port))
        .collect();
    let mut out = BTreeMap::new();
    for (name, src, port) in wanted {
        let Ok(vals) = motion
            .pump
            .cook
            .cook(&motion.doc.graph, &motion.registry, src, seconds)
        else {
            continue; // um driver que não coze deixa o param no override/default, como no cook
        };
        if let Some(v) = vals
            .get(port as usize)
            .and_then(ph2d_nodegraph::param_source::driven_value)
        {
            out.insert(name, v);
        }
    }
    out
}
