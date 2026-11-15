//! ⭐⭐⭐ **Smoke do OLHO** (suplente #21, W6). `PH2D_RAY_SMOKE=1`.
//!
//! Um poste com DOIS olhos — dois objectos filhos «Ray Sensor» (`Front Ray`, `Back Ray`), a lei de
//! 15/11 (ADR-0199) — e uma caixa que chega da direita. Mesmo componente, UMA diferença: o `Angle`
//! (`0` vê a caixa e acende a luz da direita; `180`, o CONTROLO, fica calado). Uma forma `is_sensor`
//! é simétrica e não distingue frente de trás; o raio distingue. Os nomes dos sinais diferem porque um
//! sinal é um nome global. A caixa nasce FORA do alcance (o alcance é um limite a sério) e PÁRA dentro
//! (um projéctil autorado é documento). Se `[ray-smoke]` não aparecer, a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Name, SignalAction, SignalActions, SignalFrom, SignalTarget, SignalVerb, Transform,
    Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, ProjectileMotion, RaySensor, RaySignals, RigidBody,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// ⭐⭐ **Quantas cenas este roteador serve** — CONTADO do [`montar`].
pub const CENAS: u32 = 1;

/// Até onde cada olho enxerga (m); a geometria toda deriva dele.
pub const ALCANCE: f32 = 6.0;

/// A janela útil do canvas `(x0, x1, y0, y1)`, MEDIDA na foto de 19/09 (1930×1012, câmera de
/// omissão): a Hierarquia tapa até `x ≈ −7,3`, o Inspector a partir de `+6,8`, a timeline abaixo de
/// `y ≈ −1,3`. Gate `a_cena_inteira_cabe_na_janela_util`.
pub const JANELA_UTIL: (f32, f32, f32, f32) = (-7.3, 6.8, -1.3, 5.0);

/// O centro do poste (`y`), pai dos dois olhos.
const Y_POSTE: f32 = -0.25;
/// Onde o poste está (medido na foto: em `−6` a luz do CONTROLO caía atrás da Hierarquia).
const X_POSTE: f32 = -4.0;
/// Onde a caixa nasce — **além** da ponta da linha (`X_POSTE + ALCANCE = 2`).
const X_CAIXA: f32 = 6.0;
/// Quantos metros ela anda antes de parar. `6 → −0,5`, logo a `3,1` m do olho.
const VIAGEM: f32 = 6.5;
/// A rapidez dela, m/s — devagar de propósito: o tracinho do acerto tem de **deslizar** à vista.
const RAPIDEZ: f32 = 2.0;

/// Meia-largura e meia-altura da caixa.
const CAIXA_MEIA: [f32; 2] = [0.4, 0.6];
/// A altura das duas luzes, e o lado de cada uma.
const Y_LUZ: f32 = 2.4;
const LUZ_LADO: f32 = 0.8;
/// O `y` de cada olho — ⚠️ os dois dentro da altura da caixa, senão a diferença deixava de ser
/// **só** a direcção (ver o gate `os_dois_olhos_diferem_so_na_direccao`).
const Y_OLHO: f32 = 0.35;

/// O nome do sinal de quem VÊ.
const VIU: &str = "vi";
/// E o de quem deixa de ver.
const PERDI: &str = "perdi";
/// Os do CONTROLO — ⚠️ outros nomes, senão as duas luzes acendem juntas (ver o cabeçalho).
const VIU_ATRAS: &str = "vi-atras";
const PERDI_ATRAS: &str = "perdi-atras";

/// Os nomes da cena (a Hierarquia é do app: inglês).
pub const POSTE: &str = "Post";
pub const OLHO_FRENTE: &str = "Front Ray";
pub const OLHO_TRAS: &str = "Back Ray";
pub const LUZ_FRENTE: &str = "Front Light";
pub const LUZ_TRAS: &str = "Back Light";
pub const CAIXA: &str = "Box";

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const POSTE_RGBA: [f32; 4] = [0.45, 0.47, 0.52, 1.0];
const LUZ_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
/// O SUPORTE — ver [`luz`]. Escuro, e sempre visível.
const SUPORTE_RGBA: [f32; 4] = [0.26, 0.27, 0.30, 1.0];
const CAIXA_RGBA: [f32; 4] = [0.80, 0.80, 0.84, 1.0];

/// Uma linha de tabela que age sobre **este** objecto.
fn linha(on: &str, verb: SignalVerb) -> SignalAction {
    SignalAction {
        item: Default::default(),
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg: String::new(),
        target_by: SignalTarget::Named,
        from: SignalFrom::Anyone,
    }
}

/// Uma luz do lado para onde o olho dela olha, acende e apaga pelos sinais, sobre um SUPORTE escuro
/// sempre visível (foto de 19/09: sem ele «não acendeu» e «não há luz» davam o mesmo ecrã).
fn luz(world: &mut World, nome: &str, x: f32, acende: &str, apaga: &str) -> Entity {
    world.spawn((
        Name::new(format!("Stand: {nome}")),
        Transform::from_translation(Vec2::new(x, Y_LUZ)),
        Visibility::visible(),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [LUZ_LADO * 1.4, LUZ_LADO * 1.4],
            SUPORTE_RGBA,
        ),
    ));
    world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(x, Y_LUZ)),
            Visibility::hidden(),
            Sprite::atlas(WHITE_TILE_KEY, [LUZ_LADO, LUZ_LADO], LUZ_RGBA),
            SignalActions(vec![
                linha(acende, SignalVerb::Show),
                linha(apaga, SignalVerb::Hide),
            ]),
        ))
        .id()
}

/// **Um olho**: o objecto filho «Ray Sensor» do poste (`pendura_raio`, a porta do Add ▸ Ray Sensor;
/// a lei de 15/11: o raio é um OBJECTO na Hierarquia, como a Collision Shape, ADR-0199).
///
/// ⛔ Sem `Sprite` (a foto de 19/09: as secções do Sprite enterravam o cartão Ray Sensor abaixo da
/// dobra) e sem colisor nem corpo (o raio de baixo acertaria no de cima e o CONTROLO falaria por
/// outra razão que não a direcção). `y` é o do MUNDO; a pose local é relativa ao poste.
fn olho(
    world: &mut World,
    poste: Entity,
    nome: &str,
    angle: f32,
    y: f32,
    voz: RaySignals,
) -> Entity {
    let e = world.spawn((Name::new(nome), Visibility::visible())).id();
    ph2d_physics_ecs::pendura_raio(world, e, Some(poste));
    let mut em = world.entity_mut(e);
    em.insert((
        Transform::from_translation(Vec2::new(0.0, y - Y_POSTE)),
        RaySensor {
            angle,
            reach: ALCANCE,
            layer: 0,
            turns_with_facing: false,
            ..RaySensor::default()
        },
        voz,
    ));
    e
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — o olho que vê, porque o roteiro fala do cartão
/// dele.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️ **O CHÃO PRIMEIRO**: a ordem das raízes é a de CRIAÇÃO, e quem nasce primeiro desenha por
    // baixo (a cura de 15/09).
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [24.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    // ⚠️ A altura e o centro do poste saem da foto: a régua da timeline tapa o canvas abaixo de
    // `y = −1,3`.
    let poste = world
        .spawn((
            Name::new(POSTE),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 1.8], POSTE_RGBA),
            Transform::from_translation(Vec2::new(X_POSTE, Y_POSTE)),
            Visibility::visible(),
        ))
        .id();

    // As duas luzes, cada uma do lado para onde o olho dela olha.
    luz(world, LUZ_FRENTE, X_POSTE + 1.6, VIU, PERDI);
    luz(world, LUZ_TRAS, X_POSTE - 1.6, VIU_ATRAS, PERDI_ATRAS);

    let ve = olho(
        world,
        poste,
        OLHO_FRENTE,
        0.0,
        Y_OLHO,
        RaySignals {
            on_enter: VIU.to_owned(),
            on_exit: PERDI.to_owned(),
        },
    );
    olho(
        world,
        poste,
        OLHO_TRAS,
        180.0,
        -Y_OLHO,
        RaySignals {
            on_enter: VIU_ATRAS.to_owned(),
            on_exit: PERDI_ATRAS.to_owned(),
        },
    );

    // ⭐ **A CAIXA é matéria**, e tem de o ser: a porta do motor põe `EXCLUDE_SENSORS` dentro dela
    // (*«um sensor é um marcador, não matéria»*), logo um alvo `is_sensor` seria invisível ao raio.
    //
    // ⚠️ **`face_velocity: false`**: a rotação é o RUMO do lançamento (`π` = para a esquerda) e tem
    // de ficar quieta — uma caixa a rodar não muda nada na lei e distrai de onde está a lição.
    world.spawn((
        Name::new(CAIXA),
        Transform {
            translation: Vec2::new(X_CAIXA, 0.0),
            rotation: std::f32::consts::PI,
            ..Transform::default()
        },
        Visibility::visible(),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [CAIXA_MEIA[0] * 2.0, CAIXA_MEIA[1] * 2.0],
            CAIXA_RGBA,
        ),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: CAIXA_MEIA[0],
                half_y: CAIXA_MEIA[1],
            },
            ..Collider::default()
        },
        ProjectileMotion::from_law(
            ProjectileLaw {
                initial_speed: RAPIDEZ,
                range: VIAGEM,
                face_velocity: false,
                ..ProjectileLaw::default()
            },
            0,
        ),
    ));
    ve
}

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// O OLHO QUE VÊ, que nasce escolhido.
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    ph2d_physics_ecs::migra_formas_para_filhos(world);
    ph2d_ecs::assign_missing_stable_ids(world);
    ph2d_ecs::assign_missing_root_order(world);
    println!(
        "[ray-smoke] cena=1  alcance={ALCANCE} m (600 px no painel)\n\
         (1) o poste da esquerda (`Post`) tem DOIS olhos, os dois `Ray Sensor` FILHOS dele na \
         Hierarquia (`Front Ray` e `Back Ray`), e de cada um sai uma linha fina — uma para a \
         DIREITA e outra para a ESQUERDA. Cada linha e' o alcance de um olho, e o tracinho na PONTA \
         marca onde ele acaba. Por cima do poste estao as DUAS luzes deles, as duas apagadas\n\
         (2) a caixa clara vem da direita. Enquanto ela estiver ALEM da ponta da linha nao acontece \
         nada, e no painel da direita o cartao `Ray Sensor` diz `Sees nothing right now.` — o \
         alcance e' um limite a serio\n\
         (3) quando ela ENTRA na linha: aparece um segundo tracinho SOBRE a linha, no ponto em que \
         o olho a encontrou, e a luz da DIREITA acende. A caixa continua a vir e o tracinho desliza \
         com ela ate' a caixa parar\n\
         (4) o `Front Ray` ja' nasce escolhido: no painel, a linha por baixo do titulo passa a \
         dizer `Sees Box` com a distancia em px, e o numero MUDA enquanto ela se aproxima\n\
         (5) a luz da ESQUERDA fica APAGADA o tempo todo, e esse e' o CONTROLO: o olho de baixo \
         tem o MESMO componente com UMA diferenca — o `Angle` esta' ao contrario (0 e 180). Uma forma nao \
         distingue a frente de tras; um raio distingue\n\
         (6) agora mude o `Reach` do olho escolhido para 200: a linha ENCOLHE no canvas na hora e a \
         luz da direita APAGA, porque o olho deixou de alcancar a caixa. Ponha 600 outra vez e ela \
         volta a acender\n\
         (7) deu errado se: as linhas nao aparecem · o tracinho do encontro nao desliza com a caixa \
         · a luz da esquerda acende · a distancia do painel nao muda · ou mudar o `Reach` nao muda \
         a linha"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

/// O cartão Ray Sensor e o grupo Signals abertos no olho escolhido, como na cena 26.
pub fn arruma_o_inspector(store: &mut ph2d_editor_core::interaction::WidgetStore, raio: u64) {
    use ph2d_editor_core::ids;
    for id in [ids::INSP_LIVE_RAY_SECTION, ids::INSP_RAY_FOLD_SIGNALS] {
        store.set_collapsed(id, false);
    }
    store.request_inspector_section(ids::INSP_LIVE_RAY_SECTION, raio);
}

#[cfg(test)]
#[path = "ray_smoke_tests.rs"]
mod tests;
