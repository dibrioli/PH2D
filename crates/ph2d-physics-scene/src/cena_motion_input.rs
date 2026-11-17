//! ⭐ **Cena 27 — o Motion Input** (GEF 08, onda 1; `PH2D_GAME_SMOKE=27`, com o grafo `PH2D_GPU_COOK_DEMO=134`).
//!
//! Dois alvos com vida numa fila em baixo: o «Health Dummy» publica a vida (Source Health, `vida`: a barra de 24
//! peças do grafo 134 encolhe) e o «Hit Dummy» um disparo por golpe (Source Damage, `boom`: a galáxia simulada
//! acelera no Vortex). Quatro balas com Damage (20, somem ao acertar) vêm de cada lado, uma por segundo no Play.
//! A «Lamp» tem Motion Input Source Health SEM Health — o aviso e o botão «Add Health». Abre PARADA com o
//! Inspector no «Health Dummy». Ficha: `docs/GameEngineFuncional/componentes/08_motion_input.md`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, MotionInput, MotionSource, Name, SimWorld, Transform};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::{NodeId, ids};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, OnHit, ProjectileMotion, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

pub const CENA: u32 = 27;

pub const ALVO_VIDA: &str = "Health Dummy";
pub const ALVO_GOLPE: &str = "Hit Dummy";
pub const LAMPADA: &str = "Lamp";
/// Os nomes das entradas do grafo `=134` do Motion.
pub const ENTRADA_VIDA: &str = "vida";
pub const ENTRADA_GOLPE: &str = "boom";
/// O golpe de cada bala; quatro tiram 80 % (a vida acaba em 0,2, viva).
pub const GOLPE: f32 = 20.0;
/// A rapidez das balas (m/s): com um metro entre elas, uma chega por segundo.
pub const RAPIDEZ: f32 = 1.0;
const MEIO_ALVO: f32 = 0.35;
const RAIO: f32 = 0.15;
/// `(nome, onde, fonte, entrada)`: o «Health Dummy» à esquerda da barra do grafo `=134` (y 2,6); a «Lamp» e o «Hit
/// Dummy» abaixo da grelha e da galáxia, acima das legendas (y −4,35). Fotografado (`fotos_08/`).
pub const OBJETOS: [(&str, [f32; 2], MotionSource, &str); 3] = [
    (ALVO_VIDA, [-5.0, 2.6], MotionSource::Health, ENTRADA_VIDA),
    (LAMPADA, [-1.0, -3.75], MotionSource::Health, ENTRADA_VIDA),
    (
        ALVO_GOLPE,
        [3.6, -3.75],
        MotionSource::Damage,
        ENTRADA_GOLPE,
    ),
];
/// Onda 2 — as fontes novas, sem script. O grafo lê a PRIMEIRA linha de um nome, e as linhas vão pela ordem
/// alfabética do objecto: «Health Bar Reader» (From = «Health Dummy») passa à frente do «Health Dummy» na `vida`
/// (o mesmo número: o From só muda QUEM lê); o «Max Health Reader» e o «Tap Reader» escrevem em 2.º e 3.º na `vida`
/// (a barra lê a 1.ª linha). O `estado` fica SÓ com o «Speed Reader»: o Toggle lê o campo INTEIRO, e dois
/// escritores ali desarrumavam a grelha (foto de 16/11). O cartão de cada um diz quem escreve primeiro.
pub const LEITOR_VIDA: &str = "Health Bar Reader";
pub const LEITOR_RAPIDEZ: &str = "Speed Reader";
pub const LEITOR_ACCAO: &str = "Tap Reader";
pub const LEITOR_CAMPO: &str = "Max Health Reader";
pub const PARAPEITO: &str = "Ledge";
pub const ENTRADA_ESTADO: &str = "estado";
/// A acção de fábrica do Input Map que o «Tap Reader» lê.
pub const ACCAO: &str = "jump";
/// O campo que o «Max Health Reader» lê do «Hit Dummy».
pub const CAMPO: &str = "ph2d::physics::Health/max";
/// A rapidez típica da bola (m/s): o Remap leva `0..6` a `0..1`; o `estado` vira anel a partir de 3 m/s.
pub const RAPIDEZ_TIPICA: f32 = 6.0;
pub const SUAVE_S: f32 = 0.3;
/// A bola cai de 5,5 m sobre o parapeito e quica (restitution), à direita da galáxia.
pub const BOLA: ([f32; 2], f32) = ([8.8, 6.3], 0.3);
pub const RESTITUICAO: f32 = 0.8;
pub const PARAPEITO_AT: ([f32; 2], [f32; 2]) = ([8.8, 0.8], [1.0, 0.15]);
/// Onde moram os três leitores sem corpo: a coluna à esquerda da grelha.
pub const LEITORES_AT: [(&str, [f32; 2]); 3] = [
    (LEITOR_VIDA, [-7.4, 2.6]),
    (LEITOR_CAMPO, [-7.4, 1.2]),
    (LEITOR_ACCAO, [-7.4, -0.2]),
];
const LEITOR_RGBA: [f32; 4] = [0.45, 0.70, 0.95, 1.0];
const BOLA_RGBA: [f32; 4] = [0.40, 0.85, 0.55, 1.0];
const PARAPEITO_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];

/// `(alvo, direcção do voo, distâncias ao alvo)`: as do «Health Dummy» caem de cima; as do «Hit Dummy» vêm da
/// direita, meio segundo depois.
pub const BALAS: [(&str, [f32; 2], [f32; 4]); 2] = [
    (ALVO_VIDA, [0.0, -1.0], [1.3, 2.3, 3.3, 4.3]),
    (ALVO_GOLPE, [-1.0, 0.0], [1.8, 2.8, 3.8, 4.8]),
];

const ALVO_RGBA: [f32; 4] = [0.60, 0.62, 0.66, 1.0];
const LAMPADA_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
const BALA_RGBA: [f32; 4] = [0.95, 0.35, 0.30, 1.0];

pub use crate::cena_topdown::Pecas;

fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2], sprite: Sprite) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert((Name::new(nome), sprite));
    if let Some(mut t) = em.get_mut::<Transform>() {
        t.translation = Vec2::new(at[0], at[1]);
    }
}

fn forma(sim: &mut SimWorld, e: Entity, shape: ColliderShape) {
    let w = sim.world_mut();
    let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
    if let Some(mut col) = w.get_mut::<Collider>(f) {
        col.shape = shape;
    }
}

/// O objeto filho «Motion Input» de `dono` com a config `mi` (MI-10), pela porta do `+`; devolve o filho.
fn entrada(sim: &mut SimWorld, p: &Pecas<'_>, dono: Entity, mi: MotionInput) -> Entity {
    let e = ph2d_physics_ecs::pendura_entrada(sim.world_mut(), None, Some(dono), "");
    anexa(sim, p, e, "ph2d::ecs::MotionInput");
    sim.world_mut().entity_mut(e).insert(mi);
    e
}

fn anexa(sim: &mut SimWorld, p: &Pecas<'_>, e: Entity, nome: &str) {
    if let Err(erro) = (p.anexa)(sim, e.to_bits(), nome) {
        eprintln!("[game-smoke] =27 {nome}: {erro}");
    }
}

/// Monta a cena e devolve o objecto que o Inspector abre: o filho «Motion Input» do «Health Dummy» (MI-10), ou o
/// do da [`Foto`] pedida.
pub fn montar(sim: &mut SimWorld, p: Pecas<'_>) -> Entity {
    let quer = match foto(pedida().as_deref()) {
        Some(Foto::Lamp) => LAMPADA,
        Some(Foto::Hit) => ALVO_GOLPE,
        Some(Foto::Speed | Foto::SpeedPlay) => LEITOR_RAPIDEZ,
        Some(Foto::Action) => LEITOR_ACCAO,
        Some(Foto::Property) => LEITOR_CAMPO,
        None => ALVO_VIDA,
    };
    let mut escolhido = None;
    for (nome, at, fonte, entrada) in OBJETOS {
        let lampada = nome == LAMPADA;
        let e = if lampada {
            (p.imagem)(sim)
        } else {
            (p.estatico)(sim)
        };
        let cor = if lampada { LAMPADA_RGBA } else { ALVO_RGBA };
        let lado = 2.0 * MEIO_ALVO;
        arruma(
            sim,
            e,
            nome,
            at,
            Sprite::atlas(WHITE_TILE_KEY, [lado; 2], cor),
        );
        if !lampada {
            let (half_x, half_y) = (MEIO_ALVO, MEIO_ALVO);
            forma(sim, e, ColliderShape::Cuboid { half_x, half_y });
            anexa(sim, &p, e, "ph2d::physics::Health");
            if let Some(mut h) = sim.world_mut().get_mut::<Health>(e) {
                h.invincible_s = 0.0; // um golpe por bala, sem janela que engula o seguinte
            }
        }
        let mi = MotionInput {
            name: entrada.to_owned(),
            source: fonte,
            ..MotionInput::default()
        };
        let filho = self::entrada(sim, &p, e, mi);
        if nome == quer {
            escolhido = Some(filho);
        }
    }
    for (alvo, d, distancias) in BALAS {
        let at = OBJETOS
            .iter()
            .find(|o| o.0 == alvo)
            .map_or([0.0; 2], |o| o.1);
        for k in distancias {
            let e = (p.imagem)(sim);
            let sprite = Sprite::atlas(WHITE_TILE_KEY, [2.0 * RAIO; 2], BALA_RGBA);
            arruma(
                sim,
                e,
                "Bullet",
                [at[0] - d[0] * k, at[1] - d[1] * k],
                sprite,
            );
            anexa(sim, &p, e, "ph2d::physics::ProjectileMotion");
            anexa(sim, &p, e, "ph2d::physics::Damage");
            forma(sim, e, ColliderShape::Ball { radius: RAIO });
            let w = sim.world_mut();
            if let Some(mut pm) = w.get_mut::<ProjectileMotion>(e) {
                pm.initial_speed = RAPIDEZ;
            }
            if let Some(mut d) = w.get_mut::<Damage>(e) {
                d.amount = GOLPE;
                d.on_hit = OnHit::Vanish;
            }
            if let Some(mut t) = w.get_mut::<Transform>(e) {
                t.rotation = d[1].atan2(d[0]);
            }
        }
    }
    for (dono, filho) in onda2(sim, &p) {
        let w = sim.world();
        if w.get::<Name>(dono).is_some_and(|n| n.0 == quer) {
            escolhido = Some(filho);
        }
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =27 onda 2: «Health Bar Reader» lê a vida do «Health Dummy» (From) e passa à frente dele \
         na «vida»; a bola «Speed Reader» quica no «Ledge» — a velocidade (Remap 0–6, Smooth 0.3 s) cruza a grelha \
         para o anel no «estado»; «Tap Reader» lê a acção «jump» (3.º na «vida»); «Max Health Reader» lê o \
         Health · Max do «Hit Dummy» (2.º na «vida»)."
    );
    println!(
        "[game-smoke] =27 Motion Input: o «Health Dummy» (selecionado) manda a vida ao grafo pelo nome «vida» — a \
         barra de cima; o «Hit Dummy» manda um disparo «boom» a cada golpe — a galáxia gira mais. A «Lamp» pede vida \
         e não tem: o cartão avisa, com o botão Add Health. Play (F5): uma bala por segundo em cada alvo. Use com \
         PH2D_GPU_COOK_DEMO=134 (o grafo)."
    );
    escolhido.expect("o objecto escolhido está na tabela")
}

/// Os três leitores sem corpo, a bola que quica e o parapeito; devolve os quatro `(leitor, filho Motion Input)`.
fn onda2(sim: &mut SimWorld, p: &Pecas<'_>) -> Vec<(Entity, Entity)> {
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let sid = |sim: &mut SimWorld, nome: &str| {
        let w = sim.world_mut();
        let mut q = w.query::<(&Name, &ph2d_ecs::StableId)>();
        q.iter(w).find(|(n, _)| n.0 == nome).map_or(0, |(_, s)| s.0)
    };
    let (dummy, golpe) = (sid(sim, ALVO_VIDA), sid(sim, ALVO_GOLPE));
    let mut leitores = Vec::new();
    for (nome, at) in LEITORES_AT {
        let e = (p.imagem)(sim);
        arruma(
            sim,
            e,
            nome,
            at,
            Sprite::atlas(WHITE_TILE_KEY, [2.0 * MEIO_ALVO; 2], LEITOR_RGBA),
        );
        let mi = match nome {
            LEITOR_VIDA => MotionInput {
                name: ENTRADA_VIDA.into(),
                source: MotionSource::Health,
                from: dummy,
                ..MotionInput::default()
            },
            LEITOR_CAMPO => MotionInput {
                name: ENTRADA_VIDA.into(),
                source: MotionSource::Property,
                from: golpe,
                property: CAMPO.into(),
                remap: true,
                remap_from: 0.0,
                remap_to: 200.0,
                ..MotionInput::default()
            },
            _ => MotionInput {
                name: ENTRADA_VIDA.into(),
                source: MotionSource::Action,
                action: ACCAO.into(),
                ..MotionInput::default()
            },
        };
        let filho = entrada(sim, p, e, mi);
        leitores.push((e, filho));
    }
    let (c, h) = PARAPEITO_AT;
    let ledge = (p.estatico)(sim);
    arruma(
        sim,
        ledge,
        PARAPEITO,
        c,
        Sprite::atlas(WHITE_TILE_KEY, [2.0 * h[0], 2.0 * h[1]], PARAPEITO_RGBA),
    );
    forma(
        sim,
        ledge,
        ColliderShape::Cuboid {
            half_x: h[0],
            half_y: h[1],
        },
    );
    let (at, radius) = BOLA;
    let bola = (p.estatico)(sim);
    arruma(
        sim,
        bola,
        LEITOR_RAPIDEZ,
        at,
        Sprite::atlas(WHITE_TILE_KEY, [2.0 * radius; 2], BOLA_RGBA),
    );
    forma(sim, bola, ColliderShape::Ball { radius });
    let w = sim.world_mut();
    if let Some(mut rb) = w.get_mut::<RigidBody>(bola) {
        rb.kind = BodyKind::Dynamic;
    }
    let f = ph2d_physics_ecs::forma_para_editar(w, bola).unwrap_or(bola);
    if let Some(mut col) = w.get_mut::<Collider>(f) {
        col.restitution = RESTITUICAO;
    }
    let mi = MotionInput {
        name: ENTRADA_ESTADO.into(),
        source: MotionSource::Speed,
        remap: true,
        remap_from: 0.0,
        remap_to: RAPIDEZ_TIPICA,
        smooth: SUAVE_S,
        ..MotionInput::default()
    };
    let filho = entrada(sim, p, bola, mi);
    leitores.push((bola, filho));
    leitores
}

/// O instante das fotos do cartão (`PH2D_GAME_SMOKE_FOTO`); sem a variável a cena é a do dono.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foto {
    /// `motion_input_lamp`: a «Lamp» escolhida (o aviso e o Add Health).
    Lamp,
    /// `motion_input_hit`: o «Hit Dummy» escolhido (Source Damage).
    Hit,
    /// `motion_input_speed`: o «Speed Reader» com o grupo Shape aberto.
    Speed,
    /// `motion_input_action`: o «Tap Reader» com a lista Action aberta.
    Action,
    /// `motion_input_property`: o «Max Health Reader» com a lista Property aberta.
    Property,
    /// `motion_input_speed_play`: o «Speed Reader» em Play (o «Sends …» e a grelha a cruzar).
    SpeedPlay,
}

/// Os nomes que [`foto`] aceita (censo em `ph2d_game_smoke_fotos::nomes`).
pub const NOMES: &str = "motion_input_lamp motion_input_hit motion_input_speed motion_input_action \
                         motion_input_property motion_input_speed_play";

#[must_use]
pub fn foto(nome: Option<&str>) -> Option<Foto> {
    match nome?.trim() {
        "motion_input_lamp" => Some(Foto::Lamp),
        "motion_input_hit" => Some(Foto::Hit),
        "motion_input_speed" => Some(Foto::Speed),
        "motion_input_action" => Some(Foto::Action),
        "motion_input_property" => Some(Foto::Property),
        "motion_input_speed_play" => Some(Foto::SpeedPlay),
        _ => None,
    }
}

fn pedida() -> Option<String> {
    std::env::var("PH2D_GAME_SMOKE_FOTO").ok()
}

/// Os ids do cartão Motion Input (os de `ph2d-panel-inspector::ids::inspector_motion_input`, pelo mesmo hash).
pub const FOLD_SHAPE: NodeId = ph2d_tool_registry::hash_node_id("insp_mi_fold_shape");
pub const LISTA_ACCAO: NodeId = ph2d_tool_registry::hash_node_id("insp_mi_action");
pub const LISTA_CAMPO: NodeId = ph2d_tool_registry::hash_node_id("insp_mi_property");

/// As gavetas da [`Foto`]: `(id, fechada)`; o grupo Shape abre na foto da velocidade.
#[must_use]
pub fn gavetas(foto: Option<Foto>, cartao: NodeId) -> Vec<(NodeId, bool)> {
    let mut v = vec![
        (cartao, false),
        (ids::INSP_LIVE_MOTION_INPUT_SECTION, false),
    ];
    if matches!(foto, Some(Foto::Speed | Foto::SpeedPlay)) {
        v.push((FOLD_SHAPE, false));
    }
    v
}

/// A lista que a [`Foto`] abre (Action ou Property).
#[must_use]
pub fn lista_aberta(foto: Option<Foto>) -> Option<NodeId> {
    match foto? {
        Foto::Action => Some(LISTA_ACCAO),
        Foto::Property => Some(LISTA_CAMPO),
        _ => None,
    }
}

/// O cartão da família e o Motion Input abertos, e o Inspector rola até ao Motion Input de `escolhido`.
pub fn arruma_o_inspector(store: &mut WidgetStore, cartao: NodeId, escolhido: u64) {
    let f = foto(pedida().as_deref());
    for (id, fechada) in gavetas(f, cartao) {
        store.set_collapsed(id, fechada);
    }
    if let Some(id) = lista_aberta(f) {
        store.register(
            id,
            ph2d_editor_core::interaction::InteractiveState::Dropdown {
                state: ph2d_editor_core::widget::DropdownState::Normal,
                open: true,
                selected_index: None,
            },
        );
    }
    store.request_inspector_section(ids::INSP_LIVE_MOTION_INPUT_SECTION, escolhido);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada bala chega ao seu alvo antes de passar pela «Lamp», e as dos dois lados não chegam juntas.
    #[test]
    fn as_balas_chegam_uma_por_segundo_e_nao_juntas() {
        let chegadas = |ks: [f32; 4]| ks.map(|k| (k - MEIO_ALVO - RAIO) / RAPIDEZ);
        assert_eq!((BALAS[0].0, BALAS[1].0), (ALVO_VIDA, ALVO_GOLPE));
        let (vida, golpe) = (chegadas(BALAS[0].2), chegadas(BALAS[1].2));
        assert!(vida[0] > 0.5 && golpe[0] > 0.5, "nenhuma nasce encostada");
        for par in vida.windows(2).chain(golpe.windows(2)) {
            assert!((par[1] - par[0] - 1.0).abs() < 0.01, "{par:?}");
        }
        assert!(vida.iter().zip(&golpe).all(|(a, b)| (a - b).abs() > 0.2));
        const { assert!(GOLPE * 4.0 < 100.0, "acaba viva") };
        assert_eq!(foto(Some("motion_input_lamp")), Some(Foto::Lamp));
        assert_eq!(foto(Some("ray_filter")), None);
    }
}
