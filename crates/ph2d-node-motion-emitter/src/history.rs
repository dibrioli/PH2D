//! **A HISTÓRIA DO EMISSOR** — o que a partícula guarda do movimento da fonte
//! ([ADR-0163](../../../docs/architecture/decisions/0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md);
//! doc 89, folha 01, o P1).
//!
//! Uma partícula nasce onde o emissor ESTAVA e parte com a velocidade que ele
//! tinha — o que Cavalry (*Use Emitter Velocity*) e Niagara (*Inherit Velocity*)
//! fazem, e o que este nó não podia fazer enquanto não soubesse o próprio
//! passado.
//!
//! Separado do `lib.rs` pelo tecto de LOC (HR-18), na costura que a própria
//! capacidade desenha: lá fica o emissor, aqui a memória dele.

use super::{MANIFEST, NodeOp};

/// A chave do param **do que a partícula guarda do movimento do emissor**.
pub const MOTION: &str = "emitter_motion";
/// A chave do param **de quanto da velocidade do emissor ela leva**.
pub const INHERIT: &str = "inherit";

/// Os três modos de [`MOTION`], na ordem em que o número os indexa.
///
/// - **`Carry`** (o default) — o penacho anda com o emissor. É o que sempre
///   houve, **ao bit**, e não é um bug com nome bonito: um efeito ANEXADO (a
///   chama que anda com a tocha) quer exactamente isto.
/// - **`Leave`** — a partícula fica onde nasceu. É a base de toda referência
///   (Cavalry, Niagara): arrastar um emissor deixa um rasto, e não carrega o
///   penacho.
/// - **`Inherit`** — fica onde nasceu **e** parte com a velocidade que o emissor
///   tinha nesse instante (Cavalry *Use Emitter Velocity*, Niagara *Inherit
///   Velocity*).
pub const MOTION_LABELS: &[&str] = &["Carry", "Leave", "Inherit"];

/// **A HISTÓRIA da origem: quantas amostras por segundo o leque pede.**
///
/// ⚠️ **É uma taxa, e não uma contagem — de propósito.** Uma contagem fixa
/// repartida pela vida faria a resolução PIORAR quando o artista alonga a vida
/// das partículas, que é o oposto do que ele pediu. Uma taxa mantém o passo.
///
/// **240 Hz é MEDIDO contra a referência que se quer bater**: um motor com estado
/// amostra a posição do emissor **uma vez por quadro** (60 Hz), então quatro
/// vezes isso já é estritamente melhor do que aquilo que se está a imitar.
const HISTORY_HZ: f32 = 240.0;

/// **O tecto de amostras da história**, e ele nomeia o recurso: **TEMPO**.
///
/// MEDIDO (`custo_de_uma_fatia`, release): uma fatia de leque — uma cozedura da
/// sub-árvore que dirige a origem — custa **~300-490 ns**. Daí:
///
/// | fatias | ms/quadro | % de um quadro de 16,7 |
/// |---|---|---|
/// | 512 | 0,168 | **1,0 %** |
/// | **1024** | **0,435** | **2,6 %** |
/// | 2048 | 0,913 | 5,5 % |
/// | 4096 | 2,005 | 12,0 % |
///
/// **1024** é onde isto para: 2,6% de um quadro por um knob OPCIONAL de um nó é o
/// que «fácil de usar» tolera; 5,5% já não é. Com a taxa acima, o tecto só morde
/// a partir de `life > 4,27 s` — e aí a resolução degrada suavemente em vez de o
/// custo explodir.
const MAX_HISTORY: usize = 1024;

/// Quantas amostras a história de uma vida de `life` segundos pede.
#[must_use]
pub fn history_samples(life: f32) -> usize {
    // `<=` e não `!(> 0)`: a comparação negada sobre um tipo parcialmente
    // ordenado esconde o caso `NaN`, e aqui ele existe (um `life` dirigido).
    // `NaN <= 0` é `false`, então o guarda é o mesmo — escrito de forma legível.
    if life.is_nan() || life <= 0.0 {
        return 0;
    }
    // `+1` porque N intervalos são N+1 amostras — o instante `t` e o instante
    // `t − life` têm os dois de estar lá.
    (((life * HISTORY_HZ).ceil() as usize) + 1).clamp(2, MAX_HISTORY)
}

/// Os INSTANTES da história, em deslocamento (segundos, ≤ 0) a partir de agora —
/// do mais VELHO para o AGORA.
///
/// ⚠️ **Esta é a lei, e ela tem dois leitores** — o `time_fans`, que a converte
/// nos mapas que o cook aplica, e o `emit`, que dela deriva em que fatia cada
/// partícula nasceu. Escrever a escada duas vezes poria a partícula num sítio e a
/// velocidade noutro.
#[must_use]
pub fn history_offsets(life: f32) -> Vec<f32> {
    let n = history_samples(life);
    if n == 0 {
        return Vec::new();
    }
    let last = (n - 1) as f32;
    (0..n).map(|j| -life * (last - j as f32) / last).collect()
}

/// **Onde o emissor ESTAVA no instante `birth`, e com que velocidade** — lido da
/// história por interpolação linear entre as duas amostras que o cercam.
///
/// ⚠️ **Interpolar aqui é reconstruir, não aproximar por conveniência.** A origem
/// É uma função contínua do tempo; a história são amostras dela a
/// [`HISTORY_HZ`], e ler entre duas amostras é a leitura certa dessa função. O
/// erro é de amostragem (limitado pela curvatura × passo), não de modelo — e o
/// passo é **quatro vezes mais fino** que o quadro em que a referência amostra.
///
/// ⚠️ **A velocidade sai da MESMA vizinhança**, não de um par escolhido à parte:
/// dois números que dizem *onde* e *quão depressa* têm de vir do mesmo sítio da
/// curva, senão a partícula parte de um ponto com a velocidade de outro.
pub(super) fn history_at(
    history: &[[f32; 2]],
    life: f32,
    age: f32,
) -> Option<([f32; 2], [f32; 2])> {
    let n = history.len();
    if n < 2 || life.is_nan() || life <= 0.0 {
        return None;
    }
    let last = (n - 1) as f32;
    // `age = 0` é o AGORA (a última amostra); `age = life` é a mais velha.
    let u = ((life - age.clamp(0.0, life)) / life * last).clamp(0.0, last);
    let i = (u.floor() as usize).min(n - 2);
    let f = u - i as f32;
    let (a, b) = (history[i], history[i + 1]);
    let pos = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
    // O passo entre amostras, em segundos — a régua da derivada.
    let dt = life / last;
    let vel = [(b[0] - a[0]) / dt, (b[1] - a[1]) / dt];
    Some((pos, vel))
}

/// **OS LEQUES DE TEMPO deste nó** (`ph2d_nodegraph::cook::TimeFans`, ADR-0163) —
/// a HISTÓRIA da origem, para uma partícula poder nascer onde o emissor estava.
///
/// Espelho do `ph2d_node_motion_trail::time_fans`: o substrato chaveia por
/// `NodeId` e não conhece tipo nenhum, então quem sabe o que `motion.emitter` é
/// somos nós.
///
/// ⚠️ **Vazio em três casos, e os três são o mesmo facto — «não há história»:**
/// no modo `Carry`, quando `life ≤ 0`, e **quando nem `x` nem `y` são dirigidos
/// por fio**. O último é o que importa para o custo: uma origem parada tem a
/// mesma posição em todo instante, e as três respostas coincidem *por
/// aritmética*. Um leque ali seria `N` cozeduras para reler o mesmo número.
///
/// ⚠️ **`life` é lido do override/default**, como o `time_scopes` lê os dele: um
/// `life` DIRIGIDO muda a janela por tique e o leque seguiria um quadro atrás.
/// Nomeado em vez de descoberto — a cura é o leque saber ler o próprio param
/// dirigido, que é uma recursão que o substrato não tem.
#[must_use]
pub fn time_fans(
    graph: &ph2d_nodegraph::graph::Graph,
    ops: &dyn ph2d_nodegraph::cook::OpResolver,
    _tick_seconds: f64,
) -> ph2d_nodegraph::cook::TimeFans {
    use ph2d_nodegraph::time::{TimeMap, TimeMode};
    let mut fans = ph2d_nodegraph::cook::TimeFans::new();
    for inst in graph.nodes() {
        if inst.type_name != MANIFEST.name {
            continue;
        }
        let Some(manifest) = ops.resolve(inst.type_id()).map(NodeOp::manifest) else {
            continue;
        };
        let overrides = graph.node_param_overrides(inst.id);
        let p = |name: &str| {
            overrides
                .and_then(|o| o.get(name).copied())
                .or_else(|| manifest.param_default(name))
                .unwrap_or(0.0)
        };
        if p(MOTION) < 0.5 {
            continue;
        }
        let driven = graph
            .param_sources(inst.id)
            .is_some_and(|s| s.contains_key("x") || s.contains_key("y"));
        if !driven {
            continue;
        }
        let maps: Vec<TimeMap> = history_offsets(p("life"))
            .into_iter()
            .map(|dt| TimeMap {
                mode: TimeMode::Scale,
                scale: 1.0,
                offset: f64::from(dt),
                ..TimeMap::default()
            })
            .collect();
        if maps.len() >= 2 {
            fans.insert(inst.id, maps);
        }
    }
    fans
}
