//! **A cena 98 — A PAREDE COM JANELAS** (W13, o flanco), irmã de
//! `physics_smoke_player_wall.rs`.
//!
//! O poço da cena 92 provou o ziguezague contra paredes **lisas**. Esta troca uma
//! delas por uma parede com **janelas** — e é exatamente essa geometria que o
//! sensor lateral não enxergava.
//!
//! # ⚠️ A janela mede 0,8 m, e o número não é decorativo
//!
//! O corpo do personagem mede **1,0 m** de caixa envolvente (cápsula
//! `half_height 0,3` + `radius 0,2`). Uma janela de 0,8 m deixa **10 cm de pé E
//! 10 cm de ombro** encostados na pedra — o jogador vê um personagem colado à
//! parede. O sensor lia só a **cintura**, que é justamente o que atravessa o
//! vazio, então a lei respondia *"não há parede aqui"*.
//!
//! **Medido** (`ph2d-physics-ecs/tests/measure_wall_flank.rs`), com a janela na
//! altura da cintura:
//!
//! | janela | pulo de parede |
//! |---|---|
//! | parede lisa | **2,162 m** |
//! | 0,60 m | 1,997 m (o *buffer* do pulo mascarava) |
//! | 0,70 m | 2,292 m (idem) |
//! | **0,75 m** | **0,000 m** — recusado por inteiro |
//! | **0,80 m** | **0,000 m** |
//! | 0,90 m | 0,000 m |
//!
//! ⚠️ **Abaixo de ~0,70 m o defeito era INVISÍVEL**, e não por acaso: o buffer do
//! pulo guarda o aperto até o bloco de baixo reaparecer. Uma cena com janelas
//! pequenas mostraria tudo a funcionar sobre um sensor cego — por isso as desta
//! aqui têm 0,8.
//!
//! # ⚠️ E o ESCORREGAMENTO quase não denuncia — o oráculo é o PULO
//!
//! Passar pela janela custa uns centímetros de descida a mais (0,0500 →
//! 0,0632 m/tique medidos), porque a *cola* que o `platform_wall` documenta
//! segura o personagem de qualquer jeito. O pulo não tem cola: ou a lei vê
//! parede naquele tique, ou o botão não faz nada.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O vão livre entre as duas paredes.
///
/// ⚠️ Mais estreito que os 2,4 m da cena 92 **de propósito**: lá o vão era a
/// prova (o pulo não atravessa sozinho), aqui ele é só o caminho — o que se
/// julga são as janelas, e um ziguezague difícil esconderia isso atrás de
/// pontaria.
const GAP: f32 = 1.8;
/// A altura das duas paredes.
const WALL_TOP: f32 = 14.0;
/// A altura de cada janela — ver o aviso do módulo.
const WINDOW: f32 = 0.8;
/// De quanto em quanto as janelas se repetem, para a subida cruzar VÁRIAS.
const WINDOW_EVERY: f32 = 3.0;
/// A primeira janela — acima de um pulo do chão, para a cena começar na parede
/// lisa e o artista ter um controle antes do caso.
const FIRST_WINDOW: f32 = 3.0;

impl App {
    /// **A parede com janelas** — subir um poço cuja parede direita tem buracos.
    pub(crate) fn physics_smoke_flank(&mut self) {
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

        // A parede ESQUERDA e' lisa: ela e' o CONTROLE, e o artista sente a
        // diferenca sem trocar de cena.
        let left_cx = -(GAP * 0.5) - 0.5;
        slab(
            world,
            "WallSolid",
            Vec2::new(left_cx, WALL_TOP * 0.5),
            [0.5, WALL_TOP * 0.5],
            0.0,
            [0.30, 0.34, 0.42, 1.0],
        );

        // A parede DIREITA e' a mesma parede com buracos. Ela nasce como uma
        // pilha de blocos, e o que fica entre eles sao as janelas.
        let right_cx = (GAP * 0.5) + 0.5;
        let mut lo = 0.0f32;
        let mut n = 0;
        let mut y = FIRST_WINDOW;
        while y + WINDOW < WALL_TOP {
            slab(
                world,
                "WallWindowed",
                Vec2::new(right_cx, (lo + y) * 0.5),
                [0.5, (y - lo) * 0.5],
                0.0,
                [0.42, 0.34, 0.30, 1.0],
            );
            lo = y + WINDOW;
            y += WINDOW_EVERY;
            n += 1;
        }
        slab(
            world,
            "WallWindowed",
            Vec2::new(right_cx, (lo + WALL_TOP) * 0.5),
            [0.5, (WALL_TOP - lo) * 0.5],
            0.0,
            [0.42, 0.34, 0.30, 1.0],
        );

        // Um beiral la' em cima, para a subida LEVAR a algum lugar.
        slab(
            world,
            "Ledge",
            Vec2::new(3.4, WALL_TOP - 0.4),
            [2.5, 0.4],
            0.0,
            [0.32, 0.44, 0.36, 1.0],
        );

        spawn_player(world, Vec2::new(0.0, 1.4));

        // ⚠️ A capacidade e' ARMADA aqui, e nao herdada: parede nasce desligada
        // no produto (o aviso do `physics_smoke_player_wall`).
        let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
        for (mut p, _) in q.iter_mut(world) {
            p.wall_slide_speed = 3.0;
            p.wall_jump_height = 2.0;
        }
        eprintln!(
            "{FLANK_SMOKE_MESSAGE_HEAD}{n} janelas de 0,8 m na parede direita.\n{FLANK_SMOKE_MESSAGE}"
        );
    }
}

/// A primeira linha da cena 98 — ⚠️ ela IMPRIME o que a cena montou. Se o número
/// de janelas não aparecer, a cena não montou e o resto do smoke não diz nada.
pub(crate) const FLANK_SMOKE_MESSAGE_HEAD: &str = "[physics-smoke 98] A PAREDE COM JANELAS (W13, o flanco). Poco de 1,8 m:\nesquerda LISA (o controle), direita com ";

/// O roteiro da cena 98 — o gesto é o ziguezague da 92, e o que se julga é o que
/// acontece **na janela**.
pub(crate) const FLANK_SMOKE_MESSAGE: &str = concat!(
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. CONTROLE primeiro: pule contra a parede ESQUERDA (a lisa), segure a\n",
    "    direcao dela, escorregue e pule. Isto sempre funcionou e tem de\n",
    "    continuar identico.\n",
    " 3. Agora a parede DIREITA. Escorregue por ela ate' o corpo ficar em\n",
    "    frente a uma JANELA -- pe e ombro na pedra, cintura no vazio.\n",
    "    ⚠️ APERTE PULO ALI. Ele TEM de sair da parede.\n",
    "    Antes desta wave o botao nao fazia NADA nessa posicao: 0,000 m de\n",
    "    subida contra 2,162 m na parede lisa.\n",
    " 4. Suba o poco inteiro em ziguezague ate' o beiral, cruzando todas as\n",
    "    janelas. Nenhuma pode 'engolir' um pulo.\n",
    " 5. ⚠️ O LIMITE HONESTO, para nao o confundir com bug: uma janela MAIOR\n",
    "    que o corpo (mais de 1,0 m) nao e' parede nenhuma -- nada encosta --\n",
    "    e ali a recusa esta' certa. Estas janelas medem 0,8 m de proposito.\n",
);
