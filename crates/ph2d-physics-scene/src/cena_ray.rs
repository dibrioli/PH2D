//! ⭐ **Cena 26 — o Ray Sensor é um OBJETO** (GEF 07, versão B; `PH2D_GAME_SMOKE=26`).
//!
//! O herói do menu Add (Character) leva DOIS raios filhos, criados pelo Add ▸ «Ray Sensor» com ele
//! selecionado: «Ledge Ray» à frente dos pés, a olhar para baixo (há chão?), e «Wall Ray» à altura do
//! peito, a olhar para a frente (há parede?). O chão acaba antes da parede, que desce até ao fundo do poço. No Play, anda-se para a
//! direita: a luz «Wall Light» acende quando a parede entra no alcance e a «Edge Light» quando o chão
//! acaba à frente. O «Wall Ray» nasce escolhido, com o cartão Ray Sensor e o grupo Signals abertos.
//!
//! **O filtro (versão C):** um terceiro raio, «Eye Ray», olha para cima e para a frente, onde um «Enemy» (tag
//! «Enemy», criada na árvore do projecto) está atrás de uma «Crate». Com «Only see» = Only tag Enemy e «Walls
//! block» desligado, a caixa é transparente para ele: a «Eye Light» acende já no Play, de pé. Ligar «Walls
//! block» apaga-a (a caixa tapa o inimigo).
//!
//! **As melhorias (RS, dono 15/11):** um «Guard» no alto, à direita, com o «Guard Eye» — um raio em LEQUE
//! (Cone 50°) a olhar para baixo e para a esquerda, só para a tag «Player» do herói: a «Alarm Light» acende
//! quando o herói entra no leque (perto da beirada) e apaga quando sai. E uma «Turret» no tecto, com um
//! script que dá um TIRO instantâneo a cada quadro (`ph2d.raycast`, reto para baixo, só para o «Player»): a
//! «Shot Light» acende enquanto o herói passa por baixo dela.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Name, SignalAction, SignalActions, SignalFrom, SignalTarget, SignalVerb, SimWorld,
    Transform, Visibility, ZIndexOverride,
};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::{NodeId, ids};
use ph2d_physics_ecs::{Collider, ColliderShape, RaySensor, RaySignals, SeeFilter};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

pub const CENA: u32 = 26;

pub const HEROI: &str = "Hero";
pub const RAIO_PAREDE: &str = "Wall Ray";
pub const RAIO_BEIRADA: &str = "Ledge Ray";
pub const RAIO_OLHO: &str = "Eye Ray";
pub const INIMIGO: &str = "Enemy";
pub const CAIXA: &str = "Crate";
/// A tag do inimigo, na árvore de tags do projecto.
pub const TAG_INIMIGO: &str = "Enemy";
pub const GUARDA: &str = "Guard";
pub const RAIO_GUARDA: &str = "Guard Eye";
pub const TORRETA: &str = "Turret";
/// A tag do herói — o que o guarda e a torreta procuram.
pub const TAG_HEROI: &str = "Player";
/// O guarda no alto, à direita; o leque dele olha para baixo e para a esquerda, até à beirada.
pub const GUARDA_EM: [f32; 2] = [4.5, 1.2];
/// `(ângulo, abertura, alcance)` do «Guard Eye».
pub const LEQUE_DO_GUARDA: (f32, f32, f32) = (211.0, 50.0, 6.5);
/// A torreta no tecto, sobre o chão: o tiro desce 3 m.
pub const TORRETA_EM: [f32; 2] = [0.6, 0.8];
/// O nome do script da torreta na biblioteca do projecto.
pub const SCRIPT_DA_TORRETA: &str = "turret.luau";
/// ⭐ O script da torreta: um tiro por quadro, a resposta na hora; diz-se só quando muda.
pub const TORRETA_LUAU: &str = r#"-- Turret: an instant shot straight down every frame (ph2d.raycast).
-- The Shot Light shows while the hero is under it.
local seen = false

function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  local y = ph2d.get(self.id, "y") or 0
  local hit = ph2d.raycast(ph2d.vec2(x, y), ph2d.vec2(x, y - 3), { only_see = "only_tag", tag = "Player" })[1]
  local now = hit ~= nil
  if now ~= seen then
    seen = now
    ph2d.emit(if now then "turret_hit" else "turret_miss")
  end
end
"#;

/// O herói nasce aqui; o chão vai de `x = -7` a `x = 1,5`, a parede está em `x = 6`.
pub const HEROI_EM: [f32; 2] = [-5.0, -1.0];
const FUNDO: ([f32; 2], [f32; 2]) = ([0.0, 0.0], [16.0, 9.0]);
pub const CHAO: ([f32; 2], [f32; 2]) = ([-2.75, -2.25], [4.25, 0.25]);
pub const PAREDE: ([f32; 2], [f32; 2]) = ([6.0, -1.25], [0.3, 2.75]);
const POCO: ([f32; 2], [f32; 2]) = ([5.0, -4.25], [2.0, 0.25]);
/// No caminho do «Eye Ray» de pé (a 30° da origem `(-4,7 ; -0,4)`): a caixa a ~3 m, o inimigo a ~5 m, ambos
/// acima da cabeça do herói e longe dos outros dois raios.
pub const CAIXA_EM: ([f32; 2], [f32; 2]) = ([-2.1, 1.1], [0.4, 0.4]);
pub const INIMIGO_EM: ([f32; 2], [f32; 2]) = ([-0.37, 2.1], [0.35, 0.45]);

/// `(nome, desvio no herói, ângulo, alcance, ao ver, ao deixar de ver)`.
type Raio = (&'static str, [f32; 2], f32, f32, &'static str, &'static str);
pub const RAIOS: [Raio; 3] = [
    (RAIO_PAREDE, [0.3, 0.2], 0.0, 6.0, "wall_ahead", "wall_gone"),
    (
        RAIO_BEIRADA,
        [0.6, -0.4],
        -90.0,
        1.2,
        "ground",
        "edge_ahead",
    ),
    (RAIO_OLHO, [0.3, 0.6], 30.0, 7.0, "enemy_seen", "enemy_gone"),
];

/// `(nome, x, acende com, apaga com)` — as luzes ficam no alto, apagadas.
const LUZES: [(&str, f32, &str, &str); 5] = [
    ("Wall Light", 2.0, "wall_ahead", "wall_gone"),
    ("Edge Light", -2.0, "edge_ahead", "ground"),
    ("Eye Light", 0.0, "enemy_seen", "enemy_gone"),
    ("Alarm Light", 4.0, "spotted", "lost"),
    ("Shot Light", -4.0, "turret_hit", "turret_miss"),
];
const Y_LUZ: f32 = 3.2;
const LUZ_LADO: f32 = 0.6;

const FUNDO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const CHAO_RGBA: [f32; 4] = [0.30, 0.32, 0.36, 1.0];
const PAREDE_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];
const LUZ_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
const SUPORTE_RGBA: [f32; 4] = [0.26, 0.27, 0.30, 1.0];
const CAIXA_RGBA: [f32; 4] = [0.55, 0.40, 0.24, 1.0];
const INIMIGO_RGBA: [f32; 4] = [0.85, 0.25, 0.25, 1.0];
const GUARDA_RGBA: [f32; 4] = [0.30, 0.45, 0.80, 1.0];
const TORRETA_RGBA: [f32; 4] = [0.60, 0.62, 0.66, 1.0];

/// As portas do menu que a cena usa (a mesma rota do artista).
pub struct Pecas<'a> {
    pub estatico: &'a dyn Fn(&mut SimWorld) -> Entity,
    pub imagem: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// Add ▸ «Character».
    pub heroi: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// Add ▸ «Ray Sensor» com `pai` selecionado.
    pub raio: &'a dyn Fn(&mut SimWorld, Entity) -> Entity,
    /// Marca o objecto com a tag (criada na árvore de tags do projecto se falta); o id.
    pub marca: &'a dyn Fn(&mut SimWorld, Entity, &str) -> u64,
    /// Põe no objecto o script `(nome, fonte)` da biblioteca do projecto.
    pub script: &'a dyn Fn(&mut SimWorld, Entity, &str, &str),
}

fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2]) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert(Name::new(nome));
    if let Some(mut t) = em.get_mut::<Transform>() {
        t.translation = Vec2::new(at[0], at[1]);
    }
}

fn bloco(
    sim: &mut SimWorld,
    p: &Pecas<'_>,
    nome: &str,
    (c, h): ([f32; 2], [f32; 2]),
    rgba: [f32; 4],
) -> Entity {
    let e = (p.estatico)(sim);
    arruma(sim, e, nome, c);
    let w = sim.world_mut();
    w.entity_mut(e).insert(Sprite::atlas(
        WHITE_TILE_KEY,
        [2.0 * h[0], 2.0 * h[1]],
        rgba,
    ));
    let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
    if let Some(mut col) = w.get_mut::<Collider>(f) {
        col.shape = ColliderShape::Cuboid {
            half_x: h[0],
            half_y: h[1],
        };
    }
    e
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
    sim.world_mut().entity_mut(suporte).insert(Sprite::atlas(
        WHITE_TILE_KEY,
        [lado, lado],
        SUPORTE_RGBA,
    ));
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
    bloco(sim, &p, "Floor", CHAO, CHAO_RGBA);
    bloco(sim, &p, "Wall", PAREDE, PAREDE_RGBA);
    bloco(sim, &p, "Pit Floor", POCO, CHAO_RGBA);
    bloco(sim, &p, CAIXA, CAIXA_EM, CAIXA_RGBA);
    let inimigo = bloco(sim, &p, INIMIGO, INIMIGO_EM, INIMIGO_RGBA);
    let tag = (p.marca)(sim, inimigo, TAG_INIMIGO);
    for (nome, x, acende, apaga) in LUZES {
        luz(sim, &p, nome, x, acende, apaga);
    }
    let heroi = (p.heroi)(sim);
    arruma(sim, heroi, HEROI, HEROI_EM);
    let jogador = (p.marca)(sim, heroi, TAG_HEROI);
    let olho_do_guarda = guarda(sim, &p, jogador);
    let torreta = (p.imagem)(sim);
    arruma(sim, torreta, TORRETA, TORRETA_EM);
    sim.world_mut().entity_mut(torreta).insert(Sprite::atlas(
        WHITE_TILE_KEY,
        [0.5, 0.3],
        TORRETA_RGBA,
    ));
    (p.script)(sim, torreta, SCRIPT_DA_TORRETA, TORRETA_LUAU);
    let pedida = foto(pedida().as_deref());
    let mut escolhido = heroi;
    for (nome, desvio, angle, reach, ve, perde) in RAIOS {
        let r = (p.raio)(sim, heroi);
        arruma(sim, r, nome, desvio);
        let w = sim.world_mut();
        if let Some(mut s) = w.get_mut::<RaySensor>(r) {
            s.angle = angle;
            s.reach = reach;
            s.turns_with_facing = true; // vira com o herói (a opção, ligada na cena)
            if nome == RAIO_OLHO {
                s.only_see = SeeFilter::OnlyTag;
                s.see_tag = tag;
                s.walls_block = false; // vê o inimigo através da caixa
            }
        }
        w.entity_mut(r).insert(RaySignals {
            on_enter: ve.to_owned(),
            on_exit: perde.to_owned(),
        });
        let quer = if pedida == Some(Foto::Filter) {
            RAIO_OLHO
        } else {
            RAIO_PAREDE
        };
        if nome == quer {
            escolhido = r;
        }
    }
    if pedida == Some(Foto::Cone) {
        escolhido = olho_do_guarda;
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =26 o Hero tem três Ray Sensor como filhos na Hierarchy: «Wall Ray» olha para a frente \
         (selecionado), «Ledge Ray» olha para baixo, à frente dos pés, e «Eye Ray» olha para o alto, só para a \
         tag Enemy e sem Walls block — vê o Enemy através da Crate. O Guard (alto, à direita) tem o «Guard Eye», \
         um leque de 50°; a Turret (no tecto) dá um tiro por script a cada quadro. Play (F5): a luz do meio \
         acende já; seta para a direita: a Shot Light pisca ao passar sob a Turret, a Alarm Light acende quando \
         o herói entra no leque do Guard, a Wall Light com a parede e a Edge Light quando o chão acaba."
    );
    escolhido
}

/// O «Guard» e o seu «Guard Eye» (o leque, só para o `Player`, que acende a «Alarm Light»). Devolve o raio.
fn guarda(sim: &mut SimWorld, p: &Pecas<'_>, jogador: u64) -> Entity {
    let g = (p.imagem)(sim);
    arruma(sim, g, GUARDA, GUARDA_EM);
    sim.world_mut()
        .entity_mut(g)
        .insert(Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.8], GUARDA_RGBA));
    let r = (p.raio)(sim, g);
    arruma(sim, r, RAIO_GUARDA, [0.0, 0.0]);
    let (angle, cone, reach) = LEQUE_DO_GUARDA;
    let w = sim.world_mut();
    if let Some(mut s) = w.get_mut::<RaySensor>(r) {
        s.angle = angle;
        s.cone = cone;
        s.reach = reach;
        s.only_see = SeeFilter::OnlyTag;
        s.see_tag = jogador;
    }
    w.entity_mut(r).insert(RaySignals {
        on_enter: "spotted".to_owned(),
        on_exit: "lost".to_owned(),
    });
    r
}

/// O instante das fotos do cartão (`PH2D_GAME_SMOKE_FOTO`); sem a variável a cena é a do dono.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foto {
    /// `ray_filter`: o «Eye Ray» escolhido, com o grupo Filter aberto.
    Filter,
    /// `ray_cone`: o «Guard Eye» escolhido (o leque), com o grupo Filter aberto.
    Cone,
}

/// Os nomes que [`foto`] aceita (censo em `ph2d_game_smoke_fotos::nomes`).
pub const NOMES: &str = "ray_filter ray_cone";

/// A foto pedida pelo nome.
#[must_use]
pub fn foto(nome: Option<&str>) -> Option<Foto> {
    match nome?.trim() {
        "ray_filter" => Some(Foto::Filter),
        "ray_cone" => Some(Foto::Cone),
        _ => None,
    }
}

fn pedida() -> Option<String> {
    std::env::var("PH2D_GAME_SMOKE_FOTO").ok()
}

/// O cartão Ray Sensor e o grupo Signals abertos (a foto `ray_filter` abre o Filter no lugar dele).
#[must_use]
pub fn gavetas(foto: Option<Foto>, cartao: NodeId) -> Vec<(NodeId, bool)> {
    let grupo = if matches!(foto, Some(Foto::Filter | Foto::Cone)) {
        ids::INSP_RAY_FOLD_FILTER
    } else {
        ids::INSP_RAY_FOLD_SIGNALS
    };
    vec![
        (cartao, false),
        (ids::INSP_LIVE_RAY_SECTION, false),
        (grupo, false),
    ]
}

/// Aplica as [`gavetas`] e pede ao Inspector o cartão Ray Sensor do `raio` escolhido.
pub fn arruma_o_inspector(store: &mut WidgetStore, cartao: NodeId, raio: u64) {
    for (id, fechada) in gavetas(foto(pedida().as_deref()), cartao) {
        store.set_collapsed(id, fechada);
    }
    store.request_inspector_section(ids::INSP_LIVE_RAY_SECTION, raio);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_raios_diferem_no_angulo_e_nas_vozes() {
        for (i, a) in RAIOS.iter().enumerate() {
            for b in &RAIOS[i + 1..] {
                assert_ne!(a.2, b.2, "{} e {} olham para o mesmo lado", a.0, b.0);
                for n in [a.4, a.5] {
                    assert!(![b.4, b.5].contains(&n), "nomes partilhados: {n}");
                }
            }
        }
        // Cada luz ouve um raio: acende com um nome e apaga com o outro extremo do MESMO raio.
        // O Guard Eye e a Turret (o script) publicam os seus dois nomes como os raios da tabela.
        let mut vozes: Vec<(&str, &str)> = RAIOS.iter().map(|r| (r.4, r.5)).collect();
        vozes.push(("spotted", "lost"));
        vozes.push(("turret_hit", "turret_miss"));
        assert!(
            TORRETA_LUAU.contains("\"turret_hit\"") && TORRETA_LUAU.contains("\"turret_miss\"")
        );
        for (luz, _, acende, apaga) in LUZES {
            let dono = vozes
                .iter()
                .find(|v| [v.0, v.1].contains(&acende))
                .unwrap_or_else(|| panic!("{luz}: ninguém publica «{acende}»"));
            assert!([dono.0, dono.1].contains(&apaga), "{luz}");
        }
    }

    /// O «Eye Ray» de pé passa pela caixa ANTES do inimigo, e nenhum dos dois toca a linha do «Wall Ray».
    #[test]
    fn o_olho_de_pe_atravessa_a_caixa_ate_ao_inimigo() {
        let (_, d, ang, alcance, _, _) = RAIOS[2];
        let o = [HEROI_EM[0] + d[0], HEROI_EM[1] + d[1]];
        let (s, c) = ang.to_radians().sin_cos();
        let entra = |(cen, h): ([f32; 2], [f32; 2])| {
            (0..=700).map(|i| i as f32 * alcance / 700.0).find(|t| {
                ((o[0] + c * t) - cen[0]).abs() <= h[0] && ((o[1] + s * t) - cen[1]).abs() <= h[1]
            })
        };
        let caixa = entra(CAIXA_EM).expect("a caixa está no caminho");
        let inimigo = entra(INIMIGO_EM).expect("o inimigo está no alcance");
        assert!(caixa < inimigo, "a caixa fica à frente do inimigo");
        let y_parede = HEROI_EM[1] + RAIOS[0].1[1];
        for (cen, h) in [CAIXA_EM, INIMIGO_EM] {
            assert!(cen[1] - h[1] > y_parede + 1.0, "fora da linha do Wall Ray");
        }
    }

    #[test]
    fn a_foto_ray_filter_abre_o_filter() {
        let c = ids::INSP_LIVE_PLAYER_SECTION;
        assert!(gavetas(None, c).contains(&(ids::INSP_RAY_FOLD_SIGNALS, false)));
        let f = gavetas(foto(Some("ray_filter")), c);
        assert!(f.contains(&(ids::INSP_RAY_FOLD_FILTER, false)), "{f:?}");
        assert_eq!(foto(Some("projectile_speed_change")), None);
    }

    #[test]
    fn o_chao_acaba_antes_da_parede() {
        let fim_do_chao = CHAO.0[0] + CHAO.1[0];
        let face_da_parede = PAREDE.0[0] - PAREDE.1[0];
        assert!(
            fim_do_chao + 1.0 < face_da_parede,
            "a beirada tem de vir antes"
        );
        assert!(HEROI_EM[0] < fim_do_chao);
    }
}
