//! ⭐⭐⭐ **Smoke do EMISSOR DE PARTÍCULAS** (TOP-20 #18, W4). `PH2D_PARTICLES_SMOKE=1|2`.
//!
//! # `=1` — quatro fontes, UM knob de diferença cada
//!
//! As quatro têm a **mesma** quantidade, a mesma vida e a mesma rapidez: é isso que torna cada
//! coluna legível (o molde da galeria de tiro do #14). Da esquerda para a direita:
//!
//! | fonte | o que só ela tem | o que se vê |
//! |---|---|---|
//! | **Jet** | nada — é o emissor de fábrica | um jacto que sobe e cai |
//! | **Ring** | `shape = Ring` · sem gravidade | um anel que se abre |
//! | **Fade** | `color_end` transparente · `size_end = 0` | um jacto que se apaga a meio |
//! | **Burst** | `one_shot` · `explosiveness = 1` · dispara ao SINAL | fica parado e **rebenta** de dois em dois segundos |
//!
//! ⭐⭐ **O `Burst` é o que liga o emissor ao resto da casa:** ele nasce **desligado**
//! (`emitting = false`) e o `restart_on = "boom"` é que o acende — o sinal vem de um `Timer` que
//! grita de dois em dois segundos (TOP-20 #2). *Um emissor de explosão autora-se desligado e existe
//! para ser disparado.*
//!
//! # `=2` — o RASTO e a TOCHA, lado a lado
//!
//! Dois projécteis iguais atravessam a cena, cada um com um emissor preso. A ÚNICA diferença é o
//! **Simulation Space**:
//!
//! - **Trail (World)** — as partículas ficam onde nasceram, e fica um rasto no ar;
//! - **Torch (Local)** — o penacho anda com o objecto, colado a ele.
//!
//! ⛔ **Sem os dois na MESMA cena o artista não distingue** *«o espaço funciona»* de *«é assim que
//! partículas são»* — é a lei do controlo ao lado, que o #13 e o #14 já pagaram.
//!
//! # ⛔ Cada fonte é um CORPO visível — a lição do #15
//!
//! O emissor mora num objecto que **se desenha**, porque o pick só devolve quem emite uma sprite:
//! uma fonte sem corpo seria impossível de escolher, e a secção do Inspector nunca apareceria.
//! ⚠️ E as placas **encostam sem se sobrepor** — entre arquétipos diferentes a ordem de iteração do
//! pick é indefinida por escrito (o report do dono sobre a máquina de estados, 2026-09-15).

use ph2d_core::Vec2;
use ph2d_ecs::{
    EmissionShape, Name, ParticleEmitter, ParticleSpace, Timer, Timers, Transform, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, ProjectileMotion, RigidBody};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 2;

const CHAO_RGBA: [f32; 4] = [0.14, 0.15, 0.19, 1.0];
const CORPO_RGBA: [f32; 4] = [0.55, 0.57, 0.62, 1.0];
const RELOGIO_RGBA: [f32; 4] = [0.35, 0.37, 0.42, 1.0];
const PAREDE_RGBA: [f32; 4] = [0.30, 0.32, 0.38, 1.0];

/// A quantidade e a vida que TODAS as quatro fontes da `=1` partilham — ver o cabeçalho.
const QUANTIDADE: u32 = 40;
const VIDA: f32 = 0.8;

/// O sinal que acende a rajada — o mesmo nome no `Timer` e no emissor.
const BOOM: &str = "boom";

/// ⚠️⚠️ **O ENQUADRAMENTO saiu de uma FOTO, não de um palpite** — e foram precisas três. A régua do
/// canvas com os dois painéis laterais e a timeline abertos mede **≈ 1 100 px de largura por ≈ 630
/// de altura**, e a `0,01 m/px` isso são `~11 × 6 m`. À primeira a fila estava a `±7,5 m` (duas
/// colunas fora do ecrã) e as fontes a `−5 m` (cortadas pela borda de baixo). *Uma cena que não
/// cabe no ecrã ensina o contrário do que diz.*
const CHAO_Y: f32 = -0.8;

/// O passo entre colunas, em metros. ⚠️ **Ele é o que separa DUAS lições**: a fila inteira tem de
/// caber na banda visível (`1,5 × VAO + alcance ≤ 5,2 m`) e **nenhuma coluna pode invadir a
/// vizinha** — a segunda saiu de uma mutação sobrevivente, e é a que torna a galeria legível.
const VAO: f32 = 2.4;

fn chao(world: &mut World) {
    // ⚠️ **O chão é a PRIMEIRA raiz criada**, e desde 2026-09-15 isso quer dizer que ele desenha
    // ATRÁS de tudo.
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [10.0, 5.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.8)),
    ));
}

/// Uma fonte: um corpo pequeno e visível com o emissor.
fn fonte(world: &mut World, nome: &str, x: f32, cfg: ParticleEmitter) -> u64 {
    world
        .spawn((
            Name::new(nome),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], CORPO_RGBA),
            cfg,
            Transform::from_translation(Vec2::new(x, CHAO_Y)),
        ))
        .id()
        .to_bits()
}

/// A cena `=1` — a galeria das quatro fontes. Devolve quem nasce ESCOLHIDO.
///
/// ⭐⭐ **A cena abre com uma fonte escolhida**, como a do script: com ela escolhida a secção
/// *Particles* está no painel **antes do primeiro clique** — e um painel vazio com a instrução a
/// dizer *«veja a secção»* é a mesma armadilha que a foto do #16 apanhou.
fn cena_um(world: &mut World) -> u64 {
    chao(world);
    let base = ParticleEmitter {
        amount: QUANTIDADE,
        life: VIDA,
        speed: 3.2,
        spread: 18.0,
        size: 0.18,
        // ⚠️ **Peso MAIS LEVE que o do mundo, e a foto é que o pediu:** a `−9,8` o arco tinha
        // `0,52 m` de alto e o jacto lia-se como um borrão colado à fonte. A `−3,0` ele sobe
        // `1,7 m` — meia banda visível — e as três colunas com peso partilham este número.
        gravity: [0.0, -3.0],
        ..ParticleEmitter::default()
    };
    let escolhido = fonte(world, "Jet", -1.5 * VAO, base.clone());
    let _ = fonte(
        world,
        "Ring",
        -0.5 * VAO,
        ParticleEmitter {
            shape: EmissionShape::Ring,
            shape_size: [0.6, 0.6],
            // ⚠️ Sem gravidade e a sair do CENTRO para fora: é isso que faz o anel abrir-se.
            gravity: [0.0, 0.0],
            spread: 180.0,
            // ⚠️⚠️ **A ÚNICA coluna com rapidez própria, e a foto é que a impôs:** sem gravidade
            // uma partícula viaja `v × vida` e não volta — à rapidez das outras (`5 m/s`) o anel
            // media `6 m` de raio e saía do ecrã por cima. O que as quatro partilham, e que é o
            // que as torna comparáveis, é a QUANTIDADE e a VIDA.
            // ⚠️ **E é LENTA de propósito**: o que esta coluna mostra é o sítio onde as partículas
            // NASCEM, e a qualquer rapidez o anel borra-se num disco em meia vida.
            speed: 0.25,
            color: [0.45, 0.80, 1.0, 1.0],
            ..base.clone()
        },
    );
    let _ = fonte(
        world,
        "Fade",
        0.5 * VAO,
        ParticleEmitter {
            color: [1.0, 0.75, 0.25, 1.0],
            color_end: [1.0, 0.25, 0.10, 0.0],
            size_end: 0.0,
            ..base.clone()
        },
    );
    let _ = fonte(
        world,
        "Burst",
        1.5 * VAO,
        ParticleEmitter {
            // ⭐⭐ **Nasce DESLIGADO**: quem o acende é o sinal, e é isso que a cena ensina.
            emitting: false,
            one_shot: true,
            explosiveness: 1.0,
            // ⚠️ Um leque para CIMA e não uma esfera: a `180°` a rajada voa `v × vida` para os
            // lados e sai do ecrã — a mesma régua que pôs o anel mais lento. ⚠️ E `30°` e não
            // `45°`: a `45°` ela alcançava `1,81 m` de lado e entrava na coluna do vizinho.
            spread: 30.0,
            color: [1.0, 0.95, 0.65, 1.0],
            color_end: [1.0, 0.35, 0.10, 0.0],
            restart_on: BOOM.to_string(),
            ..base
        },
    );
    // ⭐ **Quem grita é um relógio** — o emissor não sabe quem o dispara, e o artista pode trocar o
    // nome do sinal no painel das duas pontas.
    world.spawn((
        Name::new("Boom Clock"),
        // ⚠️ **Com CORPO**: a mensagem da cena diz *«o relógio ao lado»*, e um relógio invisível
        // faz dessa frase uma mentira — mais a lei do #15, que o pick só devolve quem tem sprite.
        Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], RELOGIO_RGBA),
        Timers(vec![Timer {
            name: "Boom".to_string(),
            duration_us: 2_000_000,
            repeat: true,
            autostart: true,
            signal: BOOM.to_string(),
        }]),
        Transform::from_translation(Vec2::new(1.5 * VAO, CHAO_Y - 0.9)),
    ));
    escolhido
}

/// Uma parede estática — o que faz os dois voos VOLTAREM. Ver [`cena_dois`].
fn parede(world: &mut World, nome: &str, x: f32) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.25,
                half_y: 2.4,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.5, 4.8], PAREDE_RGBA),
        Transform::from_translation(Vec2::new(x, 0.0)),
    ));
}

/// A cena `=2` — o rasto e a tocha, o mesmo voo com espaços diferentes.
///
/// ⚠️⚠️ **Os dois RICOCHETEIAM entre duas paredes, e foi uma FOTO que o impôs:** a primeira versão
/// mandava-os a direito e, passados uns segundos, a cena era um chão vazio — *um smoke que só
/// ensina nos três primeiros segundos ensina, no resto do tempo, que não há nada ali*.
fn cena_dois(world: &mut World) -> u64 {
    chao(world);
    parede(world, "Wall L", -4.6);
    parede(world, "Wall R", 4.6);
    let base = ParticleEmitter {
        amount: 60,
        life: 1.0,
        // ⚠️⚠️ **Quase PARADAS, e é isso que torna a lição legível** — medido numa foto: a `1,2 m/s`
        // cada partícula abre `1,2 m` por conta própria e as duas colunas viram duas nuvens
        // parecidas. Com elas quietas, o que se vê é só o que o ESPAÇO faz: uma fica no ar (o
        // rasto) e a outra anda com o objecto (a tocha).
        speed: 0.35,
        spread: 180.0,
        size: 0.16,
        gravity: [0.0, 0.0],
        color: [1.0, 0.80, 0.35, 1.0],
        color_end: [1.0, 0.30, 0.10, 0.0],
        ..ParticleEmitter::default()
    };
    let voo = ProjectileLaw {
        initial_speed: 2.4,
        range: 200.0,
        // ⚠️ **Vinte voltas e uma devolução INTEIRA** — é o que mantém a cena viva enquanto o dono
        // olha para ela. O tecto existe porque um ricochete perfeito para sempre é um objecto que
        // nunca acaba, e a wave do projéctil declarou-o.
        max_bounces: 20,
        bounciness: 1.0,
        ..ProjectileLaw::default()
    };
    let mut escolhido = 0;
    for (nome, y, space) in [
        ("Trail (World)", 1.4_f32, ParticleSpace::World),
        ("Torch (Local)", -1.4, ParticleSpace::Local),
    ] {
        let e = world.spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.3], CORPO_RGBA),
            ProjectileMotion::from_law(voo, 0),
            ParticleEmitter {
                space,
                ..base.clone()
            },
            // ⚠️ **Dentro do ecrã desde o primeiro quadro** — a `−9 m` os dois nasciam fora da
            // banda visível e o dono abria a cena num canvas vazio.
            Transform::from_translation(Vec2::new(-4.2, y)),
        ));
        if escolhido == 0 {
            escolhido = e.id().to_bits();
        }
    }
    escolhido
}

/// O que a cena montada deixa para a shell fazer.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// A fonte que nasce ESCOLHIDA — ver [`cena_um`].
    pub escolhido: u64,
}

/// **Monta a cena `nivel`** e devolve qual foi — o roteador.
pub fn montar(world: &mut World, nivel: u32) -> Montada {
    match nivel {
        2 => {
            let escolhido = cena_dois(world);
            println!(
                "[particles-smoke] =2 os dois voam iguais: o de cima deixa RASTO (as particulas \
                 ficam onde nasceram) e o de baixo leva o penacho COLADO. Eles vao e voltam entre \
                 as duas paredes. A unica diferenca e' o «Simulation Space» — escolha um e veja no \
                 painel"
            );
            Montada {
                nivel: 2,
                escolhido,
            }
        }
        _ => {
            let escolhido = cena_um(world);
            println!(
                "[particles-smoke] =1 quatro fontes, um knob de diferenca cada: Jet (de fabrica) · \
                 Ring (nasce num anel) · Fade (apaga-se a meio) · Burst (fica parado e REBENTA de \
                 2 em 2 s, ao sinal «{BOOM}» do relogio ao lado). Clique numa e veja a seccao \
                 Particles no painel da direita"
            );
            Montada {
                nivel: 1,
                escolhido,
            }
        }
    }
}

#[cfg(test)]
#[path = "particles_smoke_tests.rs"]
mod tests;
