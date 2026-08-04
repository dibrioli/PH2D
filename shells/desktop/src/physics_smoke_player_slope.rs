//! **A cena 88 — A LADEIRA** (W9), irmã de `physics_smoke_player.rs`.
//!
//! Cortada por CENA, o precedente do módulo. A pergunta aqui é uma só: **o
//! número escrito em *Max Slope* é o ângulo que o personagem de fato sobe?**
//!
//! # ⚠️ Por que uma cena NOVA, e não a rampa da `=81`
//!
//! Duas razões, e as duas são medidas (`measure_walk_scene`, na
//! `ph2d-physics-ecs`).
//!
//! **(1) A `=81` não é julgável a pé.** As duas rampas dela sobem *para longe*
//! do chão: o personagem passa POR BAIXO delas, cai da beirada do piso em
//! `x = ±10` e despenca — a sonda o mede em `y = −162 m` seis segundos depois,
//! sem ter tocado rampa nenhuma. O roteiro daquela cena mandava *"vá até a rampa
//! de 60°"* e não havia como chegar lá.
//!
//! **(2) A fixture tem de CONTER o fenômeno.** A `=81` mede 60°, e 60° já era
//! recusado mesmo com o defeito. O par que decide é o que **cerca** o limite:
//! **40°** (um degrau abaixo — sobe) e **50°** (um degrau acima — escorregava
//! `+4,0 m` em 3 s, e agora escorrega).
//!
//! A geometria abaixo não foi desenhada de cabeça: cada centro e cada rotação
//! saiu da sonda, que caminha nela e imprime onde o personagem para.

use ph2d_core::Vec2;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

impl App {
    /// **A ladeira** — 40° de um lado, 50° do outro, o limite autorado no meio.
    pub(crate) fn physics_smoke_slope(&mut self) {
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
        // ⚠️ **As duas paredes não são cenário: são o que impede a cena de
        // terminar num personagem caindo para sempre**, que é metade do que
        // torna a `=81` injulgável.
        for (name, x) in [("WallL", -16.5), ("WallR", 16.5)] {
            slab(
                world,
                name,
                Vec2::new(x, 2.0),
                [0.5, 2.5],
                0.0,
                [0.30, 0.30, 0.34, 1.0],
            );
        }

        // ── ESQUERDA: 40°, um grau abaixo do limite. SOBE. ───────────────────
        // ⚠️ O SINAL da rotação decide de que lado a rampa sobe, e é ele que a
        // `=81` errou. Negativo aqui: ela sobe indo para a ESQUERDA, que é o
        // lado de onde o personagem chega.
        slab(
            world,
            "Ramp40",
            Vec2::new(-6.0, 1.1),
            [3.0, 0.5],
            -40.0_f32.to_radians(),
            [0.30, 0.50, 0.35, 1.0],
        );
        // O patamar no alto — subir tem de levar a algum lugar. Ele encosta na
        // ponta da rampa (`x = −8`) e na parede (`x = −16`), então o topo é um
        // chão de verdade e não uma beirada.
        slab(
            world,
            "Plateau",
            Vec2::new(-12.0, 3.0),
            [4.0, 0.41],
            0.0,
            [0.32, 0.44, 0.36, 1.0],
        );

        // ── DIREITA: 50°, um grau acima do limite. ESCORREGA. ────────────────
        slab(
            world,
            "Ramp50",
            Vec2::new(5.0, 1.2),
            [3.0, 0.5],
            50.0_f32.to_radians(),
            [0.55, 0.32, 0.30, 1.0],
        );

        spawn_player(world, Vec2::new(0.0, 2.0));

        eprintln!("{SLOPE_SMOKE_MESSAGE}");
    }
}

/// O roteiro da cena 88 — os dois lados são o mesmo número visto por fora.
pub(crate) const SLOPE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 88] A LADEIRA (W9). Um chao entre duas paredes, uma rampa de\n",
    "40deg a ESQUERDA (com patamar no alto) e uma de 50deg a DIREITA.\n",
    "O limite autorado do personagem e' 45deg -- ele fica ENTRE as duas.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA CIMA (ou Z) pula.\n",
    "(O Espaco NAO pula -- ele e' o Play/Pause do transporte.)\n",
    "\n",
    "1. Marque **Physics** na barra de transporte e de' Play.\n",
    "2. Va' para a ESQUERDA: a rampa de 40deg e' SUBIDA ate' o patamar.\n",
    "3. Volte e va' para a DIREITA: a de 50deg NAO sobe. Segure a tecla contra\n",
    "   ela o quanto quiser -- o personagem encosta e escorrega de volta.\n",
    "   >>> Era ESTE o defeito: com o limite em 45 ele subia a de 50 assim\n",
    "       mesmo, porque o empurrao do modo-ar e' horizontal e a rampa o\n",
    "       redirecionava morro acima. O numero da caixa nao era o numero que\n",
    "       o produto honrava.\n",
    "4. O CONTROLE -- selecione o personagem e suba **Max Slope** para 55.\n",
    "   >>> Agora a de 50deg sobe, e sobe na hora. Baixe para 35 e a de 40deg\n",
    "       deixa de subir. E' o mesmo numero governando os dois lados.\n",
    "5. Baixe **Air Acceleration** para 0 e repita o passo 3.\n",
    "   >>> Nada muda. Antes desta wave o teto de subida era funcao DESTE\n",
    "       knob -- com ele em 20 subia-se 50deg, com ele em 5 grudava-se, com\n",
    "       ele em 0 escorregava-se. Um limite que muda quando voce mexe em\n",
    "       outro controle nao e' um limite.\n",
);
