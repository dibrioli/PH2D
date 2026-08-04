//! **A cena 86 — A FITA** (W7), irmã de `physics_smoke_player.rs`.
//!
//! Cortada por CENA, o precedente que este módulo de smoke já usa
//! (`physics_smoke_rigs` · `_collision` · `_props`): as outras cenas de player
//! perguntam sobre a LEI do movimento — a perna, o arco do pulo, a 3ª lei. Esta
//! pergunta sobre o **RELÓGIO**, e o gesto que ela pede não é um traço, é um
//! SCRUB.
//!
//! Os helpers compartilhados (`slab`, `spawn_player`) continuam no pai: eles
//! respondem *"como se põe um chão e um personagem numa cena"*, que é pergunta
//! de todas.

use ph2d_core::Vec2;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

impl App {
    /// **A corrida se REPETE quando a régua volta.**
    ///
    /// Até esta wave o player era a única coisa que quebrava *"o mundo é função
    /// de `(tique, cena)`"*: o laço de replay de um scrub dirigia as plataformas
    /// e deixava o personagem **sem perna e sem caminhada**, então ele caía
    /// pelos tiques replayados e parava onde a gravidade o deixasse.
    ///
    /// ⚠️ A pista tem degraus e uma saliência **para a corrida ter FORMA**: uma
    /// trajetória que só translada é uma em que um replay errado ainda parece
    /// plausível.
    pub(crate) fn physics_smoke_tape(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [9.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            "Step1",
            Vec2::new(2.5, 0.3),
            [1.2, 0.3],
            0.0,
            [0.30, 0.42, 0.36, 1.0],
        );
        slab(
            world,
            "Step2",
            Vec2::new(6.0, 0.9),
            [1.2, 0.3],
            0.0,
            [0.30, 0.42, 0.36, 1.0],
        );
        slab(
            world,
            "Ledge",
            Vec2::new(11.0, 1.6),
            [2.0, 0.3],
            0.0,
            [0.42, 0.36, 0.30, 1.0],
        );
        spawn_player(world, Vec2::new(-7.0, 1.0));

        eprintln!("{TAPE_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 86 — o gesto é o SCRUB, não o traço.
pub(crate) const TAPE_SMOKE_MESSAGE: &str = concat!(
    "\n=== PH2D_PHYSICS_SMOKE=86 -- A FITA (W7) ===\n",
    "A cena: uma pista com dois degraus e uma saliencia, o personagem a\n",
    "esquerda. A pergunta desta cena e' sobre o RELOGIO, nao sobre o pulo.\n",
    "\n",
    "1. Marque **Physics** na barra de transporte (ele nasce DESMARCADO).\n",
    "2. Play. Segure a seta DIREITA e pule (Espaco) para subir os degraus --\n",
    "   corra uns 3 s, ate a saliencia. Nao precisa ser bonito.\n",
    "3. Solte TODAS as teclas.\n",
    "4. Arraste a regua PARA TRAS, ate o meio da corrida.\n",
    "   >>> O personagem tem de estar ONDE ELE ESTAVA naquele instante, no\n",
    "       meio do mesmo salto, na mesma altura.\n",
    "   Antes desta wave ele DESPENCAVA: o replay refazia a cena e deixava o\n",
    "   personagem sem perna e sem caminhada, entao ele caia pelos tiques\n",
    "   replayados e parava no chao, longe de onde a corrida passou.\n",
    "5. Arraste para a FRENTE de novo -- a corrida se repete IGUAL, e se repete\n",
    "   com as teclas SOLTAS: e' a prova de que quem dirige o replay e' a fita\n",
    "   daquele tique, nao o seu dedo de agora.\n",
    "6. Repita o scrub duas ou tres vezes: a trajetoria nao pode DERIVAR a cada\n",
    "   ida e volta.\n",
);
