//! **A cena 99 — O AGARRAR-SE** (W23), irmã de `physics_smoke_player_wall.rs`.
//!
//! O poço da 92 pede ritmo: agarra, escorrega, pula, agarra a outra. Esta pede o
//! oposto — **parar** numa parede e escolher o momento. É a outra metade do item
//! aberto da W13, e o próprio handoff a nomeava: *"ficar parado numa parede é
//! outra mecânica, com botão próprio"*.
//!
//! # ⚠️ A reserva é o que impede a parede de virar uma beirada permanente
//!
//! A pesquisa é unânime e o que ela ensina não é o botão, é o **custo**: o
//! Celeste começou **sem reserva** e o jogo ficava resolvível pendurando-se; um
//! TEMPORIZADOR simples também foi tentado lá e abandonado por não distinguir
//! *escalar* de *pendurar*. Hollow Knight e Ori chegam ao mesmo lugar pelo lado
//! oposto — lá o que limita é a habilidade, não o recurso.
//!
//! Aqui a reserva é **um número em segundos**, e o zero desliga. ⚠️ **A
//! assimetria do Celeste (subir custa mais que pendurar) NÃO foi construída:** o
//! segundo knob teria o valor certo em função do primeiro, que é a ergonomia que
//! este repositório trata como bug de desenho.
//!
//! # ⚠️ A cena tem TRÊS estações porque a reserva só se lê comparando
//!
//! Um vão sozinho não diz se a reserva funciona: o personagem ou chega ou não
//! chega, e as duas leituras cabem em qualquer bug. Com três reservas na MESMA
//! geometria, o que se vê é a reserva a **acabar** — e o instante em que ela
//! acaba é o instante em que ele volta a escorregar.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// A altura das paredes de cada estação.
const WALL_TOP: f32 = 9.0;
/// A altura do beiral que a estação pede para alcançar.
///
/// ⚠️ **Alto de propósito**: um pulo de parede sobe ~2,0 m, e o beiral está a
/// 4,4 m do chão — ou seja **não se chega lá sem pendurar e escolher a hora**.
/// Um beiral ao alcance de um pulo mostraria a cena a funcionar com a reserva
/// desligada.
const LEDGE_Y: f32 = 4.4;
/// De quanto em quanto as estações se repetem.
const STATION: f32 = 7.0;

impl App {
    /// **O agarrar-se** — três estações, três reservas.
    pub(crate) fn physics_smoke_wall_grab(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [16.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );

        // Tres estacoes identicas: uma parede a' direita do jogador e um beiral
        // la' em cima, do lado de ca'. So' a RESERVA autorada muda.
        for cx in [-STATION, 0.0, STATION] {
            slab(
                world,
                "Wall",
                Vec2::new(cx + 1.4, WALL_TOP * 0.5),
                [0.5, WALL_TOP * 0.5],
                0.0,
                [0.30, 0.34, 0.42, 1.0],
            );
            // O beiral sai da parede para a ESQUERDA, sobre a cabeca do jogador:
            // e' preciso subir pela parede e pular para ele.
            slab(
                world,
                "Ledge",
                Vec2::new(cx - 0.4, LEDGE_Y),
                [1.3, 0.25],
                0.0,
                [0.32, 0.44, 0.36, 1.0],
            );
        }

        spawn_player(world, Vec2::new(-STATION, 1.4));

        // ⚠️ A capacidade e' ARMADA aqui, e nao herdada: parede nasce desligada
        // no produto. As tres reservas sao o que a cena existe para comparar,
        // mas o player e' UM -- o artista muda o numero na §14 entre as
        // estacoes, que e' exatamente o gesto que a wave entrega.
        let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
        for (mut p, _) in q.iter_mut(world) {
            p.wall_slide_speed = 3.0;
            p.wall_jump_height = 2.0;
            p.wall_grab_stamina = 0.0;
        }
        eprintln!("{GRAB_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 99 — o gesto é **parar** na parede, e o que se julga é a
/// reserva a acabar.
pub(crate) const GRAB_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 99] O AGARRAR-SE (W23). Tres estacoes identicas: parede a'\n",
    "direita, beiral a 4,4 m -- alto demais para um pulo de parede sozinho.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: <- / -> andam. CIMA (ou Z) pula. Q arranca. **R AGARRA.**\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. CONTROLE primeiro, com Wall Grab em 0 (como a cena abre): pule contra\n",
    "    a parede e segure a direcao. Ele ESCORREGA -- e segurar R nao faz\n",
    "    nada, porque a capacidade nasce desligada.\n",
    " 3. Selecione o Player e ponha **Wall Grab (s) = 0,5** na secao Player.\n",
    "    Repita: agora segurar R PARA a descida... por meio segundo, e depois\n",
    "    ele volta a escorregar sozinho. ⚠️ E' a reserva a acabar, e ela e' o\n",
    "    que impede a parede de virar uma beirada permanente.\n",
    " 4. Ponha **2,0**. Agora da' tempo de pendurar, escolher a hora e pular\n",
    "    para o beiral. Este e' o gesto que a wave entrega.\n",
    " 5. Toque o CHAO e volte a' parede: a reserva tem de estar CHEIA outra\n",
    "    vez. Ela enche de uma vez, no chao -- qualquer outra regra ensinaria\n",
    "    o jogador a esperar parado.\n",
    " 6. ⚠️ E o que NAO ha', para nao o confundir com bug: nao se ESCALA a\n",
    "    parede (subir e descer agarrado pede um eixo vertical que a entrada\n",
    "    deste app nao tem -- 'cima' ja' e' o pulo). Agarrado, ele PARA.\n",
);
