//! **A cena 92 — O POÇO** (W13), irmã de `physics_smoke_player_carry.rs`.
//!
//! Duas paredes de frente uma para a outra, e um chão lá embaixo. O gesto que a
//! cena existe para julgar é **subir sem plataforma nenhuma**: agarra-se numa
//! parede, pula para a outra, agarra-se, pula de volta.
//!
//! # ⚠️ A largura do poço é ESCOLHIDA pela medição, não desenhada de cabeça
//!
//! O `measure_wall` mede o afastamento de um pulo de parede com o jogador a
//! segurar a direção da parede — o gesto real, e o pior caso, porque o controle
//! aéreo trabalha contra. Com o `Wall Lockout` autorado em 0,2 s ele afasta
//! **1,74 m**; o poço tem **2,4 m** de vão livre, ou seja o personagem NÃO
//! atravessa por inércia: ele precisa de soltar a direção da parede a meio do
//! voo, que é a habilidade que o gesto ensina.
//!
//! Um poço de 1,5 m seria atravessado sem o jogador fazer nada, e a cena
//! mostraria uma capacidade que não é a que a wave entrega.
//!
//! # ⚠️ E a cena arma a capacidade EXPLICITAMENTE
//!
//! O `PlatformPlayer` nasce com as paredes **desligadas** (`wall_slide_speed` e
//! `wall_jump_height` em zero) — parede é uma capacidade opt-in do personagem.
//! Uma cena que herdasse o default mostraria um poço em que nada acontece, e
//! leria como *"o pulo de parede está quebrado"*.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O vão livre entre as duas paredes — ver o aviso do módulo.
const GAP: f32 = 2.4;
/// A altura das paredes.
const WALL_HALF_Y: f32 = 7.0;

impl App {
    /// **O poço** — subir um corredor vertical em ziguezague.
    pub(crate) fn physics_smoke_well(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [12.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        // As duas paredes. ⚠️ Elas nascem do CHÃO para cima: um poço cujas
        // paredes começassem no ar deixaria o personagem cair para fora dele
        // pelo lado, e a cena mediria a pontaria em vez do gesto.
        for (name, cx) in [("WallL", -(GAP * 0.5) - 0.5), ("WallR", (GAP * 0.5) + 0.5)] {
            slab(
                world,
                name,
                Vec2::new(cx, WALL_HALF_Y),
                [0.5, WALL_HALF_Y],
                0.0,
                [0.30, 0.34, 0.42, 1.0],
            );
        }
        // E um beiral lá em cima, para a subida LEVAR a algum lugar.
        for (name, cx) in [("LedgeL", -3.4), ("LedgeR", 3.4)] {
            slab(
                world,
                name,
                Vec2::new(cx, WALL_HALF_Y * 2.0 - 0.4),
                [2.5, 0.4],
                0.0,
                [0.32, 0.44, 0.36, 1.0],
            );
        }

        spawn_player(world, Vec2::new(0.0, 1.4));

        // ⚠️ **A capacidade é ARMADA aqui**, e não herdada: ela nasce desligada
        // no produto (ver o aviso do módulo).
        let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
        for (mut p, _) in q.iter_mut(world) {
            p.wall_slide_speed = 3.0;
            p.wall_jump_height = 2.0;
        }
        eprintln!("{WELL_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 92 — o gesto é o ZIGUEZAGUE, e o que se julga é subir sem
/// plataforma nenhuma.
pub(crate) const WELL_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 92] O POCO (W13). Duas paredes com 2,4 m de vao,\n",
    "um chao embaixo e dois beirais la' em cima.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. Pule contra uma parede e SEGURE a direcao dela enquanto cai.\n",
    "    Ele tem de ESCORREGAR devagar, nao cair nem colar.\n",
    "    ⚠️ Solte a direcao: ele volta a cair normalmente. Agarrar-se exige\n",
    "    empurrar -- raspar numa parede de passagem nao pode prender ninguem.\n",
    " 3. Escorregando, aperte PULO: ele sai da parede para cima e para o LADO.\n",
    " 4. Agora o ziguezague: agarre a outra parede, pule, repita, e SUBA ate\n",
    "    os beirais. ⚠️ O vao e' 2,4 m de proposito -- mais largo do que um\n",
    "    pulo de parede atravessa sozinho. Solte a direcao a meio do voo.\n",
    " 5. No CHAO o mesmo aperto continua a ser um pulo normal.\n",
);
