//! **O PERSONAGEM NADA** — os gates da W-Swim, na água de verdade.
//!
//! O `player_in_water.rs` ao lado pergunta *o que a água FAZ ao personagem*
//! (empuxo, arrasto, o bobeio); este pergunta *o que o personagem faz na água*.
//!
//! # ⚠️ O oráculo é DIRECIONAL, e é deliberado
//!
//! Um literal de altura seria um espelho da aritmética do servo: ele passaria a
//! afirmar *"o nado empurra exatamente 1,84 m em três segundos"*, que é uma
//! frase sobre a `swim_speed` da fixture e não sobre a lei. O que estes gates
//! afirmam é o que o jogador vê — *ele SOBE quando eu peço para subir, DESCE
//! quando peço para descer, e sem eu pedir nada não faz nem uma coisa nem
//! outra* —, sempre contra o MESMO tique da capacidade desligada.
//!
//! ⚠️ **E a paridade entre modos entra em cada um**, como no irmão: a espécie do
//! corpo não pode ser uma pergunta que a água faça.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test player_swims`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

/// A cápsula e a poça das fixtures do player — os MESMOS números do
/// `player_in_water.rs` e do `measure_the_swim_threshold`, para os três falarem
/// da mesma cena.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const FLUID: f32 = 4.0;
const DRAG: f32 = 0.6;

/// Um nado ligado. ⚠️ A capacidade nasce DESLIGADA, então toda fixture que a
/// queira tem de a ligar — e é isso que faz o `off` destes gates ser o produto
/// de ontem, e não um caso especial.
const SWIM_SPEED: f32 = 4.0;

/// A autoridade do servo — o ponto de partida da lei.
const SWIM_ACCEL: f32 = 12.0;

/// Onde o sujeito é largado: **já submerso**, para o gate medir o REGIME e não a
/// entrada na água (o transiente que o irmão mede).
const START: f32 = -1.0;

/// A poça funda: superfície em `y = 0`, nada ao alcance do sensor de chão.
fn pool(sim: &mut SimWorld) {
    pool_of(sim, FLUID);
}

/// A mesma poça com a densidade que se pedir — o que faz da **razão de
/// densidades** uma variável do gate da linha de flutuação, em vez de um número
/// fixo da fixture.
fn pool_of(sim: &mut SimWorld, fluid: f32) {
    sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 6.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(fluid),
        AreaDrag(DRAG),
        Transform::from_translation(Vec2::new(0.0, -6.0)),
    ));
}

/// Um chão sólido, para o gate de vadear.
fn floor(sim: &mut SimWorld, y: f32) {
    sim.world_mut().spawn((
        Name::new("Floor"),
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
        Transform::from_translation(Vec2::new(0.0, y - 0.5)),
    ));
}

fn player(sim: &mut SimWorld, kinematic: bool, swim: f32, accel: f32, start: f32) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            swim_speed: swim,
            swim_acceleration: accel,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, start)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn pos_of(sim: &SimWorld) -> Vec2 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation;
        }
    }
    panic!("o sujeito tem de existir");
}

/// Seis segundos de nado numa poça funda, com a entrada que se pedir.
///
/// Devolve `(x percorrido, altura MÉDIA da segunda metade)` — as duas grandezas
/// que o jogador julga.
///
/// ⚠️ **A vertical é uma MÉDIA, e a primeira versão deste harness lia um
/// instante.** O personagem na água OSCILA (o `1,44 m` que o `player_in_water`
/// mede e o plano 07 §8.4 explica), então uma amostra em `t = 180` compara a
/// FASE de duas oscilações: medido, *segurar para baixo* lia `1,191` contra
/// `1,136` de *não pedir nada* — a ordem invertida, por 5%, sobre leis que na
/// saturação produzem o MESMO motor. É a mesma lição que o gate irmão pagou
/// (*"uma amostra única de um sistema que oscila não é um repouso"*), e ela
/// reincide sempre que alguém escreve um harness novo.
fn swims(kinematic: bool, swim: f32, input: PlayerInput) -> Vec2 {
    swims_with(kinematic, swim, SWIM_ACCEL, input)
}

/// Como a [`swims`], com a AUTORIDADE do servo escolhida.
fn swims_with(kinematic: bool, swim: f32, accel: f32, input: PlayerInput) -> Vec2 {
    let mut sim = SimWorld::new();
    pool(&mut sim);
    let who = player(&mut sim, kinematic, swim, accel, START);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, input);
    let (mut sum, mut n) = (0.0f64, 0u32);
    let mut last = Vec2::new(0.0, 0.0);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        last = pos_of(&sim);
        if t > 180 {
            sum += f64::from(last.y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    Vec2::new(last.x, (sum / f64::from(n)) as f32 - START)
}

/// **ONDE ELE ASSENTA** — vinte segundos parado numa poça da densidade pedida,
/// e a altura MÉDIA do último terço.
///
/// ⚠️ **Vinte segundos e não seis**: a boia sobe pelo empuxo e depois oscila, e
/// uma janela curta mede o TRANSIENTE. O que este harness pergunta é o repouso,
/// que é a única grandeza em que a lei e a água podem ser comparadas.
///
/// ⚠️ **A autoridade é DERIVADA da poça, e a fixture não presta sem isso.** Com
/// o `SWIM_ACCEL` de partida (`12`) o servo não vence o empuxo de uma poça
/// `1,25×`, então o nadador sobe quase até à linha **sozinho** — e a mutação que
/// devolve o alvo-zero erra por `0,08 m` em vez de `0,72 m`: o gate ficaria
/// verde-por-fraqueza no dia em que a tolerância subisse um pouco. A regra é a
/// do gate do mergulho ao lado: `|g| · (razão − 1)` é o que a água cobra, e a
/// fixture paga `1,5×` isso.
///
/// ⚠️ **E ele é largado FUNDO, o que também não é folclore.** Com o
/// [`START`] dos outros gates (`−1,0`) a lei defeituosa ainda chega quase à
/// linha — ela sobe *um tique de empuxo por tique*, e a `0,7 m` de distância os
/// vinte segundos quase bastam: a mutação erraria `0,08 m` e passaria por
/// qualquer tolerância honesta. Largado a `−2,5`, onde o artista de facto está
/// depois de mergulhar, o mesmo defeito erra **`0,56 m`**. *A distância à linha
/// é parte da fixture.*
///
/// ⚠️ **E é por isso que o gate varre TRÊS densidades:** na poça `4×` o empuxo é
/// tão forte (`29,4 m/s²`) que mesmo a lei defeituosa é empurrada até à linha
/// dentro da janela — ela passaria sozinha. Quem a denuncia é a poça `2×`, onde
/// o empuxo é fraco o bastante para o congelamento durar.
fn settles(kinematic: bool, swim: f32, fluid: f32) -> f32 {
    const DEEP: f32 = -2.5;
    let mut sim = SimWorld::new();
    pool_of(&mut sim, fluid);
    let authority = 9.81 * (fluid - 1.0) * 1.5;
    let who = player(&mut sim, kinematic, swim, authority, DEEP);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());
    let (mut sum, mut n) = (0.0f64, 0u32);
    for t in 1..=1200u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 800 {
            sum += f64::from(pos_of(&sim).y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32)
}

fn holding(jump: bool, down: bool, drive: f32) -> PlayerInput {
    PlayerInput {
        drive,
        jump,
        down,
        ..PlayerInput::default()
    }
}

/// **A CAPACIDADE DESLIGADA É O MUNDO DE ONTEM** — o controle de todos os
/// outros, e o gate que qualquer termo desta wave tem de deixar em paz.
///
/// ⚠️ Com o nado em zero o personagem faz o que sempre fez numa poça: sobe pelo
/// empuxo, e os botões não significam nada dentro d'água.
#[test]
fn the_capability_off_is_yesterdays_world() {
    for kinematic in [false, true] {
        let idle = swims(kinematic, 0.0, PlayerInput::default());
        // ⚠️ **Só o BOTÃO muda entre as duas chamadas.** A primeira versão deste
        // gate variava o `drive` junto e comparava a VERTICAL — e o `drive` move
        // o personagem 17 m de lado pelo controle aéreo, o que muda o arrasto e
        // portanto a altura. *Uma ablação que mexe em duas entradas não atribui
        // a nenhuma.*
        let pressing = swims(kinematic, 0.0, holding(true, true, 0.0));
        assert!(
            (idle.y - pressing.y).abs() < 1.0e-3,
            "desligado, os botoes nao movem a vertical ({kinematic}): {idle:?} vs {pressing:?}"
        );
        assert!(
            idle.y > 0.0,
            "e o empuxo continua a levantar ({kinematic}): {idle:?}"
        );
    }
}

/// **PEDIR PARA SUBIR SOBE, PEDIR PARA DESCER DESCE** — a wave inteira, num
/// oráculo que é o do jogador.
///
/// ⚠️ **O controle é a MESMA cena com o nado desligado**, e não um número
/// escrito à mão: o que este gate afirma é que o botão MOVE a resposta, em cada
/// modo, e não que ela vale tanto.
#[test]
fn up_rises_and_down_dives_in_both_modes() {
    for kinematic in [false, true] {
        let up = swims(kinematic, SWIM_SPEED, holding(true, false, 0.0)).y;
        let idle = swims(kinematic, SWIM_SPEED, PlayerInput::default()).y;
        let down = swims(kinematic, SWIM_SPEED, holding(false, true, 0.0)).y;
        // ⚠️ **A ordem, e não um sinal absoluto:** nesta poça o empuxo líquido
        // vale `g·(4−1) ≈ 29,4 m/s²` e a autoridade default do servo é `12`,
        // então **pedir para descer ainda SOBE** — só mais devagar. Quem afirma
        // que dá para mergulhar de facto é o gate seguinte, e é ele que carrega
        // a relação medida.
        assert!(
            down < idle && idle < up,
            "{down} < {idle} < {up} ({kinematic})"
        );
    }
}

/// **O eixo horizontal responde ao `drive`**, e nos dois sentidos.
#[test]
fn the_stroke_carries_him_sideways() {
    for kinematic in [false, true] {
        let right = swims(kinematic, SWIM_SPEED, holding(false, false, 1.0)).x;
        let left = swims(kinematic, SWIM_SPEED, holding(false, false, -1.0)).x;
        assert!(right > 1.0, "nada para a direita ({kinematic}): {right}");
        assert!(left < -1.0, "e para a esquerda ({kinematic}): {left}");
        assert!(
            (right + left).abs() < 0.1 * right,
            "e os dois lados sao simetricos ({kinematic}): {right} vs {left}"
        );
    }
}

/// ⚠️ **ATRAVESSAR UMA POÇA RASA É ANDAR** — o limiar existe para isto, e o
/// preço de não o ter seria o personagem a parar de caminhar ao molhar os pés.
///
/// ⚠️ **A primeira versão deste gate montou uma cena IMPOSSÍVEL:** ela punha o
/// personagem de pé no fundo de uma poça funda e exigia que ele lá ficasse — mas
/// nesta fixture o fluido é **quatro vezes mais denso** que ele, então o empuxo
/// líquido aponta para cima com três pesos e **ninguém consegue ficar de pé lá**
/// (medido: ele subia a `+0,24` depois de andar 7,3 m). *A física estava certa e
/// a fixture é que descrevia um mundo que não existe.*
///
/// A poça que se atravessa a pé é rasa por definição: aqui o chão deixa o
/// personagem **20% submerso**, que a tabela do `measure_the_swim_threshold` lê
/// como `buoyed = 0,68` — abaixo do limiar default de `1,0`.
#[test]
fn crossing_a_shallow_puddle_is_walking() {
    for kinematic in [false, true] {
        let mut sim = SimWorld::new();
        // A poça: superfície em `y = 0`.
        sim.world_mut().spawn((
            Name::new("Puddle"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: 20.0,
                    half_y: 2.0,
                },
                ..Collider::default()
            },
            AreaBuoyancy(FLUID),
            AreaDrag(DRAG),
            Transform::from_translation(Vec2::new(0.0, -2.0)),
        ));
        // O chão a `-0,6`: o personagem paira `FLOAT` acima dele, ou seja em
        // `+0,3` — os 20% submersos da tabela.
        floor(&mut sim, -0.6);
        let who = player(&mut sim, kinematic, SWIM_SPEED, SWIM_ACCEL, 0.3);
        let mut bridge = PhysicsBridge::new();
        bridge.set_player_input(who, holding(false, false, 1.0));
        for t in 1..=120u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let p = pos_of(&sim);
        assert!(
            p.x > 2.0,
            "ele tem de ATRAVESSAR a poca ({kinematic}): {p:?}"
        );
        // E continua com os pés no chão — a caminhada nunca virou braçada.
        assert!(
            (p.y - 0.3).abs() < 0.15,
            "e a pe', na altura de flutuacao ({kinematic}): {p:?}"
        );
    }
}

/// **O EMPUXO continua a ser quem o segura** — o nado não cancela a gravidade,
/// então com o dedo parado o personagem sobe **na mesma direção** que subiria
/// sem a capacidade.
///
/// ⚠️ **Este gate afirmava a coisa errada, e a medição o corrigiu.** A versão
/// anterior pedia `idle < off` — *"ele sobe MENOS, porque o servo freia para o
/// alvo zero"* —, o que era uma descrição fiel de uma lei DEFEITUOSA: o alvo
/// zero congelava o nadador onde ele estivesse (`measure_the_float_line`: poça
/// `1,25×`, **100% submerso** com o nado ligado contra **80%** sem ele). Hoje o
/// repouso procura a LINHA, e a afirmação é de IGUALDADE — ver o gate abaixo, do
/// qual este é a metade direcional.
#[test]
fn an_idle_swimmer_still_answers_to_the_water() {
    for kinematic in [false, true] {
        let idle = swims(kinematic, SWIM_SPEED, PlayerInput::default()).y;
        assert!(
            idle > 0.0,
            "parado, o empuxo ainda o levanta ({kinematic}): {idle}"
        );
    }
}

/// ⚠️ **LIGAR O NADO NÃO PODE MOVER A LINHA DE FLUTUAÇÃO** — o gate da pergunta
/// do Enio (*"não temos parâmetros para o quanto fica submerso quando boia?"*).
///
/// **Tem, e são as duas DENSIDADES** — a submersão de repouso é `1/razão`,
/// exata (`measure_the_float_line`: **24,5% · 50,1% · 80,1%** nas poças `4×` ·
/// `2×` · `1,25×`, contra `25 · 50 · 80` previstos). O que faltava era a lei do
/// nado **honrar** essa linha em vez de a substituir por um congelamento.
///
/// ⚠️ **O oráculo é o mundo SEM a capacidade**, e é o mais forte que existe
/// aqui: um literal de altura seria um espelho da fixture, e uma desigualdade
/// (*"sobe menos"*) foi precisamente o que deixou o defeito passar. A pergunta é
/// *ligar o nado move onde ele para?* — e a resposta tem de ser **não**.
///
/// ⚠️ **A tolerância é `0,05 m` sobre um corpo de `1,0 m`** (5% da altura), e não
/// é folga escolhida: a boia OSCILA em torno da linha (amplitude medida `0,43 m`
/// a `4×`) enquanto o nadador **assenta** nela, então as duas médias não podem
/// coincidir ao dígito. A mutação que importa erra por **mais de um metro**.
#[test]
fn turning_the_swimming_on_does_not_move_the_float_line() {
    for kinematic in [false, true] {
        for fluid in [4.0f32, 2.0, 1.25] {
            let off = settles(kinematic, 0.0, fluid);
            let on = settles(kinematic, SWIM_SPEED, fluid);
            assert!(
                (on - off).abs() < 0.05,
                "com o fluido {fluid}x ({kinematic}) ele tem de boiar onde sempre boiou: \
                 nado {on:.4} vs boia {off:.4}"
            );
        }
    }
}

/// ⚠️ **MERGULHAR PEDE MAIS AUTORIDADE DO QUE A ÁGUA TEM** — o achado desta
/// wave, e a razão de a `swim_acceleration` ser um número que o artista mexe.
///
/// A doc do campo diz que ele é *autoridade contra o empuxo*; este gate torna a
/// frase um NÚMERO. Numa poça `d` vezes mais densa que o corpo, o empuxo líquido
/// sobre um corpo submerso vale `|g|·(d − 1)` — aqui `9,81 · 3 ≈ 29,4 m/s²` —,
/// então um servo de `12` **não consegue descer**: ele apenas sobe mais devagar.
///
/// ⚠️ **Isto não é um defeito, é a física da cena**, e o gate existe para que
/// ninguém o "conserte" capando o empuxo: uma rolha não mergulha por querer. O
/// que a lei promete é que **subir a autoridade acima do empuxo devolve o
/// mergulho**, e é isso que a segunda metade afirma.
#[test]
fn diving_needs_more_authority_than_the_water_has() {
    // O empuxo líquido que o servo tem de vencer, submerso — derivado da cena,
    // nunca de um literal: `|g| · (densidade_fluido/densidade_corpo − 1)`.
    let net_lift = 9.81 * (FLUID - 1.0);
    for kinematic in [false, true] {
        let weak = swims_with(kinematic, SWIM_SPEED, SWIM_ACCEL, holding(false, true, 0.0)).y;
        let strong = swims_with(
            kinematic,
            SWIM_SPEED,
            net_lift * 1.5,
            holding(false, true, 0.0),
        )
        .y;
        assert!(
            weak > 0.0,
            "com autoridade abaixo do empuxo ele ainda SOBE ({kinematic}): {weak}"
        );
        assert!(
            strong < 0.0,
            "acima do empuxo, o mergulho existe ({kinematic}): {strong}"
        );
    }
}

/// **⚠️ O PLANEIO NÃO EXISTE DENTRO D'ÁGUA** (`W-Glide`).
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO QUE SOBREVIVEU:** tirar o `!swimming` da
/// guarda do planeio no `player_motor` deixava os 194 gates da lei e os 5 do
/// produto **VERDES** — nenhum deles põe um nadador com o dedo no pulo.
///
/// ⚠️ **E o defeito é dois-donos, não uma sobreposição inofensiva:** dentro
/// d'água o botão de pulo **já significa subir** (o `swim_rise` lê o MESMO
/// `input.jump`), então sem a guarda o mesmo dedo pede as duas coisas e o
/// eixo vertical ganha um freio que ninguém autorou.
///
/// O oráculo é a subida: com o teto de planeio armado, pedir para SUBIR tem de
/// dar exatamente o que dá sem ele.
///
/// ⚠️ **E a fixture tem de o manter SUBMERSO, o que a primeira versão não
/// fazia:** com 180 tiques ele nada 1,78 m, **sai da água** (a superfície está
/// em `y = 0` e ele parte de `−1`), e fora dela o planeio age — legitimamente. O
/// gate media 1,7851 contra 1,8094 e reprovava o produto CORRETO. A premissa
/// *"ele continua dentro"* é agora afirmada, e não suposta.
///
/// ⚠️ **E encurtar a janela não bastava** — a `SWIM_SPEED` é 4 m/s, então de
/// `−1` ele cruza a superfície em ~15 tiques. Quem tinha de mudar era a
/// PROFUNDIDADE de partida: a poça tem 12 m de fundo e a fixture usava um metro
/// dela.
#[test]
fn a_glide_ceiling_never_reaches_a_swimmer() {
    let rise = |glide: f32| {
        let mut sim = SimWorld::new();
        pool(&mut sim);
        // ⚠️ FUNDO, e não o `START` raso dos irmãos — ver acima.
        let subject = player(&mut sim, false, SWIM_SPEED, SWIM_ACCEL, -8.0);
        if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(subject) {
            p.glide_fall_speed = glide;
        }
        let mut bridge = PhysicsBridge::new();
        let y0 = pos_of(&sim).y;
        for tick in 1..=45_u64 {
            bridge.set_player_input(
                subject,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, tick);
        }
        let y = pos_of(&sim).y;
        assert!(
            y < -1.0,
            "a fixture tem de o manter SUBMERSO (a superficie esta' em 0): y = {y:.4}"
        );
        y - y0
    };
    let (off, on) = (rise(0.0), rise(2.0));
    assert!(
        (on - off).abs() < 0.02,
        "o planeio nao pode alcancar um nadador: ele subiu {off:.4} m sem o teto \
         armado e {on:.4} m com ele -- o mesmo dedo estaria a pedir duas coisas"
    );
}

/// **SONDA: a guarda `!swimming` do planeio é OBSERVÁVEL?**
///
/// ⚠️ Ela existe porque o teto já cala o planeio para quem sobe, então a guarda
/// só pode morder num regime muito estreito: **já submerso** e **a descer mais
/// depressa que o teto**, com o dedo no pulo.
///
/// ⚠️ **E medir isso pela ENTRADA não funciona, o que já está medido:** com o
/// planeio armado ele chega à água a **2,00 m/s em vez de 2,83** e por isso
/// **não mergulha** (`0,000` contra `−2,209` de profundidade). Essa diferença é
/// **legítima** — é o planeio a fazer o seu trabalho *acima* da água —, e usá-la
/// como oráculo da guarda seria atribuir ao guard um efeito que não é dele.
///
/// A fixture certa larga-o **dentro** da água já a descer depressa.
#[test]
#[ignore = "sonda de medicao"]
fn measure_whether_the_swim_guard_is_observable() {
    println!("\n== o planeio DENTRO da agua (submerso, a descer a 8 m/s) ==");
    println!("  teto de planeio   profundidade maxima");
    for glide in [0.0_f32, 2.0] {
        let mut sim = SimWorld::new();
        pool(&mut sim);
        let subject = player(&mut sim, false, SWIM_SPEED, SWIM_ACCEL, -1.0);
        sim.world_mut()
            .entity_mut(subject)
            .insert(ph2d_physics_ecs::InitialVelocity {
                linvel: [0.0, -8.0],
                angvel: 0.0,
            });
        if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(subject) {
            p.glide_fall_speed = glide;
        }
        let mut bridge = PhysicsBridge::new();
        let mut deepest = 0.0_f32;
        for tick in 1..=180_u64 {
            bridge.set_player_input(
                subject,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, tick);
            deepest = deepest.min(pos_of(&sim).y);
        }
        println!("  {glide:>15.2}   {deepest:>19.3}");
    }
    println!("\n(se as duas linhas forem iguais, a guarda e' INOBSERVAVEL e quem");
    println!(" a cobre e' o proprio teto -- documente, nao invente um gate)");
}
