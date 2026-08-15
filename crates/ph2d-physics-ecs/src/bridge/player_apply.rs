//! **O QUE A PONTE FAZ com o que o laço colheu** — a metade `&mut self` do
//! tique de players.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e é o MESMO que o do `player_kinmove` ao
//! lado:** o pai [`super::player`] pergunta ao mundo e COLHE (o cast toma
//! `&self`), e este ESCREVE. O módulo pai já dizia essa lei em sete comentários
//! — *"coletar antes de aplicar porque o cast toma `&self` e o motor toma
//! `&mut self`"* — e o que muda aqui é que a metade que escreve passa a ter casa
//! própria, em vez de ser a cauda de um laço de setecentas linhas.
//!
//! ⚠️ **A ORDEM entre as listas é a lei, e ela viaja INTEIRA para cá** — cada
//! `for` traz o comentário que explica por que vem onde vem. Espalhá-los seria
//! perder exactamente o que os torna corretos.
//!
//! Módulo FILHO por `#[path]`, então `super::*` continua a alcançar o que o pai
//! não exporta.

use super::*;

/// **O que um tique de players PRODUZIU** — as oito listas que o laço colheu.
///
/// ⚠️ **Uma struct e não oito argumentos**, e não é cerimónia: metade delas são
/// `Vec` de tuplas cujo primeiro elemento é um handle do MESMO tipo, e trocar
/// duas na chamada compila, roda e escreve o motor de um corpo no outro.
pub(super) struct PlayerResults {
    pub states: Vec<(Entity, PlayerState)>,
    pub drops: Vec<(
        Entity,
        rapier2d_handle::Handle,
        ph2d_physics::ColliderHandle,
    )>,
    pub nudges: Vec<(rapier2d_handle::Handle, [f32; 2])>,
    pub motors: Vec<(rapier2d_handle::Handle, [f32; 2], [f32; 2])>,
    pub holds: Vec<(rapier2d_handle::Handle, [f32; 2])>,
    pub reactions: Vec<GroundPush>,
    pub moves: Vec<KinMove>,
    pub views: Vec<(Entity, ph2d_platformer::PlayerView)>,
    pub events: Vec<(Entity, ph2d_platformer::PlayerEvent)>,
}

impl PhysicsBridge {
    /// **Escreve tudo o que o laço colheu**, na ordem que cada bloco justifica.
    pub(super) fn apply_player_results(&mut self, dt: f32, r: PlayerResults) {
        let PlayerResults {
            states,
            drops,
            nudges,
            motors,
            holds,
            reactions,
            moves,
            views,
            events,
        } = r;
        // ⚠️ O estado é gravado para TODO player, inclusive os cujo motor é zero
        // — e isto é **defesa em camadas, não load-bearing HOJE**, medido: pôr o
        // push dentro do guard de motor **sobrevive à suíte inteira**.
        //
        // O porquê é uma coincidência da lei atual, não um desenho: todo campo do
        // `PlayerState.jump` menos o `airborne` é função pura da entrada DESTE tick
        // (`was_held = held`, `cut` re-derivado de `!held`), e o `airborne` só
        // vira em ticks que necessariamente carregam motor (a decolagem tem o
        // boost; o pouso re-arma a perna). Com os defaults, a SUBIDA tem motor
        // exatamente zero (`takeoff_gravity = 1.0` ⇒ `extra = 0`) e mesmo assim
        // nada se perde.
        //
        // ⚠️ A coincidência MORRE na W8: coyote timer e jump buffer são contadores
        // que andam por tick, e um deles congelado durante a subida é um bug **sem
        // sintoma** — o pulo continua saindo, só a tolerância deixa de existir. É
        // por isso que a linha nasce agora, e não quando alguém a perseguir.
        for (entity, next) in states {
            self.player_state.insert(entity, next);
        }
        // ⚠️ **ANTES do `step` deste tique**, que é o que torna a descida
        // observável já na resolução de contatos que vem a seguir — armá-la
        // depois daria um tique em que a plataforma ainda é sólida e o
        // personagem seria empurrado de volta para cima antes de começar a cair.
        for (entity, handle, platform) in drops {
            self.player_drop.insert(entity, platform);
            self.world.set_body_drop_through(handle, true);
        }
        // ⚠️ **O deslocamento vai ANTES dos motores, e a ordem é a lei da wave:**
        // ele corrige ONDE o corpo está, e o motor age a partir dali. Depois, o
        // impulso deste tique teria sido calculado numa posição que o corpo já
        // não ocupa — pequeno hoje, e o tipo de discordância que ninguém acha
        // quando um passe futuro passar a ler a pose entre os dois.
        for (handle, delta) in nudges {
            self.world.nudge_body(handle, delta);
        }
        for (handle, accel, boost) in motors {
            self.world.apply_player_motor(handle, accel, boost);
        }
        // ⚠️ Lista separada dos motores pelo mesmo motivo que as reações: elas
        // descrevem **quando** o impulso é pago, não só a quem. E o corpo segue
        // a ser ACORDADO — quem o faz é o `wake_up` do `apply_impulse` lá dentro,
        // não o `apply_player_motor`; um player em repouso perfeito pode ter o
        // motor agrupado exactamente zero e mesmo assim ser segurado por este
        // canal, e sem o despertar ele deixaria de ser integrado.
        for (handle, hold) in holds {
            self.world.queue_player_hold(handle, hold);
        }
        // ⚠️ **Depois dos motores, e a ordem não importa hoje** — os dois são
        // impulsos e o solver os soma —, mas as listas são separadas porque
        // descrevem corpos diferentes: um `retain` futuro que filtre players não
        // pode levar as reações que eles devem ao chão junto.
        for r in reactions {
            self.world
                .apply_player_reaction(r.ground, r.player, r.accel, r.boost, r.at);
        }
        // ── O MOVIMENTO CINEMÁTICO (W-KinMove) ───────────────────────────────
        // O laço COLHEU; quem APLICA é o irmão `player_kinmove` — ver
        // `Self::apply_kin_moves` para a ordem e o porquê dela.
        self.apply_kin_moves(moves, dt);
        // ── E A SAÍDA (`bridge::player_out`) ─────────────────────────────────
        // ⚠️ **POR ÚLTIMO, e a ordem é a lei:** a tabela de vistas é a memória
        // contra a qual o laço acabou de diferenciar, então reescrevê-la antes
        // do fim faria os players seguintes deste MESMO tique compararem-se com
        // uma vista já avançada.
        self.publish_player_tick(views, events);
    }
}
