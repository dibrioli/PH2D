//! **A cena do Projectile Motion** (`PH2D_GAME_SMOKE=25`, ficha GEF 06): o chão, uma parede à direita e cinco balas
//! que nascem juntas à esquerda — em cima o míssil (Homing), lançado para BAIXO, que dá a volta e acerta o «Target»
//! no alto à direita; por baixo a reta (morre na parede), o ricochete (Bounce), o arco (Gravity) e o alcance curto
//! (Range). Montada SÓ com peças: «Static Body» e «Sprite» do menu Add e o `+` do Inspector (o corpo nasce Kinematic
//! pela cascata). O míssil fica selecionado, com o cartão à vista, e a vista enquadra a arena inteira.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, ZIndexOverride, stable_name_id};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::{NodeId, ids};
use ph2d_physics_ecs::{Collider, ColliderShape, ProjectileMotion};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O número da cena no `PH2D_GAME_SMOKE`.
pub const CENA: u32 = 25;

/// O objecto selecionado ao abrir (nome de objecto da cena, não texto da interface).
pub const MISSIL: &str = "Missile";

/// O que o míssil persegue.
pub const ALVO: &str = "Target";

/// A rapidez de todas as balas (m/s): só um campo muda entre elas.
pub const RAPIDEZ: f32 = 9.0;

/// O raio das balas (m).
const RAIO: f32 = 0.15;

const FUNDO: ([f32; 2], [f32; 2]) = ([0.0, 0.0], [14.0, 9.0]);
/// O chão de verdade (centro, meia-medida, m): a faixa sólida na base do fundo, onde o Arc pousa.
pub const CHAO: ([f32; 2], [f32; 2]) = ([0.0, -4.35], [7.0, 0.15]);
const PAREDE: ([f32; 2], [f32; 2]) = ([5.5, 0.0], [0.3, 4.2]);
/// A parede de trás (m): o Bouncy volta da parede e ricocheteia nela em vez de sair da arena.
pub const PAREDE_ESQ: ([f32; 2], [f32; 2]) = ([-6.85, 0.0], [0.15, 4.2]);
const FUNDO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const CHAO_RGBA: [f32; 4] = [0.30, 0.32, 0.36, 1.0];
const PAREDE_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];
const ALVO_RGBA: [f32; 4] = [0.60, 0.62, 0.66, 1.0];

/// Uma bala: nome, onde nasce, cor e o que muda na lei.
type Bala = (&'static str, [f32; 2], [f32; 4], fn(&mut ProjectileMotion));

/// O Pull do míssil (m/s²): ele só VIRA (a rapidez fica [`RAPIDEZ`]), e o raio da volta é v²/a ≈ 1 m — dá a volta
/// e acerta, sem Max Speed.
pub const PUXAO: f32 = 80.0;

/// As cinco balas, de cima para baixo.
pub const BALAS: [Bala; 5] = [
    (MISSIL, [-6.0, 3.6], [0.95, 0.30, 0.30, 1.0], |p| {
        p.homing_accel = PUXAO;
        p.homing_target = stable_name_id(ALVO);
    }),
    ("Straight", [-6.0, 1.2], [0.95, 0.72, 0.25, 1.0], |_| {}),
    ("Bouncy", [-6.0, -0.4], [0.95, 0.45, 0.20, 1.0], |p| {
        p.max_bounces = 4;
        p.bounciness = 0.85;
    }),
    ("Arc", [-6.0, -2.0], [0.35, 0.65, 0.95, 1.0], |p| {
        p.gravity = 6.0
    }),
    ("Short Range", [-6.0, -3.6], [0.45, 0.85, 0.45, 1.0], |p| {
        p.range = 5.0;
    }),
];

/// Onde mora o alvo do míssil (m): no alto à direita, longe da linha de onde ele nasce, que aponta para baixo.
pub const ALVO_EM: [f32; 2] = [2.5, 2.6];

/// As peças que a cena pede à composição (as mesmas da cena 24).
pub use crate::cena_topdown::Pecas;

fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2], sprite: Sprite) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert((Name::new(nome), sprite));
    if let Some(mut t) = em.get_mut::<Transform>() {
        t.translation = Vec2::new(at[0], at[1]);
    }
}

/// A forma de colisão de `e` (a filha que o menu Add criou, ou ele mesmo) passa a `shape`.
fn forma(sim: &mut SimWorld, e: Entity, shape: ColliderShape) {
    let w = sim.world_mut();
    let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
    if let Some(mut col) = w.get_mut::<Collider>(f) {
        col.shape = shape;
    }
}

/// Monta a cena e devolve o míssil (o objecto que o Inspector abre).
pub fn montar(sim: &mut SimWorld, p: Pecas<'_>) -> Entity {
    let fundo = (p.imagem)(sim);
    let sprite = Sprite::atlas(WHITE_TILE_KEY, FUNDO.1, FUNDO_RGBA);
    arruma(sim, fundo, "Background", FUNDO.0, sprite);
    sim.world_mut().entity_mut(fundo).insert(ZIndexOverride(-1));
    let pecas = [
        ("Floor", CHAO, CHAO_RGBA),
        ("Wall", PAREDE, PAREDE_RGBA),
        ("Back Wall", PAREDE_ESQ, PAREDE_RGBA),
    ];
    for (nome, (c, h), cor) in pecas {
        let e = (p.estatico)(sim);
        let sprite = Sprite::atlas(WHITE_TILE_KEY, [2.0 * h[0], 2.0 * h[1]], cor);
        arruma(sim, e, nome, c, sprite);
        let (half_x, half_y) = (h[0], h[1]);
        forma(sim, e, ColliderShape::Cuboid { half_x, half_y });
    }
    let alvo = (p.estatico)(sim);
    let sprite = Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], ALVO_RGBA);
    arruma(sim, alvo, ALVO, ALVO_EM, sprite);
    forma(sim, alvo, ColliderShape::Ball { radius: 0.4 });
    let mut missil = alvo;
    for (nome, at, cor, lei) in BALAS {
        let e = (p.imagem)(sim);
        arruma(
            sim,
            e,
            nome,
            at,
            Sprite::atlas(WHITE_TILE_KEY, [2.0 * RAIO; 2], cor),
        );
        if let Err(erro) = (p.anexa)(sim, e.to_bits(), "ph2d::physics::ProjectileMotion") {
            eprintln!("[game-smoke] =25 o Projectile Motion de {nome}: {erro}");
        }
        forma(sim, e, ColliderShape::Ball { radius: RAIO });
        let w = sim.world_mut();
        if let Some(mut pm) = w.get_mut::<ProjectileMotion>(e) {
            pm.initial_speed = RAPIDEZ;
            lei(&mut pm);
        }
        if nome == MISSIL {
            // Aponta para BAIXO, com o alvo no alto à direita: a volta é o que se quer ver.
            if let Some(mut t) = w.get_mut::<Transform>(e) {
                t.rotation = -core::f32::consts::FRAC_PI_2;
            }
            missil = e;
        }
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =25 cinco balas e uma parede: o Missile (selecionado) sai para baixo, dá a volta e acerta o \
         Target; Straight morre na parede, Bouncy ricocheteia, Arc cai em arco até o chão e Short Range para no meio. \
         Play (F5) dispara todas."
    );
    missil
}

/// O instante das fotos do cartão (`PH2D_GAME_SMOKE_FOTO`); sem a variável a cena é a do dono.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foto {
    /// `projectile_speed_change`: a dobra Speed change aberta.
    SpeedChange,
}

/// Os nomes que [`foto`] aceita (censo em `ph2d_game_smoke_fotos::nomes`).
pub const NOMES: &str = "projectile_speed_change";

/// A foto pedida pelo nome.
#[must_use]
pub fn foto(nome: Option<&str>) -> Option<Foto> {
    match nome?.trim() {
        "projectile_speed_change" => Some(Foto::SpeedChange),
        _ => None,
    }
}

/// ⭐ **As gavetas do Inspector ao abrir a cena 25** (`(secção, fechada)`): o cartão da família e o Projectile
/// Motion abertos; a foto `projectile_speed_change` abre também a dobra.
#[must_use]
pub fn gavetas(foto: Option<Foto>, cartao: NodeId) -> Vec<(NodeId, bool)> {
    let mut g = vec![(cartao, false), (ids::INSP_LIVE_PROJECTILE_SECTION, false)];
    if foto == Some(Foto::SpeedChange) {
        g.push((ids::INSP_PJ_FOLD_SPEED_CHANGE, false));
    }
    g
}

/// Aplica as [`gavetas`] ao Inspector e pede-lhe que role até ao cartão Projectile Motion do `missil`.
pub fn arruma_o_inspector(store: &mut WidgetStore, cartao: NodeId, missil: u64) {
    let pedida = std::env::var("PH2D_GAME_SMOKE_FOTO").ok();
    for (id, fechada) in gavetas(foto(pedida.as_deref()), cartao) {
        store.set_collapsed(id, fechada);
    }
    store.request_inspector_section(ids::INSP_LIVE_PROJECTILE_SECTION, missil);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn na_cena_25_o_cartao_projectile_nasce_a_vista_e_a_foto_abre_a_dobra() {
        let c = ids::INSP_LIVE_PLAYER_SECTION;
        let p = ids::INSP_LIVE_PROJECTILE_SECTION;
        assert_eq!(gavetas(None, c), vec![(c, false), (p, false)]);
        let dobra = ids::INSP_PJ_FOLD_SPEED_CHANGE;
        assert!(gavetas(foto(Some("projectile_speed_change")), c).contains(&(dobra, false)));
        assert_eq!(foto(Some("topdown_more")), None);
        let mut s = WidgetStore::default();
        for (id, fechada) in gavetas(None, c) {
            s.set_collapsed(id, fechada);
        }
        s.request_inspector_section(p, 0xB1);
        assert_eq!(s.inspector_section_reveal(), Some((p, 0xB1)));
    }

    /// Uma bala por campo: cada uma muda UM número da lei de fábrica, e só o míssil persegue — sem Max Speed
    /// (o Pull só vira, dono 13/11).
    #[test]
    fn cada_bala_muda_um_so_campo() {
        let base = ProjectileMotion::default();
        for (nome, _, _, lei) in BALAS {
            let mut p = base;
            lei(&mut p);
            let mudou = [
                p.max_bounces != base.max_bounces,
                p.gravity != base.gravity,
                p.range != base.range,
                p.homing_accel != base.homing_accel,
            ];
            let n = mudou.iter().filter(|m| **m).count();
            assert_eq!(n, usize::from(nome != "Straight"), "{nome}");
            assert_eq!(p.homing_accel > 0.0, nome == MISSIL, "{nome}");
            assert_eq!(p.max_speed, base.max_speed, "{nome}");
        }
    }
}
