//! **QUANDO um param aparece** — as três famílias de gate de visibilidade, numa porta só.
//!
//! Irmã de `params_channel` (o que a row É), `params_range` (até onde ela alcança) e
//! `params_stream` (o que os fios carregam). Cortada aqui pelo teto de 600 LOC da shell,
//! por ASSUNTO: as três perguntam a mesma coisa — *este controle faz alguma coisa agora?* —
//! e a resposta é uma conjunção, não três testes espalhados pelo construtor.
//!
//! ⚠️ **Uma porta só, e não três `any` no meio do laço.** O construtor filtra as rows
//! escalares e as de enum pelo MESMO predicado; se ele o reimplementasse duas vezes, uma
//! família de gate nova entraria numa e não na outra — e o defeito seria um controle que
//! aparece num sítio do painel e não noutro.

use crate::motion_state::MotionState;
use ph2d_node_registry::{NodeRegistry, ParamGate, ParamGateAbove, ParamGateText};
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::node::NodeTypeId;

/// As três tabelas do tipo, lidas UMA vez.
pub(super) struct Visibility {
    /// A condição é o valor de outro param **f32 arredondado a inteiro** — exato para um
    /// `Enum` (que guarda o índice): é o que faz um `source.shape` mostrar só os controles
    /// da espécie escolhida.
    gates: Option<&'static [ParamGate]>,
    /// A condição é a PRESENÇA de um TEXT param — o nome de uma forma desenhada. É o que
    /// faz os oito sliders de polígono de controle do `motion.spline_wrap` sumirem quando o
    /// artista escolhe a curva que desenhou.
    text: Option<&'static [ParamGateText]>,
    /// A condição é uma GRANDEZA contínua passar de um limiar. O arredondamento a inteiro
    /// do [`ParamGate`] é inútil num slider de `0..1`, então *"apareça quando isto sair do
    /// zero"* precisa desta. Mantém a cor, o tracejado e o Trim do `source.shape` fora do
    /// painel enquanto não há traço — e o Trim ali é mais que um controle morto: **sem
    /// traço, aparar a forma faz a forma DESAPARECER** (um contorno aberto não tem
    /// interior).
    above: Option<&'static [ParamGateAbove]>,
}

impl Visibility {
    /// Lê as três tabelas do registry. Um tipo sem entrada nenhuma não gateia nada.
    pub(super) fn of(reg: &NodeRegistry, id: NodeTypeId) -> Self {
        Self {
            gates: reg.param_gates(id),
            text: reg.param_gates_text(id),
            above: reg.param_gates_above(id),
        }
    }

    /// `param` aparece? `value_of` dá o valor atual de outro param, `has_text` diz se um
    /// text param está presente e não vazio.
    pub(super) fn shows(
        &self,
        param: &str,
        value_of: &impl Fn(&str) -> f32,
        has_text: &impl Fn(&str) -> bool,
    ) -> bool {
        !self
            .gates
            .into_iter()
            .flatten()
            .any(|g| g.param == param && !g.values.contains(&(value_of(g.when).round() as i32)))
            && !self
                .text
                .into_iter()
                .flatten()
                .any(|g| g.param == param && has_text(g.when_text) != g.when_present)
            && !self
                .above
                .into_iter()
                .flatten()
                .any(|g| g.param == param && value_of(g.when) <= g.above)
    }

    /// **O MODO deste nó não tem este controle** — só a família [`ParamGate`], nunca as
    /// outras duas.
    ///
    /// ⚠️ **A distinção é o que separa «não existe» de «está inerte agora», e ela decide se é
    /// legítimo DESTRUIR trabalho do artista.** Um `ParamGate` lê um índice de enum: num
    /// círculo o *Tooth Depth* não existe, e voltar atrás é escolher outra espécie — um gesto
    /// deliberado. Um [`ParamGateAbove`] lê uma grandeza contínua que a mão varre: baixar o
    /// *Stroke Width* até `0` esconde o *Trim End*, e subi-lo de volta traz--lo — a mesma mão,
    /// um segundo depois.
    ///
    /// ⚠️ Medido no `source.shape` (2026-08-27): **seis** params — as duas cores, o tracejado
    /// e as duas pontas do Trim — pendem de um slider a passar por zero. Soltar um fio nessa
    /// travessia apagaria a ligação do artista **num arrasto**, e voltar não a repõe.
    /// [`ParamGateText`] é a mesma forma (limpar o nome de uma curva é reversível).
    pub(crate) fn mode_hides(&self, param: &str, value_of: &impl Fn(&str) -> f32) -> bool {
        self.gates
            .into_iter()
            .flatten()
            .any(|g| g.param == param && !g.values.contains(&(value_of(g.when).round() as i32)))
    }
}

/// **AS ROWS QUE `node` MOSTRA AGORA** — a mesma conjunção, para quem tem uma pergunta em
/// vez de um laço.
///
/// O construtor de rows monta as duas closures uma vez e pergunta N vezes; quem precisa de
/// uma resposta isolada (o menu que um fio largado oferece, a lei que solta um fio órfão)
/// reimplementaria a conjunção — e a segunda cópia é exactamente como um param passa a
/// aparecer num sítio do app e não noutro.
///
/// ⚠️ **O valor lido é o AUTORADO** (override, senão o default do manifesto), pela mesma
/// porta que o construtor usa (`params::param_value`). Um param dirigido por fio só tem
/// valor DURANTE o cook, e um gate de visibilidade que dependesse dele mudaria o painel a
/// meio de um quadro.
pub(crate) fn shown_params(motion: &MotionState, nid: NodeId) -> impl Fn(&str) -> bool + '_ {
    let type_id = motion.doc.graph.node(nid).map(|i| i.type_id());
    let vis = type_id.map(|id| Visibility::of(&motion.registry, id));
    let texts = motion.doc.graph.node_text_param_overrides(nid);
    move |param: &str| {
        let Some(vis) = &vis else {
            return false;
        };
        let value_of = |name: &str| super::param_value(motion, nid, name);
        let has_text = |name: &str| {
            texts
                .and_then(|m| m.get(name))
                .is_some_and(|v| !v.trim().is_empty())
        };
        vis.shows(param, &value_of, &has_text)
    }
}

/// **OS PARAMS QUE O MODO DESTE NÓ NÃO TEM** — a metade discreta, para a lei que solta um fio
/// órfão. Ver [`Visibility::mode_hides`] para porque não são as três famílias.
pub(crate) fn mode_hidden(motion: &MotionState, nid: NodeId) -> impl Fn(&str) -> bool + '_ {
    let vis = motion
        .doc
        .graph
        .node(nid)
        .map(|i| Visibility::of(&motion.registry, i.type_id()));
    move |param: &str| {
        let Some(vis) = &vis else {
            return false;
        };
        vis.mode_hides(param, &|name: &str| super::param_value(motion, nid, name))
    }
}
