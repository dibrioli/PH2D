//! ⭐⭐ **A cena 8 do editor de scripts** (`PH2D_CODE_SMOKE=8`, fase 8d —
//! `docs/CodeEditor/08_scripts_tocam_propriedades.md` §5 d5): os scripts tocam as propriedades do
//! Inspector PELO NOME. O herói azul corre o `aventura.luau` e anda sozinho para a direita: os espinhos
//! tiram-lhe vida (`ph2d.damage`), a moeda sobe o `health.max` e cura, a porta troca de imagem ao
//! chegar (`sprite.image`, `door_closed` → `door_open`) e passa a chamar-se `Door Open` (o 11,
//! `ph2d.set(e, "name", …)`), e a câmera, que a procura por ESSE nome, aproxima (`game_camera.height`). As
//! duas barras por cima dele são sprites que o script estica pelo que LÊ (`health.max`, `health.now`).
//! Rebobinar devolve tudo: a porta fechada, o máximo 100, a câmera de longe.
//!
//! As imagens da porta são ficheiros (`docs/CodeEditor/ferramentas/smoke/cena8/`), para a biblioteca
//! as chamar pelo nome do ficheiro como chama as do artista; sobem ao átlas em [`sobe_as_portas`], a
//! fase da shell que tem o renderer.

use crate::code_smoke::{SHAPE_R, tinta};
use bevy_ecs::prelude::Resource;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, GameCamera, Name, SimWorld, Transform, Visibility};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, Health, RigidBody};
use ph2d_render::{Sprite, SpriteRenderer, SpriteSource, WHITE_TILE_KEY};
use ph2d_script::{LuauScript, ProjectScripts};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::{VecScene, regular_polygon, star};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const AVENTURA_NAME: &str = "aventura.luau";

/// ⭐ **O script do herói** — cada bloco é um nome do Inspector que o script lê ou escreve.
pub const AVENTURA_LUAU: &str = r#"-- aventura.luau — o QUADRADO AZUL (o herói) corre ESTE script e anda sozinho para a direita.
-- Cada coisa que acontece é o script a ler ou a escrever uma propriedade do Inspector PELO NOME.
-- Rebobinar (o botão de voltar ao início): tudo volta como estava.

ph2d.property("speed", 1.5, { min = 0, max = 5 }) -- quão depressa anda (metros por segundo)

function init(self)
  self.door = ph2d.object("Door")
  self.coin = ph2d.object("Coin")
  self.spikes = ph2d.object("Spikes")
  self.camera = ph2d.object("Camera")
  self.life = ph2d.object("Life")
  self.life_max = ph2d.object("Life Max")
  -- O console mostra os nomes que a câmera tem (os mesmos do Inspector):
  print(table.concat(ph2d.properties(self.camera), "  "))
end

-- Uma barra de `pontos` de comprimento (50 pontos = 1 metro), encostada à esquerda, por cima do herói.
local function barra(bar, x, y, pontos)
  local largura = pontos / 50
  ph2d.set(bar, "scale_x", largura)
  ph2d.set(bar, "x", x - 1 + largura / 2)
  ph2d.set(bar, "y", y + 1)
end

function update(self, dt)
  local x = ph2d.get(self.id, "x")
  local y = ph2d.get(self.id, "y")
  local porta_x = ph2d.get(self.door, "x")

  -- ① Os espinhos tiram 40 de vida (a barra verde encolhe).
  if not self.ferido and x >= ph2d.get(self.spikes, "x") then
    self.ferido = true
    ph2d.damage(self.id, 40)
  end

  -- ② A moeda: o máximo da vida sobe 50 (a barra escura cresce) e cura 50.
  if not self.moeda and x >= ph2d.get(self.coin, "x") then
    self.moeda = true
    ph2d.set(self.id, "health.max", ph2d.get(self.id, "health.max") + 50)
    ph2d.heal(self.id, 50)
    ph2d.set(self.coin, "visibility.visible", false)
  end

  -- ③ A porta troca de imagem quando o herói chega (o nome é o da imagem no painel Assets)
  --   e o script dá-lhe um NOME NOVO: "Door Open".
  if not self.aberta and x >= porta_x - 1.5 then
    self.aberta = true
    ph2d.set(self.door, "sprite.image", "door_open")
    ph2d.set(self.door, "name", "Door Open")
    print("A porta chama-se agora: Door Open")
  end

  -- ④ A câmera procura a porta PELO NOME NOVO; quando a acha, aproxima até 5 metros de altura.
  --   (A Hierarquia continua a dizer "Door": ela mostra o projecto, não o jogo a correr.)
  local aberta = ph2d.find_by_name("Door Open")
  if aberta then
    local altura = ph2d.get(self.camera, "game_camera.height")
    ph2d.set(self.camera, "game_camera.height", math.max(5, altura - 4 * dt))
    local cx = ph2d.get(self.camera, "x")
    local ax = ph2d.get(aberta, "x")
    ph2d.set(self.camera, "x", cx + (ax - cx) * math.min(1, 2 * dt))
  end

  local alvo = if self.aberta then porta_x else porta_x - 1.5
  x = math.min(alvo, x + self.speed * dt)
  ph2d.set(self.id, "x", x)

  -- As barras: a escura é o máximo, a verde é a vida de agora — o script LÊ as duas.
  barra(self.life_max, x, y, ph2d.get(self.id, "health.max"))
  barra(self.life, x, y, ph2d.get(self.id, "health.now"))
end
"#;

/// A pasta das imagens da porta (o nome do ficheiro É o nome na biblioteca).
fn pasta_das_portas() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/CodeEditor/ferramentas/smoke/cena8")
}

/// As duas imagens: a autorada (fechada) primeiro.
pub const PORTAS: [&str; 2] = ["door_closed.png", "door_open.png"];

/// Onde as peças nascem (metros): herói, espinhos, moeda, porta, câmera.
pub const AT: [[f32; 2]; 5] = [
    [-5.0, 0.0],
    [-2.5, -0.25],
    [0.0, 0.0],
    [3.0, 0.5],
    [-1.0, 0.5],
];
/// A vida autorada do herói e a altura inicial da câmera (metros).
pub const VIDA: f32 = 100.0;
pub const ALTURA: f32 = 16.0;

/// A marca de que a porta espera as imagens (a cena monta-se sem o renderer; [`sobe_as_portas`] tira-a).
#[derive(Resource)]
pub struct PortaPorPintar(pub Entity);

fn sprite(
    w: &mut ph2d_ecs::World,
    nome: &str,
    at: [f32; 2],
    size: [f32; 2],
    rgba: [f32; 4],
) -> Entity {
    w.spawn((
        Name::new(nome),
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Visibility::visible(),
        Sprite::atlas(WHITE_TILE_KEY, size, rgba),
    ))
    .id()
}

/// ⭐ **A cena 8.** Devolve `[herói, espinhos, moeda, porta, câmera, vida, máximo]`.
pub fn monta(sim: &mut SimWorld, cena: &mut VecScene, mapa: &mut VecEntityMap) -> Vec<Entity> {
    let script = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(AVENTURA_NAME, AVENTURA_LUAU);
    let r = SHAPE_R * 0.6;
    let formas = [
        (
            "Spikes",
            tinta(regular_polygon([0.0, 0.0], r, r, 3), [232, 96, 80]),
        ),
        (
            "Coin",
            tinta(star([0.0, 0.0], r, r, 5, 0.45), [240, 196, 72]),
        ),
    ];
    let ids: Vec<_> = formas
        .into_iter()
        .map(|(nome, p)| (nome, cena.push_path(p)))
        .collect();
    sync(sim, cena, mapa);
    let mut vec_ents = Vec::new();
    for (i, (nome, pid)) in ids.into_iter().enumerate() {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        let at = AT[i + 1];
        sim.world_mut().entity_mut(e).insert((
            Name::new(nome),
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ));
        vec_ents.push(e);
    }
    let w = sim.world_mut();
    // A porta nasce antes: o herói entra e fica À FRENTE dela.
    let porta = sprite(w, "Door", AT[3], [1.0, 2.0], [1.0, 1.0, 1.0, 1.0]);
    let heroi = sprite(w, "Hero", AT[0], [1.0, 1.0], [0.38, 0.66, 0.92, 1.0]);
    w.entity_mut(heroi).insert((
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Health {
            max: VIDA,
            start: VIDA,
            ..Health::default()
        },
        LuauScript::library(script),
    ));
    let camera = w
        .spawn((
            Name::new("Camera"),
            Transform::from_translation(Vec2::new(AT[4][0], AT[4][1])),
            GameCamera {
                height_world: ALTURA,
                ..GameCamera::default()
            },
        ))
        .id();
    // A escura (o máximo) por baixo da verde (a vida de agora): nasce primeiro.
    let maximo = sprite(
        w,
        "Life Max",
        [AT[0][0], AT[0][1] + 1.0],
        [1.0, 0.2],
        [0.18, 0.2, 0.24, 1.0],
    );
    let vida = sprite(
        w,
        "Life",
        [AT[0][0], AT[0][1] + 1.0],
        [1.0, 0.2],
        [0.36, 0.82, 0.42, 1.0],
    );
    w.insert_resource(PortaPorPintar(porta));
    ph2d_ecs::assign_missing_stable_ids(w);
    let mut out = vec![heroi];
    out.extend(vec_ents);
    out.extend([porta, camera, vida, maximo]);
    out
}

/// ⭐ **As imagens da porta sobem ao átlas** — uma vez, no 1.º quadro com o renderer depois de a cena
/// montar (no-op sem a [`PortaPorPintar`]). A fechada vira a imagem da porta; a aberta fica na
/// biblioteca, que a chama pelo nome do ficheiro. Devolve as duas imagens.
pub fn sobe_as_portas(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    db: &AssetDb,
    next_cell: &mut u32,
    atlas_assets: &mut BTreeMap<u32, AssetId>,
) -> Option<[AssetId; 2]> {
    let porta = sim.world_mut().remove_resource::<PortaPorPintar>()?.0;
    let mut ids = Vec::new();
    let mut fechada = None;
    for nome in PORTAS {
        let path = pasta_das_portas().join(nome);
        let id = match db.load_png_path(&path) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("[code-smoke] =8 a imagem {} não abriu: {e}", path.display());
                return None;
            }
        };
        let asset = db.get(&id)?;
        let (w, h, px) = asset.image_rgba8()?;
        let key = *next_cell;
        *next_cell += 1;
        atlas_assets.insert(key, id);
        let fetch = |k: u32| -> Option<Vec<u8>> {
            db.get(atlas_assets.get(&k)?)?
                .image_rgba8()
                .map(|(_, _, p)| p.into_owned())
        };
        if renderer
            .insert_atlas_sprite_with_regrow(key, w, h, &px, fetch)
            .is_err()
        {
            atlas_assets.remove(&key);
            return None;
        }
        fechada.get_or_insert(key);
        ids.push(id);
    }
    if let (Some(key), Some(mut s)) = (fechada, sim.world_mut().get_mut::<Sprite>(porta)) {
        s.source = SpriteSource::Atlas { key };
    }
    Some([ids[0], ids[1]])
}

/// A porta da shell: monta, liga a vista pela câmera da cena e anuncia.
pub fn montar(cx: crate::scene_ctx::SceneCtx<'_>) -> Option<u64> {
    let pecas = monta(cx.sim, cx.vec_scene, cx.vec_entities);
    *cx.camera_preview = true;
    println!(
        "[code-smoke] =8 propriedades pelo nome: o herói azul corre o aventura.luau e anda para a \
         direita — os espinhos tiram vida, a moeda sobe o máximo, a porta troca de imagem e passa a \
         chamar-se Door Open, e a câmera, que a procura por esse nome, aproxima. Rebobinar devolve tudo."
    );
    pecas.first().map(|e| e.to_bits())
}

#[cfg(test)]
#[path = "code_smoke_props_tests.rs"]
mod tests;
