//! **A ponte do player de plataforma** (W2/W3) — amostra o mundo, chama a lei,
//! aplica o motor.
//!
//! Três metades, e nenhuma sabe da outra: o **cast** é do wrapper
//! (`world::cast`), a **lei** é da folha pura (`ph2d-platformer`), e **aplicar**
//! é do wrapper outra vez (`world::player`). Este módulo é só o fio — e é ele
//! que garante que os três falem do mesmo corpo, no mesmo tick.
//!
//! # ⚠️ Onde o sensor pergunta, e por que é DEPOIS do step anterior
//!
//! O cast lê o BVH do broad phase, que descreve o mundo que o **último `step`**
//! deixou (medido em `world::cast`). Rodar aqui — no topo de cada tick devido,
//! antes do `step` daquele tick — significa perguntar sobre o mundo do tick
//! anterior, que é exatamente o que um sensor pode saber: a alternativa seria
//! consultar um futuro que ainda não foi resolvido.
//!
//! **Consequência honesta, nomeada:** no primeiríssimo tick de uma cena o BVH
//! ainda está vazio, o cast devolve `None`, e o player cai por um tick antes de
//! a mola pegá-lo. É invisível a 60 Hz e é o preço de não manter um segundo
//! índice espacial só para o primeiro quadro.
//!
//! # ⚠️ A amostra do chão e a REAÇÃO são a MESMA resposta
//!
//! O plano (§7) nomeou isto como um dos dois lugares onde este módulo tentaria
//! adoecer: *quem decide "estou no chão" e quem decide "em quem eu empurro"
//! têm de ser a mesma consulta*. Por isso o [`CastHit`] inteiro é carregado
//! adiante — corpo, ponto e normal —, e não só a distância. Quando a W6 chegar,
//! a reação nasce **deste mesmo hit**, não de uma segunda pergunta.
//!
//! [`CastHit`]: ph2d_physics::CastHit

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_platformer::{
    CORNER_LOOKAHEAD, CORNER_SAMPLES, CeilingProbe, GroundSample, HEADROOM_SAMPLES, Headroom,
    PlayerConfig, PlayerInput, PlayerState, WallSample, corner_offsets, corner_probe_wanted,
    footing, headroom_offsets, headroom_probe_wanted, player_motor, relative_rise,
    wall_probe_wanted,
};

use crate::components::{BodyKind, PlatformPlayer};

use super::PhysicsBridge;

/// A direção "para cima" que este módulo assume.
///
/// ⚠️ Um número, um lugar. Ele governa o eixo da mola, o eixo de caminhada no ar
/// e o limite de rampa, e as três respostas **têm** de concordar: se o eixo da
/// mola e o do limite discordassem, existiria uma inclinação em que o
/// personagem é segurado por uma e recusado pela outra.
///
/// Gravidade lateral segue possível na cena (o mundo aceita qualquer vetor) e o
/// player não a acompanha — é a limitação honesta desta wave, e a cura, se um
/// dia for pedida, é derivar o `up` da gravidade **numa porta só**, esta.
const UP: [f32; 2] = [0.0, 1.0];

/// **Um empurrão devido ao chão** (W6) — colhido no laço e aplicado depois.
///
/// Uma struct e não uma tupla de cinco: os dois primeiros campos são handles do
/// MESMO tipo, e trocá-los daria um mundo em que o chão empurra o personagem
/// com a massa dele — compila, roda, e está errado.
struct GroundPush {
    ground: rapier2d_handle::Handle,
    player: rapier2d_handle::Handle,
    accel: [f32; 2],
    boost: [f32; 2],
    at: [f32; 2],
}

impl PhysicsBridge {
    /// **A entrada deste player.** Chamada pela shell a cada frame.
    ///
    /// Escrever `drive = 0` é uma instrução (*"pare"*), não a ausência de uma —
    /// e é por isso que a tabela guarda a entrada em vez de a consumir: um
    /// dispatch que deve vários ticks aplica a MESMA entrada a todos eles, que é
    /// o que um jogador segurando uma tecla quer dizer.
    pub fn set_player_input(&mut self, entity: Entity, input: PlayerInput) {
        self.player_input.insert(entity, input);
    }

    /// O que este player está recebendo agora (`default` = parado).
    #[must_use]
    pub fn player_input(&self, entity: Entity) -> PlayerInput {
        self.player_input.get(&entity).copied().unwrap_or_default()
    }

    /// **Este player está ATRAVESSANDO uma plataforma jump-through agora?** (W20)
    ///
    /// ⚠️ **O bit viaja no CORPO e vale para TODA plataforma one-way da cena**
    /// (`oneway::modify_solver_contacts`), então a resposta não é *"aquela
    /// prancha"* — é *"nenhuma prancha é sólida para ele neste instante"*. É
    /// exatamente isso que o contorno desenha, e é o que torna VISÍVEL uma
    /// descida que não se aposenta: hoje ela é silenciosa.
    #[must_use]
    pub fn player_is_dropping(&self, entity: Entity) -> bool {
        self.player_drop.contains_key(&entity)
    }

    /// **Algum player está atravessando alguma coisa?** — a pergunta que o
    /// overlay faz uma vez por quadro, em vez de uma por plataforma.
    #[must_use]
    pub fn any_player_is_dropping(&self) -> bool {
        !self.player_drop.is_empty()
    }

    /// Esquece toda entrada de player.
    ///
    /// Chamada por quem derruba o mundo derivado (`rebuild`): os bits de
    /// entidade são reciclados ali, então uma entrada guardada passaria a
    /// dirigir **outro** objeto — a mesma armadilha que fez as âncoras de joint
    /// viajarem por NOME em vez de por bits.
    pub fn clear_player_input(&mut self) {
        self.player_input.clear();
        // ⚠️ E o estado de PULO junto, pelo mesmo motivo e mais um: além dos bits
        // reciclados, um `airborne` sobrevivente calaria a perna de um corpo que
        // nunca pulou — o personagem cairia através do mundo sem nada na tela a
        // dizer por quê.
        self.player_state.clear();
        // ⚠️ E a DESCIDA (W12), pela razão mais forte das três: ela guarda um
        // `ColliderHandle`, e handles são reciclados junto com os corpos — uma
        // descida sobrevivente apontaria para uma forma que hoje é outra coisa,
        // e o sintoma seria uma plataforma qualquer que deixa de ser sólida sem
        // ninguém ter pedido.
        self.player_drop.clear();
    }

    /// **As descidas que já cumpriram o seu papel** (W12/W20) — rodada no topo
    /// de cada tique de player, antes de o sensor perguntar qualquer coisa.
    ///
    /// # A lei, numa frase
    ///
    /// A descida morre quando **já passei** (a caixa do personagem está
    /// inteiramente abaixo da caixa da plataforma) **E a prancha já parou de me
    /// pegar** (o gancho one-way não relatou nada neste tique).
    ///
    /// ⚠️ **As duas metades são obrigatórias, e cada uma cura um defeito que a
    /// outra tem** — as duas foram medidas
    /// (`ph2d-physics-ecs/tests/measure_drop_retire.rs`).
    ///
    /// # ⚠️ Só a geometria EXPULSA o personagem
    ///
    /// A caixa estar abaixo **não** garante que a prancha não vá agir: com o
    /// corpo **0,016 m abaixo** da prancha, sem sobreposição nenhuma, a
    /// re-solidificação o atirou de volta ao degrau de cima com um pico de
    /// **0,3267 N·s** entre sub-passos — e o `impulse` de fim de tique lia
    /// `0,0000`, que é a lição da W-ImpactForce outra vez. Faixa medida: prancha
    /// de meia-espessura 0,15, vãos **1,75 a 1,85**, onde ele **não descia de
    /// todo** e o botão parecia não fazer nada. É o livro-razão do gancho
    /// (`PhysicsWorld::drop_is_catching`) que fecha essa borda, porque ele
    /// pergunta à normal do manifold em vez de a caixas.
    ///
    /// # ⚠️ Só a evidência REGRIDE a descida
    ///
    /// Quando a prancha fica inteiramente DENTRO da caixa do personagem (prancha
    /// fina, corpo alto) não existe *lado*, e a normal do manifold **oscila**
    /// entre tiques — medido, o ponto de contato saltando de `−0,486` para
    /// `+0,490` em dois tiques. Uma lei só de evidência aposenta no primeiro
    /// "não" dessa oscilação e a prancha o empurra para cima: com prancha 0,10 e
    /// vãos 1,10 a 1,25 ele **deixava de descer**. A geometria não oscila, e é
    /// ela que segura a evidência até a travessia ter de facto acabado.
    ///
    /// # ⚠️ O que AINDA fica fantasma, e a lei disso
    ///
    /// Medido célula a célula, **a descida sobrevive exactamente onde a caixa de
    /// repouso do personagem ainda SOBREPÕE a prancha** — nenhuma exceção nas
    /// duas espessuras varridas:
    ///
    /// | meia-espessura | vão | o que acontece |
    /// |---|---|---|
    /// | 0,15 | 1,60 – 1,70 | desce, e a prancha fica **fantasma** |
    /// | 0,15 | 1,75 + | funciona (era **arremessado** até 1,85) |
    /// | 0,10 | 1,50 – 1,60 | desce, e a prancha fica **fantasma** |
    /// | 0,10 | 1,65 + | funciona |
    ///
    /// Nessa faixa a prancha **de facto o pegaria** (o cone do gancho devolve
    /// `+1,000`, medido), então as duas saídas são *fantasma* ou *cuspido* — e
    /// fantasma é a menos má.
    ///
    /// ⛔ **E o ALCANCE disso foi MEDIDO e é MENOR do que esta nota afirmava.**
    /// A frase que esteve aqui dizia *"o preço continua a ser a cena inteira —
    /// enquanto essa descida vive, nenhuma prancha é sólida para ele"*, e
    /// prescrevia a **descida por-PLATAFORMA** como cura. Ela foi construída
    /// inteira (conjunto de pares no lugar do bit, evidência por par, o gesto a
    /// levar também as plataformas que o corpo já sobrepõe, o raio a ignorar a
    /// lista) e **REVERTIDA**: numa cena com a escada apertada e uma prancha
    /// SOLTA ao lado, a solta **segura o personagem nos DOIS mundos** — pela
    /// perna, não pelo solver (`measure_whether_a_live_drop_really_dissolves_the_whole_scene`).
    ///
    /// O bit global limpa **contatos do solver**; quem segura este personagem é
    /// a **mola**, e o raio dela só ignora a plataforma da travessia. Então o
    /// que a descida viva de facto custa é a prancha que ela nomeia, e não a
    /// cena — e uma cura por-plataforma seria complexidade sem número.
    ///
    /// ⚠️ A sonda **falhou o próprio controle duas vezes** antes de decidir (o
    /// personagem não saía da escada; depois andava 400 tiques e atravessava a
    /// prancha solta a caminho do outro lado do mundo). *Um A/B em que os dois
    /// lados dão o mesmo número só vale depois de o controle dar um número
    /// diferente.*
    ///
    /// ⚠️ E a cena 91 deixou de viver dez centímetros acima de um penhasco: com
    /// `RISE = 2,0` e pranchas de 0,15 a margem passou de 0,10 para **0,25**, e
    /// a borda que sobrou é a honesta (ali o personagem não cabe).
    fn retire_drops(&mut self) {
        // O caso comum é ninguém a descer, e ele não lê um byte.
        if self.player_drop.is_empty() {
            return;
        }
        let mut done: Vec<Entity> = Vec::new();
        for (&entity, &platform) in &self.player_drop {
            let Some(b) = self.bodies.get(&entity) else {
                // O corpo morreu: não há descida a manter.
                done.push(entity);
                continue;
            };
            // ── A GEOMETRIA: já passei? ──────────────────────────────────────
            let past = match (
                self.world.collider_aabb(platform),
                self.world.body_aabb(b.handle),
            ) {
                (Some((plat_mins, _)), Some((_, body_maxs))) => body_maxs[1] <= plat_mins[1],
                // A plataforma (ou o corpo) deixou de existir — o mesmo
                // veredito, pela mesma razão.
                _ => true,
            };
            // ── A EVIDÊNCIA: e a prancha já parou de me pegar? ───────────────
            if past && !self.world.drop_is_catching(b.handle) {
                done.push(entity);
            }
        }
        for entity in done {
            self.player_drop.remove(&entity);
            if let Some(b) = self.bodies.get(&entity) {
                let handle = b.handle;
                self.world.set_body_drop_through(handle, false);
            }
        }
    }

    /// **Um tick de todos os players.** Chamado no laço de ticks devidos, antes
    /// do `step` (ver o aviso do módulo).
    ///
    /// No-op numa cena sem player — e é o que mantém esta wave byte-neutra para
    /// todo o resto do módulo.
    pub(super) fn drive_players(&mut self, sim: &SimWorld) {
        // ⚠️ **PRIMEIRO, e a ordem é a lei:** uma descida cumprida tem de deixar
        // de valer ANTES de o sensor perguntar, senão o raio deste tique ainda
        // ignoraria uma plataforma que já é sólida outra vez.
        self.retire_drops();
        let world = sim.world();
        let gravity = self.world.gravity();
        let dt = self.world.dt();
        // A ordem é a do `BTreeMap` de corpos — determinística cross-OS, a lei
        // do módulo. Coletar antes de aplicar porque o cast toma `&self` e o
        // motor toma `&mut self`.
        let mut motors: Vec<(rapier2d_handle::Handle, [f32; 2], [f32; 2])> = Vec::new();
        // O estado de pulo que este tick produz, colhido junto com os motores e
        // gravado depois — o cast toma `&self` e a tabela `&mut self`.
        let mut states: Vec<(Entity, PlayerState)> = Vec::new();
        // A reação da 3ª lei, colhida junto pelo mesmo motivo — o cast toma
        // `&self` e aplicar toma `&mut self`.
        let mut reactions: Vec<GroundPush> = Vec::new();
        // Os deslocamentos de quina (W10), pelo mesmo motivo — e vazios em todo
        // tique em que ninguém está subindo contra uma beirada, que é quase todos.
        let mut nudges: Vec<(rapier2d_handle::Handle, [f32; 2])> = Vec::new();
        // O canal que cancela a gravidade (W11), colhido pelo mesmo motivo que os
        // outros e entregue por uma porta própria — ver `PlayerStep::gravity_hold`.
        let mut holds: Vec<(rapier2d_handle::Handle, [f32; 2])> = Vec::new();
        // As descidas ARMADAS neste tique (W12), colhidas pelo mesmo motivo que
        // as outras listas — e vazias em todo tique em que ninguém pediu.
        let mut drops: Vec<(
            Entity,
            rapier2d_handle::Handle,
            ph2d_physics::ColliderHandle,
        )> = Vec::new();
        for (&entity, b) in self.bodies.iter() {
            // Dynamic-only, e é FÍSICA: um impulso não move massa infinita.
            if b.kind != BodyKind::Dynamic {
                continue;
            }
            let Some(cfg) = world.get::<PlatformPlayer>(entity) else {
                continue;
            };
            let Some(pose) = self.world.body_pose(b.handle) else {
                continue;
            };
            let origin = [pose.translation.x, pose.translation.y];
            let Some(vel) = self.world.body_velocity(b.handle) else {
                continue;
            };

            let cfg = cfg.config();
            // O alcance do sensor é o que a lei considera "no chão", e nem um
            // milímetro além: perguntar mais longe faria o cast achar coisas que
            // a lei descartaria, ao preço de descer mais no BVH.
            let reach = cfg.ride.float_height + cfg.ride.cling_distance;
            // ⚠️ **A plataforma que está a ser atravessada sai do SENSOR** (W12),
            // e não só do solver: quem segura o personagem no ar é a MOLA, e ela
            // age porque o raio achou chão. Sem esta exclusão o solver deixaria
            // passar e a perna seguraria em cima — o personagem pairaria sobre
            // exactamente aquilo que pediu para atravessar.
            let passing = self.player_drop.get(&entity).copied();
            let hit = self.world.cast_ray_skipping(
                origin,
                [0.0, -1.0],
                reach,
                Some(b.handle),
                passing,
                b.rest.layer,
            );

            let sample = hit.as_ref().map(|h| GroundSample {
                distance: h.distance,
                normal: h.normal,
                // ⚠️ **Que TIPO de chão é este?** — o único que sabe é quem
                // consultou, e a lei precisa da resposta para decidir o que o
                // botão de pulo significa neste tique.
                one_way: self.world.collider_is_one_way(h.collider),
                // ⚠️ A velocidade do PONTO, não a do centro: uma plataforma que
                // gira leva a borda mesmo com o centro parado
                // (`PhysicsWorld::point_velocity`).
                ground_velocity: h
                    .body
                    .and_then(|gb| self.world.point_velocity(gb, h.point))
                    .unwrap_or([0.0, 0.0]),
            });

            // ── O SENSOR DE TETO (W10) ───────────────────────────────────────
            // ⚠️ A pergunta *"vale a pena castar?"* é a MESMA que a lei faz para
            // decidir se age (`corner_probe_wanted`), e as duas grandezas que
            // ela toma saem das portas da lei — nunca de uma re-derivação aqui.
            // Sem sensor a lei devolve deslocamento zero, então o custo dos
            // raios só existe onde a assistência pode agir.
            let stand = footing(&cfg, sample.as_ref(), UP);
            let rel_up = relative_rise(stand, vel, UP);
            let ceiling = if corner_probe_wanted(&cfg.jump, stand.is_some(), rel_up) {
                probe_ceiling(&self.world, b.handle, b.rest.layer, &cfg, rel_up, dt)
            } else {
                None
            };

            let input = self.player_input.get(&entity).copied().unwrap_or_default();

            // ── O SENSOR LATERAL (W13) ───────────────────────────────────────
            // ⚠️ A pergunta *"vale a pena castar?"* é a MESMA que a lei faz para
            // decidir se pode agir (`wall_probe_wanted`), pelo molde exato do
            // sensor de quina. Com a capacidade desligada — que é como todo
            // player já autorado nasce — nenhum raio é lançado.
            let wall = if wall_probe_wanted(&cfg.wall, stand.is_some(), input.drive) {
                probe_wall(&self.world, b.handle, b.rest.layer, &cfg, input.drive)
            } else {
                None
            };
            let was = self.player_state.get(&entity).copied().unwrap_or_default();

            // ── O SENSOR DE TETO DO AGACHAR (W15) ────────────────────────────
            // ⚠️ A pergunta *"vale a pena castar?"* é a MESMA que a lei faz para
            // decidir se lê (`headroom_probe_wanted`), o molde exacto dos dois
            // sensores acima. Ela é falsa em quase todo tique — só quem ESTÁ
            // agachado e SOLTOU o botão tem alguma coisa a perguntar —, então o
            // custo dos raios existe apenas no instante do gesto de levantar.
            let headroom = if headroom_probe_wanted(&cfg.crouch, &cfg.ride, was.crouch, input.down)
            {
                probe_headroom(&self.world, b.handle, b.rest.layer, &cfg)
            } else {
                None
            };

            let step = player_motor(
                &cfg,
                sample.as_ref(),
                ceiling.as_ref(),
                wall.as_ref(),
                headroom.as_ref(),
                input,
                was,
                vel,
                gravity,
                UP,
                dt,
            );
            states.push((entity, step.state));
            // ⚠️ **A plataforma é a que o SENSOR viu, e é a mesma consulta que a
            // lei julgou** — o `hit` de que saiu o `one_way` que a fez dizer sim.
            // Uma segunda pergunta ("qual one-way está por perto?") poderia achar
            // outra forma, e o personagem atravessaria uma plataforma que não é a
            // que estava debaixo dos pés dele.
            if step.drop_through
                && let Some(h) = hit.as_ref()
            {
                drops.push((entity, b.handle, h.collider));
            }
            if step.nudge != [0.0, 0.0] {
                nudges.push((b.handle, step.nudge));
            }
            let motor = step.motor;
            // ⚠️ **O motor sai por DUAS portas, e a lei é quem as separa** (W11):
            // o que CANCELA a gravidade é integrado como ela (por sub-passo), o
            // resto continua a ser um impulso no topo do tique. O porquê de cada
            // metade está em [`PlayerStep::gravity_hold`]; aqui só se honra a
            // declaração — subtrair `− gravity` por conta própria seria a ponte
            // a adivinhar se a mola agiu.
            let hold = step.gravity_hold;
            let lumped = [motor.accel[0] - hold[0], motor.accel[1] - hold[1]];
            if lumped != [0.0, 0.0] || motor.boost != [0.0, 0.0] {
                motors.push((b.handle, lumped, motor.boost));
            }
            if hold != [0.0, 0.0] {
                holds.push((b.handle, hold));
            }

            // ── A 3ª LEI (W6) ────────────────────────────────────────────────
            // ⚠️ O hit **inteiro** é carregado até aqui, e é o desenho que o
            // módulo declara no topo: *quem decide "estou no chão" e quem decide
            // "em quem eu empurro" têm de ser a MESMA consulta*. Uma segunda
            // pergunta poderia achar outro corpo — o personagem seria segurado
            // por uma jangada e afundaria a de trás.
            if let (Some(r), Some(h)) = (step.reaction, hit.as_ref())
                && !r.is_zero()
                && let Some(ground) = h.body
            {
                reactions.push(GroundPush {
                    ground,
                    player: b.handle,
                    accel: r.accel,
                    boost: r.boost,
                    at: h.point,
                });
            }
        }
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
                .apply_ground_reaction(r.ground, r.player, r.accel, r.boost, r.at);
        }
    }
}

/// **O perfil do teto acima da cabeça** (W10) — a metade do sensor que este
/// módulo possui, e nada de política.
///
/// Os raios nascem no TOPO da caixa do corpo e medem `rel_up · dt ·
/// [`CORNER_LOOKAHEAD`]` — a distância que a cabeça sobe no PRÓXIMO tique. É
/// isso, e só isso, que torna a assistência preditiva: o que o perfil descreve é
/// um contato que ainda não aconteceu.
///
/// ⚠️ **A grade de deslocamentos vem da lei** ([`corner_offsets`]), nunca de uma
/// aritmética local. Duas cópias deslocariam o perfil de meia célula em relação
/// ao corpo, e o sintoma seria um personagem empurrado para dentro da quina de
/// que ele fugia.
///
/// ⚠️ **A folga lateral é medida do CENTRO e descontada a meia-largura**, porque
/// o raio nasce no centro (a origem tem de estar dentro do corpo para o
/// `exclude_body` fazer sentido) e o que a lei quer saber é quanto há de espaço
/// **além** da borda. Saturada no próprio alcance: para esta decisão, *livre até
/// onde eu poderia querer ir* e *livre até o infinito* são a mesma coisa.
fn probe_ceiling(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    rel_up: f32,
    dt: f32,
) -> Option<CeilingProbe> {
    let sweep = rel_up * dt * CORNER_LOOKAHEAD;
    if !sweep.is_finite() || sweep <= 0.0 {
        return None;
    }
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let top = maxs[1];
    let mid = (maxs[1] + mins[1]) * 0.5;
    let reach = cfg.jump.corner_reach;

    let mut blocked = [false; CORNER_SAMPLES];
    for (slot, off) in blocked
        .iter_mut()
        .zip(corner_offsets(half_width, reach).iter())
    {
        *slot = world
            .cast_ray([cx + off, top], [0.0, 1.0], sweep, Some(handle), layer)
            .is_some();
    }

    let free = |dir: f32| {
        world
            .cast_ray(
                [cx, mid],
                [dir, 0.0],
                half_width + reach,
                Some(handle),
                layer,
            )
            .map_or(reach, |h| (h.distance - half_width).clamp(0.0, reach))
    };

    Some(CeilingProbe {
        half_width,
        blocked,
        side_clear: [free(-1.0), free(1.0)],
    })
}

/// **O que há sobre a cabeça** (W15) — os raios que decidem se ele pode
/// levantar-se de um agachar.
///
/// Eles nascem no TOPO da caixa do corpo e medem exactamente
/// [`ph2d_platformer::CrouchConfig::rise`] — *quanto o corpo SOBE ao
/// levantar-se*. Nem um milímetro além: perguntar mais longe recusaria o gesto
/// por causa de um teto que o personagem, de pé, não alcança.
///
/// ⚠️ **A grade vem da lei** ([`headroom_offsets`]), nunca de uma aritmética
/// local — a mesma regra do sensor de quina, e pelo mesmo motivo: duas
/// aritméticas deslocariam as amostras em relação ao corpo.
///
/// ⚠️ **A caixa envolvente é CONSERVADORA, e a direcção do erro é a certa** (ver
/// o doc de [`headroom_offsets`]): um teto que toque só a quina da caixa recusa
/// o levantar. Ficar agachado onde caberia é um incómodo; levantar-se para
/// dentro da pedra é o solver a resolver uma penetração que ninguém autorou.
///
/// ⚠️ **O QUE NÃO ESTÁ GATEADO, e porquê:** a GRADE tem gate na lei
/// (`the_headroom_grid_spans_the_body`) e o `blocked` tem gate no produto — mas
/// a metade *"o laço percorre os TRÊS deslocamentos"* só é observável sob um
/// teto **PARCIAL**, que cubra uma borda do corpo e não o centro. Uma fixture
/// dessas teria de calibrar a aresta da laje contra a posição MEDIDA de um
/// personagem a andar, dentro de uma janela de 0,2 m — um gate que falharia por
/// deriva de fixture em vez de por defeito. Fica NOMEADO: trocar o laço por um
/// raio central sobrevive à suíte.
fn probe_headroom(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
) -> Option<Headroom> {
    let rise = cfg.crouch.rise(&cfg.ride);
    if !rise.is_finite() || rise <= 0.0 {
        return None;
    }
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let top = maxs[1];

    let mut blocked = [false; HEADROOM_SAMPLES];
    for (slot, off) in blocked.iter_mut().zip(headroom_offsets(half_width).iter()) {
        *slot = world
            .cast_ray([cx + off, top], [0.0, 1.0], rise, Some(handle), layer)
            .is_some();
    }
    Some(Headroom { blocked })
}

/// **A parede ao lado** (W13) — a metade do sensor que este módulo possui, e
/// nada de política.
///
/// Um raio, na direção em que o jogador empurra, do CENTRO do corpo até meia
/// largura mais o alcance autorado.
///
/// ⚠️ **Do centro, e o alcance desconta a meia-largura**, pela mesma razão do
/// sensor de quina: a origem tem de estar DENTRO do corpo para o `exclude_body`
/// significar alguma coisa, e o que a lei quer saber é quanto há de parede
/// **além** da borda.
///
/// ⚠️ **A altura é o MEIO do corpo, e é uma escolha com preço nomeado:** uma
/// beirada que só alcance os pés (ou só os ombros) não é vista. Um segundo par
/// de raios curaria isso, e a caixa envolvente responde hoje sem discussão — é
/// a mesma limitação honesta que a folga lateral da W10 carrega, e pela mesma
/// razão.
///
/// ⚠️ **Este módulo NÃO decide se aquilo é parede.** Ele reporta a normal; quem
/// classifica é a lei, pela régua que a perna já usa (`ph2d_platformer::cling`).
/// Uma segunda régua aqui seria o `wall_min_angle` que o `wall.rs` existe para
/// não ter.
fn probe_wall(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    drive: f32,
) -> Option<WallSample> {
    let side = if drive > 0.0 { 1.0 } else { -1.0 };
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let mid = (maxs[1] + mins[1]) * 0.5;
    let reach = half_width + cfg.wall.reach.max(0.0);
    let hit = world.cast_ray([cx, mid], [side, 0.0], reach, Some(handle), layer)?;
    Some(WallSample {
        side,
        normal: hit.normal,
    })
}

/// O handle do rapier, nomeado sem importar o rapier aqui — esta crate declara-se
/// *rapier-free* no próprio `Cargo.toml` e só carrega os tipos re-exportados.
mod rapier2d_handle {
    pub type Handle = ph2d_physics::RigidBodyHandle;
}
