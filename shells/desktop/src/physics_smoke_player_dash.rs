//! **A cena 93 — O SALTO SOBRE O ABISMO** (W14), irmã de
//! `physics_smoke_player_wall.rs`.
//!
//! Um buraco largo demais para um pulo, e uma prateleira alta demais para se
//! alcançar andando. O gesto que a cena existe para julgar é **o arranque como
//! ferramenta de travessia**: correr, pular, e arrancar no ar.
//!
//! # ⚠️ A largura do abismo é ESCOLHIDA pela medição, não desenhada de cabeça
//!
//! Um pulo default atravessa, na horizontal, o que a caminhada de 6 m/s cobre
//! durante o voo (~1,45 s) — da ordem de **8 m**, e portanto qualquer buraco que
//! um pulo vença sozinho **não mede o arranque**. O abismo tem **11 m**: com o
//! arranque autorado (18 m/s por 0,15 s = **2,7 m**) o pulo passa a alcançar
//! ~10,7 m de avanço útil, e o outro lado fica ao alcance de quem arranca **no
//! ar, a meio do voo** — e fora do alcance de quem não arranca.
//!
//! Um buraco de 6 m seria atravessado sem o jogador apertar nada, e a cena
//! mostraria uma capacidade que não é a que a wave entrega.
//!
//! # ⚠️ E a cena arma a capacidade EXPLICITAMENTE
//!
//! O `PlatformPlayer` nasce com o arranque **desligado** (`dash_speed` em zero)
//! — arranque é uma capacidade opt-in do personagem. Uma cena que herdasse o
//! default mostraria um abismo em que nada acontece, e leria como *"o arranque
//! está quebrado"*.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O vão livre do abismo — ver o aviso do módulo.
const GAP: f32 = 11.0;
/// Onde a plataforma de partida acaba.
const LEDGE_X: f32 = 6.0;

impl App {
    /// **O salto sobre o abismo** — atravessar um buraco que um pulo não vence.
    pub(crate) fn physics_smoke_dash(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // A plataforma de partida: espaço para ganhar velocidade antes da borda.
        slab(
            world,
            "Launch",
            Vec2::new(LEDGE_X - 8.0, -0.5),
            [8.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        // A do outro lado, mais comprida, para o pouso ser generoso.
        slab(
            world,
            "Landing",
            Vec2::new(LEDGE_X + GAP + 8.0, -0.5),
            [8.0, 0.5],
            0.0,
            [0.32, 0.44, 0.36, 1.0],
        );
        // ⚠️ Uma rede lá em baixo, e não o vazio: quem falha o salto tem de poder
        // voltar a tentar sem reiniciar a cena. Um smoke que exige um restart por
        // tentativa é um smoke que ninguém repete.
        slab(
            world,
            "Net",
            Vec2::new(LEDGE_X + GAP * 0.5, -9.0),
            [22.0, 0.5],
            0.0,
            [0.30, 0.30, 0.34, 1.0],
        );
        // Uma parede baixa no fim, para o personagem não sair de quadro a correr.
        slab(
            world,
            "Backstop",
            Vec2::new(LEDGE_X + GAP + 16.5, 1.5),
            [0.5, 2.0],
            0.0,
            [0.30, 0.34, 0.42, 1.0],
        );

        spawn_player(world, Vec2::new(LEDGE_X - 13.0, 1.4));

        // ⚠️ **A capacidade é ARMADA aqui**, e não herdada (ver o aviso do
        // módulo).
        let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
        for (mut p, _) in q.iter_mut(world) {
            p.dash_speed = 18.0;
            p.dash_time = 0.15;
            p.dash_cooldown = 0.2;
        }
        eprintln!("{DASH_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 93 — o gesto é ARRANCAR NO AR, e o que se julga é
/// atravessar um buraco que um pulo não vence.
pub(crate) const DASH_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 93] O SALTO SOBRE O ABISMO (W14). Duas plataformas com\n",
    "11 m de vao, uma rede la' embaixo e um encosto no fim.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "          Q arranca.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. NO CHAO, aperte Q correndo: ele dispara em linha RETA, rapido, e\n",
    "    volta a andar sozinho. Nao pode subir nem afundar no chao.\n",
    " 3. Tente atravessar o abismo SO' com o pulo: ele nao chega. E' o\n",
    "    controle -- se chegar, o vao esta' curto demais e o resto nao diz nada.\n",
    " 4. Agora corra, pule, e aperte Q A MEIO DO VOO: ele atravessa.\n",
    "    ⚠️ No arranque ele NAO cai -- a linha e' reta, nao uma curva.\n",
    " 5. Ainda no ar, aperte Q outra vez: NADA acontece. E' um arranque por\n",
    "    voo, e quem o devolve e' o chao.\n",
    " 6. Caia na rede, ande ate' a borda e repita: o arranque esta' de volta.\n",
    " 7. No chao, martele Q: ele nao encadeia -- ha' uma recuperacao entre\n",
    "    dois arranques.\n",
    " 8. Solte a direcao e aperte Q parado: ele arranca para o lado para onde\n",
    "    estava virado. Ande para a esquerda, pare, e aperte: vai para a\n",
    "    esquerda.\n",
);
