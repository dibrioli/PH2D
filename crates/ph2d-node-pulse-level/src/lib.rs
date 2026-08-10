#![forbid(unsafe_code)]
//! `pulse.level` — **o pulso vira um NÚMERO** (doc 89, folha 12, a P0 da família).
//!
//! A conferência dos seis `pulse.*` achou **uma** causa por trás de metade da tabela: um
//! pulso não tem nível. As duas metades do vocabulário — *borda* e *nível* — são unânimes
//! na referência (MiniCavalry `threshold` emite *"0/1 **+** pulse"*, Cavalry Comparison
//! devolve 1 ou 0, o **Logic CHOP** do TouchDesigner existe para converter nível↔borda,
//! Max tem `gate`), e aqui só a borda existia.
//!
//! ⚠️ **Não é um rename de coluna, é uma troca de RELÓGIO.** Um pulso é
//! `(Instances, Scalar, **Event**)` na coluna `pulse`; um valor é
//! `(Instances, Scalar, **Frame**)` na coluna `v`. Os dois tipos de porta existem
//! justamente para que um não se conecte ao outro, e a consequência estava medida antes
//! desta crate: `param_source::driven_value` lê `attr::VALUE_COLUMN` (`"v"`) e um pulso
//! emite `"pulse"` ⇒ **um pulso não dirigia parâmetro nenhum**, e nenhum nó do domínio de
//! VALOR — a família inteira `value.*`, o `motion.drive`, o `field.*` a jusante — conseguia
//! ouvir um disparo.
//!
//! ## O que isto destrava (as cadeias, tentadas contra o catálogo real)
//!
//! - **LÓGICA entre pulsos** (a P1 que a folha 12 diz colapsar nesta P0): com dois níveis
//!   `0/1`, o `value.math` **já** traz `Min` e `Max` no enum, e sobre `{0,1}` eles **são**
//!   AND e OR; `pulse.compare(rise = 0.5)` devolve ao domínio de pulso. Nada disso é
//!   exprimível sem o nível — não há uma quarta ponte pulse→value.
//! - **O PORTÃO** (o item 3 do `SUPERAR:` da folha, e a janela de atividade do `pulse.beat`):
//!   `pulse → level → value.math(Multiply, condição) → pulse.compare(rise = 0.5)` dispara
//!   só onde a condição vale. A condição pode ser QUALQUER coisa do domínio de valor,
//!   inclusive um campo espacial — que é a combinação que nenhuma referência tem, porque
//!   nenhuma tem pulso-como-campo e campos componíveis ao mesmo tempo.
//!
//! ## O que ele NÃO faz, e o nó que já faz (medido, não suposto)
//!
//! A folha 12 escreve que *"`pulse.counter` **acumula** (monotônico, nunca volta a 0)"*.
//! Isso vale para o `count_tick` que ele carrega no `pre` — **não** para o que ele emite: o
//! valor exibido é `displayed(tick, N, mode)`, e com **`count_max = 2`** ele é exatamente o
//! par que faltaria aqui:
//!
//! | quero | é | porque |
//! |---|---|---|
//! | **toggle** (cada pulso inverte) | `pulse.counter(count_max = 2, mode = Wrap)` | `tick mod 2` = 0,1,0,1 |
//! | **latch** (o 1º pulso liga e fica) | `pulse.counter(count_max = 2, mode = Clamp)` | `min(tick, 1)` = 0,1,1,1 |
//!
//! ⇒ este nó é **MOMENTÂNEO e sem estado**, e tem **zero params** de propósito. Um `mode`
//! aqui seria a segunda resposta para uma pergunta que o contador já responde — e traria
//! junto o `pre` self-loop, a detecção de borda e a memória que ele já tem. O gate
//! `the_toggle_and_the_latch_are_the_counter` (na `ph2d-node-registry-init`) mede essa
//! tabela contra o registry real, para ninguém "completar" o nó depois.
//!
//! E o **nível de um SINAL** também já existe e não é este nó: `value.step(mode = Hard)` é o
//! comparador sustentado, então o *"0/1 + pulse"* da referência é o par
//! `value.step` (o nível) + `pulse.compare` (a borda) sobre o MESMO valor — não uma saída
//! que falte ao comparador.
//!
//! ## A lei que ele honra
//!
//! O pulso carrega **só "disparou"** (doc 06 §2, a decisão contra o `{value, edge, t}` do
//! MiniCavalry). Por isso a saída é **normalizada** para `1.0`/`0.0` em vez de copiar o
//! número: é o que torna `value.math(Min)` um AND **exato**, e é o que impede um produtor
//! que um dia escrevesse `0.7` de vazar meia-verdade para dentro de uma máscara. O
//! `pulse.compare(rise = 0.5)` a lê de volta como borda ⇒ o ida-e-volta é a identidade.
//!
//! `Pure`: não lê o playhead, não carrega estado, preserva o comprimento.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// O tipo do pulso — evento discreto por instância (espelho do
/// `ph2d_node_pulse_beat::PULSE`; mantido local para esta crate seguir folha
/// drop-in — o vocabulário compartilhado é a PORTA, nunca um símbolo).
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
/// O tipo do valor — campo escalar contínuo por instância, na coluna `v`.
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// A coluna de disparo de um stream de pulso (`1.0` no tique em que dispara).
const PULSE_COL: &str = "pulse";
/// A coluna canônica do domínio de valor.
const VALUE_COL: &str = "v";

/// Acima disto o pulso já disparou. O mesmo limiar que o `pulse.counter` e o
/// `pulse.sample_hold` usam para ler a coluna — meio caminho entre os dois únicos
/// valores que a lei do pulso admite, então nenhum deles fica na fronteira.
const FIRED: f32 = 0.5;

/// O contrato estático deste tipo de nó (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.level"),
    name: "pulse.level",
    inputs: &[PortSpec {
        name: "pulse",
        ty: PULSE,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Pure: sem playhead, sem `pre`, sem memória. A saída é função do tique.
    effect: Effect::Pure,
    clock: Clock::Frame,
    // ZERO params — ver o doc do módulo: toggle e latch são `pulse.counter(2, …)`.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// Um tique: a máscara `0/1` do pulso, elemento a elemento.
fn level(pulse: &Stream) -> Stream {
    let n = pulse.count();
    let fired = match pulse.get(PULSE_COL) {
        Some(Column::Scalar(v)) => v.as_slice(),
        _ => &[],
    };
    let v = (0..n)
        .map(|i| {
            if fired.get(i).copied().unwrap_or(0.0) > FIRED {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    Stream::new(n).with(VALUE_COL, Column::Scalar(v))
}

struct PulseLevel;

impl NodeOp for PulseLevel {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let out = level(ctx.input(0));
        ctx.emit(out);
    }
}

/// Registra este nó no registry de runtime. Chamado (via codegen) do
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseLevel))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Level",
            // Cinza de utilidade: encanamento de pulso, não um transform visível.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    fn pulses(v: &[f32]) -> Stream {
        Stream::new(v.len()).with(PULSE_COL, Column::Scalar(v.to_vec()))
    }

    fn read(s: &Stream) -> Vec<f32> {
        match s.get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("o nó tem de emitir a coluna `{VALUE_COL}`"),
        }
    }

    /// FALSIFICAÇÃO da conversão: um tique que disparou vale 1, um que não
    /// disparou vale 0 — e os dois têm de estar no MESMO stream, senão um nó que
    /// devolvesse a constante 1 passaria.
    #[test]
    fn the_pulse_becomes_a_number() {
        assert_eq!(read(&level(&pulses(&[1.0, 0.0, 1.0]))), vec![1.0, 0.0, 1.0]);
    }

    /// A saída é **normalizada**, não copiada: a lei do pulso é *"disparou"*, e
    /// meia-verdade dentro de uma máscara faria do `value.math(Min)` um AND
    /// aproximado. Um produtor fora da lei é lido pelo limiar, nunca propagado.
    #[test]
    fn the_level_is_a_mask_not_the_number_the_producer_wrote() {
        assert_eq!(read(&level(&pulses(&[0.7, 0.3, 2.5]))), vec![1.0, 0.0, 1.0]);
    }

    /// O comprimento é preservado e a decisão é POR LINHA — a família dispara por
    /// instância (`pulse.on_change` já prova `[0,1]` para duas linhas), então um
    /// nível que colapsasse no primeiro elemento apagaria essa metade.
    #[test]
    fn it_decides_per_row_and_keeps_the_length() {
        let out = level(&pulses(&[0.0, 1.0, 0.0, 0.0]));
        assert_eq!(out.count(), 4);
        assert_eq!(read(&out), vec![0.0, 1.0, 0.0, 0.0]);
    }

    /// Uma porta desconectada é um stream VAZIO (`EvalCtx::input` está
    /// documentado como *"empty if unconnected"*), e o nó devolve comprimento
    /// zero em vez de inventar uma linha — o mesmo que o `pulse.counter` faz.
    #[test]
    fn an_unconnected_port_yields_nothing_not_a_phantom_row() {
        assert_eq!(level(&Stream::new(0)).count(), 0);
    }

    /// Um stream de pulso SEM a coluna (um produtor que emitiu só estado) lê
    /// como silêncio, no comprimento certo — nunca como pânico nem como disparo.
    #[test]
    fn a_stream_without_the_column_is_silence_at_full_length() {
        assert_eq!(read(&level(&Stream::new(3))), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
