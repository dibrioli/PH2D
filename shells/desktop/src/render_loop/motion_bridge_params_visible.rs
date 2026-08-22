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

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamGateAbove, ParamGateText};
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
}
