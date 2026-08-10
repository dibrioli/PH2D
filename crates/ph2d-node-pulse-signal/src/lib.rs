//! `pulse.signal` — **a saída do grafo para o resto do app**: um pulso que atravessa este
//! nó passa a GRITAR UM NOME, e quem escuta é o host (um toast hoje; som, Luau e gameplay
//! quando existirem).
//!
//! É o último item aberto da folha 12 do doc 89, e a folha mede a fronteira antes de propor:
//! `grep` nos dois sentidos dá **zero** — nenhuma crate `pulse-*` menciona `ph2d-runtime`, e a
//! `ph2d-runtime` tem zero dependências por gate estrutural. **Este nó NÃO muda isso**: ele não
//! conhece a `ph2d-runtime`, não publica nada e não chama ninguém. Ele carrega um NOME e deixa
//! o pulso passar; quem transforma isso num `Signal` é o SHELL, que já é o dono da outbox e já
//! drena as outras duas fontes (a timeline e a física). O produtor não chama ninguém (ADR-0075).
//!
//! # As duas coisas que este nó NÃO é
//!
//! ⚠️ **Não é um `Signal` por LINHA.** Um pulso é *anônimo, por linha, por tique do cook*; um
//! `Signal` é *nomeado, por quadro*. Uma grade de 576 pontos que dispara junto é **UM** evento,
//! não 576 — 576 sons no mesmo quadro é ruído, não um efeito. O host colapsa por `any`, e a
//! informação que o colapso descartaria (QUANTAS linhas dispararam) viaja no próprio sinal, em
//! vez de virar 576 deles.
//!
//! ⚠️ **Não é a direção contrária.** *Colisão vira pulso* (`runtime → grafo`) é a outra metade
//! da fronteira e continua **fora**, com o motivo medido na folha: um sinal é fato do QUADRO e o
//! cook é função do TIQUE, então ele teria de viajar como CONTEÚDO (uma coluna) e não como evento
//! efêmero — senão um scrub perde o pulso e o grafo deixa de ser função do playhead. O doc 63 §4
//! a marca como cross-line/decisão do Enio.
//!
//! # A lei que impede o scrub de virar uma metralhadora
//!
//! Ela **não mora aqui** e isso é deliberado: o cook re-roda ao arrastar a régua, então publicar
//! do lado do nó soaria N vezes num scrub. A pergunta *"o relógio está TOCANDO para a frente?"*
//! já tem uma resposta neste app — a do emissor de markers da timeline (`!jumped &&
//! playhead.is_advancing_forward()`, com o quadro que NÃO dispara ainda assim re-baselizando) —
//! e o shell faz a mesma pergunta pela mesma porta. Duas cópias divergiriam no dia em que uma
//! delas ganhasse um caso especial.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// O tipo PULSO — evento discreto por instância `(Instances, Scalar, Event)`. Redeclarado
/// localmente para esta crate seguir sendo uma FOLHA drop-in: o vocabulário compartilhado é a
/// PORTA, nunca um símbolo.
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// A coluna de disparo de um stream de pulso (`1.0` no tique em que disparou).
const PULSE_COL: &str = "pulse";

/// A chave do text param que carrega o NOME gritado (lido por `EvalCtx::text_param`).
///
/// ⚠️ **Text param, não `ParamSpec`** — o contrato congelado (§6) diz `NodeManifest = 8` campos
/// e um param é `f32`; um nome não é um número. O canal de TEXTO vive no `Graph`
/// (`set_text_param`), que é exatamente o padrão que a `motion.expression` estreou **sem tocar o
/// contrato**, e é o padrão canônico para todo param não-`f32` desde então.
pub const NAME_KEY: &str = "name";

/// O contrato estático deste tipo de nó (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.signal"),
    name: "pulse.signal",
    inputs: &[PortSpec {
        name: "pulse",
        ty: PULSE,
    }],
    // PASSTHROUGH, e é o que o torna utilizável: um nó que só consumisse teria de ser folha da
    // cadeia, e o artista teria de escolher entre NOMEAR um pulso e USÁ-lo. Assim ele entra no
    // meio do fio, como um medidor em série.
    outputs: &[PortSpec {
        name: "pulse",
        ty: PULSE,
    }],
    // Pure: não toca canal nenhum, não guarda estado, não consome `pre`. O NOME sai pelo canal
    // de texto e o pulso sai igual ao que entrou.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct PulseSignal;

impl NodeOp for PulseSignal {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // ⚠️ **O `text_param` é LIDO aqui, e não só pelo shell.** Ler o nome no eval é o que o
        // põe no FINGERPRINT do cook — sem isso, renomear o sinal não invalidaria o memo e o
        // nó seguiria devolvendo o stream cozido sob o nome antigo. O valor lido não muda um
        // byte da saída; ele muda quando a saída é recomputada.
        let _ = ctx.text_param(NAME_KEY);
        let input = ctx.input(0);
        ctx.emit(passthrough(input));
    }
}

/// O stream de saída: o de entrada, verbatim.
///
/// ⚠️ Existe como função (em vez de um `clone()` inline) porque é ela que o gate de
/// byte-identidade chama — *este nó não pode alterar o pulso que atravessa*, e um `clone` inline
/// não tem onde ser afirmado.
#[must_use]
pub fn passthrough(input: &Stream) -> Stream {
    input.clone()
}

/// Quantas linhas dispararam neste stream — `0` quando nenhuma.
///
/// ⚠️ **Esta é a porta que o SHELL pergunta**, e ela mora aqui de propósito: *o que conta como
/// um disparo* é a lei do domínio do pulso (`> 0.5`, a mesma soleira que a família inteira usa),
/// não uma re-derivação do lado de quem publica. Uma segunda cópia no shell divergiria no dia em
/// que a soleira mudasse, e o modo de falha seria um sinal que não sai.
#[must_use]
pub fn fired_rows(stream: &Stream) -> usize {
    match stream.get(PULSE_COL) {
        Some(Column::Scalar(v)) => v.iter().filter(|&&x| x > 0.5).count(),
        _ => 0,
    }
}

/// Registra o tipo no registry.
///
/// # Errors
/// Devolve [`RegistryError`] se o tipo já estiver registrado.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseSignal))?;
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: NAME_KEY,
    label: "Signal Name",
    min: 0.0,
    max: 0.0,
    step: 0.0,
    widget: ParamWidget::Text,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
