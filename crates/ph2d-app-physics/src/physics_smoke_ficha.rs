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
    ph2d_physics_scene::ficha::nomeia_para_as_fotos(ctx.world);
    ph2d_physics_scene::ficha::olho_para_as_fotos(ctx.world);
    let excecao = ph2d_physics_scene::ficha::excecao_para_as_fotos(ctx.world);
    let outline = ph2d_physics_scene::ficha::outline_para_as_fotos(ctx.world);
    if let Some((forma, centro)) = outline {
        ctx.want.camera_center = Some(centro);
        // A foto dos pontos (`PH2D_PHYSICS_SMOKE_OUTLINE_PONTOS=1`): a forma já nasce em edição no canvas.
        if let Some(v) = std::env::var_os("PH2D_PHYSICS_SMOKE_OUTLINE_PONTOS") {
            ph2d_shape_handles::alterna(ctx.world, forma.to_bits());
            // `=2`: três pontos de cada anel já selecionados.
            let i = (0..4)
                .flat_map(|anel| [0, 4, 8].map(|i| ph2d_shape_handles::pontos::Ponto { anel, i }));
            ctx.world
                .insert_resource(ph2d_shape_handles::pontos::PontosNoCanvas {
                    selecionados: if v == "2" { i.collect() } else { Vec::new() },
                    ..Default::default()
                });
        }
    }
    ctx.want.select = excecao.or(outline.map(|o| o.0)).map(Entity::to_bits);
    if let Ok(nome) = std::env::var("PH2D_PHYSICS_SMOKE_SELECT") {
        let mut q = ctx.world.query::<(Entity, &Name)>();
        ctx.want.select = q
            .iter(ctx.world)
            .find(|(_, n)| n.0 == nome.trim())
            .map(|(e, _)| e.to_bits());
    }
    println!(
        "[physics-smoke 120] The whole item, PAUSED. Space plays: the Crate falls through the Loose Shape and \
         lands on the Floor; the Stretched Ball falls into the Updraft Zone and the zone pushes it up. Left/Right \
         walk, Up jumps: the Player goes up through the One-way Platform from below and lands on top; he passes \
         through the Crate because his layer only hits Ground. The Rock's shape is an Outline traced from its \
         picture: Detail, Alpha cutoff and Ignore specks under each reshape it. It lands on the Floor. The Marble falls inside the Ring's hole and rests on its bottom. The Table \
         stands on its two legs: one body, three shapes. Stop puts everything back."
    );
}

#[cfg(test)]
#[test]
fn a_cena_nasce_parada() {
    assert!(crate::smoke::PAUSED_SCENES.contains(&"120"));
}
