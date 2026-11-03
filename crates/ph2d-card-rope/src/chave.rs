//! ⭐ **O alfinete sobrevive a reabrir o projecto** (`docs/CartoesUI/01_decisoes_do_dono.md`, T36):
//! os bits do sujeito preso são da sessão (e o respawn do undo troca-os); a chave que se grava é o
//! `StableId` com o nome (o `sections.txt` é da máquina, não do projecto: o nome impede que o mesmo
//! número de OUTRO projecto prenda o cartão a um estranho). Cada quadro, antes de os painéis lerem
//! os presos: bits sem chave ganham-na; chave com bits mortos (ou sem bits) volta a achá-los.

use ph2d_ecs::{Entity, Name, StableId, World, entity_with_stable_id};
use ph2d_editor_core::WidgetStore;

/// O quadro: liga os bits de cada solto preso à chave durável dele, nos dois sentidos.
pub fn resolve(store: &mut WidgetStore, world: &World) {
    for (k, bits, chave) in store.loose_pin_keys() {
        match (bits.and_then(|b| chave_de(world, b)), chave) {
            // O mesmo objecto (talvez renomeado: a chave segue o nome).
            (Some(c), Some(d)) if c.0 == d.0 => store.set_loose_pin_key(k, c),
            (Some(c), None) => store.set_loose_pin_key(k, c),
            // Bits mortos ou de outro objecto (o respawn do undo, reabrir): a chave decide.
            (_, Some((id, nome))) => store.rebind_loose_pin(k, achar(world, id, &nome)),
            (None, None) => {}
        }
    }
}

fn chave_de(world: &World, bits: u64) -> Option<(u64, String)> {
    let e = Entity::try_from_bits(bits)?;
    let id = world.get::<StableId>(e).filter(|s| !s.is_none())?;
    Some((id.0, nome(world, e)))
}

fn achar(world: &World, id: u64, nome_gravado: &str) -> Option<u64> {
    let e = entity_with_stable_id(world, StableId(id))?;
    (nome(world, e) == nome_gravado).then(|| e.to_bits())
}

fn nome(world: &World, e: Entity) -> String {
    world
        .get::<Name>(e)
        .map_or_else(String::new, |n| n.as_str().to_owned())
}

/// O Transform local e o nome de `bits` — o retrato de um objecto preso por um solto do Inspector
/// (as mesmas contas da seleção, `snapshots_inspector`).
#[must_use]
pub fn transform_de(
    world: &World,
    bits: u64,
) -> Option<(ph2d_editor_core::InspectorTransformInfo, String)> {
    let e = Entity::try_from_bits(bits)?;
    let t = world.get::<ph2d_ecs::Transform>(e)?;
    let info = ph2d_editor_core::InspectorTransformInfo {
        entity_bits: bits,
        translation: [t.translation.x, t.translation.y],
        rotation_rad: t.rotation,
        scale: [t.scale.x, t.scale.y],
        skew_rad: [t.skew_x, t.skew_y],
    };
    let nome = world.get::<Name>(e).map(|n| n.as_str().to_owned());
    Some((info, nome.unwrap_or_else(|| format!("Entity_{bits:x}"))))
}

/// A origem de `bits` no MUNDO (a ponta da corda de um objecto preso pelo Inspector).
#[must_use]
pub fn origem(world: &World, bits: u64) -> Option<[f32; 2]> {
    let t = ph2d_ecs::world_transform(world, Entity::try_from_bits(bits)?)?;
    Some([t.translation.x, t.translation.y])
}
