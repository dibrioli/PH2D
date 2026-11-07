//! **O item inteiro numa tela** (`PH2D_PHYSICS_SMOKE=120`): os corpos e os gates moram em
//! [`ph2d_physics_scene::ficha`]; aqui só o que a cena pede à shell.

use ph2d_ecs::{Entity, Name};
use ph2d_physics_scene::ficha::{VISTA, VISTA_ALTURA, monta, regras};

pub fn physics_smoke_ficha(ctx: &mut crate::SceneCtx<'_>) {
    monta(ctx.world);
    ctx.want.settings = Some(regras(ctx.settings));
    ctx.want.camera_center = Some(VISTA);
    ctx.want.camera_height_world = Some(
        std::env::var("PH2D_PHYSICS_SMOKE_VISTA")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(VISTA_ALTURA),
    );
    if std::env::var_os("PH2D_PHYSICS_SMOKE_PAINEL").is_some() {
        ctx.want.panels.push("physics");
    }
    if let Ok(nome) = std::env::var("PH2D_PHYSICS_SMOKE_SELECT") {
        let mut q = ctx.world.query::<(Entity, &Name)>();
        ctx.want.select = q
            .iter(ctx.world)
            .find(|(_, n)| n.0 == nome.trim())
            .map(|(e, _)| e.to_bits());
    }
    println!(
        "[physics-smoke 120] O item inteiro, PARADO. Espaço dá Play: o Crate atravessa a Collision Shape solta e \
         pousa no Floor; a Stretched Ball cai na Updraft Zone e a zona empurra-a para cima. ←/→ andam, ↑ salta: o \
         Player sobe pela One-way Platform por baixo e pousa em cima; ele atravessa o Crate porque a camada dele só \
         toca Ground. O Stop devolve tudo."
    );
}

#[cfg(test)]
#[test]
fn a_cena_nasce_parada() {
    assert!(crate::smoke::PAUSED_SCENES.contains(&"120"));
}
