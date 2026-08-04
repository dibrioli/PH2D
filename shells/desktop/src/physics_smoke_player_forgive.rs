//! **A cena 87 — O PERDÃO** (W8), irmã de `physics_smoke_player.rs`.
//!
//! Cortada por CENA, o precedente do módulo. A pergunta aqui não é sobre a
//! trajetória nem sobre o relógio: é sobre **a distância entre o que o jogador
//! quis e o que ele fez**, que é o único assunto do platformer que não se mede
//! num gate — os dois números têm de ser julgados pelo dedo.

use ph2d_core::Vec2;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

impl App {
    /// **Coyote time e jump buffer**, com o controle ao lado.
    ///
    /// ⚠️ A cena tem DUAS armadilhas, uma por janela, e elas testam erros
    /// OPOSTOS: a beirada pune quem aperta **tarde**, o poço pune quem aperta
    /// **cedo**. Uma cena com só uma delas deixaria metade da wave sem olho.
    pub(crate) fn physics_smoke_forgive(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // (1) A BEIRADA — corra e caia dela. Aperte DEPOIS de sair: o coyote.
        slab(
            world,
            "Run",
            Vec2::new(-4.0, -0.5),
            [6.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        // O vão. Do outro lado, a plataforma que só se alcança pulando da quina.
        slab(
            world,
            "Landing",
            Vec2::new(6.0, -0.5),
            [3.0, 0.5],
            0.0,
            [0.30, 0.42, 0.36, 1.0],
        );
        // (2) O POÇO — desça e aperte ANTES de tocar o fundo: o buffer.
        slab(
            world,
            "PitFloor",
            Vec2::new(13.0, -3.5),
            [3.0, 0.5],
            0.0,
            [0.42, 0.36, 0.30, 1.0],
        );
        slab(
            world,
            "PitWall",
            Vec2::new(16.5, -1.5),
            [0.5, 2.5],
            0.0,
            [0.42, 0.36, 0.30, 1.0],
        );
        spawn_player(world, Vec2::new(-8.0, 1.0));

        eprintln!("{FORGIVE_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 87 — os dois números se julgam com o DEDO.
pub(crate) const FORGIVE_SMOKE_MESSAGE: &str = concat!(
    "\n=== PH2D_PHYSICS_SMOKE=87 -- O PERDAO (W8) ===\n",
    "Duas armadilhas, uma por janela, e elas punem erros OPOSTOS.\n",
    "\n",
    "1. Marque **Physics** na barra de transporte e de' Play.\n",
    "2. COYOTE -- corra para a direita e caia da beirada. Aperte Espaco\n",
    "   *depois* de ja' ter saido do chao, um instante tarde demais.\n",
    "   >>> O pulo tem de SAIR, e voce alcanca a plataforma verde.\n",
    "3. BUFFER -- va' ate' o poco e caia nele. Aperte Espaco *antes* de o pe'\n",
    "   tocar o fundo.\n",
    "   >>> O pulo tem de sair NO tique do pouso, nao um quadro depois. Sem\n",
    "       o buffer o aperto morre no ar e voce fica preso la' dentro.\n",
    "4. O CONTROLE -- selecione o personagem no Inspector, zere\n",
    "   **Coyote Time (s)** e **Jump Buffer (s)**, e repita 2 e 3.\n",
    "   >>> Agora os dois erros PUNEM: cair da beirada e' cair, e o aperto\n",
    "       antes do pouso e' um aperto perdido. E' esse o contraste que diz\n",
    "       que a assistencia esta' agindo, e nao que o jogo e' facil.\n",
    "5. Suba as duas para 0,3 s e repita: fica GENEROSO demais -- e' de\n",
    "   proposito que o teto seja 0,5, onde a queda dentro da janela ja'\n",
    "   passa de uma altura de corpo e o pulo le' como se fosse do ar.\n",
);
