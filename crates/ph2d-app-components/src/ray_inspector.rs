//! **A secção RAY SENSOR: o instantâneo e o dreno** (suplente #21).
//!
//! ⛔⛔ **Ele mora na CRATE DA FAMÍLIA e não na shell**, ao contrário dos irmãos mais velhos
//! (`inspector_topdown`, `inspector_projectile`): a catraca `the_shell_only_shrinks` reprovou na
//! primeira tentativa (`197 219` contra o tecto de `196 990`), e a cura que ela prescreve por
//! escrito é **MOVER para `crates/ph2d-app-<família>`, nunca subir o número**. ⭐ O molde já
//! existia ao lado — o gatilho, a vigia do contador, a cutscene, o emissor e o HUD já vivem aqui.
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o mesmo molde do
//! [`super::inspector_projectile`]: quem lê o mundo para o painel e quem escreve a edição de volta
//! fazem a MESMA tradução, e separá-los seria a porta pela qual as duas divergem.
//!
//! # ⭐⭐⭐ O que esta secção faz que as irmãs não fazem: ela lê a CORRIDA
//!
//! Os campos saem dos dois componentes, como sempre. **O que o raio VÊ** sai do mapa da ponte
//! (`PhysicsBridge::ray_sensor_hits`) — não há campo nenhum onde ele viva, e a lei do #11/#20 diz
//! porquê: *o que nasce numa corrida não é documento*.
//!
//! ⚠️ **E o alvo vira um NOME aqui**, nunca bits: o painel mostra texto, e os bits de uma entidade
//! não sobrevivem a um `Ctrl+Z` (o undo respawna o mundo inteiro com bits novos). *Guardar bits num
//! snapshot que dura um quadro seria inofensivo; traduzi-los aqui é o que mantém a regra numa porta
//! só.*

use ph2d_ecs::{Entity, Name, SimWorld, World};
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayFieldEdit};
use ph2d_physics_ecs::{RaySensor, RaySignals};

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
pub fn build_ray_info(
    world: &World,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
    visto: Option<(Entity, f32)>,
) -> Option<InspectorRayInfo> {
    let e = Entity::from_bits(bits);
    let r = *world.get::<RaySensor>(e)?;
    // ⚠️ **Os nomes são OPCIONAIS**, e a ausência lê-se como duas strings vazias — que é
    // exactamente *«ele vê e cala-se»*, a terceira queixa. ⛔ Não há aqui um `?`: um raio sem
    // `RaySignals` é um raio legítimo, e devolver `None` esconderia a secção inteira dele.
    let s = world.get::<RaySignals>(e);
    // ⭐ O NOME de quem ele vê — e `None` quando o alvo já não está no mundo, que é honesto: o
    // canal publica a entidade do tique, e um corpo apagado entre o tique e o quadro não tem nome.
    let (sees, sees_at) = match visto {
        Some((alvo, d)) => (
            world
                .get::<Name>(alvo)
                .map(|n| n.as_str().to_string())
                .unwrap_or_default(),
            d,
        ),
        None => (String::new(), 0.0),
    };
    Some(InspectorRayInfo {
        entity_bits: bits,
        origin_x: r.origin.x,
        origin_y: r.origin.y,
        dir_x: r.dir.x,
        dir_y: r.dir.y,
        reach: r.reach,
        layer: u32::from(r.layer),
        on_enter: s.map(|s| s.on_enter.clone()).unwrap_or_default(),
        on_exit: s.map(|s| s.on_exit.clone()).unwrap_or_default(),
        sees,
        sees_at,
        clock_playing,
        selected_count,
    })
}

/// **O dreno.** `true` = tocou no mundo.
pub fn apply_ray_edit(world: &mut World, bits: u64, edit: &RayFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    // ⚠️ **Os dois NOMES vivem noutro componente**, e ele pode não estar lá — um raio que ganha voz
    // pela primeira vez recebe o componente aqui. ⛔ Fazê-lo no painel seria o painel a escrever no
    // mundo, que é o que esta porta existe para impedir.
    match edit {
        RayFieldEdit::OnEnter(nome) => {
            let mut s = world.get_mut::<RaySignals>(e).map(|s| s.clone());
            if let Some(v) = s.as_mut() {
                v.on_enter.clone_from(nome);
            } else {
                s = Some(RaySignals {
                    on_enter: nome.clone(),
                    on_exit: String::new(),
                });
            }
            world.entity_mut(e).insert(s.unwrap_or_default());
            return true;
        }
        RayFieldEdit::OnExit(nome) => {
            let mut s = world.get_mut::<RaySignals>(e).map(|s| s.clone());
            if let Some(v) = s.as_mut() {
                v.on_exit.clone_from(nome);
            } else {
                s = Some(RaySignals {
                    on_enter: String::new(),
                    on_exit: nome.clone(),
                });
            }
            world.entity_mut(e).insert(s.unwrap_or_default());
            return true;
        }
        _ => {}
    }
    let Some(mut r) = world.get_mut::<RaySensor>(e) else {
        return false;
    };
    match edit {
        RayFieldEdit::OriginX(v) => r.origin.x = *v,
        RayFieldEdit::OriginY(v) => r.origin.y = *v,
        RayFieldEdit::DirX(v) => r.dir.x = *v,
        RayFieldEdit::DirY(v) => r.dir.y = *v,
        RayFieldEdit::Reach(v) => r.reach = *v,
        // ⚠️ **A camada é um ÍNDICE**, e o tecto é o do mundo (`MAX_LAYERS`): um número maior não
        // tem linha na matriz, e a porta do motor leria uma coluna que não existe.
        RayFieldEdit::Layer(v) => {
            r.layer = u8::try_from(*v)
                .unwrap_or(0)
                .min(ph2d_physics_ecs::MAX_LAYERS as u8 - 1);
        }
        // Os dois nomes já saíram acima.
        RayFieldEdit::OnEnter(_) | RayFieldEdit::OnExit(_) => return false,
    }
    true
}

/// **Aplica as edições que o painel emitiu neste quadro.** `true` = o documento mudou.
///
/// ⚠️ Irmã das `apply_all` das outras secções desta crate — a shell chama UMA função por secção, e
/// o laço vive com quem sabe o que cada edição significa.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, RayFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if apply_ray_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}

#[cfg(test)]
#[path = "ray_inspector_tests.rs"]
mod tests;
