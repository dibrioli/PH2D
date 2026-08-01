//! **A CHAVE E A FENDA** (`PH2D_PHYSICS_SMOKE=70`, W-PartFace).
//!
//! A cena 69 mostra que uma PEÇA existe. Esta mostra que ela é **editável**, e o
//! oráculo é binário: uma chave cujo palhetão é largo demais **ENTALA** na fenda;
//! afinar o palhetão — que é uma peça, e até esta wave não tinha nenhum campo no
//! Inspector — a faz **PASSAR**.
//!
//! ⚠️ **A rotação é travada nas duas chaves** (`LockRotation`), e não é enfeite:
//! sem ela um corpo rígido pode girar 90° e escorregar de lado pela fenda, e o
//! resultado deixaria de medir a LARGURA que o artista edita.
//!
//! Os números da mensagem saem da sonda `probe_smoke_70`.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, LockRotation, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Meia-largura da FENDA. A chave passa se o palhetão couber nela.
const SLOT_HALF: f32 = 0.40;
/// Onde cada fenda está, em `x`.
pub(crate) const LANES: [f32; 2] = [-2.0, 2.0];
/// O muro: topo em `y = 1.2`, centro em `1.0`.
const WALL_Y: f32 = 1.0;
const WALL_HALF_Y: f32 = 0.2;

/// O cabo (o CORPO) — estreito o bastante para passar em qualquer fenda.
const HANDLE_HALF: [f32; 2] = [0.15, 0.5];
/// A guarda: a segunda PEÇA, pequena e inofensiva. Ela existe para o readout do
/// dono dizer *"+ 2 more shapes from children"* e para provar que as peças são
/// editáveis **uma a uma**.
const GUARD_HALF: [f32; 2] = [0.30, 0.10];
/// O palhetão de cada chave — a ÚNICA diferença entre as duas faixas.
const BIT_HALF_X: [f32; 2] = [0.62, 0.22];
const BIT_HALF_Y: f32 = 0.15;

const DROP_Y: f32 = 4.2;

const STEEL: [f32; 4] = [0.72, 0.74, 0.80, 1.0];
const BIT_STUCK: [f32; 4] = [0.85, 0.40, 0.35, 1.0];
const BIT_FREE: [f32; 4] = [0.40, 0.75, 0.50, 1.0];
const GUARD_RGBA: [f32; 4] = [0.55, 0.50, 0.70, 1.0];
const WALL_RGBA: [f32; 4] = [0.34, 0.36, 0.42, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, 1.6];
const CAMERA_HEIGHT: f32 = 9.0;

/// **MEDIDO** pela sonda (`probe_smoke_70`): onde o CABO de cada chave descansa
/// depois de 5 s.
///
/// A da esquerda para **em cima do muro** (topo em `1,20`) porque o palhetão não
/// cabe na fenda; a da direita atravessa e pousa no chão (topo em `−0,80`). Os
/// palhetões ficam em `1,349` e `−0,651`.
pub(crate) const MEASURED_HANDLE_Y: [f32; 2] = [2.00, 0.00];

/// Um trecho de muro, de `x0` a `x1`.
fn wall(world: &mut World, x0: f32, x1: f32) {
    let half_x = (x1 - x0) * 0.5;
    world.spawn((
        Name::new(format!("Wall {x0:+.1}")),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x,
                half_y: WALL_HALF_Y,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [half_x * 2.0, WALL_HALF_Y * 2.0], WALL_RGBA),
        Transform::from_translation(Vec2::new((x0 + x1) * 0.5, WALL_Y)),
    ));
}

/// Uma chave: cabo (corpo) + palhetão e guarda (peças).
fn key(world: &mut World, i: usize, name: &str) -> Entity {
    let x = LANES[i];
    let bit_half_x = BIT_HALF_X[i];
    let handle = world
        .spawn((
            Name::new(format!("{name} Handle")),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: HANDLE_HALF[0],
                    half_y: HANDLE_HALF[1],
                },
                ..Collider::default()
            },
            // ⚠️ Sem isto a chave gira e escorrega de lado pela fenda — o
            // resultado deixaria de medir a LARGURA que o artista edita.
            LockRotation,
            Sprite::atlas(
                WHITE_TILE_KEY,
                [HANDLE_HALF[0] * 2.0, HANDLE_HALF[1] * 2.0],
                STEEL,
            ),
            Transform::from_translation(Vec2::new(x, DROP_Y)),
        ))
        .id();
    // O PALHETÃO — a peça que decide tudo.
    world.spawn((
        Name::new(format!("{name} Bit")),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: bit_half_x,
                half_y: BIT_HALF_Y,
            },
            ..Collider::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [bit_half_x * 2.0, BIT_HALF_Y * 2.0],
            if bit_half_x > SLOT_HALF {
                BIT_STUCK
            } else {
                BIT_FREE
            },
        ),
        Transform::from_translation(Vec2::new(0.0, -(HANDLE_HALF[1] + BIT_HALF_Y))),
        ChildOf(handle),
    ));
    // A GUARDA — a segunda peça, no topo, fora do caminho da fenda.
    world.spawn((
        Name::new(format!("{name} Guard")),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: GUARD_HALF[0],
                half_y: GUARD_HALF[1],
            },
            ..Collider::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [GUARD_HALF[0] * 2.0, GUARD_HALF[1] * 2.0],
            GUARD_RGBA,
        ),
        Transform::from_translation(Vec2::new(0.0, HANDLE_HALF[1] + GUARD_HALF[1])),
        ChildOf(handle),
    ));
    handle
}

pub(crate) fn build_keys(world: &mut World) {
    for lane in LANES {
        // Duas fendas: uma por faixa, cada uma com `SLOT_HALF` de meia-largura.
        wall(world, lane - 1.8, lane - SLOT_HALF);
        wall(world, lane + SLOT_HALF, lane + 1.8);
    }
    key(world, 0, "Wide");
    key(world, 1, "Slim");
}

#[cfg(test)]
#[path = "physics_smoke_part_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 70 (W-PartFace).** A chave entala; afinar a PEÇA a faz passar.
    pub(crate) fn physics_smoke_part(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_keys(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 70] A CHAVE E A FENDA -- uma PECA e mais uma forma do corpo\n  \
               ancestral (um filho com `Collider` e SEM `RigidBody`). A wave anterior deu\n  \
               o gesto de CRIAR uma; esta da a volta que faltava: EDITA-LA.\n\n  \
               Ate agora, selecionar uma peca abria a face VAZIA do painel, que dizia\n  \
               \"Not simulated\" -- o oposto da verdade -- e mostrava SEMENTES (caixa\n  \
               0,50 x 0,50, offset 0, densidade 1,00) em vez da forma autorada. E a porta\n  \
               que a criou seguia oferecida: clica-la reescrevia o collider com os\n  \
               defaults, EM SILENCIO.\n\n  \
               Duas chaves IDENTICAS, cada uma com um cabo (o CORPO) e duas pecas: o\n  \
               palhetao embaixo e a guarda em cima. So a LARGURA do palhetao difere.\n  \
               A fenda tem {slot:.2} m de meia-largura.\n\n  \
               1. Toque Play.\n     \
                  - ESQUERDA (palhetao VERMELHO, meia-largura {w0:.2}) -- largo demais:\n       \
                    ENTALA no muro, cabo parado em {h0:.2} m.\n     \
                  - DIREITA (palhetao VERDE, meia-largura {w1:.2}) -- passa limpo e cai\n       \
                    no chao: cabo em {h1:.2} m.\n\n  \
               2. A ENTREGA DA WAVE. Pause, toque Reset, e selecione 'Wide Bit' na\n     \
                  Hierarquia. A secao Physics Body abre a TERCEIRA face:\n     \
                  - o cabecalho diz \"Shape of Wide Handle -- simulated as part of it\",\n       \
                    e NAO \"Not simulated\";\n     \
                  - Collider / Half Width / Half Height / Offset / Density / Bounce /\n       \
                    Friction / Layer / Trigger / One-Way sao exatamente as propriedades\n       \
                    que a PONTE le de uma peca -- nem uma a mais;\n     \
                  - nao ha Gravity, Mass, CCD, Freeze nem Bake: sao do CORPO, e o solver\n       \
                    os ignora numa peca.\n     \
                  Mude **Half Width** para {w1:.2} e toque Play. A chave PASSA.\n     \
                  Antes desta wave esse campo nao existia -- e, se existisse, o valor\n     \
                  digitado nao chegava ao ECS.\n\n  \
               3. O SEGUNDO CAMINHO. Reset, selecione 'Wide Bit' e aperte **Remove\n     \
                  Shape**. O palhetao deixa de existir para o solver (o desenho fica) e\n     \
                  o cabo passa. Ate esta wave uma peca era porta de MAO UNICA: criada\n     \
                  por um clique e desfeita so apagando o objeto.\n\n  \
               4. O TERCEIRO. Reset, selecione 'Wide Bit' e aperte **Make Independent\n     \
                  Body**. Ela deixa de ser peca e vira um corpo PROPRIO: agora sao duas\n     \
                  massas que o solver pode separar, e a chave se desmonta.\n\n  \
               5. O READOUT. Selecione 'Wide Handle' (o CORPO). Abaixo das dimensoes\n     \
                  DELE aparece \"+ 2 more shapes from children\". Sem essa linha, com o\n     \
                  contorno desligado nada distinguia um corpo de forma unica de um que\n     \
                  carrega mais duas -- uma peca era invisivel dos DOIS lados.\n\n  \
               6. A RECUSA. Em 'Wide Bit' NAO ha mais 'Add Shape to ...'. Ela e a porta\n     \
                  da face VAZIA; clica-la sobre algo que ja tem forma so apagava o que o\n     \
                  artista afinou (medido: a barra virava a caixa do sprite, com\n       \
                  offset, densidade e camada zerados).\n\n  \
               === Os DOIS defeitos do smoke anterior ===\n  \
               7. A MAO. Toque Play e, com a ferramenta de interacao em Hand, ARRASTE\n     \
                  o palhetao (a peca) com o mouse. A chave inteira vem junto.\n     \
                  Antes: a mao procurava um corpo que a peca nao tem, recusava em\n     \
                  silencio, e o press caia adiante para o GIZMO.\n  \
               8. E era o gizmo que produzia a penetracao: com o relogio ANDANDO, o\n     \
                  re-describe da peca era gateado em REPOUSO, entao arrastar movia o\n     \
                  DESENHO e deixava o collider onde estava -- a forma atravessava o\n     \
                  estatico, sem erro e sem warning. Agora, com o Play rodando,\n     \
                  selecione 'Wide Bit' e mude Half Width: a fenda responde NA HORA.\n\n  \
               (!) Toque B para o contorno: as pecas sao desenhadas na cor do DONO,\n     \
                   porque e o corpo dele que as governa. Com o Play rodando, o\n     \
                   contorno da peca tem de acompanhar o desenho dela em toda edicao.\n",
            slot = SLOT_HALF,
            w0 = BIT_HALF_X[0],
            w1 = BIT_HALF_X[1],
            h0 = MEASURED_HANDLE_Y[0],
            h1 = MEASURED_HANDLE_Y[1],
        );
    }
}
