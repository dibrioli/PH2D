//! ⭐ **O bench do RESTAURO** — o outro lado da F2, e a condição de pronto da **F8**.
//!
//! ```text
//! cargo test -p ph2d-ecs --release --test measure_restore -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]` de propósito** (família de flakes do `CLAUDE.md` §5.0): isto mede um RELÓGIO, e
//! um relógio sob fan-out não mede nada. Ele imprime o `load` que encontrou, para que um número
//! fora de esquadria se leia como carga e não como regressão. Irmão exacto do
//! [`measure_incremental_capture`], com a mesma forma para os dois números serem comparáveis.
//!
//! # A pergunta que este ficheiro existe para responder
//!
//! A F2 tornou a **captura** incremental (`0,189 ms` parada, `0,613` a 10 % de mundo mudado, contra
//! `23,8` do rebuild). O **restauro** continua `O(mundo)` por construção — `ProjectState::restore`
//! faz *despawn de tudo o que tem `Transform`* + `snapshot_to_world` + reconstrução dos mapas.
//!
//! ⛔⛔ **Isto é MEDIÇÃO, não uma proposta.** O plano da F8 escreve a barra `≤ 1 ms @10 k`; se o
//! número medido já estiver perto de um quadro, a fase justifica-se, e se estiver longe **a fase
//! não se justifica** e o número fica aqui escrito para quem voltar a perguntar. *Um teto sem a
//! tabela ao lado é um palpite* (`CLAUDE.md` §0.0).

use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::scene::save::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{Name, RootOrder, Transform};
use ph2d_ecs::{TransformPropagationState, WorklistBuf};
use std::time::Instant;

const ITERS: usize = 25;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    reg
}

fn load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    v[v.len() / 2]
}

fn populate(world: &mut bevy_ecs::world::World, n: u32) {
    for i in 0..n {
        world.spawn((
            Transform::IDENTITY,
            Name::new(format!("obj{i}")),
            RootOrder(i),
        ));
    }
}

/// A mediana de `ITERS` restauros completos, em ms — **exactamente os três passos que o
/// `ProjectState::restore` do shell faz**, menos as pontes (que são do shell).
fn measure_restore(n: u32, reg: &ComponentRegistry) -> (f64, usize) {
    let mut world = bevy_ecs::world::World::new();
    populate(&mut world, n);
    let mut snap = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(&mut world);
    let mut worklist = WorklistBuf::default();
    world_to_snapshot(&mut world, &mut prop, &mut worklist, reg, &mut snap).expect("captura");
    let rows = snap.entities.len();

    let mut samples = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let t0 = Instant::now();
        // 1. Limpa tudo o que é editável (a definição do shell: tem `Transform`).
        let editable: Vec<bevy_ecs::entity::Entity> = {
            let mut q = world
                .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::prelude::With<Transform>>();
            q.iter(&world).collect()
        };
        for e in editable {
            let _ = world.despawn(e);
        }
        // 2. Re-spawna do snapshot.
        let _ = snapshot_to_world(&mut world, &snap, reg);
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    (median(samples), rows)
}

#[test]
#[ignore = "mede um relogio; rode com a maquina calma e --release"]
fn the_restore_costs_the_world_and_this_is_the_number() {
    let reg = registry();
    println!(
        "\n  restauro completo (despawn tudo + snapshot_to_world) — mediana de {ITERS}, load = {:.2}",
        load_average()
    );
    println!("  ┌──────────┬──────────┬───────────────────────────────┐");
    println!("  │ entidades│  medido  │ contra um quadro de 16,7 ms   │");
    println!("  ├──────────┼──────────┼───────────────────────────────┤");
    for n in [100u32, 1_000, 10_000] {
        let (ms, rows) = measure_restore(n, &reg);
        assert_eq!(
            rows, n as usize,
            "o snapshot nao tem uma linha por entidade"
        );
        println!(
            "  │ {n:>8} │ {ms:>6.3} ms │ {:>26.1} % │",
            ms / 16.7 * 100.0
        );
    }
    println!("  └──────────┴──────────┴───────────────────────────────┘");
    println!("  (a CAPTURA, para comparar: 0,189 ms parada · 0,613 a 10 % — F2, 2026-08-25)\n");
}

// ⛔⛔⛔ **A RESIDÊNCIA do MUNDO já está resolvida, e a 1.ª versão deste ficheiro mediu-a ERRADO.**
//
// Eu escrevi aqui um teste que somava `postcard::to_allocvec(&snapshot).len() × UNDO_CAP` e
// imprimia **189 MB** a 10 k entidades. O número está certo e a pergunta estava errada: o
// [`WorldSnapshot`] guarda `Arc<EntitySnapshotRow>` **por linha** desde a F2, e a pilha de undo
// **partilha a linha de quem não mudou entre passos** — a residência real, medida na F2, é
// **~12,5 MB** contra os ~614 MB que custaria sem partilha.
//
// ⚠️ **E o doc do `WorldSnapshot` diz, na mesma frase, que a partilha NÃO viaja no fio** — a
// serde escreve um `Arc<T>` como o próprio `T`. ⇒ o tamanho SERIALIZADO é, por construção, o único
// número que **não** reflecte a residência. *Escolhi a régua que era garantidamente cega ao que eu
// queria medir, e ela devolveu um número grande e plausível.*
//
// ⇒ **este teste não existe**, e a nota fica no lugar dele. O que sobra por medir é o outro lado —
// a `VecScene`, que a captura clona INTEIRA por passo e que não partilha nada:
// [`ph2d-vec-scene/tests/measure_scene_clone.rs`].
