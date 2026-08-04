//! **As cenas 89 e 90 — A CHAMINÉ e O VAGÃO** (W10), irmãs de
//! `physics_smoke_player.rs`.
//!
//! Cortadas por CENA, o precedente do módulo, e **duas e não uma** porque as duas
//! assistências desta wave respondem a perguntas diferentes com gestos
//! diferentes: uma é sobre *onde a cabeça passa*, a outra sobre *o que se leva ao
//! sair de uma plataforma*. Uma cena só teria de as pôr no mesmo espaço, e o
//! vagão atravessa a cena inteira.
//!
//! # ⚠️ A geometria da chaminé foi MEDIDA, não desenhada de cabeça
//!
//! `measure_corner::measure_the_chimney_window` varre o vão e o desvio e imprime
//! o pico do pulo (o pulo livre é 0,833 m; qualquer coisa abaixo é a cabeça a
//! bater). Com o vão de **0,60 m** e o corpo de 0,40:
//!
//! | desvio do centro | `Corner Reach = 0` | `= 0,12` |
//! |---|---|---|
//! | 0,09 m | livre | livre |
//! | 0,12 m | **bate** | livre |
//! | 0,18 m | bate | livre |
//! | 0,21 m | bate | **livre** |
//! | 0,24 m | bate | bate |
//!
//! A janela em que o pulo passa sai de **±0,10 m** para **±0,22 m** — ela mais
//! que DOBRA, e é isso que a cena existe para o olho julgar. Um vão de 0,70
//! deixaria a folga geométrica grande demais para a diferença ser visível; um de
//! 0,50 deixaria a assistência a salvar quase nada.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, CombineRule, GravityScale, InitialVelocity, LockRotation,
    MassOverride, MaterialCombine, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O vão da chaminé — o número da tabela do topo do módulo.
const GAP: f32 = 0.6;

impl App {
    /// **A chaminé** — um vão estreito, e a janela em que o pulo passa por ele.
    pub(crate) fn physics_smoke_chimney(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [10.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        for (name, x) in [("WallL", -10.5), ("WallR", 10.5)] {
            slab(
                world,
                name,
                Vec2::new(x, 2.0),
                [0.5, 2.5],
                0.0,
                [0.30, 0.30, 0.34, 1.0],
            );
        }

        // ⚠️ **A face de baixo em 2,2 é o que torna a cena julgável:** a cabeça
        // do personagem em repouso está em 1,4 (`float 0,9` + meia-altura 0,5),
        // então há 0,8 m de subida antes do vão — tempo de sobra para o sensor
        // ver a quina — e um pulo de altura cheia (~2,1 m) atravessa a chaminé
        // com folga. Uma laje mais alta faria o pulo cortado bater sem nunca
        // chegar lá, e a cena mediria o `cut_gravity`.
        let under = 2.2;
        for (name, cx) in [
            ("ShelfL", -(GAP * 0.5) - 4.0),
            ("ShelfR", (GAP * 0.5) + 4.0),
        ] {
            slab(
                world,
                name,
                Vec2::new(cx, under + 0.4),
                [4.0, 0.4],
                0.0,
                [0.42, 0.36, 0.30, 1.0],
            );
        }
        // O piso de cima — atravessar a chaminé tem de levar a algum lugar.
        for (name, cx) in [("TopL", -6.0), ("TopR", 6.0)] {
            slab(
                world,
                name,
                Vec2::new(cx, 4.4),
                [3.5, 0.4],
                0.0,
                [0.32, 0.44, 0.36, 1.0],
            );
        }

        spawn_player(world, Vec2::new(0.0, 1.4));
        eprintln!("{CHIMNEY_SMOKE_MESSAGE}");
    }

    /// **O vagão** — pular parado sobre uma plataforma que anda.
    pub(crate) fn physics_smoke_wagon(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Um chão lá embaixo, só para o personagem não cair para sempre quando
        // errar o pouso — que é exatamente o que a cena existe para mostrar.
        slab(
            world,
            "Ground",
            Vec2::new(0.0, -4.0),
            [40.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );

        // ⚠️ **O vagão é dinâmico com gravidade ZERO e massa 1000**, não
        // cinemático: um corpo cinemático é dirigido por uma pose por tique (o
        // `SceneAtTick` da timeline) e esta cena não tem curva nenhuma. Sem
        // gravidade e sem arrasto ele viaja a velocidade constante, e a massa
        // grande faz a reação do personagem (a 3ª lei, W6) não o desviar.
        world.spawn((
            Name::new("Wagon"),
            Transform::from_translation(Vec2::new(-12.0, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [8.0, 0.6], [0.45, 0.40, 0.30, 1.0]),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 4.0,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
            LockRotation,
            GravityScale(0.0),
            MassOverride(1000.0),
            MaterialCombine {
                restitution: CombineRule::Average,
                friction: CombineRule::Average,
            },
            InitialVelocity {
                linvel: [3.0, 0.0],
                angvel: 0.0,
            },
        ));

        // Em cima do vagão, no meio dele.
        spawn_player(world, Vec2::new(-12.0, 0.3 + 0.9));
        eprintln!("{WAGON_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 89 — o gesto é MIRAR, e o número é a largura da mira.
pub(crate) const CHIMNEY_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 89] A CHAMINE (W10). Duas lajes deixam um vao de 0,60 m\n",
    "sobre o personagem, que mede 0,40 m de largura -- 10 cm de folga de cada lado.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "(O Espaco NAO pula -- ele e' o Play/Pause do transporte.)\n",
    "\n",
    "1. Marque **Physics** na barra de transporte e de' Play.\n",
    "2. Ande um pouco para o lado e pule pela chamine, SEGURANDO o pulo.\n",
    "   Repita mirando cada vez pior -- 10 cm, 15 cm, 20 cm fora do centro.\n",
    "   >>> Ate' cerca de 20 cm de desvio ele PASSA: a cabeca encosta na quina\n",
    "       e o personagem desliza de lado o suficiente para escapar dela.\n",
    "3. O CONTROLE -- selecione o personagem e ponha **Corner Reach** em 0.\n",
    "   >>> Agora so' passa quem mira dentro dos 10 cm de folga geometrica. Fora\n",
    "       disso a cabeca BATE, o pulo morre no ar e ele cai de volta.\n",
    "   (Medido: a janela vai de +-0,10 m para +-0,22 m. Ela mais que dobra.)\n",
    "4. Volte o **Corner Reach** para 0,12 e mire MUITO mal, uns 30 cm fora.\n",
    "   >>> Ele bate. A assistencia perdoa um encosto, nao um erro de mira --\n",
    "       um teto continua um teto, e e' isso que a separa de teletransporte.\n",
);

/// O roteiro da cena 90 — o gesto é pular PARADO, e a resposta é onde se pousa.
pub(crate) const WAGON_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 90] O VAGAO (W10). Uma plataforma de 8 m viaja para a\n",
    "direita a 3 m/s com o personagem em cima dela.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: SETA PARA CIMA (ou Z) pula. NAO ande -- o gesto e' pular PARADO.\n",
    "\n",
    "1. Marque **Physics** na barra de transporte e de' Play.\n",
    "2. Sem tocar nas setas laterais, PULE.\n",
    "   >>> Ele pousa mais ou menos onde saiu, EM CIMA do vagao. Voce viaja com\n",
    "       ele: pular parado num trem nao te deixa para tras.\n",
    "3. O CONTROLE -- selecione o personagem e ponha **Lift Momentum** em 0.\n",
    "   Rebobine a regua para o comeco e pule de novo.\n",
    "   >>> O vagao SAI DEBAIXO DELE e ele cai atras. Era este o defeito: o corpo\n",
    "       sempre manteve a velocidade (isso e' o solver), mas o controle aereo\n",
    "       mira 'parado em relacao ao CHAO' e no ar o chao valia zero -- entao a\n",
    "       assistencia freava justamente o que a fisica tinha dado.\n",
    "   (Medido: o avanco no voo cai para 11% do balistico com a janela em 0.)\n",
    "4. Ponha **Lift Momentum** em 0,25 e repita.\n",
    "   >>> Ele quase alcanca o vagao. A janela e' quanto tempo a memoria dura,\n",
    "       e o default (1,5 s) cobre o pulo mais longo que a config produz.\n",
);
