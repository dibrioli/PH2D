//! **As cenas do PLAYER DE PLATAFORMA** — `=80` (a mola, W2) e `=81` (andar, W3).
//!
//! ⚠️ **A cena do CAST (`=80` no plano) foi CORTADA, e o corte é a decisão.**
//! O plano prometia *"três raios contra chão plano / rampa / vão, imprimindo
//! distância e normal"* — e isso é um TESTE, não um smoke: o `world::cast` já
//! tem sete gates, com os dois lados do BVH medidos, e uma cena que imprime
//! números pede ao artista para conferir exatamente o que uma máquina confere
//! melhor. Uma cena de smoke existe para julgar o que **só o olho julga**.
//! As duas cenas daqui sobem um número cada, e o plano foi corrigido junto.
//!
//! ⚠️ **A `float_height` destas cenas é 0,9 e não o `0,5` do ponto de partida**,
//! e o motivo é geometria medida, não gosto: flutuar de verdade exige
//! `float_height > half_height + radius / cos(max_slope)`
//! ([`ph2d_platformer::RideConfig::min_float_height`], com a tabela). Para a
//! cápsula daqui (`0,3` / `0,2`) o mínimo já é `0,5` **no plano** — ou seja, o
//! ponto de partida a deixa TANGENTE, e ela deixa de pairar na primeira rampa.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::stable_name_id;
use ph2d_ecs::{Entity, Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, LockRotation, PhysicsJoint, PlatformPlayer,
    RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::{PropKind, TimelineDoc};

#[cfg(test)]
#[path = "physics_smoke_reaction_tests.rs"]
mod reaction_tests;

/// A altura de flutuação destas cenas — ver o aviso do módulo.
const FLOAT: f32 = 0.9;

/// O personagem: cápsula dinâmica, rotação travada (D4), com a config do plano.
pub(crate) fn spawn_player(world: &mut bevy_ecs::world::World, at: Vec2) -> Entity {
    world
        .spawn((
            Name::new("Player"),
            Transform::from_translation(at),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.25, 0.85, 1.0, 1.0]),
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
        ))
        .id()
}

/// Um bloco estático, opcionalmente inclinado.
pub(crate) fn slab(
    world: &mut bevy_ecs::world::World,
    name: &str,
    at: Vec2,
    half: [f32; 2],
    rot: f32,
    tint: [f32; 4],
) -> Entity {
    world
        .spawn((
            Name::new(name.to_string()),
            Transform {
                rotation: rot,
                ..Transform::from_translation(at)
            },
            Sprite::atlas(WHITE_TILE_KEY, [half[0] * 2.0, half[1] * 2.0], tint),
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
        ))
        .id()
}

/// A plataforma vai e volta.
///
/// ⚠️ **Dentro de 4 s de propósito** — é a duração com que toda composição
/// nasce, e uma track mais longa ficaria congelada no corte sem que nada na tela
/// dissesse por quê. Para o vagão repetir, arme o Loop na barra de transporte.
pub(crate) fn author_platform_track(doc: &mut TimelineDoc, platform: Entity) {
    for (t, x) in [(0.0, 6.0_f32), (2.0, 11.0), (4.0, 6.0)] {
        doc.insert_key(
            platform.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(x),
            Interp::Linear,
        );
    }
}

impl crate::App {
    /// **Cena 80 (W2).** O personagem PAIRA.
    ///
    /// A pergunta é de olho e é uma só: *ele fica no ar, na mesma altura, sem
    /// tremer?* Depois, empurre-o com a **MÃO** (a ferramenta de interação) e
    /// solte: ele tem de voltar à altura, sem pipocar.
    pub(crate) fn physics_smoke_float(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [14.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        // Um degrau: subir nele é o caso que a `cling_distance` governa.
        slab(
            world,
            "Step",
            Vec2::new(4.0, 0.15),
            [1.2, 0.65],
            0.0,
            [0.45, 0.4, 0.35, 1.0],
        );
        spawn_player(world, Vec2::new(-3.0, 2.5));

        eprintln!(
            "[physics-smoke 80] O PLAYER PAIRA (W2). Um chao, um degrau e um personagem.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             Julgue, de olho:\n\
             · ele CAI e para NO AR, {FLOAT:.2} m acima do chao -- nao encostado.\n\
             · em repouso ele NAO treme nem sobe e desce (a mola nao pode oscilar).\n\
             · pegue-o com a MAO (a ferramenta de interacao) e puxe para cima: ao\n\
               soltar ele volta a MESMA altura, sem pipocar.\n\
             · empurre-o para o degrau: a perna o levanta sem que ele bata nele."
        );
    }

    /// **Cena 82 (W5).** A AUTORIA — nenhum player na cena, e você faz um.
    ///
    /// A irmã da cena 3 do W2a, um degrau acima: lá o gesto que faltava era
    /// *"tornar um sprite físico"*, aqui é *"tornar um corpo um personagem"*. Um
    /// componente sem este gesto roda em toda cena de smoke — que constrói com
    /// código — e é **inalcançável no produto**.
    pub(crate) fn physics_smoke_author_player(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [14.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            "Ramp",
            Vec2::new(-8.0, 1.0),
            [3.5, 0.5],
            35.0_f32.to_radians(),
            [0.3, 0.5, 0.35, 1.0],
        );
        // Um corpo Dynamic com CÁPSULA e sem comportamento — o estado exato em
        // que a face vazia da §14 é a única coisa que a seção oferece.
        world.spawn((
            Name::new("Hero"),
            Transform::from_translation(Vec2::new(0.0, 2.0)),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.25, 0.85, 1.0, 1.0]),
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
        ));
        // E um sprite PELADO, para a outra metade do gesto: Add Physics Body
        // (§11) e só depois Make Platform Player (§14).
        world.spawn((
            Name::new("Prop"),
            Transform::from_translation(Vec2::new(3.0, 2.0)),
            Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.9], [0.9, 0.7, 0.3, 1.0]),
        ));

        eprintln!(
            "[physics-smoke 82] A AUTORIA (W5). Um chao, uma rampa de 35deg, um corpo\n\
             Dynamic SEM comportamento (Hero) e um sprite pelado (Prop).\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             Julgue, de olho:\n\
             · selecione Hero: a secao 'Platform Player' aparece com UM botao.\n\
             · clique 'Make Platform Player': o personagem passa a PAIRAR, e as\n\
               setas <- / -> passam a anda-lo. Ele nasce acima do chao, nao encostado.\n\
             · re-selecione: as caixas mostram o que foi autorado, nao o seed.\n\
             · baixe 'Float Height' para 0.2: o botao passa a dizer o MINIMO que a\n\
               forma exige. Clique nele e o personagem volta a pairar.\n\
             · suba 'Max Slope' acima de 35: a rampa passa a ser escalavel.\n\
             · selecione Prop: a secao NAO aparece (ele nao tem corpo). Use a §11\n\
               'Add Physics Body' primeiro; ai' a §14 nasce.\n\
             · 'Remove Platform Player' devolve o corpo -- e a secao continua la',\n\
               com o botao, para voce refaze-lo."
        );
    }

    /// **Cena 83 (W4).** PULAR — a cena que mede a própria promessa.
    ///
    /// ⚠️ **As duas saliências são a régua, e as alturas saíram da sonda**
    /// (`measure_the_jump`, em `ph2d-physics-ecs`): com o `Jump Height` de
    /// fábrica (2,0) o personagem alcança um topo em **2,10 m**, então a de
    /// **1,6** é confortável e a de **2,8** é inalcançável — até o artista subir o
    /// knob para 3,0, que compra **3,12**. É isso que torna o número da §14 uma
    /// coisa que se vê, em vez de um campo que se acredita.
    pub(crate) fn physics_smoke_jump(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // O chão, com um VÃO no meio (x de 3 a 6) — atravessá-lo é a pergunta do
        // controle aéreo, e cair nele é a resposta honesta de quem soltou cedo.
        slab(
            world,
            "FloorLeft",
            Vec2::new(-4.0, -0.5),
            [7.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            "FloorRight",
            Vec2::new(11.0, -0.5),
            [5.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        // A saliência BAIXA (topo 1,6): alcançável de fábrica.
        slab(
            world,
            "LedgeLow",
            Vec2::new(-8.0, 1.35),
            [1.6, 0.25],
            0.0,
            [0.3, 0.5, 0.35, 1.0],
        );
        // A saliência ALTA (topo 2,8): fora de alcance ate' o knob subir.
        slab(
            world,
            "LedgeHigh",
            Vec2::new(-8.0, 2.55),
            [1.6, 0.25],
            0.0,
            [0.55, 0.32, 0.3, 1.0],
        );

        // O vagão outra vez — pular DELE tem de levar a velocidade dele junto.
        let platform = world
            .spawn((
                Name::new("Wagon"),
                Transform::from_translation(Vec2::new(6.0, 1.6)),
                Sprite::atlas(WHITE_TILE_KEY, [4.0, 0.5], [0.6, 0.55, 0.25, 1.0]),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 2.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
            ))
            .id();

        spawn_player(world, Vec2::new(-2.0, 2.0));
        author_platform_track(&mut self.timeline.doc, platform);

        eprintln!(
            "[physics-smoke 83] PULAR (W4). Chao com um VAO, duas saliencias (topo 1.6 e\n\
             2.8), e o vagao.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n\
             O 'Jump Height' autorado e' 2.0 m.\n\
             \n\
             Julgue, de olho:\n\
             · pule parado: ele sobe cerca de 2 m e volta. A DESCIDA e' visivelmente\n\
               mais rapida que a subida -- e' o que faz um pulo parecer um pulo.\n\
             · TOQUE a tecla e solte na hora: o pulo sai BAIXO. Segure ate' o topo:\n\
               sai CHEIO. E' o mesmo botao, e a altura obedece ao dedo.\n\
             · SEGURE a tecla apertada e espere: ele pula UMA vez, cai, e NAO pula\n\
               de novo sozinho. Ele so' pula quando a tecla e' apertada de novo.\n\
             · no ar, aperte de novo: nada acontece. Nao ha' pulo duplo.\n\
             · va' para a ESQUERDA ate' as saliencias: a de BAIXO (verde) voce\n\
               alcanca; a de CIMA (vermelha) nao. Suba 'Jump Height' para 3.0 na\n\
               secao Platform Player e a vermelha passa a ser alcancavel.\n\
             · corra para a DIREITA e pule o VAO: com impulso voce atravessa.\n\
             · suba no VAGAO e pule enquanto ele anda: voce sobe JUNTO com ele, e\n\
               cai de volta em cima -- o pulo e' relativo ao chao que se move.\n\
               (O vagao faz uma ida-e-volta nos primeiros 4 s; arme o Loop.)"
        );
    }

    /// **Cena 85 (W6).** A REAÇÃO — o personagem deixa de ser um fantasma.
    ///
    /// ⚠️ **A cena tem um CONTROLE, e ele é metade da entrega:** duas jangadas
    /// idênticas, e só a da esquerda tem `Weight on Ground`. Sem o par, *"ela
    /// afundou"* é uma frase sobre um número que o olho não tem com o que
    /// comparar — e foi exatamente assim que o fantasma sobreviveu a três waves
    /// de smoke.
    pub(crate) fn physics_smoke_reaction(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_reaction_scene(gfx.sim.world_mut());
        eprintln!("{REACTION_SMOKE_MESSAGE}");
    }
}

/// A cena 85, sem a `App` — é o que deixa a sonda a medir **a cena que shipa**
/// em vez de uma cópia dela.
pub(crate) fn build_reaction_scene(world: &mut bevy_ecs::world::World) {
    slab(
        world,
        "Floor",
        Vec2::new(0.0, -3.0),
        [24.0, 0.5],
        0.0,
        [0.35, 0.35, 0.4, 1.0],
    );

    // Duas jangadas iguais, penduradas por molas iguais: só o escalar difere.
    for (i, (name, support, at_x, tint)) in [
        ("RaftLive", 1.0_f32, -8.0_f32, [0.3, 0.5, 0.35, 1.0]),
        ("RaftGhost", 0.0, 4.0, [0.5, 0.35, 0.35, 1.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let raft = world
            .spawn((
                Name::new(name.to_string()),
                Transform::from_translation(Vec2::new(at_x, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [6.0, 0.5], tint),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 3.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
            ))
            .id();
        let _ = raft;
        // ⚠️ **Duas molas por jangada, e não uma** — uma no centro seguraria
        // a altura e deixaria a jangada girar livre em torno dela, e a
        // metade INCLINAR da cena viraria um pião. Duas, nas pontas, dão
        // altura E nivelamento: afundar de um lado é o que o torque faz.
        for (side, dx) in [("L", -2.4_f32), ("R", 2.4)] {
            let hook = format!("Hook{i}{side}");
            world.spawn((
                Name::new(hook.clone()),
                Transform::from_translation(Vec2::new(at_x + dx, 4.0)),
                Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], [0.5, 0.5, 0.55, 1.0]),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.15,
                        half_y: 0.15,
                    },
                    ..Collider::default()
                },
            ));
            world.spawn((
                Name::new(format!("Rope{i}{side}")),
                Transform::from_translation(Vec2::new(at_x + dx, 4.0)),
                PhysicsJoint {
                    body_a: stable_name_id(&hook),
                    body_b: stable_name_id(name),
                    kind: JointKind::Spring,
                    rest_length: 4.0,
                    // ⚠️ **A rigidez saiu da CONTA, não do gosto:** o
                    // afundamento é `m·g / (2k)`, e com o personagem de
                    // 20 kg desta cena `k = 165` dá ~0,6 m — grande o
                    // bastante para o olho, pequeno o bastante para a
                    // jangada não bater no chão.
                    stiffness: 165.0,
                    damping: 15.0,
                    // ⚠️ **As âncoras são AUTORADAS**, e sem isso a cena
                    // não funciona: numa mola o lado B ancora no CENTRO do
                    // corpo (a política do W3), então duas molas "nas
                    // pontas" seriam duas molas no MESMO ponto — sem
                    // nenhuma resistência a torque, que é metade do que
                    // esta cena existe para mostrar.
                    anchored: true,
                    local_a: [0.0, 0.0],
                    local_b: [dx, 0.0],
                    ..PhysicsJoint::default()
                },
            ));
        }

        world.spawn((
            Name::new(format!("Hero{i}")),
            // ⚠️ No CENTRO: assim a jangada afunda NIVELADA, e inclinar é o que o
            // artista faz andando — se ele já nascesse na borda, a cena
            // chegaria torta e o roteiro perderia a metade do torque.
            Transform::from_translation(Vec2::new(at_x, 1.2)),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.25, 0.85, 1.0, 1.0]),
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
            // ⚠️ 20 kg, e o número é da CENA e não do personagem: o
            // afundamento é proporcional à massa, e com os ~0,37 kg que a
            // cápsula pesa por densidade ele seria de 3 cm — verdadeiro e
            // invisível, que é o pior resultado possível para um smoke.
            ph2d_physics_ecs::MassOverride(20.0),
            PlatformPlayer {
                float_height: FLOAT,
                reaction_support: support,
                ..PlatformPlayer::default()
            },
        ));
    }
}

/// A mensagem da cena 85 — uma const para a sonda poder afirmar sobre ela.
pub(crate) const REACTION_SMOKE_MESSAGE: &str = "[physics-smoke 85] A REACAO (W6). Duas jangadas iguais penduradas em molas,\n\
             cada uma com um personagem. So' a da ESQUERDA tem 'Weight on Ground'.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             CONTROLE: setas <- / -> andam OS DOIS ao mesmo tempo (a entrada e' do\n\
             teclado e chega a todo player). E' de proposito: o par anda junto e a\n\
             comparacao fica lado a lado.\n\
             \n\
             Julgue, de olho:\n\
             · a jangada da ESQUERDA AFUNDA sob o personagem; a da direita nao se\n\
               mexe. O da direita e' um fantasma que a balanca nao pesa.\n\
             · ande ate' a BORDA da jangada esquerda: ela INCLINA para o lado dele.\n\
               No meio ela fica nivelada -- o braco e' zero, logo o torque tambem.\n\
             · PULE na jangada esquerda: ela e' empurrada para BAIXO no instante da\n\
               decolagem, e volta enquanto voce esta' no ar.\n\
             · selecione o Hero0 e baixe 'Weight on Ground' para 0: ele vira o\n\
               fantasma da direita, ao vivo.\n\
             · suba 'Push on Ground' para 1 e ande: a jangada escorrega para TRAS\n\
               como um tapete. E' atrito honesto, e e' por isso que nasce em zero.";

impl crate::App {
    /// **Cena 81 (W3).** ANDAR — a cena que se dirige.
    ///
    /// ⚠️ É a primeira cena desta jornada com **controle**: as setas ←/→ (ou
    /// A/D) andam. Sem isso o W3 seria invisível — a caminhada é uma resposta a
    /// um dedo, e nada num log a mostra.
    pub(crate) fn physics_smoke_walk(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Um chão longo entre duas paredes, e uma rampa RASA (30°) que se sobe.
        //
        // ⚠️ **A GEOMETRIA foi CORRIGIDA na W9, e o defeito era mudo** (medido em
        // `measure_walk_scene`, na `ph2d-physics-ecs`): as duas rampas antigas
        // subiam *para longe* do chão, então o personagem passava POR BAIXO
        // delas, caía da beirada do piso em `x = ±10` e despencava — `y = −162 m`
        // seis segundos depois, sem ter encostado em rampa nenhuma. O roteiro
        // mandava *"vá até a rampa"* e não havia como chegar lá.
        //
        // O que decide é o **SINAL da rotação**: negativo faz a rampa subir indo
        // para a ESQUERDA, que é o lado de onde o personagem chega. E a rampa
        // ÍNGREME saiu daqui para a `=88`, a cena que existe para o par que
        // cerca o limite (40° × 50°) — 60° já era recusado mesmo com o defeito
        // do *Max Slope*, então ela nunca foi a fixture que continha o fenômeno.
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [16.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        for (name, x) in [("WallL", -16.5), ("WallR", 16.5)] {
            slab(
                world,
                name,
                Vec2::new(x, 2.0),
                [0.5, 2.5],
                0.0,
                [0.30, 0.30, 0.34, 1.0],
            );
        }
        slab(
            world,
            "Ramp30",
            Vec2::new(-7.0, 1.3),
            [4.0, 0.5],
            -30.0_f32.to_radians(),
            [0.3, 0.5, 0.35, 1.0],
        );
        // O patamar no alto — subir tem de levar a algum lugar.
        slab(
            world,
            "Plateau",
            Vec2::new(-13.0, 3.3),
            [3.5, 0.41],
            0.0,
            [0.32, 0.44, 0.36, 1.0],
        );

        // A plataforma KINEMATIC, dirigida pela timeline — o vagão.
        let platform = world
            .spawn((
                Name::new("Wagon"),
                Transform::from_translation(Vec2::new(6.0, 1.6)),
                Sprite::atlas(WHITE_TILE_KEY, [4.0, 0.5], [0.6, 0.55, 0.25, 1.0]),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 2.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
            ))
            .id();

        spawn_player(world, Vec2::new(0.0, 2.0));
        author_platform_track(&mut self.timeline.doc, platform);

        eprintln!(
            "[physics-smoke 81] ANDAR (W3). Chao entre duas paredes + rampa de 30deg\n\
             com patamar no alto + um vagao.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             CONTROLE: as setas <- / -> (ou A / D) andam. O limite de rampa autorado e' 45deg.\n\
             \n\
             Julgue, de olho:\n\
             · ande: ele acelera ate' uma velocidade CONSTANTE e nao passa dela.\n\
             · SOLTE a tecla: ele freia e PARA -- e depois nao escorrega mais nada.\n\
             · segure a tecla oposta em movimento: virar responde mais rapido que\n\
               arrancar do zero (e' o fator de mudanca de direcao, ate' 2x).\n\
             · va' para a ESQUERDA, ate' a rampa de 30deg: ele SOBE ate' o patamar,\n\
               e sobe com a mesma velocidade com que andava no plano (a velocidade\n\
               e' a do PERCURSO, nao a horizontal).\n\
             · a rampa INGREME mudou de cena: ela agora e' a =88, com o par que\n\
               cerca o limite (40deg sobe / 50deg escorrega).\n\
             · suba no VAGAO e SOLTE a tecla: ele viaja junto, parado em relacao\n\
               ao vagao. Ande em cima dele: anda normal, sobre um chao que se move.\n\
               (O vagao faz uma ida-e-volta nos primeiros 4 s; arme o Loop na barra\n\
                de transporte para ele repetir.)"
        );
    }
}
