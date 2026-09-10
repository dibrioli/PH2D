//! ⭐⭐⭐ **Smoke da câmera de jogo** (TOP-20 #7). `PH2D_GAME_CAMERA_SMOKE=1`.
//!
//! # O que esta cena prova
//!
//! Até 2026-09-09 a câmera deste app era um **recurso do editor**: ela enquadrava o que o artista
//! edita, e nada na cena podia dizer *«a câmera segue-me»*. O levantamento chama a isto *«a maior
//! lacuna do PH2D e de metade da indústria»*.
//!
//! ```text
//!   Heroi (o quadrado amarelo) ──── ARRASTE-O ────► a câmera segue
//!   Camera (segue o Heroi)  ·  amortecimento 5  ·  janela morta 0,25
//!   Cerca (os quatro cantos vermelhos) ─────────► a JANELA pára neles, não o centro
//!   Postes (de 3 em 3 m) ───────────────────────► é por eles que o movimento se vê
//! ```
//!
//! # ⚠️ O que provar
//!
//! - **A janela morta:** arrastar o Heroi um bocadinho **não** mexe a câmera. Ela só começa a
//!   seguir quando ele passa de ~¼ do ecrã do centro. *É isto que faz um plataforma não enjoar.*
//! - **O amortecimento:** ao largar, a câmera **assenta** — ela não salta para o Heroi.
//! - ⭐⭐ **A CERCA prende a JANELA, não o centro:** leve o Heroi para lá dos cantos vermelhos. A
//!   câmera pára com o canto **na borda do ecrã** — nunca com o canto no meio dele. É a diferença
//!   que o levantamento nomeia como a entrega do P0, e a que todo jogo escreve à mão.
//! - ⚠️ **O nascimento não viaja:** ao abrir, a câmera já está no Heroi. Ela não vem da origem.
//!
//! ⚠️ **Enquanto a pré-visualização está ligada, a câmera da cena é dona do pan E do zoom** — a
//! roda e o arrasto de vista ficam por conta dela. É o que uma câmera de jogo é.
//!
//! ⏳ **O que esta wave NÃO entrega, e é nomeado:** o botão que liga e desliga a pré-visualização
//! vive na secção do Inspector, que é a wave seguinte (W3). Aqui ela é ligada pela cena.
//!
//! ⚠️ Se a linha `[camera-2d-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{CameraFollow, CameraLimits, GameCamera, Name, Transform};
use ph2d_render::Sprite;

/// A cerca da fase, em metros. ⚠️ **Larga de propósito**: com a altura de `10 m` e um ecrã
/// panorâmico a meia-janela passa de `8 m`, então uma cerca apertada prenderia a câmera antes de
/// ela chegar a mover-se — e a cena ensinaria que a cerca é que está partida.
const CERCA: f32 = 30.0; // LITERAL-PX-OK: metros
const CERCA_Y: f32 = 12.0; // LITERAL-PX-OK: metros

impl crate::App {
    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn game_camera_smoke(&mut self) {
        if self.game_camera_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_GAME_CAMERA_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda não há mundo; tenta no quadro seguinte
        }
        self.game_camera_smoke_done = true;

        {
            let gfx = self.gfx.as_mut().expect("gfx");
            let world = gfx.sim.world_mut();

            // **OS POSTES** — sem eles o ecrã é liso e o movimento da câmera é invisível.
            let mut n = 0;
            let mut x = -CERCA - 6.0;
            while x <= CERCA + 6.0 {
                let dentro = x.abs() <= CERCA;
                world.spawn((
                    Transform::from_translation(Vec2::new(x, -4.0)),
                    Sprite::atlas(
                        0,
                        [0.4, 2.0],
                        if dentro {
                            [0.30, 0.32, 0.38, 1.0]
                        } else {
                            // ⚠️ Os de FORA da cerca são mais escuros — é como se vê que a câmera
                            // parou de os alcançar, em vez de se concluir que ela travou.
                            [0.16, 0.16, 0.20, 1.0]
                        },
                    ),
                    Name::new(format!("Poste {n}")),
                ));
                n += 1;
                x += 3.0;
            }

            // **OS CANTOS DA CERCA** — a cerca só é uma promessa se ela se vir.
            for (i, (sx, sy)) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
                .into_iter()
                .enumerate()
            {
                world.spawn((
                    Transform::from_translation(Vec2::new(CERCA * sx, CERCA_Y * sy)),
                    Sprite::atlas(0, [1.2, 1.2], [0.90, 0.25, 0.25, 1.0]),
                    Name::new(format!("Cerca {i}")),
                ));
            }

            // **O HERÓI** — o que o dono arrasta.
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Sprite::atlas(0, [1.4, 1.4], [0.95, 0.85, 0.20, 1.0]),
                Name::new("Heroi"),
            ));

            // **A CÂMERA.** ⚠️ Ela nasce LONGE do herói de propósito: se o nascimento amortecesse,
            // a cena abriria com a vista a viajar de `(20, 8)` até `(0, 0)` — e é exactamente esse
            // o defeito que o `settled` recusa. Ao abrir, tem de estar já no herói.
            world.spawn((
                Transform::from_translation(Vec2::new(20.0, 8.0)),
                Name::new("Camera"),
                GameCamera::default(),
                CameraFollow {
                    target: "Heroi".into(),
                    damping: [5.0, 5.0], // LITERAL-PX-OK: 1/s, o default medido do oráculo
                    // ⭐ **A janela morta é o que se vem cá ver**: um quarto da meia-janela.
                    dead_zone: [0.25, 0.25], // LITERAL-PX-OK: fracção da meia-janela
                    lookahead: [0.0, 0.0],
                    offset: [0.0, 0.0],
                },
                CameraLimits {
                    min: [-CERCA, -CERCA_Y],
                    max: [CERCA, CERCA_Y],
                },
            ));
        }

        // ⚠️ **A cena LIGA a pré-visualização**, senão ela montaria uma câmera correcta que não se
        // vê — que é o defeito de *«um motor vivo sem botão nenhum»* que o pincel de tecido pagou.
        self.game_camera_preview = true;

        eprintln!(
            "[camera-2d-smoke] a vista e' da CAMERA DA CENA · ARRASTE o «Heroi» amarelo · a camera \
             so' segue depois de um quarto de ecra' (janela morta) e ASSENTA ao largar · leve-o \
             para la' dos cantos VERMELHOS e a JANELA para' neles"
        );
    }
}
