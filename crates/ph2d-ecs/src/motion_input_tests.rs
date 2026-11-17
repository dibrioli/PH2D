use super::*;

const BASE: &str = "Motion Input";

/// ⭐ O seletor `source` do catálogo tem UMA opção por [`MotionSource::ALL`], na ordem do índice, e o
/// valor de cada uma é o nome serde da variante — senão o Luau `set(.., "source", "velocity")` falha
/// e o canal da Timeline não tem palavra (GEF fecho, D2).
///
/// Mutação que deve sangrar: tirar `Move` do `options!` do catálogo · trocar duas opções de ordem.
#[test]
fn motion_input_source_catalog_matches_every_motion_source() {
    let d = ph2d_component_desc::desc_for("ph2d::ecs::MotionInput").expect("no catálogo");
    let campo = d
        .fields
        .iter()
        .find(|f| f.name == "source")
        .expect("campo source");
    let ph2d_component_desc::View::Options(opcoes) = campo.view else {
        panic!("source não é um seletor: {:?}", campo.view);
    };
    let valores: Vec<&str> = opcoes.iter().map(|o| o.valor).collect();
    let serde: Vec<String> = MotionSource::ALL
        .iter()
        .map(|s| {
            serde_json::to_string(s)
                .expect("serde")
                .trim_matches('"')
                .to_owned()
        })
        .collect();
    assert_eq!(valores, serde, "uma opção por variante, na ordem");
    for (i, s) in MotionSource::ALL.iter().enumerate() {
        assert_eq!(usize::from(s.index()), i, "{s:?}: o índice é a posição");
    }
}

fn com_nome(
    nome_entrada: &str,
    nome: Option<&str>,
) -> (bevy_ecs::world::World, bevy_ecs::entity::Entity) {
    let mut w = bevy_ecs::world::World::new();
    let m = MotionInput {
        name: nome_entrada.to_owned(),
        ..MotionInput::default()
    };
    let e = match nome {
        Some(n) => w.spawn((m, crate::Name::new(n))).id(),
        None => w.spawn(m).id(),
    };
    (w, e)
}

fn nome_de(w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> String {
    w.get::<crate::Name>(e)
        .map(|n| n.as_str().to_owned())
        .unwrap_or_default()
}

/// ⭐ D3: só o nome AUTOMÁTICO segue a entrada; o que o artista escreveu, mesmo começando pela base, fica.
///
/// Mutação que deve sangrar: voltar ao `starts_with(base)` (o «(pulo)» é apagado) · `rotulo` com nome
/// vazio deixar o « · » no fim.
#[test]
fn motion_input_label_follows_only_the_automatic_name() {
    for (antes, entrada, depois) in [
        ("Motion Input (pulo)", "novo", "Motion Input (pulo)"),
        ("Motion Inputs", "novo", "Motion Inputs"),
        ("Hero", "novo", "Hero"),
        ("Motion Input · velho", "novo", "Motion Input · novo"),
        ("Motion Input", "novo", "Motion Input · novo"),
        ("Motion Input 2", "novo", "Motion Input · novo"),
        ("Motion Input · velho", "  ", "Motion Input"),
    ] {
        let (mut w, e) = com_nome(entrada, Some(antes));
        rotula_entradas(&mut w, BASE);
        assert_eq!(
            nome_de(&w, e),
            depois,
            "{antes:?} com a entrada {entrada:?}"
        );
    }
    let (mut w, e) = com_nome("x", None);
    rotula_entradas(&mut w, BASE);
    assert_eq!(nome_de(&w, e), "Motion Input · x", "sem nome nenhum");
}
