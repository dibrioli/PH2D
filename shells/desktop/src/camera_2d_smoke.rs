//! ⭐⭐⭐ **Smoke da câmera de jogo** (TOP-20 #7). `PH2D_GAME_CAMERA_SMOKE=1`.
//!
//! # O que esta cena prova
//!
//! Até 2026-09-09 a câmera deste app era um **recurso do editor**: ela enquadrava o que o artista
//! edita, e nada na cena podia dizer *«a câmera segue-me»*. O levantamento chama a isto *«a maior
//! lacuna do PH2D e de metade da indústria»*.
//!
//! ```text
//!   Heroi (o quadrado amarelo) ──── SETAS do teclado ────► a câmera segue
//!   Camera (segue o Heroi)  ·  amortecimento 5  ·  janela morta 0,25
//!   Cerca (os quatro cantos vermelhos) ─────────► a JANELA pára neles, não o centro
//!   Postes (de 3 em 3 m) ───────────────────────► é por eles que o movimento se vê
//! ```
//!
//! # ⛔⛔ Porque o herói anda pelo TECLADO e não pelo rato — o report de 2026-09-09
//!
//! A primeira redacção desta cena mandava **arrastar** o herói, e o dono devolveu: *«arrastar o
//! herói provoca umas travadas no movimento dele porque o rato sai de cima do player»*. Ele tem
//! razão, e não é um defeito da câmera: **é um LAÇO**.
//!
//! Um arrasto de canvas ancora o objecto na posição de MUNDO debaixo do cursor, e essa posição é
//! derivada da câmera. Com uma câmera que segue o objecto, a cadeia fecha-se sobre si mesma:
//!
//! ```text
//!   rato parado → a câmera ainda vem a caminho → o mundo debaixo do cursor MUDA
//!               → o objecto «move-se» sem ninguém lhe tocar → a câmera segue de novo → …
//! ```
//!
//! Ele converge (o amortecimento é `< 1`), mas o caminho até lá é a trepidação que o dono viu.
//! ⛔ **Não há afinação que o cure** — a realimentação é da geometria, não de um número.
//!
//! ⭐ **A cura é a FONTE DO MOVIMENTO.** Num jogo, o sujeito de uma câmera nunca é arrastado pelo
//! ponteiro: ele anda por **entrada, em metros por segundo**, e a câmera segue. Trocada a fonte, o
//! laço deixa de existir — o herói passa a depender só do teclado e a câmera só do herói.
//! *Uma cena de smoke tem de encenar o que o produto faz, senão ela mede um caminho que ninguém
//! percorre.*
//!
//! ⚠️ **Arrastar o herói continua a funcionar** (é o canvas de sempre) e continua a trepidar
//! enquanto a pré-visualização estiver ligada. Isso é **declarado**, não um defeito por corrigir:
//! é o que qualquer editor faz ao arrastar dentro de uma vista que se move sozinha.
//!
//! # ⚠️ O que provar
//!
//! - **A janela morta:** dar um toque numa seta **não** mexe a câmera. Ela só começa a seguir
//!   quando o herói passa de ~¼ do ecrã do centro. *É isto que faz um plataforma não enjoar.*
//! - **O amortecimento:** ao largar a seta, a câmera **assenta** — ela não pára a seco.
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
use ph2d_render::{Sprite, WHITE_TILE_KEY};

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
                        WHITE_TILE_KEY,
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
                    Sprite::atlas(WHITE_TILE_KEY, [1.2, 1.2], [0.90, 0.25, 0.25, 1.0]),
                    Name::new(format!("Cerca {i}")),
                ));
            }

            // **O HERÓI** — o que as setas movem. Ver o doc do módulo sobre porque não é o rato.
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [1.4, 1.4], [0.95, 0.85, 0.20, 1.0]),
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
            "[camera-2d-smoke] a vista e' da CAMERA DA CENA · use as SETAS (ou A/D/Z/S) para mover \
             o «Heroi» amarelo · a camera so' segue depois de um quarto de ecra' (janela morta) e \
             ASSENTA ao largar · leve-o para la' dos cantos VERMELHOS e a JANELA para' neles"
        );
    }
}

/// **O herói anda com o dedo do jogador**, em metros por segundo.
///
/// ⚠️ **Ela corre ANTES do passe da câmera**, e a ordem é a mesma lei que aquele passe já honra: a
/// câmera segue o mundo **deste** quadro. Ao contrário, ela enquadraria a posição do quadro
/// anterior — e com um herói a `8 m/s` isso lê-se como *«a câmera atrasa»*.
///
/// ⚠️ **Função LIVRE e não método**, e não é estilo: no sítio onde ela corre a `AppGfx` já está
/// desmontada em empréstimos por campo, e um `&mut self` emprestaria a `App` uma segunda vez.
/// *Receber o mundo é o que a torna chamável de onde ela precisa de correr.*
///
/// ⚠️ **`Transform` É componente registado**, então isto escreve documento. ⛔ Não é problema
/// **nesta cena** — é um smoke, e cada tecla é um gesto do artista como qualquer arrasto —, mas um
/// personagem de jogo a sério move-se pelo solver da física, que já tem a separação
/// `preview_drive` para isto. *A cena encena o movimento; ela não é o modelo dele.*
pub(crate) fn drive_smoke_hero(
    sim: &mut ph2d_ecs::SimWorld,
    input: ph2d_physics_ecs::PlayerInput,
    dt: f32,
) {
    // ⚠️ **`jump`/`down` valem por CIMA e BAIXO aqui**, e é uma escolha da cena: o mapa de omissão
    // já os tem nas setas (`↑/Z` e `↓/S`), então o dono não precisa de ligar nada para provar a
    // cerca no eixo Y. ⛔ Não é o que aquelas acções significam num plataforma.
    let dy = f32::from(u8::from(input.jump)) - f32::from(u8::from(input.down));
    if input.drive == 0.0 && dy == 0.0 {
        return;
    }
    let world = sim.world_mut();
    let id = ph2d_ecs::StableId(ph2d_ecs::stable_id_for_name(world, "Heroi"));
    let Some(heroi) = ph2d_ecs::entity_of_stable_id(world, id) else {
        return;
    };
    if let Some(mut t) = world.get_mut::<ph2d_ecs::Transform>(heroi) {
        t.translation.x += input.drive * HEROI_M_POR_S * dt;
        t.translation.y += dy * HEROI_M_POR_S * dt;
    }
}

/// A velocidade do herói, em metros por segundo.
///
/// ⚠️ **Ela sai da CENA, não do gosto**: a cerca tem `60 m` de lado, e a `8 m/s` atravessá-la leva
/// `7,5 s` — devagar o bastante para a janela morta e o amortecimento se verem, depressa o bastante
/// para o dono chegar à cerca sem se aborrecer. *Um número de smoke também tem de dizer de onde é.*
const HEROI_M_POR_S: f32 = 8.0; // LITERAL-PX-OK: metros por segundo
