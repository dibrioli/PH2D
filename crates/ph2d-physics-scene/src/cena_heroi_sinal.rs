//! ⭐ **Cena 28 — o herói manda** (GEF 08 Motion Input, onda 3; `PH2D_GAME_SMOKE=28` com o grafo
//! `PH2D_GPU_COOK_DEMO=143`).
//!
//! O «Hero» (Add ▸ Character, setas/WASD, com Health) tem DOIS filhos Motion Input: «vida» (Health → a barra) e
//! «rastro» (Velocity, o par → os dois ventos do rastro de estrelas). Quatro pedras com Damage (20) caem sobre ele
//! no Play: a vida cruza 0.25 e o nó «Send signal» manda `low_health` uma vez. Ouvem-no a regra da «Warning Lamp»
//! (When signal low_health → Show) e o script da «Script Meter» (`on_signal` e `motion.get("vida")`). Abre PARADA
//! com o Inspector no filho «rastro». Ficha: `docs/GameEngineFuncional/componentes/08_motion_input.md`.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, MotionInput, MotionSource, Name, SignalAction, SignalActions, SignalVerb, SimWorld,
    Transform, Visibility,
};
use ph2d_physics_ecs::{Collider, ColliderShape, Damage, Health, OnHit, ProjectileMotion};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

pub const CENA: u32 = 28;

pub const HEROI: &str = "Hero";
pub const CHAO: &str = "Floor";
pub const LAMPADA: &str = "Warning Lamp";
pub const MEDIDOR: &str = "Script Meter";
pub const PEDRA: &str = "Rock";
/// Os nomes do grafo `=143`.
pub const ENTRADA_VIDA: &str = "vida";
pub const ENTRADA_RASTRO: &str = "rastro";
pub const SINAL: &str = "low_health";
/// `(entrada, fonte)` dos filhos do «Hero», pela ordem em que nascem.
pub const FILHOS: [(&str, MotionSource); 2] = [
    (ENTRADA_VIDA, MotionSource::Health),
    (ENTRADA_RASTRO, MotionSource::Velocity),
];
/// Quatro pedras de 20 tiram 80 %: a vida passa por 0.4 e cai a 0.2 (cruza 0.25 uma vez, viva).
pub const GOLPE: f32 = 20.0;
pub const RAPIDEZ: f32 = 1.0;
pub const HEROI_AT: [f32; 2] = [-1.5, -2.0];
/// As alturas das pedras acima do herói: uma chega por segundo.
pub const PEDRAS: [f32; 4] = [1.6, 2.6, 3.6, 4.6];
const RAIO: f32 = 0.15;
pub const CHAO_AT: ([f32; 2], [f32; 2]) = ([0.0, -3.0], [8.0, 0.25]);
/// A «Warning Lamp» e a «Script Meter» à esquerda da barra (y 2,6) do grafo `=143`.
pub const LAMPADA_AT: [f32; 2] = [-5.2, 2.6];
pub const MEDIDOR_AT: [f32; 2] = [-5.2, 1.4];
pub const SCRIPT_NOME: &str = "meter.luau";
/// O script: a largura segue o que o grafo lê em «vida»; ao ouvir `low_health` engorda e escreve no Output.
pub const SCRIPT: &str = r#"function update(self, dt)
  local v = motion.get("vida")
  if v then ph2d.set(self.id, "scale_x", v) end
end

function on_signal(self, signal, from)
  if signal == "low_health" then
    ph2d.set(self.id, "scale_y", 2)
    print("Low health! vida = " .. tostring(motion.get("vida")))
  end
end
"#;
const CHAO_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];
const PEDRA_RGBA: [f32; 4] = [0.95, 0.35, 0.30, 1.0];
const LAMPADA_RGBA: [f32; 4] = [1.0, 0.25, 0.2, 1.0];
const MEDIDOR_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];

pub struct Pecas<'a> {
    pub estatico: &'a dyn Fn(&mut SimWorld) -> Entity,
    pub imagem: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// Add ▸ «Character».
    pub heroi: &'a dyn Fn(&mut SimWorld) -> Entity,
    pub anexa: &'a dyn Fn(&mut SimWorld, u64, &str) -> Result<(), String>,
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

fn anexa(sim: &mut SimWorld, p: &Pecas<'_>, e: Entity, nome: &str) {
    if let Err(erro) = (p.anexa)(sim, e.to_bits(), nome) {
        eprintln!("[game-smoke] =28 {nome}: {erro}");
    }
}

fn forma(sim: &mut SimWorld, e: Entity, shape: ColliderShape) {
    let w = sim.world_mut();
    let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
    if let Some(mut col) = w.get_mut::<Collider>(f) {
        col.shape = shape;
    }
}

/// Monta a cena e devolve o filho «Motion Input · rastro» (o que o Inspector abre).
pub fn montar(sim: &mut SimWorld, p: Pecas<'_>) -> Entity {
    let (c, h) = CHAO_AT;
    let chao = (p.estatico)(sim);
    arruma(sim, chao, CHAO, c);
    sim.world_mut().entity_mut(chao).insert(Sprite::atlas(
        WHITE_TILE_KEY,
        [2.0 * h[0], 2.0 * h[1]],
        CHAO_RGBA,
    ));
    let (half_x, half_y) = (h[0], h[1]);
    forma(sim, chao, ColliderShape::Cuboid { half_x, half_y });

    let heroi = (p.heroi)(sim);
    arruma(sim, heroi, HEROI, HEROI_AT);
    anexa(sim, &p, heroi, "ph2d::physics::Health");
    if let Some(mut vida) = sim.world_mut().get_mut::<Health>(heroi) {
        vida.invincible_s = 0.0; // um golpe por pedra
    }
    let mut rastro = None;
    for (nome, source) in FILHOS {
        let e = ph2d_physics_ecs::pendura_entrada(sim.world_mut(), None, Some(heroi), "");
        anexa(sim, &p, e, "ph2d::ecs::MotionInput");
        let mi = MotionInput {
            name: nome.to_owned(),
            source,
            ..MotionInput::default()
        };
        sim.world_mut().entity_mut(e).insert(mi);
        if nome == ENTRADA_RASTRO {
            rastro = Some(e);
        }
    }
    for k in PEDRAS {
        let e = (p.imagem)(sim);
        arruma(sim, e, PEDRA, [HEROI_AT[0], HEROI_AT[1] + k]);
        let sprite = Sprite::atlas(WHITE_TILE_KEY, [2.0 * RAIO; 2], PEDRA_RGBA);
        sim.world_mut().entity_mut(e).insert(sprite);
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
            t.rotation = -std::f32::consts::FRAC_PI_2; // para baixo
        }
    }
    let lampada = (p.imagem)(sim);
    arruma(sim, lampada, LAMPADA, LAMPADA_AT);
    let regra = SignalAction {
        on: SINAL.into(),
        verb: SignalVerb::Show,
        ..SignalAction::default()
    };
    sim.world_mut().entity_mut(lampada).insert((
        Sprite::atlas(WHITE_TILE_KEY, [0.7; 2], LAMPADA_RGBA),
        Visibility::hidden(),
        SignalActions(vec![regra]),
    ));
    let medidor = (p.imagem)(sim);
    arruma(sim, medidor, MEDIDOR, MEDIDOR_AT);
    let barra = Sprite::atlas(WHITE_TILE_KEY, [2.0, 0.3], MEDIDOR_RGBA);
    sim.world_mut().entity_mut(medidor).insert(barra);
    (p.script)(sim, medidor, SCRIPT_NOME, SCRIPT);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =28 o herói manda: o «Hero» (setas/WASD) tem dois filhos Motion Input — «vida» (Health → a \
         barra verde) e «rastro» (Velocity → as estrelas voam para onde ele corre). Play (F5) e fique parado: \
         quatro pedras tiram 80 % da vida; abaixo de 0.25 o nó Send signal manda «low_health» uma vez — a \
         «Warning Lamp» vermelha aparece (a regra) e a «Script Meter» amarela engorda (o script; a largura dela \
         é o motion.get(\"vida\")). Use com PH2D_GPU_COOK_DEMO=143 (o grafo)."
    );
    rastro.expect("o filho rastro nasceu")
}
