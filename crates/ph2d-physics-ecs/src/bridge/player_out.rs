//! **A SAÍDA do player** — o que o resto do app pode ler sobre ele
//! (`W-PlayerOut`).
//!
//! # ⚠️ Dois canais, e a fronteira entre eles é a pergunta
//!
//! O [`player_view`](PhysicsBridge::player_view) é estado **CONTÍNUO**: *ele
//! está no chão, olha para a direita, tem um pulo de ar sobrando*. Ele descreve
//! o **AGORA**, então é o do ÚLTIMO tique do dispatch — quem precisa dos tiques
//! do meio precisa da outra metade.
//!
//! Os [`player_events`](PhysicsBridge::player_events) são **TRANSIÇÕES**:
//! *acabou de aterrar*. Eles nascem **por TIQUE**, dentro do laço, comparando o
//! par de vistas consecutivas — e é essa a metade que um diff entre dois
//! quadros da shell **não consegue dar**: um dispatch pode dever vários tiques,
//! e um pulo que sai e aterra dentro do mesmo dispatch não teria acontecido. É
//! literalmente o defeito que o `W-TickContacts` mediu no canal de contatos
//! (uma queda de 3 m não gerava evento nenhum), e a cura aqui é a mesma.
//!
//! # ⚠️ A ponte NÃO conhece o `SignalOutbox`
//!
//! Ela expõe eventos **tipados** e quem os funde numa saída é a **shell**, que
//! já é dona daquele consumidor e já drena a outra fonte. É o precedente literal
//! do [`bridge::signals`](super::signals) — e é o que mantém a `ph2d-runtime`
//! fora das dependências deste módulo.

use ph2d_ecs::Entity;
use ph2d_platformer::{PlayerEvent, PlayerView};

use crate::PhysicsBridge;

impl PhysicsBridge {
    /// **O que este player estava a fazer no fim do último tique.**
    ///
    /// `None` para toda entidade que não é um player — e também **enquanto a
    /// física está desarmada ou logo depois de um scrub**, pela razão que o
    /// [`discard_player_history`](Self::discard_player_history) explica: sem
    /// passo não há lei, e um readout de uma corrida que acabou é um número
    /// errado apresentado como certo.
    #[must_use]
    pub fn player_view(&self, entity: Entity) -> Option<&PlayerView> {
        self.player_views.get(&entity)
    }

    /// **As transições deste dispatch**, na ordem em que os tiques as
    /// produziram — e, dentro de um tique, na ordem determinística do
    /// `BTreeMap` de corpos.
    ///
    /// Vazia em todo quadro em que nada aconteceu, e vazia no primeiro quadro
    /// depois de uma descontinuidade (a baseline nasce vazia).
    ///
    /// ⚠️ **Um scrub RE-PRODUZ os eventos dos tiques que replaya**, e isso é o
    /// correcto: o que se está a assistir é aquela passagem do tempo.
    #[must_use]
    pub fn player_events(&self) -> &[(Entity, PlayerEvent)] {
        &self.player_events
    }

    /// Esquecer o que o player estava a fazer, **sem reportar nada como tendo
    /// terminado**.
    ///
    /// Os dois chamadores são as duas descontinuidades — um scrub/Reset
    /// ([`rewind_to`](Self::rewind_to)) e desarmar o toggle **Physics**
    /// ([`hold`](Self::hold)) —, exactamente os mesmos do
    /// [`discard_contact_history`](Self::discard_contact_history) e da lista de
    /// marcas dos sensores. Os dois movem o relógio de um jeito que a simulação
    /// não percorreu, e estes canais só falam do que ela percorreu.
    ///
    /// ⚠️ **A vista vai junto com os eventos, e não é higiene:** ela é a memória
    /// contra a qual o tique seguinte diferencia, então mantê-la faria o
    /// primeiro tique depois do salto comparar-se com um estado de outra
    /// corrida — um `Landed` que ninguém viveu.
    pub(super) fn discard_player_history(&mut self) {
        self.player_views.clear();
        self.player_events.clear();
    }

    /// O que o tique acabou de publicar: a vista de cada player que correu, e as
    /// transições que a comparação com o tique anterior revelou.
    ///
    /// ⚠️ **A lista de vistas é o CENSO dos players que correram**, então ela
    /// SUBSTITUI a tabela em vez de a completar: quem deixou de ser player
    /// (componente removido, corpo apagado) cai fora por construção, e não por
    /// um `retain` que alguém tem de lembrar de escrever.
    pub(super) fn publish_player_tick(
        &mut self,
        views: Vec<(Entity, PlayerView)>,
        events: Vec<(Entity, PlayerEvent)>,
    ) {
        self.player_views.clear();
        self.player_views.extend(views);
        self.player_events.extend(events);
    }
}
