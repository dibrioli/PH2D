//! **A cena 95 — A CORRIDA VIRA ANIMAÇÃO** (W16), irmã de
//! `physics_smoke_player_crouch.rs`.
//!
//! Um percurso curto com um degrau e um vão. O gesto que a cena existe para
//! julgar é **assar a própria corrida**: o artista joga, aperta Bake, e o que
//! ele acabou de fazer vira curva na timeline.
//!
//! # ⚠️ A CONTRADIÇÃO, e ela está no roteiro de propósito
//!
//! Assar vira o corpo **Kinematic**, e a lei do player não dirige um corpo
//! kinematic — ela escreve força e velocidade, que massa infinita ignora. Logo,
//! **depois do bake o personagem para de responder ao teclado**. Isso não é um
//! defeito: é o que *assar* significa, e é a mesma contradição que a W-BakeJoint
//! já mediu do outro lado (*um Kinematic do bake não é movido por joint*).
//!
//! O roteiro faz o artista **encontrá-la de propósito** em vez de a descobrir —
//! um passo que manda tentar dirigir o boneco assado, e diz o que tem de
//! acontecer. Uma cena que a escondesse trocaria uma decisão de desenho por um
//! bug reportado.
//!
//! # ⚠️ E o alcance do bake é o que o BOTÃO diz
//!
//! Sem loop armado, a janela vai de zero até a extensão do documento (ou 5 s
//! numa cena fresca), e o número aparece **no botão**. Por isso o roteiro pede
//! uma corrida de poucos segundos: o que ficar fora do alcance é simulado e
//! descartado, não gravado.

use ph2d_core::Vec2;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

impl App {
    /// **A corrida vira animação** — jogar, assar, e assistir ao que se jogou.
    pub(crate) fn physics_smoke_bake_run(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Um chão com um DEGRAU e um VÃO: o suficiente para a corrida ter forma,
        // e curto o bastante para caber no alcance default do bake.
        slab(
            world,
            "Ground A",
            Vec2::new(2.0, -0.5),
            [8.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            "Step",
            Vec2::new(12.0, 0.0),
            [3.0, 0.5],
            0.0,
            [0.38, 0.36, 0.44, 1.0],
        );
        // O vão: largo o bastante para exigir um pulo, estreito o bastante para
        // um pulo default o vencer.
        slab(
            world,
            "Ground B",
            Vec2::new(22.0, 0.0),
            [5.0, 0.5],
            0.0,
            [0.32, 0.42, 0.36, 1.0],
        );
        slab(
            world,
            "Backstop",
            Vec2::new(28.0, 2.0),
            [0.5, 2.0],
            0.0,
            [0.30, 0.34, 0.42, 1.0],
        );

        spawn_player(world, Vec2::new(-4.0, 1.4));
        eprintln!("{BAKE_RUN_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 95 — o gesto é JOGAR e depois ASSAR, e o passo 6 é a
/// contradição.
pub(crate) const BAKE_RUN_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 95] A CORRIDA VIRA ANIMACAO (W16). Um degrau, um vao e um\n",
    "encosto -- percurso curto de proposito, para caber no alcance do bake.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. JOGUE por uns 3 segundos: ande para a direita, suba o degrau, pule o\n",
    "    vao. E' esta corrida que vai ser assada -- faca uma que se reconheca.\n",
    " 3. Pause. Selecione o personagem na Hierarquia.\n",
    " 4. No Inspector, secao Physics Body, aperte o botao Bake (ele diz o\n",
    "    alcance: 'Bake 5.0s to Timeline'). O toast confirma 1 corpo.\n",
    " 5. DESMARQUE Physics no transporte e de Play do inicio: o personagem\n",
    "    REFAZ a corrida que voce deu -- o mesmo caminho, o mesmo pulo.\n",
    "    ⚠️ Se ele ficar parado, ou andar para outro lado, o bake gravou o dedo\n",
    "    do instante do clique em vez da corrida. E' o defeito que a wave\n",
    "    remove, e o numero dele: 8.765 m contra -8.765 m.\n",
    " 6. ⚠️ AGORA TENTE DIRIGI-LO com as setas: NADA acontece, e isso esta'\n",
    "    CERTO. Assar virou o corpo Kinematic, e a lei do player nao dirige um\n",
    "    corpo de massa infinita -- ele deixou de ser um personagem e passou a\n",
    "    ser animacao. E' o que assar SIGNIFICA.\n",
    " 7. Ctrl+Z devolve o corpo a Dynamic (as curvas ficam, num segundo passo:\n",
    "    sao duas filas de undo, e isso esta' nomeado no handoff do W4).\n",
);
