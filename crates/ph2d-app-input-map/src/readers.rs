//! Quem lê cada acção (plano §3 «Leitores»): os Triggers da cena (`SignalOnAction`, pelo nome do objecto) e os
//! scripts Luau do projecto (`ProjectScripts`), juntados em `ReaderFacts` para `ph2d_input::readers::readers`.

use std::collections::BTreeMap;

use ph2d_ecs::{Name, SignalOnAction, World};
use ph2d_input::readers::{Reader, ReaderFacts, readers};

/// Os factos da cena e dos scripts deste quadro, por NOME (a ordem dos arquétipos muda com um componente).
#[must_use]
pub fn facts(world: &World) -> ReaderFacts {
    let mut f = ReaderFacts::default();
    for e in world.iter_entities() {
        let Some(g) = e.get::<SignalOnAction>() else {
            continue;
        };
        let object = e.get::<Name>().map_or_else(String::new, |n| n.0.clone());
        for row in g.0.iter().filter(|r| !r.action.is_empty()) {
            f.triggers.push((object.clone(), row.action.clone()));
        }
    }
    if let Some(lib) = world.get_resource::<ph2d_script::ProjectScripts>() {
        for (_, doc) in lib.0.iter() {
            f.scripts.push((doc.name.clone(), doc.text.clone()));
        }
    }
    f.triggers.sort();
    f.scripts.sort_by(|a, b| a.0.cmp(&b.0));
    f
}

/// Os leitores de cada nome de acção (os estáticos entram sempre).
#[must_use]
pub fn of_world(world: &World) -> BTreeMap<String, Vec<Reader>> {
    readers(&facts(world))
}
