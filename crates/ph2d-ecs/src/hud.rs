//! **O HUD: o placar, a vida e o menu** (TOP-20 #20) — os quatro componentes.
//!
//! O que o artista monta: uma entidade com [`UiCanvas`] (a RAIZ, que se cola à vista do jogo),
//! filhos com [`UiLabel`] (o número que muda) e [`UiButton`] (o clique que publica um sinal), e um
//! [`Counter`] algures na cena para haver o que mostrar.
//!
//! # ⛔ O que este módulo NÃO traz, e a ausência é a wave inteira
//!
//! **Não há aqui uma âncora, um fluxo, um texto nem um estado de botão** — as quatro coisas já
//! existem nesta casa e foram medidas antes da primeira linha
//! ([plano](../../../docs/Components/15_plano_hud.md) §1):
//!
//! * prender aos cantos é o [`crate::VecAnchors`], e o doc do passe vivo dele **nomeia o HUD como
//!   o caso de uso**: *«É o HUD: a vida no canto de cima, a pontuação colada no canto oposto»*;
//! * empilhar é o [`crate::VecLayout`] (taffy, ADR-0153);
//! * o texto é o [`crate::VecShape::Text`];
//! * os estados visuais são a `ph2d-ui-state`.
//!
//! *Construir uma segunda lei de âncora teria sido a forma mais cara de ignorar a §5.0.*
//!
//! # A pose da raiz é CONDUZIDA, e por isso não entra no ficheiro
//!
//! O `Transform` de um [`UiCanvas`] é reescrito a cada quadro pela fase do HUD (é o que o cola à
//! vista). Isso passa pelo ledger do `ph2d-preview-drive` — a mesma lei do solver, do script e da
//! máquina de estados: *o documento é o valor AUTORADO; o que um motor escreve agora é
//! pré-visualização — vê-se, não se guarda nem se desfaz.*

use bevy_ecs::component::Component;
use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};

use crate::SimComponent;
use crate::timer::{TimerRuntime, Timers};

pub use ph2d_hud::Fit;

/// **A RAIZ do HUD** — a caixa em que o artista desenhou, e como ela se acomoda na vista do jogo.
///
/// ⚠️ **A caixa é CENTRADA na entidade** (mundo Y-up, poses no centro — o idioma desta casa), e é
/// isso que faz o centramento do [`Fit::Keep`] não custar aritmética nenhuma: a translação é o
/// centro da vista, e mais nada. A lei vive na folha [`ph2d_hud`], medida contra o oráculo.
///
/// ⚠️ **Sem câmera de jogo na cena, o canvas fica onde o artista o pôs** — e o painel di-lo em voz
/// alta. *Um HUD que salta para um sítio arbitrário porque ninguém definiu a vista é pior do que
/// um que não se move.*
#[derive(Component, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct UiCanvas {
    /// Largura da caixa de referência, em unidades de mundo.
    pub ref_w: f32,
    /// Altura da caixa de referência.
    pub ref_h: f32,
    /// Ver [`Fit`].
    pub fit: Fit,
}

impl Default for UiCanvas {
    /// `32 × 18` — a caixa de `16:9` que a cena de fábrica deste repo usa, e o `Keep` do alvo.
    fn default() -> Self {
        Self {
            ref_w: 32.0,
            ref_h: 18.0,
            fit: Fit::Keep,
        }
    }
}

impl SimComponent for UiCanvas {}

/// **De onde vem o número que um rótulo mostra.**
///
/// ⚠️ **Todas as fontes são somas ou extremos, nunca «o primeiro»** — um mundo ECS não promete
/// ordem de iteração entre arquétipos (a wave da máquina de estados pagou isso com o dedo do dono
/// a acertar na placa errada), e *«o primeiro»* leria um valor diferente conforme a ordem de
/// criação. Uma soma de um único contador É o valor dele.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum LabelSource {
    /// O texto que está no documento — o NEUTRO: nada é derivado e o desenho é byte-idêntico.
    #[default]
    Authored,
    /// A **soma** de todos os [`Counter`] com este nome.
    Counter(String),
    /// O **menor** tempo que falta, em segundos, entre os `Timer` com este nome (o que vai tocar
    /// primeiro). ⛔ Somar tempos que correm em paralelo não significa nada.
    TimerLeft(String),
    /// Quantos objectos carregam esta etiqueta (TOP-20 #9).
    TagCount(String),
}

/// **Um rótulo cujo texto o JOGO muda.**
///
/// ⚠️ **O documento não é reescrito.** A string derivada é cozida em glyphs para a `LiveGeometry`
/// do quadro — a lei do ADR-0153, *o passe publica o que a coisa MOSTRA; ele não escreve o que ela
/// É*. Com [`LabelSource::Authored`] nada é derivado e o caminho é o de sempre, ao bit.
#[derive(Component, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct UiLabel {
    /// Ver [`LabelSource`].
    pub source: LabelSource,
    /// O que vem antes do número (`"Pontos: "`).
    pub prefix: String,
    /// O que vem depois (`" s"`).
    pub suffix: String,
}

impl SimComponent for UiLabel {}

/// **Um botão que publica um sinal** — o idioma do ADR-0075: o produtor não chama ninguém.
///
/// A lei do clique é a do oráculo (bloco L3): dispara **uma vez, ao LARGAR**, e só se o carregar
/// **e** o largar caírem dentro; desactivado nunca dispara.
#[derive(Component, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct UiButton {
    /// O nome do sinal. ⚠️ **Em branco não é um sinal** — a mesma regra do `SignalOnHit` e do
    /// marcador da timeline: um nome em branco não é um contrato que alguém possa casar.
    pub signal: String,
    /// Recusa o clique, em silêncio e por desenho.
    pub disabled: bool,
}

impl UiButton {
    /// O nome, ou `None` se estiver em branco.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        let t = self.signal.trim();
        (!t.is_empty()).then_some(t)
    }
}

impl SimComponent for UiButton {}

/// **Um número que o jogo soma** — os pontos, as vidas, as chaves.
///
/// Quem o move é o verbo `AddToCounter` da tabela do TOP-20 #5, logo *«bateu na moeda → soma 1»*
/// fecha com o `SignalOnHit` que a física já publica, sem uma linha de código do artista.
///
/// ⚠️ **`i64` e não `f32`:** um placar é uma contagem, e somar `0,1` trezentas vezes em vírgula
/// flutuante não dá `30`. *O tipo é a lei.*
///
/// ⛔⛔ **O valor VIVO não está aqui, e a ausência é a lei da casa.** Este componente é CONFIG e
/// grava-se; o número que a corrida mexe vive no [`CounterRuntime`], que **não deriva
/// `Serialize`** — logo registá-lo nem compila. É o precedente exacto do `Timer`/`TimerRuntime`,
/// do `Factory`/`FactoryRuntime` e do `StateMachine`/`StateMachineRuntime`, e sem ele **cada ponto
/// marcado seria um passo de `Ctrl+Z`** e ficaria dentro do ficheiro gravado.
#[derive(Component, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Counter {
    /// O nome pelo qual um rótulo e a tabela de acções lhe chegam.
    pub name: String,
    /// O valor com que ele começa **e com que renasce** ao rebobinar.
    pub start: i64,
}

impl SimComponent for Counter {}

/// **O valor VIVO de um contador** — o que a corrida mexe, e o que o ficheiro nunca vê.
///
/// ⚠️ **Nascer aqui é `start`, e não `Default`** ([`crate::rewind_runtime`]): é a quinta espécie de
/// «nascer» daquela tabela, e a primeira que **lê a config** em vez de uma constante. Um `Default`
/// poria toda a gente a zero e apagaria em silêncio um contador que o artista quis começar em três
/// vidas.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CounterRuntime {
    /// O valor de agora.
    pub value: i64,
}

/// **O que este rótulo mostra AGORA**, ou `None` se não houver nada derivado (o
/// [`LabelSource::Authored`], que é o neutro) ou se a fonte não existir na cena.
///
/// ⚠️ **`None` e «zero» são coisas diferentes, e a distinção é visível:** um rótulo preso a um
/// contador que ninguém criou tem de continuar a mostrar o que o artista escreveu — e não um `0`
/// inventado, que se leria como *«o jogo está a funcionar e a pontuação é zero»*.
#[must_use]
pub fn valor(
    world: &mut World,
    tree: &ph2d_tags::TagTree,
    label: &UiLabel,
) -> Option<ph2d_hud::Valor> {
    match &label.source {
        LabelSource::Authored => None,
        // ⚠️ SOMA, e não «o primeiro»: a ordem de iteração entre arquétipos não é prometida.
        LabelSource::Counter(nome) => {
            let alvo = nome.trim();
            if alvo.is_empty() {
                return None;
            }
            let mut achou = false;
            let mut total: i64 = 0;
            let mut q = world.query::<(&Counter, &CounterRuntime)>();
            for (cfg, rt) in q.iter(world) {
                if cfg.name.trim() == alvo {
                    achou = true;
                    total = total.saturating_add(rt.value);
                }
            }
            achou.then_some(ph2d_hud::Valor::Inteiro(total))
        }
        // ⚠️ O MENOR tempo que falta — o relógio que vai tocar primeiro. ⛔ Somar tempos que
        // correm em paralelo não significa nada.
        LabelSource::TimerLeft(nome) => {
            let alvo = nome.trim();
            if alvo.is_empty() {
                return None;
            }
            let mut menor: Option<u64> = None;
            let mut q = world.query::<(&Timers, &TimerRuntime)>();
            for (timers, rt) in q.iter(world) {
                for (t, st) in timers.0.iter().zip(rt.0.iter()) {
                    if t.name.trim() == alvo {
                        let falta = t.duration_us.saturating_sub(st.elapsed_us);
                        menor = Some(menor.map_or(falta, |m: u64| m.min(falta)));
                    }
                }
            }
            #[allow(clippy::cast_precision_loss)]
            menor.map(|us| ph2d_hud::Valor::Segundos(us as f32 / 1_000_000.0))
        }
        // ⚠️ Uma etiqueta que não existe na árvore devolve `None`, e não `0`: *«ninguém tem esta
        // etiqueta»* e *«esta etiqueta não existe»* são dois factos, e o segundo é um erro de
        // autoria que o artista tem de poder ver.
        LabelSource::TagCount(caminho) => {
            let id = tree.find(caminho.trim())?;
            let n = crate::tags::tagged(world, tree, id).len();
            i64::try_from(n).ok().map(ph2d_hud::Valor::Inteiro)
        }
    }
}

/// **A linha inteira que o rótulo mostra** — prefixo + o número + sufixo.
///
/// `None` quando não há nada derivado: quem chama deixa o texto AUTORADO no lugar.
#[must_use]
pub fn texto(world: &mut World, tree: &ph2d_tags::TagTree, label: &UiLabel) -> Option<String> {
    let v = valor(world, tree, label)?;
    Some(format!(
        "{}{}{}",
        label.prefix,
        ph2d_hud::formata(v),
        label.suffix
    ))
}

#[cfg(test)]
#[path = "hud_tests.rs"]
mod tests;
