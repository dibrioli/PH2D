//! **AS ÂNCORAS** — a regra de ancoragem no *resize*, para o filho que NÃO está num fluxo (plano
//! UI/UX W3).
//!
//! É a outra metade da responsividade. O [`crate::VecLayout`] responde *"como estes filhos se
//! empilham?"*; isto responde *"quando a moldura muda de tamanho, para onde vai este filho que eu
//! coloquei à mão?"* — a pontuação que gruda no canto direito, a barra que estica com a tela.
//!
//! # A escolha: âncoras PROCEDURAIS, e não um solver
//!
//! ⚠️ Cassowary (o simplex de restrições do AutoLayout da Apple) fica **FORA**, com a cerca
//! escrita. O caso que justificaria um solver é o RELACIONAL (*"alinhe A com B"*), e ele já tem
//! resposta neste app — o align/distribute e o snap de reivindicação 2-D. Para o resto, a regra
//! é aritmética fechada, e uma aritmética fechada não tem passo de convergência para gastar num
//! `advance(dt)` de runtime.
//!
//! # O modelo é o da Unity, com uma diferença que decide a wave
//!
//! A Unity guarda **âncora normalizada** (`min`/`max` em `0..1` dentro da caixa do pai) **mais os
//! OFFSETS** do filho em píxeis. O plano previa os dois. ⚠️ **Os offsets NÃO estão aqui, e a
//! ausência é a decisão inteira:** guardá-los tornaria a pose do filho uma função só do
//! componente, e **arrastar o filho deixaria de fazer alguma coisa** — o passe sobrescreveria o
//! gesto a cada frame, que é exactamente o controlo morto que a política de UI deste repo existe
//! para impedir. Aqui o offset é **DERIVADO** por frame da caixa que o filho de facto tem, então o
//! arrasto continua a ser a autoria da posição, como em toda a outra parte do editor.
//!
//! # O que fica guardado é a RÉGUA: a moldura contra a qual a regra foi autorada
//!
//! [`VecAnchors::base`] é a caixa **LOCAL** da moldura no instante em que o artista armou a regra
//! — o *Capture Base State* do Rive, o `rest` que cada binding da timeline guarda. Sem ela não há
//! *"mudou de tamanho"* nenhum: a moldura só tem UM tamanho (o `w`/`h` do retângulo vivo que ela
//! é), e comparar um número consigo próprio nunca dá delta.
//!
//! ⚠️ Ela é **LOCAL**, e não de mundo: mover a moldura, ou mover um ancestral dela, muda a caixa
//! de MUNDO sem que nada tenha sido redimensionado — e uma régua de mundo tornaria cada arrasto
//! da moldura num *resize* fantasma que empurraria os filhos ancorados para longe.
//!
//! # A lei, herdada do ADR-0153
//!
//! > **O passe publica ONDE as coisas ficam. Ele não escreve ONDE elas estão.**
//!
//! O resultado é uma pose derivada, publicada por frame. Nada aqui toca `Transform` — o undo deste
//! editor é por DIFF do mundo ECS, e um passe que escrevesse a pose faria cada frame de um
//! redimensionamento virar um passo de undo.
//!
//! # Componente NOVO não bumpa nada
//!
//! Ele cunha `stable_type_id = blake3(NOME)[..8]` próprio ⇒ **zero bump de `PROJECT_SCHEMA`** — o
//! precedente literal do [`crate::VecLayout`] (W2), do [`crate::VecFrame`] (W0) e do
//! `PhysicsJoint`. Ausência do componente = o mundo é byte-idêntico ao de antes desta feature.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A regra de ancoragem de UM filho.**
///
/// [`Self::min`] e [`Self::max`] são a fracção da caixa da moldura que cada ponta do filho segue,
/// em `0..1`: **`0` é a aresta MÍNIMA** (esquerda, e — como o mundo é Y-up — a de BAIXO) e **`1` a
/// MÁXIMA** (direita, topo). Os quatro casos que o painel oferece saem todos daqui:
///
/// | `min` | `max` | o que faz |
/// |---|---|---|
/// | `0` | `0` | fica colado na aresta mínima (o neutro) |
/// | `1` | `1` | gruda na aresta máxima |
/// | `0.5` | `0.5` | fica no meio |
/// | `0` | `1` | **estica**: uma ponta fica, a outra acompanha |
///
/// ⚠️ O modelo é mais largo do que a UI de propósito (um `0.25` é exprimível e ninguém o oferece
/// hoje): é o mesmo par *motor mais largo que o catálogo* do perfil de largura, e é o que faz
/// âncoras proporcionais nascerem sem tocar no schema. Um par que a tabela de chips não conhece
/// simplesmente **não acende chip nenhum**, que é a verdade.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecAnchors {
    /// A fracção que a ponta MÍNIMA do filho segue, por eixo.
    pub min: [f64; 2],
    /// A fracção que a ponta MÁXIMA do filho segue, por eixo.
    pub max: [f64; 2],
    /// **A caixa LOCAL da moldura quando a regra foi armada** — `[x0, y0, x1, y1]`.
    ///
    /// A régua contra a qual o *"mudou de tamanho"* é medido. Ver o cabeçalho do módulo.
    pub base: [f64; 4],
}

impl SimComponent for VecAnchors {}

/// A caixa local `[x0, y0, x1, y1]` de uma moldura.
type Box2 = [f64; 4];

impl VecAnchors {
    /// **A regra recém-armada**: colada na aresta mínima, com a régua tirada da moldura de AGORA.
    ///
    /// Nascer no neutro é deliberado — armar uma âncora não pode mover a arte no clique que a
    /// arma. O artista escolhe a aresta a seguir logo a seguir, e aí o delta aparece.
    #[must_use]
    pub fn armed(now: Box2) -> Self {
        Self {
            min: [0.0, 0.0],
            max: [0.0, 0.0],
            base: now,
        }
    }

    // ⚠️ **O `is_neutral` MORREU, e a cerca que ele carregava disparou** (report do Enio,
    // 2026-08-03: *"Constrain Left e Bottom pararam de funcionar"*).
    //
    // Ele existia para DESTACAR a regra neutra — *"uma regra que não move nada não tem por que
    // viajar em todo save"* — e o próprio doc dele nomeava o gatilho que a revogava: isso
    // *"coincide com a ausência do componente **enquanto a aresta mínima da moldura ficar
    // parada**"*, garantido pelo `w`/`h` do painel, que escala em torno do canto mínimo, e
    // *"uma alça futura que cresça a moldura para a ESQUERDA quebra a coincidência"*.
    //
    // A alça chegou. `Left` e `Bottom` SÃO as duas arestas mínimas, então escolher as duas
    // produzia exactamente a regra que o destaque apagava — e o filho ficava sem regra
    // nenhuma, que não é a mesma coisa: **sem regra ele não anda; com a regra neutra ele anda
    // COM a aresta mínima**. As duas só coincidiam enquanto aquela aresta estivesse presa.
    //
    // A régua (`base`) é o que se perdia junto, e é ela a carga útil: sem um `base` guardado
    // não há contra o que medir *"quanto a moldura mudou"*, então a regra neutra **tem de ser
    // gravada como qualquer outra**. Um destaque que apaga a régua não economiza um no-op,
    // apaga a única coisa que a regra sabe.

    /// **O deslocamento das duas pontas do filho, em unidades LOCAIS da moldura** — `[dmin, dmax]`.
    ///
    /// É a wave inteira em duas linhas. Sendo `d` o quanto a aresta mínima da moldura andou e `g` o
    /// quanto ela cresceu, a ponta que segue a fracção `a` anda `d + a·g`:
    ///
    /// - âncora `0`: anda com a aresta mínima;
    /// - âncora `1`: anda com a aresta mínima **e** com todo o crescimento — ou seja, com a aresta
    ///   máxima;
    /// - `min = 0`, `max = 1`: a ponta de baixo fica e a de cima acompanha ⇒ **estica**.
    ///
    /// ⚠️ **Moldura intocada dá zero EXACTO**, e não *"quase zero"*: `now == base` faz as duas
    /// subtracções darem `0.0` em IEEE-754 seja qual for a escala a jusante. É isso que mantém o
    /// mundo pré-âncora byte-idêntico — o afim que sai daqui é a identidade, e o passe nem paga a
    /// cópia da geometria.
    #[must_use]
    pub fn delta_local(&self, now: Box2) -> [[f64; 2]; 2] {
        let mut dmin = [0.0; 2];
        let mut dmax = [0.0; 2];
        for i in 0..2 {
            // Quanto a aresta MÍNIMA andou, e quanto a moldura CRESCEU, neste eixo.
            let moved = now[i] - self.base[i];
            let grew = (now[i + 2] - now[i]) - (self.base[i + 2] - self.base[i]);
            dmin[i] = moved + self.min[i] * grew;
            dmax[i] = moved + self.max[i] * grew;
        }
        [dmin, dmax]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A moldura de referência: 10 de largura, 4 de altura, canto mínimo na origem.
    const BASE: Box2 = [0.0, 0.0, 10.0, 4.0];

    fn rule(min: [f64; 2], max: [f64; 2]) -> VecAnchors {
        VecAnchors {
            min,
            max,
            base: BASE,
        }
    }

    /// **O controlo, e ele vem primeiro**: sem redimensionar, TODA regra dá zero — e zero exacto.
    ///
    /// Sem esta metade, um gate que só medisse o caso esticado ficaria verde sobre um passe que
    /// empurra a arte toda no primeiro frame.
    #[test]
    fn an_untouched_frame_moves_nothing_whatever_the_rule() {
        for (mn, mx) in [
            ([0.0, 0.0], [0.0, 0.0]),
            ([1.0, 1.0], [1.0, 1.0]),
            ([0.5, 0.5], [0.5, 0.5]),
            ([0.0, 0.0], [1.0, 1.0]),
        ] {
            let [dmin, dmax] = rule(mn, mx).delta_local(BASE);
            assert_eq!(dmin, [0.0, 0.0], "min de {mn:?}/{mx:?}");
            assert_eq!(dmax, [0.0, 0.0], "max de {mn:?}/{mx:?}");
        }
    }

    /// Alargar 10 para 16 pela DIREITA: quem segue a aresta máxima anda os 6 inteiros, quem segue
    /// a mínima não anda, e o meio anda 3.
    #[test]
    fn growing_to_the_right_moves_each_rule_by_its_share() {
        let now = [0.0, 0.0, 16.0, 4.0];
        assert_eq!(rule([1.0, 0.0], [1.0, 0.0]).delta_local(now)[0][0], 6.0);
        assert_eq!(rule([0.0, 0.0], [0.0, 0.0]).delta_local(now)[0][0], 0.0);
        assert_eq!(rule([0.5, 0.0], [0.5, 0.0]).delta_local(now)[0][0], 3.0);
    }

    /// **Esticar é o par de pontas a discordar**: a de baixo fica, a de cima leva o crescimento
    /// inteiro. Um gate que só olhasse UMA ponta não distinguiria esticar de grudar.
    #[test]
    fn a_stretched_child_keeps_one_end_and_carries_the_other() {
        let [dmin, dmax] = rule([0.0, 0.0], [1.0, 1.0]).delta_local([0.0, 0.0, 16.0, 9.0]);
        assert_eq!(dmin, [0.0, 0.0]);
        assert_eq!(dmax, [6.0, 5.0]);
    }

    /// **A aresta MÍNIMA a andar é um caso próprio, e é por ele que a régua guarda a caixa toda e
    /// não só o tamanho.** A moldura cresce para a ESQUERDA (o canto mínimo recua, o máximo fica):
    /// quem segue a máxima não pode andar, e quem segue a mínima anda com ela.
    #[test]
    fn growing_leftwards_moves_the_min_follower_and_pins_the_max_one() {
        let now = [-6.0, 0.0, 10.0, 4.0]; // mesma aresta direita, 6 a mais de largura
        assert_eq!(rule([1.0, 0.0], [1.0, 0.0]).delta_local(now)[0][0], 0.0);
        assert_eq!(rule([0.0, 0.0], [0.0, 0.0]).delta_local(now)[0][0], -6.0);
    }

    /// Os dois eixos são independentes: mexer na largura não mexe em quem segue o eixo Y.
    #[test]
    fn the_axes_do_not_talk_to_each_other() {
        let [dmin, _] = rule([1.0, 1.0], [1.0, 1.0]).delta_local([0.0, 0.0, 16.0, 4.0]);
        assert_eq!(dmin, [6.0, 0.0]);
    }

    /// A regra recém-armada cola-se na aresta mínima, e a régua dela é a moldura de agora.
    #[test]
    fn an_armed_rule_hugs_the_min_edge_and_remembers_the_frame_it_saw() {
        let a = VecAnchors::armed(BASE);
        assert_eq!([a.min, a.max], [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(a.base, BASE);
    }

    /// **A regra NEUTRA anda com a aresta mínima — ela não é a ausência de regra.**
    ///
    /// ⚠️ É o gate que a cerca do `is_neutral` pedia e que ninguém tinha escrito: as duas
    /// coisas coincidem enquanto a aresta mínima ficar parada, e divergem no instante em que
    /// uma alça a move. Sem regra o filho não anda; com esta, anda os 4 inteiros — e é por
    /// isso que ela precisa de ser GRAVADA (o `base` é a carga útil).
    #[test]
    fn the_neutral_rule_is_not_the_absence_of_a_rule() {
        // A moldura cresceu para a ESQUERDA: a aresta mínima andou −4, o tamanho subiu 4.
        let now = [-4.0, 0.0, 10.0, 4.0];
        let [dmin, dmax] = rule([0.0, 0.0], [0.0, 0.0]).delta_local(now);
        assert_eq!(dmin, [-4.0, 0.0], "a ponta minima segue a aresta minima");
        assert_eq!(dmax, [-4.0, 0.0], "e a maxima tambem: e' uma translacao");
    }
}
