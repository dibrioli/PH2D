//! **Desfazer e refazer o aplicar devolve a pintura** (W18): o restauro do undo respawna o mundo do
//! instantâneo (`ProjectState::restore` da shell: tira tudo o que tem `Transform`, `snapshot_to_world`)
//! e a imagem volta com bits novos — o documento do Painter passa a eles pela identidade estável
//! ([`Reata`], a porta que a shell chama à volta do restauro). Sem ele a imagem mostraria só o atlas.

use crate::aplicar::{banca, grava, pasta};
use ph2d_app_painter::documentos_no_projecto::Reata;
use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, incremental, snapshot_to_world};
use ph2d_ecs::{ChildOf, Entity, Macro, Name, SimWorld, Transform, With};
use ph2d_editor_core::macro_edits::{MacroRequest, OndeAplicar};
use ph2d_tool_painter::PainterTool;
use std::collections::BTreeMap;

fn captura(sim: &mut SimWorld, reg: &ComponentRegistry) -> WorldSnapshot {
    let mut s = WorldSnapshot::new();
    let mut cache = incremental::CaptureCache::new();
    incremental::capture_incremental(sim.world_mut(), &mut cache, reg, &mut s).expect("captura");
    s
}

/// O restauro da shell, com a porta dos documentos à volta.
fn restaura(
    s: &WorldSnapshot,
    sim: &mut SimWorld,
    p: &mut PainterTool,
    r: &mut Reata,
    reg: &ComponentRegistry,
) {
    r.antes(sim, p);
    let w = sim.world_mut();
    let todas: Vec<Entity> = w
        .query_filtered::<Entity, With<Transform>>()
        .iter(w)
        .collect();
    for e in todas {
        let _ = w.despawn(e);
    }
    snapshot_to_world(w, s, reg).expect("restaura");
    r.depois(sim, p);
}

/// Os píxeis do documento que o Painter guarda sob `bits` (ligado ou guardado), sem o ligar.
fn pixeis(p: &PainterTool, bits: u64) -> Option<(Vec<u8>, u32, u32)> {
    let doc = p.collect_documents(&BTreeMap::from([(bits, 0)])).pop()?;
    let mut t = PainterTool::default();
    t.install_document(1, doc);
    t.baked_document_pixels(1)
}

fn por_nome(sim: &mut SimWorld, nome: &str) -> Vec<u64> {
    let w = sim.world_mut();
    w.query::<(Entity, &Name)>()
        .iter(w)
        .filter(|(_, n)| n.0 == nome)
        .map(|(e, _)| e.to_bits())
        .collect()
}

#[test]
fn desfazer_e_refazer_o_aplicar_devolve_a_pintura() {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    let lib = pasta("desfazer");
    let (mut b, dono) = banca(96, 64, &lib);
    grava(&mut b, [10.0, 30.0], [80.0, 30.0]);
    b.m.pede(MacroRequest::Guarda { entity_bits: dono });
    b.quadro();
    // A imagem pintada à mão também atravessa: a da banca, com a pincelada.
    let solta = b.bits;
    b.sim
        .world_mut()
        .entity_mut(Entity::from_bits(solta))
        .remove::<ChildOf>();
    let pre = captura(&mut b.sim, &reg);
    b.m.pede(MacroRequest::Aplica {
        nome: "Splash".into(),
        onde: OndeAplicar::Novo { em: None },
    });
    b.quadro();
    let raiz = b.m.toma_o_aplicado().expect("aplicou");
    let img = *b.imagens.borrow().keys().next().expect("a imagem nasceu");
    assert_eq!(
        b.sim
            .world()
            .get::<ChildOf>(Entity::from_bits(img))
            .map(|c| c.0.to_bits()),
        Some(raiz)
    );
    let antes = pixeis(&b.painter, img).expect("o editor tem-na");
    let pos = captura(&mut b.sim, &reg);
    // Depois do último quadro (a água do Wet Paint anda a cada um).
    let da_solta = pixeis(&b.painter, solta).expect("pintada");

    let mut r = Reata::default();
    restaura(&pre, &mut b.sim, &mut b.painter, &mut r, &reg);
    let w = b.sim.world_mut();
    let com_gravacao = w
        .query::<&Macro>()
        .iter(w)
        .filter(|m| !m.doc.is_empty())
        .count();
    assert_eq!(
        com_gravacao, 1,
        "desfazer tira o objecto Macro aplicado (fica o gravado)"
    );
    restaura(&pos, &mut b.sim, &mut b.painter, &mut r, &reg);
    let img2: Vec<u64> = por_nome(&mut b.sim, "Tela")
        .into_iter()
        .filter(|&x| x != img && b.sim.world().get::<ChildOf>(Entity::from_bits(x)).is_some())
        .collect();
    assert_eq!(
        img2.len(),
        1,
        "a imagem nascida voltou, dentro do objecto Macro"
    );
    let nascida = img2
        .iter()
        .copied()
        .find(|&x| pixeis(&b.painter, x).as_ref() == Some(&antes));
    assert!(
        nascida.is_some(),
        "o documento do aplicar voltou à imagem nascida, com os bits novos"
    );
    assert_ne!(nascida, Some(img), "o refazer respawna: bits novos");
    let solta2 = por_nome(&mut b.sim, "Tela")
        .into_iter()
        .find(|&x| b.sim.world().get::<ChildOf>(Entity::from_bits(x)).is_none());
    assert_eq!(
        solta2.and_then(|x| pixeis(&b.painter, x)),
        Some(da_solta),
        "a imagem pintada à mão também"
    );
}
