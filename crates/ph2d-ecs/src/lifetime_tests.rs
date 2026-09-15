//! Os gates da lei de ciclo de vida (`docs/Components/09_plano_spawner.md` §5, W1).

use super::*;
use crate::{ChildOf, Name};
use bevy_ecs::world::World as BevyWorld;
use ph2d_core::Vec2;

const DT: u64 = 16_667; // um tique a 60 Hz, em µs

/// Uma cópia como a fábrica a põe: pose, nome, identidade e a marca de nascimento.
fn nascida(world: &mut BevyWorld, nome: &str, id: u64, x: f32, born: u64) -> Entity {
    world
        .spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Name::new(nome),
            StableId(id),
            Spawned {
                by: 7,
                born_tick: born,
            },
        ))
        .id()
}

/// ⭐⭐ **A vida acaba no tique EXACTO, e o resto do período fica** — a lei que o oráculo mediu
/// (Godot: um período de `6,3` tiques dá `[6,6,7,6,6,7]`, média `6,273`; re-zerar daria `7,7,7`).
///
/// Aqui a vida é de `100 000 µs` = `6` tiques de `16 667` + `2 µs`: ela **não** pode morrer ao 6.º
/// (`100 002 >= 100 000`… ao 6.º já passou) — o oráculo do teste é o tique em que a morte aparece.
///
/// ⭐⭐ **E ele mede também que se morre UMA vez.** A 1.ª redacção da lei reportava a morte em
/// **todo** tique a partir do sexto (`[(6),(7),(8)]`), porque quem remove a entidade é o dreno da
/// shell, um quadro depois — este gate apanhou-o antes de existir uma linha de shell. A cura é a do
/// oráculo pelo outro lado: no Godot um segundo `queue_free()` não faz nada porque a bandeira já
/// está posta, e a nossa bandeira é o relógio ter **atravessado** a duração.
///
/// (Mutação: trocar `>=` por `>` ⇒ a morte escorrega um tique. Tirar o `antes <` ⇒ ela repete-se.)
#[test]
fn a_life_ends_on_the_tick_its_time_is_up() {
    // ⚠️⚠️ **A duração é um MÚLTIPLO EXACTO do tique, e isso é a fixtura** — medido em 2026-09-14
    // por uma mutação que SOBREVIVEU: com `100 000 µs` (6,000 tiques e um resto) o relógio chega a
    // `100 002` e tanto `>=` como `>` matam no mesmo tique, logo a fronteira não é observável. *Uma
    // fixtura que não contém o fenómeno deixa a lei sem gate, com a suíte verde.*
    let mut w = BevyWorld::new();
    let e = nascida(&mut w, "bala", 1, 0.0, 0);
    w.entity_mut(e).insert(Lifetime {
        duration_us: 6 * DT,
        on_death: "sumiu".into(),
    });
    let mut vistos = Vec::new();
    for t in 1..=8u64 {
        let mortes = tick_lifetimes(&mut w, DT, t);
        if !mortes.is_empty() {
            vistos.push((t, mortes[0].signal.clone(), mortes[0].why));
        }
    }
    assert_eq!(
        vistos,
        vec![(6, "sumiu".to_string(), DeathCause::Aged)],
        "o 6.º tique e' o que COMPLETA a vida — nem o 5.º nem o 7.º"
    );
}

/// ⭐⭐⭐ **Um objecto AUTORADO nunca morre de velho** — a lei do §2.6: *uma corrida não apaga
/// documento*.
///
/// (Mutação: tirar o `&Spawned` da query ⇒ o objecto desenhado morre e isto fica RED.)
#[test]
fn a_lifetime_on_an_authored_object_never_kills() {
    let mut w = BevyWorld::new();
    let e = w
        .spawn((
            Transform::IDENTITY,
            Name::new("Parede"),
            StableId(1),
            Lifetime {
                duration_us: 1,
                on_death: "nunca".into(),
            },
        ))
        .id();
    for t in 1..=10u64 {
        assert!(
            tick_lifetimes(&mut w, DT, t).is_empty(),
            "a corrida apagou um objecto que o artista desenhou"
        );
    }
    assert!(w.get_entity(e).is_ok(), "e a lei nao toca no mundo");
    // O controlo: a MESMA entidade, com a marca de nascimento, morre.
    w.entity_mut(e).insert(Spawned {
        by: 7,
        born_tick: 0,
    });
    assert_eq!(tick_lifetimes(&mut w, DT, 1).len(), 1, "o controlo");
}

/// ⭐⭐ **O recém-nascido não envelhece no tique em que nasce** — a lei D do oráculo (nascido no
/// tique `3`, tem `1` tique no `4`).
///
/// (Mutação: trocar `>=` por `>` no guarda do `born_tick` ⇒ ele morre no tique do nascimento.)
#[test]
fn the_newborn_does_not_age_on_the_tick_it_was_born() {
    let mut w = BevyWorld::new();
    let e = nascida(&mut w, "faisca", 1, 0.0, 5);
    w.entity_mut(e).insert(Lifetime {
        duration_us: 1,
        on_death: String::new(),
    });
    assert!(
        tick_lifetimes(&mut w, DT, 5).is_empty(),
        "ela morreu no tique em que nasceu — ninguem a viu"
    );
    assert_eq!(
        tick_lifetimes(&mut w, DT, 6).len(),
        1,
        "e morre no seguinte"
    );
}

/// **Duração `0` não mata** — a mesma recusa embutida do `Timer`: um campo por preencher não pode
/// ser um gesto destrutivo.
#[test]
fn a_zero_duration_never_kills() {
    let mut w = BevyWorld::new();
    let e = nascida(&mut w, "eterno", 1, 0.0, 0);
    w.entity_mut(e).insert(Lifetime {
        duration_us: 0,
        on_death: "x".into(),
    });
    for t in 1..=100u64 {
        assert!(tick_lifetimes(&mut w, DT, t).is_empty());
    }
}

/// ⭐ **As mortes saem pela ordem da IDENTIDADE, nunca pela da query** — senão o replay publica os
/// sinais noutra ordem e diverge.
///
/// ⚠️ **A fixtura nasce ao contrário de propósito:** os ids `3, 1, 2` são criados nessa ordem, então
/// a ordem do arquétipo e a da identidade são diferentes — sem isso o gate passaria por acaso.
#[test]
fn the_deaths_come_in_identity_order_not_in_query_order() {
    let mut w = BevyWorld::new();
    for id in [3u64, 1, 2] {
        let e = nascida(&mut w, &format!("m{id}"), id, 0.0, 0);
        w.entity_mut(e).insert(Lifetime {
            duration_us: 1,
            on_death: format!("morreu{id}"),
        });
    }
    let nomes: Vec<String> = tick_lifetimes(&mut w, DT, 1)
        .into_iter()
        .map(|d| d.signal)
        .collect();
    assert_eq!(nomes, vec!["morreu1", "morreu2", "morreu3"]);
}

/// ⭐⭐ **A margem CRESCE o rectângulo**, e uma margem negativa não o encolhe.
///
/// (Mutação: tirar o `.max(0.0)` ⇒ uma margem negativa mata o que está à vista.)
#[test]
fn the_margin_grows_the_rectangle_and_a_negative_one_does_not_shrink_it() {
    let mut w = BevyWorld::new();
    // Meia-janela de 5 m; o objecto está a 6 m — fora da borda, dentro da margem de 2 m.
    let e = nascida(&mut w, "bala", 1, 6.0, 0);
    w.entity_mut(e).insert(DestroyOutside { margin: 2.0 });
    assert!(
        reap_outside(&mut w, [0.0, 0.0], [5.0, 5.0]).is_empty(),
        "a margem tem de o segurar"
    );
    w.entity_mut(e).insert(DestroyOutside { margin: 0.0 });
    assert_eq!(
        reap_outside(&mut w, [0.0, 0.0], [5.0, 5.0]).len(),
        1,
        "sem margem, 6 m esta' fora de 5 m"
    );
    // E uma margem negativa não pode matar quem está DENTRO.
    let d = nascida(&mut w, "dentro", 2, 1.0, 0);
    w.entity_mut(d).insert(DestroyOutside { margin: -10.0 });
    let mortos: Vec<Entity> = reap_outside(&mut w, [0.0, 0.0], [5.0, 5.0])
        .into_iter()
        .map(|m| m.entity)
        .collect();
    assert!(!mortos.contains(&d), "uma margem negativa encolheu o ecra'");
}

/// ⭐⭐ **O ponto testado é o de MUNDO, nunca o local** — uma bala filha de uma nave leria a posição
/// dela em relação à nave e nunca sairia do ecrã.
///
/// (Mutação: ler `Transform::translation` em vez de `world_transform` ⇒ RED.)
#[test]
fn outside_is_measured_in_world_space_not_local() {
    let mut w = BevyWorld::new();
    let nave = w
        .spawn((
            Transform::from_translation(Vec2::new(100.0, 0.0)),
            Name::new("Nave"),
            StableId(1),
        ))
        .id();
    // A bala está na ORIGEM da nave (local `0`), logo a 100 m do centro do mundo.
    let bala = w
        .spawn((
            Transform::IDENTITY,
            Name::new("Bala"),
            StableId(2),
            Spawned {
                by: 7,
                born_tick: 0,
            },
            DestroyOutside { margin: 0.0 },
            ChildOf(nave),
        ))
        .id();
    let mortos = reap_outside(&mut w, [0.0, 0.0], [5.0, 5.0]);
    assert_eq!(
        mortos.iter().map(|m| m.entity).collect::<Vec<_>>(),
        vec![bala],
        "a pose LOCAL dela e' (0,0) — so' a de mundo a poe fora do ecra'"
    );
}

/// **Um objecto autorado também não morre por sair do ecrã** — a mesma lei do §2.6, pelo outro
/// caminho. (Uma cerca só num dos dois é a forma que a §5.0 do `CLAUDE.md` chama de *um braço só*.)
#[test]
fn an_authored_object_never_dies_outside() {
    let mut w = BevyWorld::new();
    let e = w
        .spawn((
            Transform::from_translation(Vec2::new(999.0, 0.0)),
            Name::new("Montanha"),
            StableId(1),
            DestroyOutside { margin: 0.0 },
        ))
        .id();
    assert!(reap_outside(&mut w, [0.0, 0.0], [5.0, 5.0]).is_empty());
    // O controlo, outra vez: com a marca, ela morre.
    w.entity_mut(e).insert(Spawned {
        by: 7,
        born_tick: 0,
    });
    assert_eq!(reap_outside(&mut w, [0.0, 0.0], [5.0, 5.0]).len(), 1);
}

/// **O relógio nasce sozinho em quem tem vida e marca de nascimento**, e não em mais ninguém.
#[test]
fn the_clock_is_only_given_to_those_who_were_born_in_a_run() {
    let mut w = BevyWorld::new();
    let viva = nascida(&mut w, "viva", 1, 0.0, 0);
    w.entity_mut(viva).insert(Lifetime::default());
    let autorada = w
        .spawn((
            Transform::IDENTITY,
            Name::new("autorada"),
            StableId(2),
            Lifetime::default(),
        ))
        .id();
    reconcile_lifetimes(&mut w);
    assert!(w.get::<LifetimeRuntime>(viva).is_some());
    assert!(
        w.get::<LifetimeRuntime>(autorada).is_none(),
        "o relogio foi parar a um objecto que nunca vai correr"
    );
}
