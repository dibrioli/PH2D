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
        // A foto do «Merge» (`PH2D_PHYSICS_SMOKE_OUTLINE_MERGE=<px>`): funde todos os pontos logo depois do 1.º traço.
        if let Some(px) = std::env::var("PH2D_PHYSICS_SMOKE_OUTLINE_MERGE")
            .ok()
            .and_then(|v| v.trim().parse().ok())
        {
            ctx.world
                .insert_resource(ph2d_physics_outline::PendingMerge { shape: forma, px });
        }
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
    let traco = ph2d_physics_scene::ficha::traco_para_as_fotos(ctx.world);
    ctx.want.select = excecao
        .or(outline.map(|o| o.0))
        .or(traco)
        .map(Entity::to_bits);
    if let Ok(nome) = std::env::var("PH2D_PHYSICS_SMOKE_SELECT") {
        let mut q = ctx.world.query::<(Entity, &Name)>();
        ctx.want.select = q
            .iter(ctx.world)
            .find(|(_, n)| n.0 == nome.trim())
            .map(|(e, _)| e.to_bits());
    }
    // A foto do Look da forma (GEF02 onda 8): o pivô da «Loose Shape» no canto de baixo à esquerda, a forma no lugar.
    if std::env::var_os("PH2D_PHYSICS_SMOKE_PIVO_NO_CANTO").is_some() {
        pivo_no_canto(ctx.world);
    }
    if std::env::var_os("PH2D_PHYSICS_SMOKE_CAMADA_NA_TIMELINE").is_some() {
        camada_na_timeline(ctx);
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

/// A foto da camada de colisão na timeline: a do Player vai de «Player» (0 s) a «Props» (1 s), Timeline aberta.
fn camada_na_timeline(ctx: &mut crate::SceneCtx<'_>) {
    let mut q = ctx.world.query::<(Entity, &Name)>();
    let Some(player) = q
        .iter(ctx.world)
        .find(|(_, n)| n.0 == "Player")
        .map(|(e, _)| e)
    else {
        return;
    };
    let prop = ph2d_timeline::PropKind::Field(ph2d_ecs::FieldKey::new(
        ph2d_ecs::canal::familia::FISICA,
        ph2d_ecs::canal_fisica::CAMADA,
        0,
        0,
    ));
    let dono = crate::canais_corpo::dono_da_chave(ctx.world, player.to_bits(), prop);
    for (t, camada) in [(0.0, "Player"), (1.0, "Props")] {
        let Some(i) = ph2d_physics_scene::ficha::CAMADAS
            .iter()
            .position(|c| *c == camada)
        else {
            continue;
        };
        #[allow(clippy::cast_precision_loss)]
        ctx.timeline.insert_key(
            dono,
            prop,
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(i as f32),
            ph2d_anim::Interp::Linear,
        );
    }
    ctx.want.panels.push("timeline");
    ctx.want.select = Some(player.to_bits());
}

#[cfg(test)]
#[test]
fn a_cena_nasce_parada() {
    assert!(crate::smoke::PAUSED_SCENES.contains(&"120"));
}

/// O pivô da «Loose Shape» no canto de baixo à esquerda pela lei da ferramenta (`arrasta_pivo`): a forma fica.
fn pivo_no_canto(world: &mut ph2d_ecs::World) {
    let mut q = world.query::<(Entity, &Name)>();
    let Some(e) = q
        .iter(world)
        .find(|(_, n)| n.0 == "Loose Shape")
        .map(|(e, _)| e)
    else {
        return;
    };
    let (Some((c, h)), Some(t)) = (
        ph2d_shape_handles::look::anchor_half(world, e),
        world.get::<ph2d_ecs::Transform>(e).copied(),
    ) else {
        return;
    };
    let canto = [c[0] - h[0], c[1] - h[1]];
    let origem = [
        t.translation.x + canto[0] * t.scale.x,
        t.translation.y + canto[1] * t.scale.y,
    ];
    let _ = ph2d_shape_handles::look::arrasta_pivo(world, e, origem, [h[0], h[1]]);
}
