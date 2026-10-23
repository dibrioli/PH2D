//! ⭐ **A cena 9 do editor de scripts ensina o que diz** (`PH2D_CODE_SMOKE=9`, 8b + 8c —
//! `docs/CodeEditor/07_arrastar_para_o_codigo.md` §3.4 e §3.10): a cena montada como a shell a monta; os
//! passos do script feitos pelas MESMAS leis que a soltura usa — o nome da linha do Inspector
//! (`property_drag::carga`), o texto escrito (`dropping`), o campo de objecto (`into_script_slot`) — e o
//! texto corrido pela VM real: a nave gira, a porta sobe e desce, e depois é a lâmpada.

use ph2d_app_components::code_smoke::CENAS;
use ph2d_app_components::code_smoke_inspector::{LIGAR_LUAU, LIGAR_NAME, monta};
use ph2d_app_components::object_drag::into_script_slot;
use ph2d_app_components::property_drag::{carga, names_of_label};
use ph2d_code_syntax::dropping::{alt_dropped_into, property_insertion};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform};
use ph2d_editor_core::interaction::drag_payload::DragPayload;
use ph2d_editor_core::widget::arrasto::Linha;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_app_components::script_inspector::build_info;
use ph2d_script::{ProjectScripts, ScriptDrafts, ScriptHost};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecScene;

fn pose(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("pose");
    (t.translation.y, t.rotation)
}

fn corre(host: &mut ScriptHost, sim: &mut SimWorld, n: usize) {
    for _ in 0..n {
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(sim, 1.0 / 60.0);
        assert!(r.failed.is_empty(), "o script parou: {:?}", r.failed);
    }
}

fn linha(cartao: &str, texto: &str, script: Option<&str>) -> Linha {
    Linha {
        area: Rect::new(0.0, 0.0, 1.0, 1.0),
        texto: texto.to_owned(),
        cartao: cartao.to_owned(),
        script: script.map(str::to_owned),
    }
}

/// Larga `names` logo depois da 1.ª `depois_de` a partir do byte `de` (a linha que o dono escreveu).
fn largar(src: &str, de: usize, depois_de: &str, names: &[String], own: bool) -> String {
    let b = src[de..].find(depois_de).expect("o buraco") + de + depois_de.len();
    let i = property_insertion(b, names, own);
    let mut s = src.to_owned();
    s.insert_str(i.at, &i.text);
    s
}

#[test]
fn os_quatro_passos_da_cena_9_fazem_o_que_o_script_diz() {
    const { assert!(CENAS >= 9) };
    let (mut sim, mut cena, mut mapa) = (
        SimWorld::new(),
        VecScene::default(),
        VecEntityMap::default(),
    );
    let pecas = monta(&mut sim, &mut cena, &mut mapa);
    let nomes: Vec<&str> = pecas
        .iter()
        .map(|&e| sim.world().get::<Name>(e).expect("nome").as_str())
        .collect();
    assert_eq!(nomes, ["Ship", "Door", "Lamp"]);
    assert!(
        pecas
            .iter()
            .all(|&e| sim.world().get::<StableId>(e).is_some())
    );
    let (nave, porta, lampada) = (pecas[0], pecas[1], pecas[2]);
    let id = sim
        .world()
        .resource::<ProjectScripts>()
        .by_name(LIGAR_NAME)
        .expect("script");
    let reg = ComponentRegistry::default();

    // ① e ② — as linhas do Inspector, pelo nome com que o script as lê.
    let rotacao = linha(
        tr("panel.inspector.transform.transform"),
        tr("panel.inspector.transform.rotation"),
        None,
    );
    let (p, rot) = carga(&reg, &sim, nave, &rotacao).expect("Rotation arrasta");
    assert_eq!(p, DragPayload::Property { script: 0 });
    let giro = linha(
        tr("panel.inspector.script.script"),
        "giro",
        Some(LIGAR_NAME),
    );
    let (p, g) = carga(&reg, &sim, nave, &giro).expect("giro arrasta");
    assert_eq!(p, DragPayload::Property { script: id.0 });
    let fmt = ph2d_code_format::Program::pinned().expect("`bash scripts/stylua-fetch.sh`");
    assert_eq!(
        fmt.format(LIGAR_LUAU).as_deref(),
        Ok(LIGAR_LUAU),
        "começa limpo e válido"
    );
    let ate = "para logo ANTES do  )\n";
    let vazia = LIGAR_LUAU.rfind(ate).expect("o passo ②") + ate.len();
    let mut um = LIGAR_LUAU.to_owned();
    // o `(` escrito traz o `)` (o par automático do editor): larga-se antes dele
    um.insert_str(vazia, "  ph2d.set(self.id, )");
    let um = largar(
        &um,
        vazia,
        "ph2d.set(self.id, ",
        &names_of_label(&rot),
        false,
    );
    let pos = um[vazia..].find("\"rotation\"").expect("①") + vazia + "\"rotation\"".len();
    let mut dois = um.clone();
    dois.insert_str(pos, ", tempo * ");
    let dois = largar(&dois, vazia, "tempo * ", &names_of_label(&g), true);
    assert!(
        dois.contains("  ph2d.set(self.id, \"rotation\", tempo * self.giro)\n"),
        "{dois}"
    );

    // ③ — Alt + a «Door» na linha vazia do topo.
    let vazia = dois.find("SEM salvar:\n").expect("o passo ③") + "SEM salvar:\n".len();
    let tres = alt_dropped_into(&dois, vazia, &["Door".to_owned()]);
    assert!(
        tres.contains(
            "SEM salvar:\nph2d.property(\"door\", ph2d.object(\"Door\"))\n\nlocal tempo"
        ),
        "{tres}"
    );
    assert_eq!(
        fmt.format(&tres).as_deref(),
        Ok(tres.as_str()),
        "o texto final sai limpo"
    );

    let mut host = ScriptHost::new().expect("vm");
    // ③ antes do Ctrl+S (1b): o campo «door» já está no cartão — a lista do texto da aba.
    sim.world_mut()
        .resource_mut::<ProjectScripts>()
        .set_text(id, &dois);
    host.scene_sync(sim.world_mut());
    let antes = |sim: &SimWorld, host: &ScriptHost| -> Vec<String> {
        build_info(sim, Some(host), nave.to_bits(), 1, false)
            .expect("cartão")
            .props
            .into_iter()
            .map(|p| p.name)
            .collect()
    };
    assert_eq!(antes(&sim, &host), ["giro"], "salvo: só o giro");
    let rascunho = host.draft_decls(LIGAR_NAME, &tres).expect("o texto da aba carrega");
    sim.world_mut()
        .insert_resource(ScriptDrafts([(id, rascunho)].into_iter().collect()));
    assert_eq!(antes(&sim, &host), ["giro", "door"], "③ sem salvar: o campo aparece");
    sim.world_mut().remove_resource::<ScriptDrafts>();
    sim.world_mut()
        .resource_mut::<ProjectScripts>()
        .set_text(id, &tres);
    host.request_reload(id);
    let (porta0, lamp0) = (pose(&sim, porta), pose(&sim, lampada));
    corre(&mut host, &mut sim, 30);
    assert_ne!(pose(&sim, nave).1, 0.0, "① + ②: a nave gira");
    assert_ne!(pose(&sim, porta), porta0, "③: a porta sobe e desce");
    assert_eq!(pose(&sim, lampada), lamp0, "a lâmpada ainda não");

    // ④ — a «Lamp» largada no campo «door» do cartão Script.
    let lamp_id = sim.world().get::<StableId>(lampada).expect("id").0;
    assert!(into_script_slot(&mut sim, nave.to_bits(), "door", lamp_id));
    let porta1 = pose(&sim, porta);
    corre(&mut host, &mut sim, 30);
    assert_ne!(pose(&sim, lampada), lamp0, "④: agora é a lâmpada");
    assert_eq!(pose(&sim, porta), porta1, "e a porta pára");
}
