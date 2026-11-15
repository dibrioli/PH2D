//! ⭐ **Cena 26 — o Ray Sensor é um OBJETO** (GEF 07, versão B; `PH2D_GAME_SMOKE=26`).
//!
//! O herói do menu Add (Character) leva DOIS raios filhos, criados pelo Add ▸ «Ray Sensor» com ele
//! selecionado: «Ledge Ray» à frente dos pés, a olhar para baixo (há chão?), e «Wall Ray» à altura do
//! peito, a olhar para a frente (há parede?). O chão acaba antes da parede, que desce até ao fundo do poço. No Play, anda-se para a
//! direita: a luz «Wall Light» acende quando a parede entra no alcance e a «Edge Light» quando o chão
//! acaba à frente. O «Wall Ray» nasce escolhido, com o cartão Ray Sensor e o grupo Signals abertos.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Name, SignalAction, SignalActions, SignalFrom, SignalTarget, SignalVerb, SimWorld,
    Transform, Visibility, ZIndexOverride,
};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::{NodeId, ids};
use ph2d_physics_ecs::{Collider, ColliderShape, RaySensor, RaySignals};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

pub const CENA: u32 = 26;

pub const HEROI: &str = "Hero";
pub const RAIO_PAREDE: &str = "Wall Ray";
pub const RAIO_BEIRADA: &str = "Ledge Ray";

/// O herói nasce aqui; o chão vai de `x = -7` a `x = 1,5`, a parede está em `x = 6`.
pub const HEROI_EM: [f32; 2] = [-5.0, -1.0];
const FUNDO: ([f32; 2], [f32; 2]) = ([0.0, 0.0], [16.0, 9.0]);
pub const CHAO: ([f32; 2], [f32; 2]) = ([-2.75, -2.25], [4.25, 0.25]);
pub const PAREDE: ([f32; 2], [f32; 2]) = ([6.0, -1.25], [0.3, 2.75]);
const POCO: ([f32; 2], [f32; 2]) = ([5.0, -4.25], [2.0, 0.25]);

/// `(nome, desvio no herói, ângulo, alcance, ao ver, ao deixar de ver)`.
type Raio = (&'static str, [f32; 2], f32, f32, &'static str, &'static str);
pub const RAIOS: [Raio; 2] = [
    (RAIO_PAREDE, [0.3, 0.2], 0.0, 6.0, "wall_ahead", "wall_gone"),
    (RAIO_BEIRADA, [0.6, -0.4], -90.0, 1.2, "ground", "edge_ahead"),
];

/// `(nome, x, acende com, apaga com)` — as luzes ficam no alto, apagadas.
const LUZES: [(&str, f32, &str, &str); 2] = [
    ("Wall Light", 2.0, "wall_ahead", "wall_gone"),
    ("Edge Light", -2.0, "edge_ahead", "ground"),
];
const Y_LUZ: f32 = 3.2;
const LUZ_LADO: f32 = 0.6;

const FUNDO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const CHAO_RGBA: [f32; 4] = [0.30, 0.32, 0.36, 1.0];
const PAREDE_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];
const LUZ_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
const SUPORTE_RGBA: [f32; 4] = [0.26, 0.27, 0.30, 1.0];

/// As portas do menu que a cena usa (a mesma rota do artista).
pub struct Pecas<'a> {
    pub estatico: &'a dyn Fn(&mut SimWorld) -> Entity,
    pub imagem: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// Add ▸ «Character».
    pub heroi: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// Add ▸ «Ray Sensor» com `pai` selecionado.
    pub raio: &'a dyn Fn(&mut SimWorld, Entity) -> Entity,
}

fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2]) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert(Name::new(nome));
    if let Some(mut t) = em.get_mut::<Transform>() {
        t.translation = Vec2::new(at[0], at[1]);
    }
}

fn bloco(sim: &mut SimWorld, p: &Pecas<'_>, nome: &str, (c, h): ([f32; 2], [f32; 2])) {
    let e = (p.estatico)(sim);
    arruma(sim, e, nome, c);
    let w = sim.world_mut();
    w.entity_mut(e).insert(Sprite::atlas(
        WHITE_TILE_KEY,
        [2.0 * h[0], 2.0 * h[1]],
        if nome == "Wall" { PAREDE_RGBA } else { CHAO_RGBA },
    ));
    let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
    if let Some(mut col) = w.get_mut::<Collider>(f) {
        col.shape = ColliderShape::Cuboid {
            half_x: h[0],
            half_y: h[1],
        };
    }
}

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

fn luz(sim: &mut SimWorld, p: &Pecas<'_>, nome: &str, x: f32, acende: &str, apaga: &str) {
    let suporte = (p.imagem)(sim);
    arruma(sim, suporte, &format!("{nome} Base"), [x, Y_LUZ]);
    let lado = LUZ_LADO * 1.4;
    sim.world_mut()
        .entity_mut(suporte)
        .insert(Sprite::atlas(WHITE_TILE_KEY, [lado, lado], SUPORTE_RGBA));
    let l = (p.imagem)(sim);
    arruma(sim, l, nome, [x, Y_LUZ]);
    sim.world_mut().entity_mut(l).insert((
        Sprite::atlas(WHITE_TILE_KEY, [LUZ_LADO, LUZ_LADO], LUZ_RGBA),
        Visibility::hidden(),
        SignalActions(vec![
            linha(acende, SignalVerb::Show),
            linha(apaga, SignalVerb::Hide),
        ]),
    ));
}

/// Monta a cena e devolve o raio escolhido («Wall Ray»).
pub fn montar(sim: &mut SimWorld, p: Pecas<'_>) -> Entity {
    let fundo = (p.imagem)(sim);
    arruma(sim, fundo, "Background", FUNDO.0);
    sim.world_mut().entity_mut(fundo).insert((
        Sprite::atlas(WHITE_TILE_KEY, FUNDO.1, FUNDO_RGBA),
        ZIndexOverride(-1),
    ));
    bloco(sim, &p, "Floor", CHAO);
    bloco(sim, &p, "Wall", PAREDE);
    bloco(sim, &p, "Pit Floor", POCO);
    for (nome, x, acende, apaga) in LUZES {
        luz(sim, &p, nome, x, acende, apaga);
    }
    let heroi = (p.heroi)(sim);
    arruma(sim, heroi, HEROI, HEROI_EM);
    let mut escolhido = heroi;
    for (nome, desvio, angle, reach, ve, perde) in RAIOS {
        let r = (p.raio)(sim, heroi);
        arruma(sim, r, nome, desvio);
        let w = sim.world_mut();
        if let Some(mut s) = w.get_mut::<RaySensor>(r) {
            s.angle = angle;
            s.reach = reach;
        }
        w.entity_mut(r).insert(RaySignals {
            on_enter: ve.to_owned(),
            on_exit: perde.to_owned(),
        });
        if nome == RAIO_PAREDE {
            escolhido = r;
        }
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =26 o Hero tem dois Ray Sensor como filhos na Hierarchy: «Wall Ray» olha para a frente \
         (selecionado) e «Ledge Ray» olha para baixo, à frente dos pés. Play (F5) e seta para a direita: a luz \
         da direita acende quando a parede entra no raio, a da esquerda quando o chão acaba."
    );
    escolhido
}

/// O cartão Ray Sensor e o grupo Signals abertos, e o Inspector no raio escolhido.
#[must_use]
pub fn gavetas(cartao: NodeId) -> Vec<(NodeId, bool)> {
    vec![
        (cartao, false),
        (ids::INSP_LIVE_RAY_SECTION, false),
        (ids::INSP_RAY_FOLD_SIGNALS, false),
    ]
}

pub fn arruma_o_inspector(store: &mut WidgetStore, cartao: NodeId, raio: u64) {
    for (id, fechada) in gavetas(cartao) {
        store.set_collapsed(id, fechada);
    }
    store.request_inspector_section(ids::INSP_LIVE_RAY_SECTION, raio);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_dois_raios_diferem_no_angulo_e_nas_vozes() {
        let [(_, _, a, _, ve_a, perde_a), (_, _, b, _, ve_b, perde_b)] = RAIOS;
        assert_ne!(a, b, "um olha para a frente e o outro para baixo");
        for n in [ve_a, perde_a] {
            assert!(![ve_b, perde_b].contains(&n), "nomes partilhados: {n}");
        }
        // Cada luz ouve um raio: acende com um nome e apaga com o outro extremo do MESMO raio.
        for (luz, _, acende, apaga) in LUZES {
            let dono = RAIOS
                .iter()
                .find(|r| [r.4, r.5].contains(&acende))
                .unwrap_or_else(|| panic!("{luz}: ninguém publica «{acende}»"));
            assert!([dono.4, dono.5].contains(&apaga), "{luz}");
        }
    }

    #[test]
    fn o_chao_acaba_antes_da_parede() {
        let fim_do_chao = CHAO.0[0] + CHAO.1[0];
        let face_da_parede = PAREDE.0[0] - PAREDE.1[0];
        assert!(fim_do_chao + 1.0 < face_da_parede, "a beirada tem de vir antes");
        assert!(HEROI_EM[0] < fim_do_chao);
    }
}
