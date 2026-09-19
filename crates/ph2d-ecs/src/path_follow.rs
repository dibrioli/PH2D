//! **O SEGUIDOR DE CAMINHO** (suplente #23) — o objecto anda sobre a curva que o artista desenhou.
//!
//! # ⛔⛔ O que este módulo NÃO tem, e porquê
//!
//! Ele **não tem relógio**, **não tem estado vivo** e **não tem geometria**.
//!
//! O relógio é o [`Timer`](crate::Timer) — a mesma medição que o [`crate::tween`] e o
//! `SequencePlayer` pagaram antes dele: duração · repetir · `autostart` · um sinal a arrancá-lo
//! ([`crate::SignalVerb::StartTimer`]) · e o [`rewind_runtime`](crate::rewind_runtime) a fazê-lo
//! **renascer**. ⇒ o seguidor é uma **FUNÇÃO PURA do relógio**, logo rebobinar já funciona.
//!
//! ⭐⭐⭐ **E a geometria não está aqui porque ela não PODE estar aqui, e isso foi MEDIDO** (§5.0 do
//! suplente #23): o `ph2d-ecs` não declara o `ph2d-vec-scene` nas `[dependencies]`, que é a
//! doutrina do [`VecPathRef`](crate::VecPathRef) — *«não põe geometria no ECS»*. O componente
//! guarda o **NOME** da forma; quem lê a curva é a ponte da família, que vê os dois lados.
//!
//! # ⚠️ UM por entidade, e não uma lista
//!
//! Ao contrário dos [`Tweens`](crate::Tweens), este componente é **único**: um objecto tem UMA
//! posição, e dois seguidores no mesmo corpo seriam duas respostas à mesma pergunta — com o
//! segundo a ler a pré-visualização do primeiro como se fosse o documento.
//!
//! # ⚠️ O NOME é a referência
//!
//! É a lei do [`stable_name_id`](crate::stable_name_id) escrita no `CLAUDE.md` §5 (*«referência
//! durável entre objetos é o NOME, nunca os bits»* — o undo respawna tudo com bits novos), e é a
//! mesma forma do [`CameraFollow::target`](crate::CameraFollow). ⭐ O `motion.path` desta casa
//! chegou **independentemente** à mesma resposta, e escreve-a no cabeçalho dele: *«o nome É a
//! referência, não uma consulta a uma»*.

use bevy_ecs::prelude::Component;
use ph2d_tween::{AoAcabar, Ciclo, Easing, Relogio};
use serde::{Deserialize, Serialize};

use crate::SimComponent;
use crate::timer::{Timer, TimerState};

/// **O seguidor de caminho de uma entidade** — o componente registado.
///
/// ⚠️ **É CONFIG inteira** — o que ele guarda é o que o artista escreveu. O *«onde está agora»* é
/// o relógio, que não é registado; é isso que o mantém fora do `Ctrl+Z` sem uma declaração.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathFollow {
    /// O **NOME** da forma vectorial a percorrer. ⚠️ **Vazio = inerte** (e calado: um objecto sem
    /// caminho não é um erro, é um componente acabado de anexar).
    pub caminho: String,
    /// Qual [`Timer`](crate::Timer) o faz andar — o índice na lista de relógios da entidade.
    ///
    /// ⛔ Um índice sem timer é **inerte**, e o painel di-lo em voz alta: a lei que o `TWEENS_MAX`
    /// já escreve (*um modelo que aceita o que não pode correr produz estado inalcançável*).
    pub relogio: u8,
    /// O que ele faz DENTRO de um período — [`Ciclo::PingPong`] é ir e voltar pela mesma curva.
    pub ciclo: Ciclo,
    /// A forma do andamento no tempo — o motor do `ph2d-anim`, nunca uma cópia dele.
    pub easing: Easing,
    /// O que fazer quando o relógio acaba.
    pub ao_acabar: AoAcabar,
    /// **Onde no percurso ele começa**, em fracção `0..1`.
    ///
    /// ⚠️ **DÁ A VOLTA** (`rem_euclid`), e isso não é escolha: é a lei que o `motion.path` desta
    /// casa já declara por escrito para o param do mesmo nome. Com `0` a saída é **byte-idêntica**
    /// a não existir campo nenhum.
    pub deslocamento: f32,
    /// **Roda para a direcção do caminho?** (o `rotates` do Godot, o `rotateToPath` do Phaser).
    pub alinha: bool,
    /// …e **mais este ângulo, em GRAUS**.
    ///
    /// ⭐ Ele existe porque a arte do artista não aponta necessariamente para `+X`: sem ele, uma
    /// nave desenhada a apontar para cima percorre a pista **de lado**. É o `rotationOffset` do
    /// Phaser, e é o único knob desta wave que nenhuma lei geométrica podia adivinhar.
    ///
    /// ⚠️ **GRAUS, e não radianos** — é a unidade AUTORADA deste app (o `FieldKind::Angle` do
    /// descritor declara-o por escrito), e a conversão vive num sítio só: a ponte.
    pub angulo: f32,
    /// Deslocamento **PERPENDICULAR** ao caminho, em metros — duas faixas na mesma pista.
    ///
    /// O `h_offset` do Godot. A normal é a tangente rodada um quarto de volta, logo o sinal diz o
    /// lado.
    pub lado: f32,
}

impl SimComponent for PathFollow {}

impl Default for PathFollow {
    fn default() -> Self {
        // ⚠️ **Alinhado por omissão**, e não parado a apontar para `+X`: um seguidor acabado de
        // anexar que desliza de lado lê-se como partido — a mesma lei que o `Tween::default`
        // (*«um fade-out linear, e não um tween INERTE»*) e o `Timer::default` já escrevem.
        Self {
            caminho: String::new(),
            relogio: 0,
            ciclo: Ciclo::Reinicia,
            easing: Easing::LINEAR,
            ao_acabar: AoAcabar::Hold,
            deslocamento: 0.0,
            alinha: true,
            angulo: 0.0,
            lado: 0.0,
        }
    }
}

/// ⭐⭐⭐ **A FRACÇÃO do percurso que este seguidor pede NESTE quadro.** `None` = ele não escreve.
///
/// ⚠️ **Ela devolve uma fracção e nunca um arco** — o comprimento da curva vive do outro lado da
/// fronteira, e misturar os dois aqui obrigaria a fundação a conhecer a geometria.
///
/// ⚠️ **A volta vem DEPOIS da curva**: o `easing` diz *quando* se anda, o deslocamento diz *onde se
/// entrou na pista*. Ao contrário, uma curva não-linear deformaria o ponto de entrada em vez do
/// andamento.
///
/// ⛔⛔ **E a volta NÃO é um `rem_euclid` cru — o gate red-first apanhou-o:** `1,0.rem_euclid(1,0)`
/// é **`0,0`**, logo um *one-shot* que chega ao fim com `AoAcabar::Hold` **teletransportava-se para
/// o princípio da pista** em vez de descansar na ponta. ⇒ a lei tira **uma** volta ao que passa de
/// `1`, e `1` continua a ser `1`: com o deslocamento normalizado a `[0, 1)` e o andamento em
/// `[0, 1]`, a soma vive em `[0, 2)` e uma subtracção chega.
#[must_use]
pub fn fraccao_de(pf: &PathFollow, timer: &Timer, estado: &TimerState) -> Option<f64> {
    let u = ph2d_tween::andamento(relogio_de(timer, estado), pf.ciclo, pf.easing, pf.ao_acabar)?;
    let s = u + f64::from(pf.deslocamento).rem_euclid(1.0);
    Some(if s > 1.0 { s - 1.0 } else { s })
}

/// **O relógio, como a lei do [`ph2d_tween`] o vê** — a porta que traduz, e a MESMA do
/// [`crate::tween`].
///
/// ⚠️ Ela é re-exportada daqui em vez de reescrita: o dia em que o `acabou` mudasse de sentido
/// deixaria as duas a discordar em silêncio.
#[must_use]
pub fn relogio_de(timer: &Timer, estado: &TimerState) -> Relogio {
    crate::tween::relogio_de(timer, estado)
}

/// **O que os seguidores da cena pedem neste quadro** — a entidade, a fracção, e a config.
///
/// ⚠️ **Um seguidor sem timer no índice dele sai da lista**, e ⛔ *não* cai no timer `0`: correr no
/// relógio errado lê-se como um defeito do motor, e não como uma lista curta.
///
/// ⚠️ **Um caminho VAZIO também sai** — sem nome não há curva a resolver, e deixá-lo entrar faria
/// a ponte pagar uma busca por nome em toda entidade acabada de anexar.
#[must_use]
pub fn a_seguir(world: &mut bevy_ecs::world::World) -> Vec<Percurso> {
    let mut q = world.query::<(
        bevy_ecs::entity::Entity,
        &PathFollow,
        &crate::Timers,
        &crate::TimerRuntime,
    )>();
    let mut fora = Vec::new();
    for (entity, pf, cfg, rt) in q.iter(world) {
        if pf.caminho.is_empty() {
            continue;
        }
        let i = usize::from(pf.relogio);
        let (Some(timer), Some(estado)) = (cfg.0.get(i), rt.0.get(i)) else {
            continue;
        };
        if let Some(fraccao) = fraccao_de(pf, timer, estado) {
            fora.push(Percurso {
                entity,
                fraccao,
                caminho: pf.caminho.clone(),
                alinha: pf.alinha,
                angulo: pf.angulo,
                lado: pf.lado,
            });
        }
    }
    fora
}

/// **Um percurso pedido neste quadro** — o que a ponte precisa para resolver a curva.
#[derive(Clone, Debug, PartialEq)]
pub struct Percurso {
    /// Quem anda.
    pub entity: bevy_ecs::entity::Entity,
    /// Onde no percurso, em fracção `0..1`.
    pub fraccao: f64,
    /// O nome da forma a percorrer.
    pub caminho: String,
    /// Roda para a tangente?
    pub alinha: bool,
    /// O ângulo somado à tangente, em radianos.
    pub angulo: f32,
    /// O deslocamento perpendicular, em metros.
    pub lado: f32,
}

#[cfg(test)]
#[path = "path_follow_tests.rs"]
mod tests;
