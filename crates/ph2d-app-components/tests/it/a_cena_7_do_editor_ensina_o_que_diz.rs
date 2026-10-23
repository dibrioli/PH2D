//! ⭐ **A cena 7 do editor de scripts ensina o que diz** (`PH2D_CODE_SMOKE=7`, item 8 —
//! `docs/CodeEditor/07_arrastar_para_o_codigo.md`): a cena montada como a shell a monta; o passo ① do
//! script (Ctrl + soltar a «Door» na linha vazia) feito pela LEI da soltura (`ph2d_code_syntax::dropping`)
//! e o texto corrido pela VM real — a porta tem de passar a mexer, e antes do passo não.

use ph2d_app_components::code_smoke::{ARRASTAR_LUAU, ARRASTAR_NAME, CENAS, monta_em};
use ph2d_code_syntax::dropping::dropped_into;
use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform};
use ph2d_script::{ProjectScripts, ScriptEditRequests, ScriptHost};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecScene;

fn y(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("pose").translation.y
}

fn corre(host: &mut ScriptHost, sim: &mut SimWorld, n: usize) {
    for _ in 0..n {
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(sim, 1.0 / 60.0);
        assert!(r.failed.is_empty(), "o script parou: {:?}", r.failed);
    }
}

#[test]
fn ctrl_soltar_a_porta_na_linha_vazia_poe_a_porta_a_mexer() {
    const { assert!(CENAS >= 7) };
    let (mut sim, mut cena, mut mapa) = (
        SimWorld::new(),
        VecScene::default(),
        VecEntityMap::default(),
    );
    let pecas = monta_em(&mut sim, &mut cena, &mut mapa, 7);
    let nomes: Vec<String> = pecas
        .iter()
        .map(|&e| {
            sim.world()
                .get::<Name>(e)
                .expect("nome")
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(nomes, ["Ship", "Door", "Gold Coin"]);
    assert!(
        pecas
            .iter()
            .all(|&e| sim.world().get::<StableId>(e).is_some()),
        "arrastáveis"
    );
    assert_eq!(
        sim.world().resource::<ScriptEditRequests>().0.len(),
        1,
        "a aba abre"
    );
    let porta = pecas[1];
    let mut host = ScriptHost::new().expect("vm");
    let antes = y(&sim, porta);
    corre(&mut host, &mut sim, 30);
    assert_eq!(y(&sim, porta), antes, "antes do passo ① a porta não mexe");

    let linha = ARRASTAR_LUAU.find("NA LINHA VAZIA").expect("o passo ①");
    let vazia = ARRASTAR_LUAU[linha..].find("\n\n").expect("a linha vazia") + linha + 1;
    let novo = dropped_into(ARRASTAR_LUAU, vazia, &["Door".to_owned()], true, "  ");
    assert!(
        novo.contains("  local door = ph2d.object(\"Door\")\n"),
        "{novo}"
    );
    let id = {
        let lib = sim.world().resource::<ProjectScripts>();
        lib.iter()
            .find(|(_, s)| s.name == ARRASTAR_NAME)
            .expect("script")
            .0
    };
    sim.world_mut()
        .resource_mut::<ProjectScripts>()
        .set_text(id, &novo);
    host.request_reload(id);
    corre(&mut host, &mut sim, 30);
    assert_ne!(
        y(&sim, porta),
        antes,
        "depois do Ctrl+soltar a porta sobe e desce"
    );
}
