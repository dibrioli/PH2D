//! **A SUPERFÍCIE CHEGA À LEI** (`W-Surface`) — os gates da ponte, e as sondas
//! que numeraram a cena.
//!
//! ⚠️ **O que estes gates cobrem que os do kernel não cobrem:** lá a lei recebe
//! uma `GroundSample` já montada, e a pergunta é *o que ela faz com o `grip`*.
//! Aqui a pergunta é a outra metade — **de onde o `grip` vem** —, e ela tem três
//! modos de falhar que só a ponte pode ter: perguntar ao corpo em vez do raio que
//! ganhou, montar a correia em eixos de mundo, e não voltar a olhar para o
//! componente depois de o artista o mexer.
//!
//! ⚠️ **Tudo pela porta do PRODUTO** (`PhysicsBridge::dispatch`) — as cenas são
//! montadas com os mesmos componentes que o Inspector escreve.
//!
//! # ⚠️ O RELÓGIO é do rig, e isso é estrutural
//!
//! O `dispatch` é função do TICK: pedir um tique **anterior** ao último é um
//! `rewind_to`, e a ponte reconstrói o mundo a partir da pose de REPOUSO. Uma
//! primeira versão destes gates tinha o contador dentro do helper de correr, que
//! recomeçava em `1` a cada chamada — então a fase de "derrapagem" era na verdade
//! uma **re-simulação do zero sem input**, e ela devolvia o personagem a `x = 0`.
//! Medido: a distância da 2ª fase saía **exatamente `−distância da 1ª`** em todo
//! `grip`, e dois gates ficavam VERDES comparando dois números NEGATIVOS.
//!
//! Por isso o tique mora no [`Rig`] e nunca é argumento de quem corre: *não há
//! como reiniciá-lo sem apagar o campo*.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, RigidBody, WalkSurface,
};

const FLOAT: f32 = 0.9;

/// A aceleração e o cruzeiro das cenas destes gates.
///
/// ⚠️ **BAIXOS de propósito, e medidos:** com os 60 m/s² do default o personagem
/// chega ao cruzeiro em poucos tiques mesmo com um quarto do `grip`, e a
/// diferença some no ruído. É a mesma nota que a cena 114 da `W-Brake` já
/// carrega — não é um default de produto, é o que torna a wave mensurável.
const RUN_ACCEL: f32 = 8.0;
const RUN_SPEED: f32 = 8.0;

/// A velocidade com que o personagem é LANÇADO nos gates de derrapagem.
///
/// ⚠️ **Lançar é o oráculo certo, e a distância de arranque NÃO é** — no gelo o
/// personagem nunca chega ao cruzeiro, então comparar quanto ele derrapa *depois
/// de arrancar sozinho* mede a mistura de duas coisas e sai ao contrário do que
/// a palavra "gelo" significa (medido: arranque de 1 s dá 5,25 m na madeira e
/// 1,35 m no gelo, logo a derrapagem da madeira é a MAIOR). Com a MESMA
/// velocidade inicial nos dois, o que sobra é só a tração.
const LAUNCH: f32 = 5.0;

/// Tiques de eixo SOLTO depois do spawn, antes de qualquer medição de arranque.
///
/// ⚠️ **A janela do spawn é AÉREA, e isso não é um defeito:** o corpo nasce à
/// altura de flutuação e a perna leva alguns tiques a achar o chão, então a lei
/// corre pelo braço `None` — controle aéreo, com o `grip` NEUTRO por construção
/// (não há superfície no ar). Medido com o eixo apertado desde o tique 1, um
/// `grip = 0` colhia **0,5 m/s ali e guardava-os para sempre** (0,250 m em 30
/// tiques e 0,500 m nos 60 seguintes — a MESMA taxa), que é exatamente o que
/// *gelo perfeito* promete: ele não arranca **e não pára**.
///
/// Soltar o eixo enquanto a perna assenta deixa a janela de arranque medir só a
/// tração.
const SETTLE_SPAWN: u64 = 30;

/// O banco: mundo, ponte e **o relógio**.
struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    tick: u64,
}

impl Rig {
    fn new() -> Self {
        Self {
            sim: SimWorld::new(),
            bridge: PhysicsBridge::new(),
            tick: 0,
        }
    }

    /// Corre `ticks` tiques com o eixo em `drive` e devolve **quanto andou em x**.
    fn run(&mut self, p: Entity, ticks: u64, drive: f32) -> f32 {
        let start = self.x(p);
        for _ in 0..ticks {
            self.tick += 1;
            self.bridge.set_player_input(
                p,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
            self.bridge.dispatch(&mut self.sim, true, self.tick);
        }
        self.x(p) - start
    }

    fn x(&self, p: Entity) -> f32 {
        self.sim
            .world()
            .get::<Transform>(p)
            .expect("transform")
            .translation
            .x
    }

    fn y(&self, p: Entity) -> f32 {
        self.sim
            .world()
            .get::<Transform>(p)
            .expect("transform")
            .translation
            .y
    }

    fn floor(&mut self, name: &str, at: Vec2, half: [f32; 2]) -> Entity {
        self.sim
            .world_mut()
            .spawn((
                Name::new(name),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: half[0],
                        half_y: half[1],
                    },
                    ..Collider::default()
                },
                Transform::from_translation(at),
            ))
            .id()
    }

    /// Uma rampa inclinada `deg` graus, centrada na ORIGEM, já com superfície.
    ///
    /// ⚠️ **Centrada na origem de propósito:** a 1ª versão punha a rampa em
    /// `(18, -0.5)` e o topo dela em `x = 0` caía a `y = -6,53` — o personagem
    /// nascia sete metros no AR e o gate media uma QUEDA (`0,900 -> -5,490`).
    fn ramp(&mut self, deg: f32, surface: WalkSurface) -> Entity {
        self.sim
            .world_mut()
            .spawn((
                Name::new("Ramp"),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 20.0,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                surface,
                Transform {
                    rotation: deg.to_radians(),
                    ..Transform::from_translation(Vec2::ZERO)
                },
            ))
            .id()
    }

    /// Onde o pé pousa sobre a [`Self::ramp`] em `x = 0`.
    fn ramp_top_y(deg: f32) -> f32 {
        let (s, c) = (deg.to_radians().sin(), deg.to_radians().cos());
        (0.5 * s / c) * s + 0.5 * c
    }

    fn player(&mut self, at: Vec2) -> Entity {
        self.sim
            .world_mut()
            .spawn((
                Name::new("Player"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: FLOAT,
                    speed: RUN_SPEED,
                    acceleration: RUN_ACCEL,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(at),
            ))
            .id()
    }

    /// O mesmo personagem, LANÇADO para a direita.
    fn launched_player(&mut self, at: Vec2) -> Entity {
        let p = self.player(at);
        self.sim.world_mut().entity_mut(p).insert(InitialVelocity {
            linvel: [LAUNCH, 0.0],
            angvel: 0.0,
        });
        p
    }
}

/// **A SONDA que numerou a FAIXA do slider** — onde o `grip` deixa de fazer
/// diferença, com os defaults de PRODUTO (`WalkConfig::STARTING_POINT`).
///
/// `cargo test -p ph2d-physics-ecs --test walk_surface measure_where_the_grip_saturates -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_where_the_grip_saturates() {
    eprintln!("  grip   0,1 s (m)   1,0 s (m)");
    for g in [1.0_f32, 2.0, 4.0, 6.0, 8.0, 12.0, 20.0] {
        let mut r = Rig::new();
        let deck = r.floor("Deck", Vec2::new(20.0, -0.5), [40.0, 0.5]);
        r.sim
            .world_mut()
            .entity_mut(deck)
            .insert(WalkSurface { grip: g, belt: 0.0 });
        // ⚠️ **O pincel de PRODUTO, não o desta suíte** — as consts daqui são
        // baixas de propósito para a diferença ser mensurável, e a faixa de um
        // slider tem de sair do que o artista de facto encontra.
        let p = r
            .sim
            .world_mut()
            .spawn((
                Name::new("Player"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: FLOAT,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(Vec2::new(0.0, FLOAT)),
            ))
            .id();
        r.run(p, SETTLE_SPAWN, 0.0);
        let x0 = r.x(p);
        let _ = r.run(p, 6, 1.0);
        let short = r.x(p) - x0;
        let _ = r.run(p, 54, 1.0);
        eprintln!("  {g:4.1}   {short:9.4}   {:9.4}", r.x(p) - x0);
    }
}

/// **A SONDA da correia em rampa** — quanto ARCO ela carrega por inclinação.
///
/// `cargo test -p ph2d-physics-ecs --test walk_surface measure_the_belt_on_a_ramp -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_belt_on_a_ramp() {
    eprintln!("  graus   arco (m)   subida (m)");
    for deg in [0.0_f32, 20.0, 40.0] {
        let mut r = Rig::new();
        r.ramp(
            deg,
            WalkSurface {
                grip: 1.0,
                belt: 3.0,
            },
        );
        let p = r.player(Vec2::new(0.0, Rig::ramp_top_y(deg) + FLOAT));
        r.run(p, SETTLE_SPAWN, 0.0);
        let (x0, y0) = (r.x(p), r.y(p));
        let _ = r.run(p, 120, 0.0);
        let (dx, dy) = (r.x(p) - x0, r.y(p) - y0);
        eprintln!(
            "  {deg:5.1}   {:8.3}   {dy:10.3}",
            (dx * dx + dy * dy).sqrt()
        );
    }
}

/// **A SONDA que numerou a cena** — o que cada `grip` faz num piso desta config.
///
/// `cargo test -p ph2d-physics-ecs --test walk_surface measure_the_ice -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_ice() {
    eprintln!("  grip   arranca 1s (m)   derrapa de 5 m/s (m)");
    for g in [0.0_f32, 0.1, 0.15, 0.25, 0.5, 1.0] {
        // Arranque do zero.
        let mut r = Rig::new();
        let deck = r.floor("Deck", Vec2::new(20.0, -0.5), [40.0, 0.5]);
        r.sim
            .world_mut()
            .entity_mut(deck)
            .insert(WalkSurface { grip: g, belt: 0.0 });
        let p = r.player(Vec2::new(0.0, FLOAT));
        r.run(p, SETTLE_SPAWN, 0.0);
        let started = r.run(p, 60, 1.0);

        // Derrapagem a partir da MESMA velocidade.
        let mut r = Rig::new();
        let deck = r.floor("Deck", Vec2::new(20.0, -0.5), [40.0, 0.5]);
        r.sim
            .world_mut()
            .entity_mut(deck)
            .insert(WalkSurface { grip: g, belt: 0.0 });
        let p = r.launched_player(Vec2::new(0.0, FLOAT));
        let skid = r.run(p, 600, 0.0);

        eprintln!("  {g:4.2}   {started:12.3}   {skid:19.3}");
    }
}

/// **Gelo derrapa mais e arranca mais devagar — e o chão é quem diz.**
///
/// ⚠️ **É o gate que prova que a autoria CHEGA à lei.** Os gates do kernel
/// recebem a amostra pronta; este monta a cena com o componente que o Inspector
/// escreve e mede o `Transform` que o artista vê.
///
/// ⚠️ **As duas metades medem coisas DIFERENTES e nenhuma basta sozinha:** a
/// derrapagem parte da MESMA velocidade (senão ela mede o arranque de novo, ao
/// contrário — ver [`LAUNCH`]) e o arranque parte do MESMO repouso.
///
/// **Mutação que deve sangrar:** o `sample` da ponte usar `WalkSurface::NEUTRAL`
/// em vez de `self.surfaces.at(h)`.
#[test]
fn an_icy_floor_makes_him_slide_and_a_normal_one_does_not() {
    fn deck_with(grip: f32) -> (Rig, Entity) {
        let mut r = Rig::new();
        let deck = r.floor("Deck", Vec2::new(20.0, -0.5), [40.0, 0.5]);
        if grip != 1.0 {
            r.sim
                .world_mut()
                .entity_mut(deck)
                .insert(WalkSurface { grip, belt: 0.0 });
        }
        (r, deck)
    }

    fn skid(grip: f32) -> f32 {
        let (mut r, _) = deck_with(grip);
        let p = r.launched_player(Vec2::new(0.0, FLOAT));
        r.run(p, 600, 0.0)
    }

    fn start(grip: f32) -> f32 {
        let (mut r, _) = deck_with(grip);
        let p = r.player(Vec2::new(0.0, FLOAT));
        r.run(p, SETTLE_SPAWN, 0.0);
        r.run(p, 60, 1.0)
    }

    let (wood_skid, ice_skid) = (skid(1.0), skid(0.15));
    assert!(
        ice_skid > wood_skid * 3.0,
        "do MESMO lancamento, o gelo derrapa muito mais: {ice_skid:.3} vs {wood_skid:.3}"
    );

    let (wood_start, ice_start) = (start(1.0), start(0.15));
    assert!(
        ice_start < wood_start * 0.5,
        "e do MESMO repouso arranca muito menos: {ice_start:.3} vs {wood_start:.3}"
    );
}

/// **A superfície é da PEÇA que o pé encontrou, não do corpo dono dela.**
///
/// ⚠️ **É a lei *"quem responde é o MESMO raio que ganhou"*, no único sítio onde
/// ela pode falhar.** Um corpo composto (W-Compound) tem várias formas; uma
/// plataforma com uma metade de gelo e outra de madeira é o caso de uso, e
/// perguntar ao CORPO daria a mesma resposta nas duas — o personagem andaria no
/// gelo e derraparia na madeira dentro do mesmo tique.
///
/// ⚠️ **O corpo tem de carregar uma superfície PRÓPRIA, e sem ela o gate não
/// pode falhar:** numa 1ª versão só a peça era autorada, então `by_body` ficava
/// VAZIO e trocar a ordem da consulta não mudava resposta nenhuma — a mutação
/// abaixo passava verde. É o caso de uso real que fecha o buraco: uma plataforma
/// com uma face de BORRACHA e outra de GELO, em que a peça **sobrepõe** o corpo.
///
/// **Mutação que deve sangrar:** consultar `by_body` antes de `by_collider` na
/// `Surfaces::at`.
#[test]
fn the_surface_comes_from_the_part_the_foot_found() {
    fn skid_over(x0: f32) -> f32 {
        let mut r = Rig::new();
        // O corpo: a metade de BORRACHA, à esquerda (x de -20 a 0).
        let deck = r.floor("Deck", Vec2::new(-10.0, -0.5), [10.0, 0.5]);
        r.sim.world_mut().entity_mut(deck).insert(WalkSurface {
            grip: 4.0,
            belt: 0.0,
        });
        // A PEÇA: a metade de GELO, à direita (x de 0 a 40) — filho com
        // `Collider` e sem `RigidBody`, que é a definição de peça.
        let ice = r
            .sim
            .world_mut()
            .spawn((
                Name::new("Ice"),
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 20.0,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                WalkSurface {
                    grip: 0.15,
                    belt: 0.0,
                },
                // ⚠️ **LOCAL, não mundo** (a lição do W5): o pai está em
                // `(-10, -0.5)`, então esta translação tem de ser a diferença
                // para pousar em `(20, -0.5)`. A 1ª versão escreveu a coordenada
                // de MUNDO aqui e a peça caiu meio metro abaixo do deck — o
                // personagem passava a janela toda no AR, onde o eixo solto
                // FREIA, e o gate media a travagem aérea (0,473 m) em vez da
                // tração da peça.
                Transform::from_translation(Vec2::new(30.0, 0.0)),
            ))
            .id();
        r.sim.world_mut().entity_mut(ice).insert(ChildOf(deck));

        let p = r.launched_player(Vec2::new(x0, FLOAT));
        r.run(p, 600, 0.0)
    }

    // Sobre a borracha (a forma do próprio corpo) ele pára curto; sobre a peça
    // de gelo ele derrapa. É a MESMA cena e o MESMO corpo dono.
    let on_rubber = skid_over(-15.0);
    let on_ice = skid_over(2.0);
    assert!(
        on_ice > on_rubber * 3.0,
        "a peca de gelo derrapa e a forma do corpo nao: {on_ice:.3} vs {on_rubber:.3}"
    );
}

/// **A ESTEIRA leva quem está de pé, e a correia é lida ao longo da SUPERFÍCIE.**
///
/// ⚠️ **A metade da rampa é o que torna a correia um ESCALAR:** um vetor autorado
/// em eixos de mundo teria componente ao longo da NORMAL, e uma superfície não
/// empurra ninguém para dentro nem para fora de si mesma. Sobre uma rampa a
/// esteira tem de SUBIR — e uma montada em `[belt, 0]` empurraria contra o chão.
///
/// ⚠️ **O personagem tem de NASCER sobre a rampa**, e a 1ª fixture não o punha
/// lá: com a rampa centrada longe da origem a superfície dela passava 7 m ABAIXO
/// do ponto de spawn, e o gate media uma QUEDA (0,900 -> -5,490). A geometria
/// abaixo põe o topo da rampa a `x = 0` e o personagem em cima dele.
///
/// **Mutação que deve sangrar:** montar a correia em `[belt, 0]` na
/// `ground_velocity_with_belt` (a rampa deixa de subir).
#[test]
fn a_belt_carries_him_along_the_surface_uphill_included() {
    // Plano: a correia leva quem não toca em nada.
    let mut r = Rig::new();
    let deck = r.floor("Belt", Vec2::new(20.0, -0.5), [40.0, 0.5]);
    r.sim.world_mut().entity_mut(deck).insert(WalkSurface {
        grip: 1.0,
        belt: 3.0,
    });
    let p = r.player(Vec2::new(0.0, FLOAT));
    // ⚠️ **As duas metades assentam a perna antes de medir, e é o que torna a
    // comparação honesta:** sem isto o plano media 5,476 m (a janela do spawn é
    // AÉREA e a correia não alcança quem não a toca) contra 6,000 na rampa —
    // 9,6% de desvio contra uma barra de 10%, um gate a um fio de flakar sobre
    // produto correto.
    r.run(p, SETTLE_SPAWN, 0.0);
    let carried = r.run(p, 120, 0.0);
    assert!(
        carried > 4.0,
        "a correia leva quem esta de pe sem tocar em nada: {carried:.3} m em 2 s"
    );

    // Rampa: a MESMA correia carrega o MESMO ARCO por ela.
    //
    // ⚠️ **O oráculo é a INVARIÂNCIA do arco, não a subida** — e a diferença
    // custou uma mutação sobrevivente. Numa encosta caminhável a tangente e o
    // eixo-x do mundo têm SEMPRE o mesmo sinal em x, então *"ele sobe"* é
    // verdade nas duas leis e não separa nada. O que separa é que uma correia
    // é um ESCALAR ao longo da tangente: o comprimento percorrido não sabe da
    // inclinação. Medido (`measure_the_belt_on_a_ramp`), 2 s a 3 m/s:
    //
    // | graus | arco (m) | com o eixo de MUNDO |
    // |---|---|---|
    // | 0  | 6,000 | 6,000 |
    // | 20 | 6,000 | **22,832** |
    // | 40 | 6,000 | **63,526** |
    //
    // ⚠️ E o desvio **não é o `cos θ`** que a aritmética sugere: um vetor de
    // mundo tem componente ao longo da NORMAL, a PERNA a lê como um chão que
    // afunda, e o personagem é ARREMESSADO (55 m de subida a 40°). É a frase do
    // doc do campo, medida — *uma superfície não empurra ninguém para dentro
    // nem para fora de si mesma*.
    const SLOPE_DEG: f32 = 40.0;

    let mut r = Rig::new();
    r.ramp(
        SLOPE_DEG,
        WalkSurface {
            grip: 1.0,
            belt: 3.0,
        },
    );
    let p = r.player(Vec2::new(0.0, Rig::ramp_top_y(SLOPE_DEG) + FLOAT));
    // Deixa a perna assentar antes de medir — a queda do spawn não é o que
    // este gate mede.
    r.run(p, SETTLE_SPAWN, 0.0);
    let (x0, y0) = (r.x(p), r.y(p));
    let _ = r.run(p, 120, 0.0);
    let (dx, dy) = (r.x(p) - x0, r.y(p) - y0);
    let arc = (dx * dx + dy * dy).sqrt();
    assert!(
        (arc - carried).abs() < 0.1 * carried,
        "o arco da correia nao sabe da inclinacao: {arc:.3} m na rampa contra {carried:.3} m no plano"
    );
    assert!(
        dy > 0.5,
        "e a correia da rampa SOBE, nao desce: {y0:.3} -> {:.3}",
        r.y(p)
    );
}

/// **A correia leva por TRAÇÃO: sem `grip` ela não leva nada.**
///
/// ⚠️ **É a propriedade EMERGENTE da wave** — nada no código diz isto, ele cai da
/// composição: a correia chega como velocidade do chão, e o que fecha a distância
/// entre o corpo e o chão é o orçamento, que o `grip` multiplica.
///
/// **Mutação que deve sangrar:** neutralizar o `grip` **e deixar a correia** —
/// `grip: NEUTRAL_GRIP` com o `belt` intacto ⇒ ele é levado 6 m onde deve ficar
/// parado.
///
/// ⚠️ **E NÃO a mutação do primeiro gate — a nota anterior aqui dizia que era, e
/// era FALSA.** Ignorar a superfície INTEIRA devolve o neutro, e o neutro é
/// `grip: 1.0` **e** `belt: 0.0`: sem correia ninguém leva nada, e este gate
/// fica VERDE pelo motivo errado. As duas metades do componente têm de ser
/// mutadas em SEPARADO, senão uma cobre a outra.
#[test]
fn a_frictionless_belt_carries_nothing() {
    let mut r = Rig::new();
    let deck = r.floor("Belt", Vec2::new(20.0, -0.5), [40.0, 0.5]);
    r.sim.world_mut().entity_mut(deck).insert(WalkSurface {
        grip: 0.0,
        belt: 3.0,
    });
    let p = r.player(Vec2::new(0.0, FLOAT));
    let carried = r.run(p, 120, 0.0);
    assert!(
        carried.abs() < 0.05,
        "uma correia sem grip nao tem por onde puxar: {carried:.4} m"
    );
}

/// **Mexer no componente morde no tique SEGUINTE** — sem respawn, sem re-carimbo.
///
/// ⚠️ **É a razão de a superfície não ridar o `BodyDesc`.** Se ela viajasse na
/// receita de spawn, arrastar o slider exigiria re-nascer o collider — ou o passe
/// de re-carimbo por dispatch que o `bridge::damping` teve de construir por ter
/// escolhido o outro caminho.
///
/// ⚠️ **O oráculo é o ARRANQUE, não a derrapagem:** as três janelas partem do
/// repouso (a fase de assentamento entre elas é longa o bastante para o gelo
/// parar), então o que muda entre elas é só a tração — e é ela que o slider move.
///
/// **Mutação que deve sangrar:** construir a tabela uma vez (mover o
/// `reconcile_surfaces` para fora do `prepare`).
#[test]
fn editing_the_surface_mid_play_bites_on_the_next_tick() {
    /// Tiques de eixo solto entre janelas — o gelo é o lento a parar, e a 5 m/s
    /// ele leva ~4 s.
    const SETTLE: u64 = 600;
    const WINDOW: u64 = 60;

    let mut r = Rig::new();
    let deck = r.floor("Deck", Vec2::new(20.0, -0.5), [40.0, 0.5]);
    let p = r.player(Vec2::new(0.0, FLOAT));
    r.run(p, SETTLE_SPAWN, 0.0);

    // Chão normal.
    let before = r.run(p, WINDOW, 1.0);
    r.run(p, SETTLE, 0.0);

    // O artista põe gelo com o relógio A ANDAR.
    r.sim.world_mut().entity_mut(deck).insert(WalkSurface {
        grip: 0.15,
        belt: 0.0,
    });
    let after = r.run(p, WINDOW, 1.0);
    r.run(p, SETTLE, 0.0);
    assert!(
        after < before * 0.5,
        "o gelo posto com o relogio a andar morde: {after:.3} vs {before:.3}"
    );

    // E TIRAR o componente devolve o chão de sempre.
    r.sim.world_mut().entity_mut(deck).remove::<WalkSurface>();
    let restored = r.run(p, WINDOW, 1.0);
    assert!(
        (restored - before).abs() < before * 0.1,
        "tirar o componente devolve o chao de sempre: {restored:.3} vs {before:.3}"
    );
}
