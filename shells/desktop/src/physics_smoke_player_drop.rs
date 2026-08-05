//! **A cena 91 — A ESCADA DE PRANCHAS** (W12), irmã de
//! `physics_smoke_player_carry.rs`.
//!
//! A plataforma jump-through existe desde a W-OneWay (cena `=23`) e ali ela é
//! julgada por BAIXO: a caixa sobe através dela e pousa em cima. O que faltava
//! era a outra metade do idioma — **sair dela por baixo de propósito** —, e é
//! isso que esta cena põe na mão do artista.
//!
//! # ⚠️ Uma ESCADA, e não uma prancha só
//!
//! Com uma prancha apenas, *"o personagem atravessou"* e *"o personagem caiu"*
//! são a mesma imagem. Com três, o gesto ganha a propriedade que o torna útil e
//! que um defeito não reproduz: **ele desce UM andar por aperto**, porque a
//! descida mira a prancha de baixo dos pés e acaba quando ele a passou. Um bug
//! que simplesmente desligasse a solidez o levaria ao chão de uma vez, e a cena
//! mostra a diferença sem nenhum número.
//!
//! # ⚠️ O vão de 2,0 m é ESCOLHIDO por duas leis, não desenhado de cabeça
//!
//! *Para BAIXO:* a descida acaba quando a caixa do personagem está inteiramente
//! abaixo da prancha (`bridge::player::retire_drops`). Em repouso sobre uma
//! prancha ele ocupa até `topo + 1,4`; a prancha de cima começa em `+2,0 −
//! 0,15`, ou seja **0,3 m de margem** — a descida se retira, e a prancha volta a
//! ser sólida.
//!
//! *Para CIMA:* um pulo de altura cheia sobe ~2,1 m, então subir um andar de 2,0
//! é possível **por pouco**, que é o que faz a escada ser subida E descida com o
//! mesmo par de botões. Um vão de 2,5 tornaria a subida impossível e a cena
//! mediria só metade do idioma.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, OneWayPlatform, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// A distância entre os andares — ver o aviso do módulo.
const RISE: f32 = 2.0;
/// Meia-espessura de uma prancha.
const PLANK_HALF_Y: f32 = 0.15;
/// A altura da prancha mais baixa.
const FIRST: f32 = 2.0;
/// Quantos andares a escada tem.
const FLOORS: usize = 3;

impl App {
    /// **A escada de pranchas** — descer com baixo + pulo, subir só com o pulo.
    pub(crate) fn physics_smoke_pass_through(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // O chão SÓLIDO, e ele é o controle da cena inteira: o mesmo gesto que
        // atravessa as pranchas não o atravessa, e o personagem para ali.
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [12.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );

        // As pranchas. ⚠️ **Um tom próprio, e não é enfeite:** uma plataforma
        // jump-through comporta-se de forma diferente de um chão, e uma cena em
        // que as duas se parecem obriga o artista a descobrir qual é qual
        // tentando atravessar cada uma.
        for i in 0..FLOORS {
            let y = FIRST + RISE * i as f32;
            let half = [4.0, PLANK_HALF_Y];
            world.spawn((
                Name::new(format!("Plank{}", i + 1)),
                Transform::from_translation(Vec2::new(0.0, y)),
                Sprite::atlas(
                    WHITE_TILE_KEY,
                    [half[0] * 2.0, half[1] * 2.0],
                    [0.86, 0.70, 0.38, 1.0],
                ),
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
                OneWayPlatform,
            ));
        }

        // Em cima de tudo: a escada é para DESCER, e o gesto novo é o primeiro
        // que o artista vai querer experimentar.
        let top = FIRST + RISE * (FLOORS - 1) as f32;
        spawn_player(world, Vec2::new(0.0, top + PLANK_HALF_Y + 0.9));
        eprintln!("{DROP_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 91 — o gesto é **BAIXO + PULO**, e o que se julga é *um
/// andar por aperto*.
pub(crate) const DROP_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 91] A ESCADA DE PRANCHAS (W12). Tres plataformas\n",
    "jump-through (as douradas) a 2,0 m uma da outra, e um chao solido embaixo.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "          SETA PARA BAIXO (ou S) e o botao novo.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. So o PULO: ele sobe e volta a pousar na MESMA prancha -- por cima ela e\n",
    "    solida, como sempre foi.\n",
    " 3. So o BAIXO, segurado: nao acontece nada. Segurar baixo enquanto se anda\n",
    "    NAO pode derrubar ninguem.\n",
    " 4. BAIXO + PULO: ele atravessa a prancha e pousa na de baixo.\n",
    "    ⚠️ UM ANDAR POR APERTO -- se ele for ao chao de uma vez, pare e reporte.\n",
    " 5. Repita ate o chao. No chao SOLIDO o mesmo gesto volta a ser um PULO:\n",
    "    o botao so muda de significado onde a descida e possivel.\n",
    " 6. Suba de volta so com o pulo. Cada prancha tem de o SEGURAR outra vez --\n",
    "    e a prova de que a descida se retira quando ele ja passou.\n",
);
