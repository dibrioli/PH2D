//! ⭐⭐⭐ **Smoke do ABANÃO DA VISTA** (suplente #25). `PH2D_SHAKE_SMOKE=1`.
//!
//! # A cena: **a mesma bomba, e a única variável é ONDE ESTÁ QUEM VÊ**
//!
//! Um pátio de postes, um herói que anda com as setas, e **uma** bomba que explode ao `Q`. A câmera
//! do jogo segue o herói ⇒ afastar-se da bomba é afastar a VISTA dela, e o mesmo estrondo abana
//! menos. *O controlo não é um segundo objecto: é a mesma bomba a três distâncias.*
//!
//! ⚠️⚠️ **O PÁTIO DE POSTES não é decoração, e é o que torna a wave visível:** um abanão é um
//! deslocamento da vista, e sobre um chão de cor CHAPADA ele é **rigorosamente invisível** — nada
//! na tela muda de sítio. *Uma cena que não deixa ver o que ensina é a espécie que o `CLAUDE.md`
//! §5.0 chama de pior que uma cena ausente*, e há gate a contar os postes.
//!
//! # ⛔ E a VISTA tem de ser TOMADA pela câmera do jogo
//!
//! O abanão é um offset da [`ph2d_ecs::CameraRuntime`], que só chega ao ecrã com a
//! pré-visualização ligada. Sem esse passo o prólogo monta tudo certo e **nada treme** — o dono
//! leria *«o abanão não funciona»* sobre um motor que está a funcionar.
//!
//! # ⚠️ A tecla é a do gatilho, e é MEDIDA
//!
//! `Q` — a mesma do suplente #24, pela mesma medição (ver [`crate::trigger_smoke::TECLA`]): o
//! espaço é o Play/Pause do transporte, e o dono já devolveu uma cena que o usava.
//!
//! ⚠️ Se a linha `[shake-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ActionEdge, ActionTriggerRow, CameraFollow, CameraShake, Entity, GameCamera, Name,
    ShakeEmitter, ShakeSource, SignalFrom, SignalOnAction, Transform, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do corpo do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 1;

/// A acção do Input Map que o prólogo cria — ⚠️ **lida nos DOIS sítios pela mesma const**.
pub const ACCAO: &str = "boom";
/// O sinal que o gatilho publica e que a fonte ouve — idem.
pub const SINAL: &str = "boom";
/// A tecla, e o nome dela na tela — ver o cabeçalho.
pub const TECLA: u32 = crate::trigger_smoke::TECLA;
/// Ver [`TECLA`].
pub const TECLA_NOME: &str = crate::trigger_smoke::TECLA_NOME;

/// ⭐⭐⭐ **Quantos POSTES o pátio tem, e porque eles existem** — ver o cabeçalho.
///
/// ⚠️ **O número é MEDIDO pela vista E pelo ALCANCE, e a 1.ª redacção foi REPROVADA por um gate
/// desta crate:** com `7 × 5` a `3` m o pátio media `18` m de largura e a fonte alcançava `18` ⇒
/// *o dono não conseguia ANDAR PARA FORA do alcance sem sair do pátio*, e o passo (2) do roteiro
/// era inalcançável. A grelha é `13 × 9` a `3` m ⇒ **`36 × 24` m**, o dobro do alcance.
///
/// ⚠️⚠️ **E o PASSO é menor que METADE da altura da vista, e a FOTO é que o corrigiu:** a régua
/// media a JANELA (`11,25` m) e o que o dono vê é a **BANDA** que sobra com a timeline aberta, que é
/// ~metade dela. Com `5` m de passo a foto mostrou **uma** fileira; com `3` mostra três. *Um gate que
/// mede o enquadramento da janela aprova uma cena que o artista vê cortada ao meio.*
pub const POSTES_X: i32 = 13;
/// Ver [`POSTES_X`].
pub const POSTES_Y: i32 = 9;
/// O passo da grelha, em metros. Ver [`POSTES_X`].
pub const POSTE_PASSO: f32 = 3.0;

/// ⭐⭐⭐ **Onde o herói nasce, e a FOTO é que o decidiu** (`docs/Components/ferramentas/fotografa_cena.sh`).
///
/// ⛔⛔ **A 1.ª redacção punha-o a `−5` m da bomba e a FOTO mostrou a bomba FORA DO ECRÃ**, com a
/// suíte inteira verde: a câmera segue o herói, e a banda do canvas que sobra com a timeline aberta
/// enquadra **~6 m** — logo `5` m de distância põem o sujeito do passo (1) fora da vista. *Um passo
/// que manda olhar para um quadrado vermelho que não está na tela é a espécie que o `CLAUDE.md`
/// §5.0 chama de pior que uma cena ausente.*
///
/// ⚠️ **E ele não nasce EM CIMA dela** — a `1,5` m: em cima, a bomba fica tapada pelo herói e pelo
/// realce da selecção, e o dono não vê o que explodiu.
pub const HEROI_Y: f32 = -1.5;

const CHAO_RGBA: [f32; 4] = [0.13, 0.14, 0.17, 1.0];
const POSTE_RGBA: [f32; 4] = [0.42, 0.45, 0.52, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const BOMBA_RGBA: [f32; 4] = [0.95, 0.35, 0.30, 1.0];

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// A BOMBA, que nasce escolhida — ver [`montar`].
    pub escolhido: u64,
}

/// O pátio: um chão e a grelha de postes que torna o abanão VISÍVEL.
fn patio(world: &mut World) {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, e isto não é estilo: desde a cura de 15/09 a ordem das raízes é a
    // ordem de CRIAÇÃO, logo quem nasce primeiro desenha por baixo.
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [44.0, 32.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    for iy in 0..POSTES_Y {
        for ix in 0..POSTES_X {
            #[allow(clippy::cast_precision_loss)]
            let x = (ix - POSTES_X / 2) as f32 * POSTE_PASSO;
            #[allow(clippy::cast_precision_loss)]
            let y = (iy - POSTES_Y / 2) as f32 * POSTE_PASSO;
            world.spawn((
                Name::new(format!("Post {ix}x{iy}")),
                Sprite::atlas(WHITE_TILE_KEY, [0.45, 1.6], POSTE_RGBA),
                Transform::from_translation(Vec2::new(x, y)),
            ));
        }
    }
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO**.
fn cena_um(world: &mut World) -> Entity {
    patio(world);

    // ⭐ O HERÓI: anda com as setas, e a câmera segue-o ⇒ **andar é afastar a VISTA da bomba**.
    //
    // ⛔⛔⛔ **O CORPO CINEMÁTICO NÃO É DECORAÇÃO — sem ele o passo (2) é IMPOSSÍVEL** (report do
    // dono, 19/09: *«vc esqueceu de colocar física no jogador»*). A ponte do mover varre
    // `self.bodies`, logo **quem não tem corpo nunca entra no laço**: a 1.ª redacção desta cena
    // dava-lhe o componente e mais nada, e a seta segurada movia `0,0000 m`. As três cenas irmãs
    // que carregam este componente (`topdown_smoke` · `trigger_smoke` · `dano_smoke`) dão-lhe as
    // três peças; esta dava uma.
    //
    // ⚠️⚠️ **E `Kinematic` é LEI e não gosto:** um corpo `Dynamic` é do SOLVER
    // (`bridge::pose_owner`), logo o mover fica inerte **e** a gravidade leva-o — medido nesta
    // cena, `y = −492 m` ao fim de dez segundos, com a câmera a segui-lo. *Foi isso que o dono viu
    // como «travou»: em ~2 s não há um poste no ecrã, e nada do que ele carregue traz o pátio de
    // volta.* O Inspector diz-no em vermelho na secção *Top-Down Player* — mas o roteiro manda
    // escolher a BOMBA, logo ele nunca olha para lá.
    //
    // ⚠️ O raio é o do sprite (`0,9` de lado ⇒ `0,45`), como nas irmãs: nada nesta cena tem
    // collider, logo ele não bate em nada — ele existe para o corpo ENTRAR no mundo.
    let heroi = world
        .spawn((
            Name::new("Heroi"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.45 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.9], HEROI_RGBA),
            Transform::from_translation(Vec2::new(0.0, HEROI_Y)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 6.0,
                direction: DirectionMode::Free,
                ..TopDownLaw::default()
            }),
        ))
        .id();

    // ⭐⭐⭐ **A BOMBA:** o gatilho do #24 publica, e a fonte do #25 ouve — no MESMO objecto.
    //
    // ⚠️ **`SignalFrom::Myself` e não `Anyone`**, e não é indiferente: o sinal de um gatilho carrega
    // o SUJEITO (`SignalOrigin::Action { source }`), logo com `Myself` a distância é medida a
    // partir **desta** bomba. Com a cerca aberta uma segunda bomba ouviria o estrondo da primeira e
    // a cena deixaria de ensinar o que ensina.
    let bomba = world
        .spawn((
            Name::new("Bomba"),
            Sprite::atlas(WHITE_TILE_KEY, [1.2, 1.2], BOMBA_RGBA),
            Transform::from_translation(Vec2::ZERO),
            SignalOnAction(vec![ActionTriggerRow {
                action: ACCAO.to_owned(),
                edge: ActionEdge::Press,
                signal: SINAL.to_owned(),
            }]),
            ShakeEmitter(vec![ShakeSource {
                on: SINAL.to_owned(),
                de: SignalFrom::Myself,
                // ⚠️ **Força `1,0` e não a de fábrica:** esta cena existe para o abanão ser VISTO,
                // e o dono desce o número no painel se quiser. *Uma cena que ensina com o volume
                // baixo faz o dono duvidar da ferramenta.*
                forca: 1.0,
                dentro: 4.0,
                fora: 18.0,
            }]),
        ))
        .id();

    // ⭐⭐ **A CÂMERA DO JOGO, a seguir o herói, e é ELA que treme.**
    //
    // ⚠️ `damping` a zero de propósito: com amortecimento o dono não sabe se o que vê é o abanão ou
    // a câmera a apanhar o herói.
    world.spawn((
        Name::new("Camera"),
        Transform::from_translation(Vec2::new(0.0, HEROI_Y)),
        GameCamera::default(),
        CameraFollow {
            target: "Heroi".to_owned(),
            ..CameraFollow::default()
        },
        CameraShake::default(),
    ));

    // ⛔ **A BOMBA nasce escolhida, e é a FOTO que o exige:** o roteiro manda ver a secção *Shake
    // Emitter* no painel da direita, e com ninguém escolhido o Inspector diz *«Select an entity in
    // the Hierarchy»*. *Um passo que nomeia uma secção AFIRMA que ela está na tela.*
    let _ = heroi;
    bomba
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    ph2d_ecs::assign_missing_stable_ids(world);
    println!(
        "[shake-smoke] cena=1  accao=«{ACCAO}» (tecla {TECLA_NOME})  sinal=«{SINAL}»\n\
         (1) carregue no {TECLA_NOME}: a vista TREME — os postes saltam, e o quadrado VERMELHO e' \
         a bomba que explodiu\n\
         (2) ande para LONGE com as SETAS (a camera segue o quadrado azul) e carregue no \
         {TECLA_NOME} outra vez: o MESMO estrondo abana MENOS. Va' ainda mais longe e ele deixa de \
         chegar\n\
         (3) role o painel da direita ate' ao fim (a «Bomba» ja' esta' escolhida): a seccao \
         SHAKE EMITTER e' onde ela grita. \
         suba o «Nothing Beyond» de 18 para 40 e repita o passo (2) — agora ele chega de longe\n\
         (4) na Hierarchy escolha a «Camera»: a seccao CAMERA SHAKE e' o COMO. Suba a «Amplitude» \
         e baixe o «Decay» para 0,5; carregue no {TECLA_NOME} e veja um abanao grande e longo\n\
         (5) ainda na CAMERA SHAKE, o «Punch» de 1 a 3 muda o CARACTER: a 1 a cauda fica a pairar, \
         a 3 o abanao morre depressa\n\
         (6) na barra de CIMA carregue em `Pause`: o {TECLA_NOME} deixa de abanar e volta a ser do \
         editor. `Play`, ao lado, devolve-o\n\
         (7) deu errado se: nada treme ao carregar no {TECLA_NOME} · tremer o mesmo de perto e de \
         longe · o abanao nunca parar · ou a vista ficar DESLOCADA depois de ele acabar"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "shake_smoke_tests.rs"]
mod tests;
