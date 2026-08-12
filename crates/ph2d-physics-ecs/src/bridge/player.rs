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
use ph2d_physics::{CharacterHit, CharacterParams};
use ph2d_platformer::{
    Buoyed, CeilingProbe, Fluid, GroundSample, Headroom, KinematicState, MAX_CORNER_SAMPLES,
    MAX_WALL_SAMPLES, PlayerConfig, PlayerInput, PlayerState, ReactionConfig, WallHit, WallProbe,
    corner_offsets, corner_probe_wanted, footing, headroom_probe_wanted, kinematic_advance,
    kinematic_settle, odd_samples, player_motor, relative_rise, wall_offsets, wall_probe_wanted,
};

use super::player_view::{ProbeKind, ProbeMark};
use crate::components::PlatformPlayer;

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
/// **Um deslocamento cinemático devido** (W-KinMove) — colhido no laço e
/// aplicado depois, pelo motivo de sempre: perguntar ao mundo toma `&self` e
/// escrever a pose toma `&mut self`.
struct KinMove {
    entity: Entity,
    handle: rapier2d_handle::Handle,
    layer: u8,
    passing: Option<ph2d_physics::ColliderHandle>,
    params: CharacterParams,
    /// O deslocamento que a lei PEDIU.
    wanted: [f32; 2],
    /// A velocidade depois do avanço e **antes** do assentamento.
    advanced: KinematicState,
    /// Os escalares da 3ª lei — o EMPURRÃO (W-KinPush) lê o `push` daqui.
    ///
    /// ⚠️ Viaja na lista porque a config do player é lida no primeiro laço (com
    /// `&self`) e o empurrão é aplicado no segundo (com `&mut self`): relê-la
    /// depois seria uma segunda consulta ao mundo ECS a meio do tique, e as
    /// duas divergiriam no frame em que o artista arrasta o slider.
    react: ReactionConfig,
}

struct GroundPush {
    ground: rapier2d_handle::Handle,
    player: rapier2d_handle::Handle,
    accel: [f32; 2],
    boost: [f32; 2],
    at: [f32; 2],
}

impl PhysicsBridge {
    /// **Um tick de todos os players.** Chamado no laço de ticks devidos, antes
    /// do `step` (ver o aviso do módulo).
    ///
    /// No-op numa cena sem player — e é o que mantém esta wave byte-neutra para
    /// todo o resto do módulo.
    pub(super) fn drive_players(&mut self, sim: &SimWorld) {
        // ⚠️ **PRIMEIRO, e a ordem é a lei:** uma descida cumprida tem de deixar
        // de valer ANTES de o sensor perguntar, senão o raio deste tique ainda
        // ignoraria uma plataforma que já é sólida outra vez.
        drops::retire_drops(self);
        // ⚠️ **A leitura dos sensores descreve UM tique**, então ela é
        // reconstruída aqui e não acumulada: um dispatch que deve vários tiques
        // desenharia o rasto de todos eles, e o artista leria como *"o sensor
        // está a tremer"* aquilo que é só a história a somar-se. A capacidade do
        // `Vec` fica.
        self.player_probes.clear();
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
        // Os deslocamentos do modo CINEMÁTICO (W-KinMove), colhidos pelo mesmo
        // motivo que todas as outras listas: perguntar ao mundo quanto coube toma
        // `&self` e escrever a pose toma `&mut self`.
        let mut moves: Vec<KinMove> = Vec::new();
        for (&entity, b) in self.bodies.iter() {
            // ⚠️ **Quem escreve a pose deste corpo?** — a porta única do
            // `bridge::pose_owner`, e ela decide DUAS coisas de uma vez: se este
            // laço dirige o corpo, e **o que o segura** (o `Support` da lei).
            // Perguntadas em dois lugares elas poderiam discordar, e a lei de
            // Snap num corpo dinâmico deixa o personagem cair através do mundo.
            let owner = super::pose_owner::pose_owner(world, entity, b.kind);
            if owner.driven_by_scene() {
                // A cena manda nele (um corpo estático, um bake, uma curva) — e
                // um impulso não move massa infinita de qualquer forma.
                continue;
            }
            let Some(cfg) = world.get::<PlatformPlayer>(entity) else {
                continue;
            };
            let Some(pose) = self.world.body_pose(b.handle) else {
                continue;
            };
            let origin = [pose.translation.x, pose.translation.y];
            let Some(solver_vel) = self.world.body_velocity(b.handle) else {
                continue;
            };
            let was = self.player_state.get(&entity).copied().unwrap_or_default();

            let mut cfg = cfg.config();
            // ⚠️ **Sob Snap a PERNA é o próprio corpo** — ver
            // `PhysicsWorld::body_foot_distance`. O `float_height` autorado
            // descreve onde a cápsula flutuante paira, e sob Snap ela não paira:
            // ela pousa. Deixar a régua da perna aqui faz a lei dizer *"estou no
            // chão"* com o personagem a 0,4 m no ar.
            if owner.writes_own_pose()
                && let Some(foot) = self.world.body_foot_distance(b.handle, UP)
            {
                cfg.ride.float_height = foot;
            }
            // ⚠️ **O MUNDO É CENÁRIO** (W-KinPure) — e este é o ÚNICO sítio onde
            // isso é dito, de propósito. As duas metades da 3ª lei saem daqui: o
            // PESO (a `reaction`, que lê `cfg.react`) e o EMPURRÃO lateral (o
            // `KinMove.react`, copiado deste mesmo `cfg`). Silenciar as duas num
            // ponto é o que impede que uma wave futura acrescente uma terceira
            // metade e se esqueça de a calar.
            //
            // ⚠️ Os escalares AUTORADOS não são tocados: o modo decide se eles
            // são ouvidos, não o que eles valem. Voltar para `Kinematic` devolve
            // o que o artista escreveu.
            if !owner.transmits() {
                cfg.react = ReactionConfig::OFF;
            }
            let cfg = cfg;
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
            // ⚠️ **A perna é um LEQUE, não um raio** (`W-Probes2`). Um raio só no
            // centro afunda **0,411 m — 46% do `float_height`** parado sobre uma
            // fenda de 10 cm que as bordas do corpo suportam fisicamente
            // (`measure_what_a_single_ground_ray_costs_over_a_gap`); é a mesma
            // doença que o flanco teve na W13, e a mesma cura.
            //
            // ⚠️ **A redução é o MAIS PRÓXIMO, e não é uma regra nova** — é a que
            // o `cling` já ship a no flanco: o chão é o degrau mais alto que
            // qualquer parte do pé alcança **e que seja CAMINHÁVEL** (a segunda
            // metade nasceu de uma medição, logo abaixo). O `<` estrito faz o
            // raio do MEIO ganhar todo empate (ele é o índice 0 do
            // `wall_offsets`), o que mantém um corpo sobre chão plano
            // byte-idêntico ao que era.
            let leg = player_leg::cast_leg(
                &self.world,
                b.handle,
                &cfg,
                origin,
                reach,
                passing,
                b.rest.layer,
            );
            let leg_hits = leg.per_foot;
            let hit = leg.hit;

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

            // ⚠️ **Sob Snap a velocidade é a NOSSA** (K5): um corpo cinemático
            // não tem velocidade que o solver possua — o que o rapier reporta é
            // derivado da pose que nós escrevemos no tique ANTERIOR, ou seja um
            // eco atrasado do que a lei já sabe. Ler o eco faria o pulo e a
            // caminhada decidirem sobre um estado de um tique atrás.
            //
            // ⚠️ **E ela vem SEM a parte que o chão já segura** — a mesma porta
            // que o integrador usa (`supported_velocity`), chamada aqui porque a
            // ORDEM importa: o `settle` deixa no estado a queda que o mundo
            // bloqueou, o `kinematic_advance` a apaga, e entre os dois corre a
            // LEI. Enquanto ela via essa queda, o freio da caminhada a lia como
            // escorregão e empurrava morro acima — ver `supported_velocity`.
            //
            // ⚠️ **E quem responde *"há chão?"* é o `footing`, NUNCA o
            // `was.kin.grounded`** — as duas perguntas parecem a mesma e não
            // são: a do integrador é *"eu TOQUEI no mundo?"*, respondida pelo
            // tique ANTERIOR, e no tique do CONTATO ela ainda diz *no ar*. A da
            // lei é o `footing` (K4), e ele já respondeu — o cast acima corre
            // sobre a pose de DEPOIS do `settle`. Por isso este bloco mora aqui
            // embaixo e não junto do `was`: **a ordem é a correção**. Com a
            // pergunta velha o tique do contato entrava na lei com a queda
            // inteira e saía deslocado ao longo da NORMAL da rampa.
            // ⚠️ **O PISO da absorção (§8.1), e ele é uma pergunta DIFERENTE da
            // de baixo — por isso mora aqui fora.**
            //
            // A absorção pergunta *"há chão AGORA?"* e a resposta é o `footing`
            // fresco (o parágrafo acima). O piso pergunta *"ele JÁ vinha a
            // andar no chão?"*, e essa é a do INTEGRADOR (`was.kin.grounded`,
            // o tique anterior): num POUSO a referência é uma QUEDA, cuja
            // projeção tangencial numa rampa é indistinguível de uma caminhada
            // — medido, o personagem pousava 1,4 cm ao lado. Ver
            // `surface_descent`.
            //
            // ⚠️ **Juntá-las num `if` só dentro do braço abaixo é o que faz o
            // arch-gate vizinho disparar**, e ele está certo: o texto daquele
            // ramo é o que prova que a absorção não usa a resposta velha.
            let floor = if was.kin.grounded {
                // A MESMA referência e a MESMA superfície que o integrador vai
                // usar — é isso que faz as duas chamadas concordarem.
                ph2d_platformer::surface_descent(
                    was.kin.velocity,
                    stand.map_or(UP, |s| s.normal),
                    UP,
                )
            } else {
                0.0
            };
            let vel = if owner.writes_own_pose() {
                ph2d_platformer::supported_velocity(was.kin.velocity, stand.is_some(), UP, floor)
            } else {
                solver_vel
            };

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

            // ── A LEITURA DOS SENSORES (W-Probes) ────────────────────────────
            // ⚠️ **Aqui, e não antes de cada cast:** as três consultas acima já
            // responderam, e o que falta é guardar o par *onde olhou · o que
            // viu*. A geometria sai das MESMAS portas que castaram — este bloco
            // não recalcula uma origem.
            //
            // ⚠️ **E ela é gravada mesmo quando a lei não perguntou.** O `Idle`
            // que sai daí é a metade que o artista não consegue ver hoje: sem
            // ela, *"a capacidade está desligada"* e *"o alcance é curto demais"*
            // produzem o mesmo nada na tela.
            probes::record_marks(
                &mut self.player_probes,
                &self.world,
                entity,
                b.handle,
                &cfg,
                origin,
                input.drive,
                // A lei PERGUNTOU (a perna e' castada em todo tique) — o `Some`
                // exterior e' isso, e o interior e' o que CADA raio achou.
                Some(leg_hits),
                ceiling
                    .as_ref()
                    .map(|c| (c, probes::corner_rise(rel_up, dt, &cfg))),
                wall.as_ref(),
                headroom.as_ref(),
            );

            // ── O FLUIDO (W-Submerged) ───────────────────────────────────────
            // ⚠️ **Não há `*_wanted` aqui, e a assimetria é deliberada:** os três
            // sensores acima lançam RAIOS, e a pergunta *"vale a pena castar?"*
            // existe para os manter fora do caminho quando a lei não pode agir.
            // Esta é uma consulta de LEITURA que sai por um early-out no primeiro
            // `if` de uma cena sem zona de empuxo — e um `wanted` para ela seria
            // uma segunda pergunta sobre a MESMA condição que a porta já faz.
            // ⚠️ **UMA consulta, DOIS consumidores** (W-KinFluid): a `Buoyed` é o
            // que a lei do pulo pergunta (a trava do fluido) e o `Fluid` inteiro é
            // o que o integrador cinemático integra. Perguntar duas vezes pagaria o
            // passeio pelo grafo de interseção em dobro por tique de player, e
            // deixaria dois números para o mesmo fato.
            let at = self.world.fluid_at(b.handle);
            let buoyed = Buoyed(at.buoyed);
            let fluid = Fluid {
                buoyed,
                drag: at.drag,
                // ⚠️ **A MESMA varredura devolve as TRÊS grandezas** (W-ZoneForce): o
                // empurrão entrou na consulta que já andava pelo grafo de interseção,
                // e não numa segunda pergunta ao lado — perguntá-lo à parte pagaria o
                // passeio em dobro por tique de player e deixaria dois números para o
                // mesmo fato, que é a doença que este `at` existe para não ter.
                push: at.push,
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
                buoyed,
                // ⚠️ **A MESMA resposta que decidiu quem escreve a pose** — ver o
                // aviso no topo do laço e o `bridge::pose_owner`.
                owner.support(),
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
            if owner.writes_own_pose() {
                // ── O MODO CINEMÁTICO (W-KinMove) ────────────────────────────
                // ⚠️ **A `gravity_hold` NÃO é consultada aqui, e é deliberado:**
                // ela existe para a ponte DINÂMICA dividir o impulso entre
                // sub-passos, e a lei cinemática integra a gravidade ela mesma
                // (ver o topo de `ph2d_platformer::kinematic`). O `motor.accel`
                // já traz o cancelamento embutido quando ele existe — somá-lo
                // outra vez o pagaria duas vezes.
                let (next_kin, wanted) = kinematic_advance(
                    was.kin, motor,
                    // ⚠️ **A plataforma entra por AQUI** (K7): a amostra já
                    // viaja desde a W3, e resolvê-la fora da lei seria uma
                    // segunda resposta para *"quanto o chão me leva?"*.
                    //
                    // ⚠️ E o que atravessa é a AMOSTRA, não a `ground_velocity`
                    // dela: quem decide quanto o chão ainda deve é o
                    // `ground_carry`, porque a caminhada já paga a tangente.
                    stand, gravity, UP, dt,
                    // ⚠️ **A ÁGUA entra por aqui, e é a MESMA leitura que a lei do
                    // pulo já recebeu** (o `buoyed` acima sai deste `fluid`): uma
                    // segunda consulta daria duas respostas para *"em que meio
                    // estou?"*, e elas divergiriam no tique em que o personagem
                    // cruza a superfície — precisamente onde ninguém confere.
                    fluid,
                );
                moves.push(KinMove {
                    entity,
                    handle: b.handle,
                    layer: b.rest.layer,
                    // A MESMA plataforma que o sensor está a atravessar (W12):
                    // sem esta exclusão o controlador o pararia em cima do que
                    // ele pediu para atravessar.
                    passing,
                    params: CharacterParams {
                        up: UP,
                        // ⚠️ Os dois números saem da configuração que o artista
                        // JÁ autora — um default do rapier em qualquer um deles
                        // seria o produto a discordar de si mesmo (ver
                        // `CharacterParams`).
                        snap_distance: cfg.ride.cling_distance,
                        max_slope_deg: cfg.walk.max_slope_deg,
                        step_height: cfg.ride.cling_distance,
                    },
                    wanted,
                    advanced: next_kin,
                    react: cfg.react,
                });
                // ⚠️ **O estado guardado leva a velocidade ANTES do assentamento**
                // — a lista de moves a corrige depois de o mundo responder. É a
                // razão de o `states` ser gravado primeiro: o `settle` precisa do
                // que o mundo deixou acontecer, e isso só existe com `&mut self`.
                if let Some(last) = states.last_mut() {
                    last.1.kin = next_kin;
                }
            } else {
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
            }

            // ── A 3ª LEI (W6) — NOS DOIS MODOS ───────────────────────────────
            // ⚠️ **Fora do ramo de propósito (K6):** a `reaction` toma o suporte
            // como ARGUMENTO e o chão vem da `footing` — nada nela depende de o
            // corpo ser dinâmico. Sob Snap o `push` é zero, e zero **já É o
            // peso**, porque o `support_accel` sempre carregou o `− gravity` ao
            // lado do empurrão: o chão sente quem está em cima dele nos dois
            // modos, pela MESMA função.
            //
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
                .apply_player_reaction(r.ground, r.player, r.accel, r.boost, r.at);
        }
        // ── O MOVIMENTO CINEMÁTICO (W-KinMove) ───────────────────────────────
        // ⚠️ **Por último, e a ordem importa:** a reação já foi enfileirada
        // contra o chão, e o `move_shape` lê o BVH que o `step` ANTERIOR deixou
        // (o mesmo contrato do sensor, ver o topo do módulo) — nada aqui depende
        // dos impulsos deste tique.
        // ⚠️ **UM buffer para o laço inteiro**, limpo pela porta que o preenche
        // (ver `move_character`): uma lista por personagem seria uma alocação
        // por player por tique, para carregar no máximo um punhado de contatos.
        let mut hits: Vec<CharacterHit> = Vec::new();
        let mut pushes: Vec<Push> = Vec::new();
        for m in moves {
            let got = self
                .world
                .move_character(m.handle, m.wanted, m.params, m.passing, m.layer, &mut hits);
            // ── O EMPURRÃO (W-KinPush) ───────────────────────────────────────
            //
            // ⚠️ **Um corpo cinemático tem massa INFINITA para o solver**, então
            // o `move_shape` desliza contra um caixote solto sem lhe transmitir
            // nada — medido, o dinâmico o empurra 16,55 m em 3 s e o cinemático
            // 0,0000. A 3ª lei já atravessa o modo no eixo VERTICAL (a
            // `reaction`, K6); isto é a metade lateral dela.
            //
            // ⚠️ **UM empurrão por CORPO, e é o que impede a contagem dupla:**
            // o controlador desliza em iterações e pode relatar o mesmo corpo
            // várias vezes; somar cada relatório entregaria a mesma quantidade
            // de movimento duas vezes ao mesmo caixote.
            push::push_from_hits(&m.react, m.wanted, got.translation, &hits, &mut pushes);
            for &(body, transfer, at) in &pushes {
                // ⚠️ **A conversão é a da reação vertical**: um deslocamento de
                // um tique É uma velocidade quando dividido pelo tique, que é
                // exatamente o que o canal `boost` significa (`Δv·m`).
                //
                // ⚠️ **Mas a ENTREGA é por SUB-PASSO** (§8.2, escolha do Enio),
                // e é o que separa esta metade da vertical. Entregue de uma vez,
                // o bloqueio inteiro do tique entrava como UMA martelada num
                // ponto alto do caixote e `r × F` fazia o resto — medido, um
                // caixote pequeno dava **12 voltas em 3 s** (74,29 rad) contra
                // 0,3175 do corpo dinâmico, e o giro seguia a ALAVANCA (some
                // quando o contato desce até o centro de massa).
                //
                // O dinâmico empurra com força SUSTENTADA por sub-passo, e essa
                // é literalmente a diferença medida — então a cura é entregar
                // pelo mesmo mecanismo, não capar o torque com um número novo.
                let _ = at;
                self.world
                    .apply_player_push(body, m.handle, [transfer[0] / dt, transfer[1] / dt]);
            }
            if let Some(pose) = self.world.body_pose(m.handle) {
                // ⚠️ **`set_next_kinematic_pose`, nunca uma escrita direta:** é
                // ela que faz o solver tratar o corpo como MOVENDO-SE, e é isso
                // que o faz empurrar o que toca em vez de o atravessar.
                self.world.set_next_kinematic_pose(
                    m.handle,
                    pose.translation.x + got.translation[0],
                    pose.translation.y + got.translation[1],
                    pose.rotation.angle(),
                );
            }
            // ⚠️ **E o que o mundo NÃO deixou acontecer volta como velocidade**
            // — sem isto o personagem acelera contra uma parede para sempre e
            // sai disparado quando ela acaba (ver `kinematic_settle`).
            if let Some(st) = self.player_state.get_mut(&m.entity) {
                st.kin = kinematic_settle(
                    m.advanced,
                    m.wanted,
                    got.translation,
                    // ⚠️ **A pergunta do INTEGRADOR** (ver `KinematicState::grounded`)
                    // — quem responde é quem TOCOU no mundo. A resposta da LEI
                    // sobre chão continua a `footing`, nos dois modos (K4).
                    got.grounded,
                    dt,
                );
            }
        }
    }
}

/// **A PERNA** — como ela procura o chão e o que aceita como chão
/// (`W-FootFan`). Irmão por RESPONSABILIDADE: aqui mora o que o tique FAZ com
/// a resposta, lá mora como a resposta é obtida.
#[path = "player_leg.rs"]
mod player_leg;
#[path = "player_probes.rs"]
mod probes;
use probes::{probe_ceiling, probe_headroom, probe_wall};

/// O canal com o mundo de fora — ver o cabeçalho do módulo filho.
#[path = "player_channel.rs"]
mod channel;

#[path = "player_drops.rs"]
mod drops;

#[path = "player_push.rs"]
mod push;
use push::Push;

/// O handle do rapier, nomeado sem importar o rapier aqui — esta crate declara-se
/// *rapier-free* no próprio `Cargo.toml` e só carrega os tipos re-exportados.
mod rapier2d_handle {
    pub type Handle = ph2d_physics::RigidBodyHandle;
}
