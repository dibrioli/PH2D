//! **Quem liga o som ao objeto:** uma função só. A ligação é escrita no mundo, e a fila de desfazer da cena a vê pelo diff.

use ph2d_ecs::{AudioSource2D, Entity, Name, SoundSet, StableId, World, assign_missing_stable_ids};

/// Liga o som ao objeto: entrada do conjunto, se a lista é quem manda, ou o campo Sound. Objeto sem fonte ganha uma com
/// Autoplay, senão nada a faria tocar; fonte que já existe fica com o Autoplay que o artista deixou.
pub fn liga(world: &mut World, objeto: Entity, som: &str) {
    let lista_manda = world
        .get::<SoundSet>(objeto)
        .is_some_and(SoundSet::has_sounds);
    if world.get::<AudioSource2D>(objeto).is_none() {
        world.entity_mut(objeto).insert(AudioSource2D {
            autoplay: true,
            ..AudioSource2D::default()
        });
    }
    if lista_manda {
        let campo = world
            .get::<AudioSource2D>(objeto)
            .map(|f| f.sound.clone())
            .unwrap_or_default();
        if let Some(mut lista) = world.get_mut::<SoundSet>(objeto)
            && lista.add_keeping(&campo, som)
        {
            lista.sanitize();
        }
    } else if let Some(mut f) = world.get_mut::<AudioSource2D>(objeto)
        && f.sound != som
    {
        f.sound = som.to_owned();
    }
}

/// Os objetos que a lista do Use in Game oferece, por nome.
#[must_use]
pub fn objetos_da_cena(world: &mut World) -> Vec<(u64, String)> {
    assign_missing_stable_ids(world);
    let mut v: Vec<(u64, String)> = world
        .query::<(&Name, &StableId)>()
        .iter(world)
        .map(|(n, id)| (id.0, n.as_str().to_owned()))
        .collect();
    v.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    v
}
