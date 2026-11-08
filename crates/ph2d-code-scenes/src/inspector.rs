//! A cena 9 (arrastar nomes do Inspector): o script e as três peças.

use crate::{SHAPE_R, tinta};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::{VecScene, rectangle, regular_polygon, star};

pub const LIGAR_NAME: &str = "ligar.luau";

/// ⭐ **O script da cena 9.** Começa VÁLIDO: o cartão Script mostra as propriedades da última versão
/// boa do texto (o «giro» do passo ②; a «door» do ③ aparece na pausa da escrita, sem Ctrl+S — 1b). O
/// dono escreve o começo da linha e arrasta os nomes para o fim dela (o idioma da cena 7).
pub const LIGAR_LUAU: &str = r#"-- ligar.luau — a NAVE AZUL corre ESTE script. Aqui você arrasta NOMES do Inspector (à direita)
-- e objetos da Hierarquia (à esquerda) para o código. Para ver a cena mexer: Ctrl+S, e a aba «Cena».
ph2d.property("giro", 1, { min = 0, max = 5 })

-- ③ Segure Alt e arraste «Door» da Hierarquia para a LINHA VAZIA aqui em baixo. O campo «door»
--    aparece no cartão Script logo que você pára, SEM salvar:

local tempo = 0

function update(self, dt)
  tempo = tempo + dt

  -- ① Na linha vazia de baixo escreva  ph2d.set(self.id,  (o editor fecha o parêntese sozinho) e
  --    arraste o nome «Rotation» (cartão Transform) para logo ANTES do  )
  -- ② Escreva  , tempo *  e arraste o nome «giro» (abra o cartão Script) para logo ANTES do  )

  if self.door then
    ph2d.set(self.door, "y", math.sin(tempo * 2) * 1.5) -- o objeto do campo «door» sobe e desce
  end
end
"#;

/// Onde as peças nascem (metros): a nave, a porta e a lâmpada.
pub const LIGAR_AT: [[f32; 2]; 3] = [[-4.0, 0.0], [0.5, 0.0], [4.5, 0.0]];

/// ⭐ Monta a cena 9: `[Ship, Door, Lamp]`; só a nave tem script.
pub fn monta(sim: &mut SimWorld, cena: &mut VecScene, mapa: &mut VecEntityMap) -> Vec<Entity> {
    let script = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(LIGAR_NAME, LIGAR_LUAU);
    let r = SHAPE_R;
    let pecas = [
        (
            "Ship",
            tinta(regular_polygon([0.0, 0.0], r, r, 3), [96, 168, 235]),
        ),
        (
            "Door",
            tinta(
                rectangle([-r * 0.5, -r * 1.5], [r * 0.5, r * 1.5]),
                [176, 120, 80],
            ),
        ),
        (
            "Lamp",
            tinta(star([0.0, 0.0], r, r, 8, 0.6), [240, 196, 72]),
        ),
    ];
    let ids: Vec<_> = pecas
        .into_iter()
        .map(|(nome, p)| (nome, cena.push_path(p)))
        .collect();
    sync(sim, cena, mapa);
    let mut out = Vec::new();
    for (i, (nome, pid)) in ids.into_iter().enumerate() {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        let at = LIGAR_AT[i];
        let mut em = sim.world_mut().entity_mut(e);
        em.insert((
            Name::new(nome),
            Transform::from_translation(ph2d_core::Vec2::new(at[0], at[1])),
        ));
        if i == 0 {
            em.insert(LuauScript::library(script));
        }
        out.push(e);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    sim.world_mut()
        .get_resource_or_init::<ScriptEditRequests>()
        .0
        .push(script);
    out
}
