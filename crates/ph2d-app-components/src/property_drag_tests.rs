use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_ecs::scene::property::{self, full_name};
use ph2d_ecs::scene::property_ref::RefNames;
use ph2d_ecs::{StableId, Transform};
use ph2d_editor_core::zones::Rect;
use ph2d_physics_ecs::Health;
use ph2d_script::{LuauScript, ProjectScripts};

fn linha(cartao: &str, texto: &str, script: Option<&str>) -> Linha {
    Linha {
        area: Rect::new(0.0, 0.0, 10.0, 10.0),
        texto: texto.to_owned(),
        cartao: cartao.to_owned(),
        script: script.map(str::to_owned),
    }
}

fn health() -> &'static ph2d_component_desc::ComponentDesc {
    ph2d_component_desc::all()
        .find(|d| d.code == "health")
        .expect("Health no catálogo")
}

fn rotulo_de(campo: &str) -> &'static str {
    let f = health()
        .fields
        .iter()
        .find(|f| f.name == campo)
        .expect("campo");
    tr(f.label_key)
}

#[test]
fn uma_linha_do_inspector_da_o_nome_com_que_o_script_a_le() {
    let reg = registo();
    let mut sim = SimWorld::new();
    let id = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("heroi.luau", "ph2d.property(\"velocidade\", 3)\n");
    let heroi = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            StableId(7),
            Health::default(),
            LuauScript::library(id),
        ))
        .id();
    let porta = sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(8)))
        .id();
    let cartao = tr(health().display_key);
    let max = linha(cartao, rotulo_de("max"), None);
    assert_eq!(
        carga(&reg, &sim, heroi, &max),
        Some((DragPayload::Property { script: 0 }, "health.max".to_owned()))
    );
    assert_eq!(
        carga(&reg, &sim, porta, &max),
        None,
        "um objecto sem Vida não tem a linha"
    );
    let transform = tr("panel.inspector.transform.transform");
    for (rotulo, nomes) in [
        ("panel.inspector.transform.position_m", "x, y"),
        ("panel.inspector.transform.position_px", "x, y"),
        ("panel.inspector.transform.rotation", "rotation"),
        ("panel.inspector.transform.scale", "scale_x, scale_y"),
        ("panel.inspector.transform.skew", "skew_x, skew_y"),
    ] {
        assert_eq!(
            carga(&reg, &sim, porta, &linha(transform, tr(rotulo), None)).map(|c| c.1),
            Some(nomes.to_owned()),
            "{rotulo}"
        );
    }
    let vel = linha(
        tr("panel.inspector.script.script"),
        "velocidade",
        Some("heroi.luau"),
    );
    assert_eq!(
        carga(&reg, &sim, heroi, &vel),
        Some((
            DragPayload::Property { script: id.0 },
            "velocidade".to_owned()
        ))
    );
    let fonte = linha(tr("panel.inspector.script.script"), "Source", None);
    assert_eq!(
        carga(&reg, &sim, heroi, &fonte),
        None,
        "uma linha que não é propriedade"
    );
    assert_eq!(names_of_label("x, y"), ["x", "y"]);
}

/// Dois componentes com `code` que partilham o texto de um rótulo e cabem no mesmo objecto.
fn par_de_rotulo_comum(
    reg: &ComponentRegistry,
    sim: &mut SimWorld,
) -> Option<(Entity, &'static str, String, &'static str, String, String)> {
    let descs: Vec<_> = ph2d_component_desc::all()
        .filter(|d| !d.code.is_empty())
        .collect();
    for (i, a) in descs.iter().enumerate() {
        for b in &descs[i + 1..] {
            if tr(a.display_key) == tr(b.display_key) {
                continue;
            }
            for fa in a.fields.iter().filter(|f| !f.name.is_empty()) {
                for fb in b.fields.iter().filter(|f| !f.name.is_empty()) {
                    if tr(fa.label_key) != tr(fb.label_key) {
                        continue;
                    }
                    let e = sim
                        .world_mut()
                        .spawn((Transform::IDENTITY, StableId(99)))
                        .id();
                    let mut ok = true;
                    for d in [a, b] {
                        let ins = reg
                            .get_by_name(d.canonical_name)
                            .and_then(|en| en.insert_default);
                        ok &= ins.is_some_and(|f| f(sim.world_mut(), e).is_ok());
                    }
                    let na = full_name(a.code, fa.name);
                    let nb = full_name(b.code, fb.name);
                    let pega = |sim: &SimWorld, n: &str| {
                        property::get(reg, sim.world(), e, n, RefNames::default()).is_ok()
                    };
                    if ok && pega(sim, &na) && pega(sim, &nb) {
                        return Some((
                            e,
                            tr(a.display_key),
                            na,
                            tr(b.display_key),
                            nb,
                            tr(fa.label_key).to_owned(),
                        ));
                    }
                    sim.world_mut().despawn(e);
                }
            }
        }
    }
    None
}

#[test]
fn dois_componentes_com_o_mesmo_rotulo_decidem_se_pelo_titulo_do_cartao() {
    let reg = registo();
    let mut sim = SimWorld::new();
    let (e, cartao_a, nome_a, cartao_b, nome_b, rotulo) =
        par_de_rotulo_comum(&reg, &mut sim).expect("há dois componentes com o mesmo rótulo");
    let carga_de = |cartao: &str| carga(&reg, &sim, e, &linha(cartao, &rotulo, None)).map(|c| c.1);
    assert_eq!(carga_de(cartao_a), Some(nome_a));
    assert_eq!(carga_de(cartao_b), Some(nome_b));
    assert_eq!(
        carga_de("Nonsense"),
        None,
        "cartão que não é de nenhum: nunca um nome à sorte"
    );
}
