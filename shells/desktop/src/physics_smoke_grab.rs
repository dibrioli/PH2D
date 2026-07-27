//! **A cena do CUTUCÃO** (`PH2D_PHYSICS_SMOKE=52`, W-Grab).
//!
//! Até esta wave o Play era só de LEITURA: a pose de um corpo dinâmico é escrita
//! pelo readback a cada frame, então um arrasto durante o play era sobrescrito no
//! mesmo frame e a cena não podia ser cutucada. Esta cena é sobre a mão.
//!
//! ⚠️ **Ela abre TOCANDO** (as outras de autoria abrem pausadas): a mão existe
//! precisamente enquanto o solver corre.
//!
//! Três estações, cada uma respondendo a uma pergunta diferente:
//!
//! - **A DUPLA**: uma bola leve e um caixote 25× mais denso, do mesmo tamanho.
//!   Arrastar os dois é IGUAL — a mão é uma ferramenta, não uma mola física
//!   (`MotorModel::AccelerationBased`; a mola do artista faria o pesado afundar).
//! - **A PAREDE**: puxar um caixote contra um muro **não o atravessa**. É o que
//!   separa a mão de um teleporte: ela entra pelo solver, então o contato vale.
//! - **A TORRE**: pegar a bola, girar e SOLTAR em movimento **arremessa** —
//!   soltar não zera a velocidade, e é metade da razão de existir do gesto.
//!
//! Os números da mensagem saíram da sonda `probe_smoke_52` (`body_grab`), rodada
//! sobre ESTAS peças antes de a mensagem ser escrita.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// As três estações **com o chão**, sem mensagem — a MESMA construção que a sonda
/// headless mede, para os números serem sobre a cena que o artista abre.
///
/// ⚠️ **Chão PRÓPRIO, largo, e a sonda o pegou:** o `physics_smoke::spawn_floor`
/// compartilhado mede `half_x = 4`, e as três estações desta cena vão de −7,2 a
/// +5,5 ⇒ o caixote da parede e a torre nasciam **fora do chão** e simplesmente
/// caíam. O sintoma na trajetória medida era outro (*"o caixote atravessa o
/// muro"*) porque um corpo em queda passa por baixo dele — e eu quase escrevi
/// isso no doc como *"a mão tunela"*, que a varredura do wrapper desmentiu (vão
/// de 0 a 2 m, com e sem CCD: ela **nunca** tunela). O padrão de chão largo é o
/// mesmo das cenas 29/25/33.
pub(crate) fn spawn_props(world: &mut World) {
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -0.25)),
        Sprite::atlas(WHITE_TILE_KEY, [20.0, 0.5], [0.40, 0.42, 0.48, 1.0]),
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
    ));
    const GREY: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
    const HOT: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
    const COOL: [f32; 4] = [0.4, 0.8, 0.95, 1.0];

    // ── A PAREDE (ponta esquerda): muro ESTÁTICO alto e um caixote à esquerda
    // dele. Puxar para a direita não o faz atravessar.
    //
    // ⚠️ O muro fica na PONTA de propósito: no meio da cena ele ficaria ENTRE a
    // bola e a torre, e o arremesso teria de contorná-lo. Cada estação tem de
    // poder ser exercitada sem atravessar a de outra.
    crate_(
        world,
        "Pusher",
        [-7.2, 1.0],
        [0.5, 0.5],
        1.0,
        BodyKind::Dynamic,
        COOL,
    );
    crate_(
        world,
        "Wall",
        [-6.0, 2.0],
        [0.25, 2.0],
        1.0,
        BodyKind::Static,
        GREY,
    );

    // ── A DUPLA (meio): MESMO tamanho, densidades 1 e 25. A mão as carrega
    // igual; uma mola física não carregaria.
    ball(world, "Light Ball", [-3.0, 1.0], 0.4, 1.0, COOL);
    crate_(
        world,
        "Heavy Crate",
        [-1.5, 1.0],
        [0.4, 0.4],
        25.0,
        BodyKind::Dynamic,
        HOT,
    );

    // ── A TORRE (direita): cinco caixotes leves empilhados, para o arremesso ter
    // CONSEQUÊNCIA (a lição da cena 30: um impacto contra chão imóvel não mostra
    // o efeito). Nenhum obstáculo entre ela e a bola.
    for i in 0..5u16 {
        crate_(
            world,
            &format!("Tower {}", i + 1),
            [5.5, 0.75 + f32::from(i) * 0.62],
            [0.3, 0.3],
            0.5,
            BodyKind::Dynamic,
            if i % 2 == 0 { HOT } else { COOL },
        );
    }
}

/// Uma bola dinâmica com densidade escolhida (o que a DUPLA precisa variar).
fn ball(world: &mut World, name: &str, at: [f32; 2], r: f32, density: f32, rgba: [f32; 4]) {
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Sprite::atlas(WHITE_TILE_KEY, [r * 2.0, r * 2.0], rgba),
        Name::new(name.to_string()),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: r },
            density,
            ..Collider::default()
        },
    ));
}

/// Um caixote — o tipo é parâmetro porque o muro é ESTÁTICO e o resto não.
fn crate_(
    world: &mut World,
    name: &str,
    at: [f32; 2],
    half: [f32; 2],
    density: f32,
    kind: BodyKind,
    rgba: [f32; 4],
) {
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Sprite::atlas(WHITE_TILE_KEY, [half[0] * 2.0, half[1] * 2.0], rgba),
        Name::new(name.to_string()),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: half[0],
                half_y: half[1],
            },
            density,
            ..Collider::default()
        },
    ));
}

impl crate::App {
    /// **Cena 52 (W-Grab).** Três estações, TOCANDO.
    pub(crate) fn physics_smoke_grab(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_props(gfx.sim.world_mut());

        eprintln!(
            "[physics-smoke 52] A cena esta TOCANDO. Arraste um corpo com o mouse.\n  \
               1. Aperte B (mostra os colliders; a MAO aparece como um zigzag VERDE-LIMAO\n     \
                  do cursor ate o ponto que voce pegou -- ela E uma mola).\n  \
               2. A DUPLA (meio): a bola azul e o caixote laranja tem o MESMO tamanho e o\n     \
                  caixote e 25x mais denso. Arraste um, depois o outro: seguem IGUAL.\n     \
                  (medido: 3,00 m e 2,99 m sob o mesmo gesto -- razao 1,004)\n  \
               3. A PAREDE (esquerda): arraste 'Pusher' (o caixote azul da ponta) para a\n     \
                  DIREITA, tentando atravessar o muro cinza. Ele PARA encostado nele --\n     \
                  a mao cede contra o contato, nao teleporta.\n     \
                  (medido: o cursor foi para x=-3,0 e o caixote parou em x=-6,75,\n      \
                   que e exatamente encostado; penetracao zero)\n  \
               4. A TORRE (direita): pegue a bola azul da DUPLA, ganhe velocidade para a\n     \
                  direita e SOLTE em movimento. Ela voa e DERRUBA a pilha -- soltar nao\n     \
                  zera a velocidade.\n     \
                  (medido: soltando a 8 m/s a bola viaja 2,62 m depois do release)\n  \
               5. O muro cinza e ESTATICO: a MAO nao o pega (uma mola nao move massa\n     \
                  infinita) -- o clique cai no caminho de sempre, que SELECIONA e\n     \
                  ARRASTA. E desde a W-Hand o collider vai junto: arraste o muro e o\n     \
                  caixote passa a bater onde ele esta AGORA (era o bug do collider\n     \
                  fantasma; a cena 53 o demonstra com uma testemunha em cima).\n  \
               6. Desmarque 'Physics' no transporte e tente arrastar: a mao NAO aparece\n     \
                  (sem passo de solver ela nao puxaria nada). Marque de novo e volta.\n  \
               7. Pause: em repouso o arrasto volta a ser AUTORIA de pose (e com ALT\n     \
                  carrega o rig -- W-JG). O relogio e o interruptor.\n  \
               8. Arraste a regua para TRAS: o cutucao NAO volta -- ele nao esta no\n     \
                  documento, e a cena re-simula da pose autorada."
        );
    }
}
