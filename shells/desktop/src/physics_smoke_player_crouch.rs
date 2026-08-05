//! **A cena 94 — O CORREDOR BAIXO** (W15), irmã de
//! `physics_smoke_player_dash.rs`.
//!
//! Dois túneis: um baixo demais para se atravessar de pé, e um alto que serve de
//! CONTROLE. O gesto que a cena existe para julgar é o **agachar como passagem**
//! — e, debaixo do primeiro, a recusa de se levantar.
//!
//! # ⚠️ As alturas são ARITMÉTICA da cápsula, não desenho de cabeça
//!
//! O corpo mede `half_height + radius = 0,50` do centro para cima, e o centro
//! paira à altura da perna. Logo:
//!
//! | pose | centro | topo |
//! |---|---|---|
//! | de pé | 0,90 | **1,40** |
//! | agachado | 0,55 | **1,05** |
//!
//! O túnel baixo tem o fundo em **1,20** — entre os dois, com 0,15 m de folga
//! para cada lado. Um túnel a 1,45 seria atravessado de pé e a cena mostraria
//! uma capacidade que não é a que a wave entrega; um a 1,00 não deixaria passar
//! nem agachado, e leria como *"o agachar não funciona"*.
//!
//! # ⚠️ E a altura agachada tem um PISO GEOMÉTRICO
//!
//! `0,55` está acima dos **0,50** que a cápsula mede do centro ao pé — abaixo
//! disso ela enterra no chão e quem resolve é o solver. **A lei pura não pode
//! clampar isso** (ela não conhece formas, de propósito), então o piso vive onde
//! a geometria vive: aqui, e no aviso do componente.
//!
//! # ⚠️ E a cena arma a capacidade EXPLICITAMENTE
//!
//! O `PlatformPlayer` nasce com o agachar **desligado** (`crouch_height` em
//! zero) — é uma capacidade opt-in do personagem. Uma cena que herdasse o
//! default mostraria um túnel em que nada acontece.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// A altura de flutuação agachado desta cena — ver o piso geométrico no topo.
const CROUCH_HEIGHT: f32 = 0.55;
/// A velocidade de cruzeiro agachado, m/s (a de pé é 6,0).
const CROUCH_SPEED: f32 = 2.0;
/// O fundo do túnel BAIXO — entre o topo de pé (1,40) e o agachado (1,05).
const LOW_BOTTOM: f32 = 1.2;
/// O fundo do túnel ALTO, o CONTROLE: acima do topo de pé.
const TALL_BOTTOM: f32 = 1.6;

impl App {
    /// **O corredor baixo** — passar por onde não se cabe de pé.
    pub(crate) fn physics_smoke_crouch(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        slab(
            world,
            "Floor",
            Vec2::new(20.0, -0.5),
            [40.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );

        // O túnel BAIXO: comprido de propósito, para o artista ter tempo de
        // soltar o botão lá dentro e ver que ele NÃO se levanta.
        tunnel(
            world,
            "Low Tunnel",
            6.0,
            20.0,
            LOW_BOTTOM,
            [0.42, 0.32, 0.34, 1.0],
        );
        // O CONTROLE: o mesmo gesto de andar, e este ele atravessa de pé.
        tunnel(
            world,
            "Tall Tunnel",
            26.0,
            38.0,
            TALL_BOTTOM,
            [0.32, 0.42, 0.36, 1.0],
        );

        // Uma parede no fim, para o personagem não sair de quadro a correr.
        slab(
            world,
            "Backstop",
            Vec2::new(44.5, 1.5),
            [0.5, 2.0],
            0.0,
            [0.30, 0.34, 0.42, 1.0],
        );

        spawn_player(world, Vec2::new(0.0, 1.4));

        // ⚠️ **A capacidade é ARMADA aqui**, e não herdada (ver o topo).
        let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
        for (mut p, _) in q.iter_mut(world) {
            p.crouch_height = CROUCH_HEIGHT;
            p.crouch_speed = CROUCH_SPEED;
        }
        eprintln!("{CROUCH_SMOKE_MESSAGE}");
    }
}

/// Uma laje-teto entre `x0` e `x1`, com a face de baixo em `bottom`.
fn tunnel(
    world: &mut bevy_ecs::world::World,
    name: &str,
    x0: f32,
    x1: f32,
    bottom: f32,
    tint: [f32; 4],
) {
    const HALF_Y: f32 = 1.0;
    let half_x = (x1 - x0) * 0.5;
    slab(
        world,
        name,
        Vec2::new(x0 + half_x, bottom + HALF_Y),
        [half_x, HALF_Y],
        0.0,
        tint,
    );
}

/// O roteiro da cena 94 — o gesto é SEGURAR BAIXO, e o que se julga é passar
/// por onde não se cabe de pé.
pub(crate) const CROUCH_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 94] O CORREDOR BAIXO (W15). Dois tuneis: o primeiro com o\n",
    "fundo em 1.20 m (o topo do personagem de pe' mede 1.40, agachado 1.05) e o\n",
    "segundo, o CONTROLE, com o fundo em 1.60.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA BAIXO (ou S) agacha.\n",
    "          SETA PARA CIMA (ou Z) pula.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. Ande para a direita ate' o primeiro tunel: ele PARA na entrada.\n",
    "    De pe' ele nao cabe -- e' o controle do gesto.\n",
    " 3. Segure BAIXO: ele abaixa (o corpo desce 0.35 m, e o desenho desce\n",
    "    junto) e passa a andar mais devagar. Agora entre no tunel.\n",
    " 4. LA' DENTRO, solte o botao de baixo: ele NAO se levanta. Nao ha' para\n",
    "    onde subir, e a ferramenta recusa em vez de o enfiar na pedra.\n",
    " 5. Continue ate' sair pelo outro lado: assim que houver ceu, ele levanta\n",
    "    sozinho.\n",
    " 6. Siga para o segundo tunel e atravesse-o DE PE'. E' o CONTROLE: se este\n",
    "    tambem o parasse, a cena estaria a medir a laje e nao o agachar.\n",
    " 7. Agachado, aperte PULAR: ele pula (o agachar nao e' uma prisao).\n",
);
