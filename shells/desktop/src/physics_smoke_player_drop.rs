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
//!
//! # ⚠️ E os 2,0 m estão DEZ CENTÍMETROS acima de um penhasco
//!
//! Os `0,3 m de margem` acima são a folga que a lei de aposentadoria EXIGE, e
//! a medição (`ph2d-physics-ecs/tests/measure_drop_retire.rs`) mostra que ela é
//! mais apertada do que parece. Nesta geometria exacta — pranchas de
//! meia-espessura `0,15`, um aperto e uma tentativa de voltar:
//!
//! | `RISE` | antes (W19) | **hoje** |
//! |---|---|---|
//! | 1,60 – 1,70 | fantasma para sempre — e ele ficava **PRESO** lá | **funciona** (W27) |
//! | **1,75 – 1,85** | **arremessado de volta** | **funciona** (W20) |
//! | 1,90 + | funciona | funciona |
//!
//! ⚠️ **As duas bordas fecharam, e a de baixo custou uma medição para se
//! entender:** ela estava registada como *"as pranchas ficam fantasma"*, que é o
//! sintoma; o preço era o personagem descer um degrau e **ficar lá para sempre**
//! (`−0,598 → −0,598` a 1,60, em toda célula da janela). A cura é uma cláusula
//! de intenção — **subir aposenta a descida** —, e o porquê está no aviso de
//! `bridge::player::retire_drops`.
//!
//! ⚠️ **O `RISE` continua em 2,0**, e não porque seja apertado: a cena existe
//! para o artista julgar o GESTO, e um vão folgado é o que a torna legível. O
//! que mudou é que apertá-lo já não a transforma num defeito.

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

/// A distância entre os andares de cada uma das três escadas da cena 97 — as
/// três células que a W20 mediu, e que o artista tem de conseguir distinguir a
/// olho (`ph2d-physics-ecs/tests/measure_drop_retire.rs`).
const WALL_RISES: [f32; 3] = [1.80, 2.00, 1.60];
/// Onde cada escada fica, no eixo X.
const WALL_X: [f32; 3] = [-7.0, 0.0, 7.0];

impl App {
    /// **A cena 97 — AS DUAS BORDAS DA DESCIDA** (W20).
    ///
    /// Três escadas da MESMA prancha (meia-espessura 0,15) e nada diferente
    /// entre elas além do VÃO. É esse o desenho: o que muda o comportamento é um
    /// número só, e as três células estão nos dois lados e no meio da lei.
    pub(crate) fn physics_smoke_drop_edges(&mut self) {
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

        for (i, (&rise, &x)) in WALL_RISES.iter().zip(WALL_X.iter()).enumerate() {
            for f in 0..FLOORS {
                let y = FIRST + rise * f as f32;
                let half = [2.5, PLANK_HALF_Y];
                world.spawn((
                    Name::new(format!("Plank{}-{}", i + 1, f + 1)),
                    Transform::from_translation(Vec2::new(x, y)),
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
        }

        // No topo da escada do MEIO — a que sempre funcionou —, para o artista
        // começar pelo controle e só depois julgar as bordas.
        let top = FIRST + WALL_RISES[1] * (FLOORS - 1) as f32;
        spawn_player(world, Vec2::new(WALL_X[1], top + PLANK_HALF_Y + 0.9));
        eprintln!("{EDGES_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 97 — o gesto é o MESMO da 91, e o que se julga é a
/// diferença entre as três escadas.
pub(crate) const EDGES_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 97] AS DUAS BORDAS DA DESCIDA (W20). Tres escadas de\n",
    "pranchas identicas; so' o VAO entre os degraus muda.\n",
    "  ESQUERDA vao 1,80  ·  MEIO vao 2,00  ·  DIREITA vao 1,60\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: <- / -> andam. CIMA pula. BAIXO + PULO desce um andar.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play. Voce comeca no MEIO.\n",
    " 2. MEIO (2,00): desca ate' o chao e volte a subir. E' o controle --\n",
    "    sempre funcionou, e tem de continuar identico.\n",
    " 3. ESQUERDA (1,80): desca. ⚠️ ESTE era o vao em que ele NAO DESCIA --\n",
    "    a prancha voltava a ser solida a meio da queda e o arremessava de\n",
    "    volta. Agora tem de descer UM andar por aperto, como o meio.\n",
    " 4. DIREITA (1,60): desca UM andar. Ele desce -- e a partir dai as\n",
    "    pranchas ficam FANTASMA: um pulo ja' nao pousa nelas.\n",
    "    ⚠️ Isso e' o LIMITE HONESTO, nao um bug novo: ali a caixa dele ainda\n",
    "    sobrepoe a prancha, que de facto o pegaria -- as saidas eram\n",
    "    'fantasma' ou 'cuspido', e escolhemos fantasma.\n",
    " 5. E OLHE O CONTORNO (tecla B): enquanto ele atravessa, TODA prancha da\n",
    "    cena fica apagada. Na direita ela fica apagada para sempre -- que e'\n",
    "    exatamente o estado que antes era invisivel.\n",
);
