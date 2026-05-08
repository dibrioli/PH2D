//! C2 spike — query overhead bench (simplified).
//!
//! Mede overhead do Luau interpreter vs Rust nativo iterando 1000
//! estruturas com 3 "components" (posição+velocidade+health). Atualiza
//! position += velocity e decrementa health.
//!
//! **Limitação importante:** o plano L48 fala em "sistema iterando 1000
//! entidades com 3 componentes em Luau vs Rust nativo". Para fazer a
//! versão fiel, precisaríamos de bridge Luau↔ECS world (não existe
//! ainda). Esta bench mede o proxy: overhead do interpreter Luau vs
//! Rust direto. Pós-bridge, refazer com cross-FFI real.
//!
//! Threshold C2: Luau ≤ 5× Rust nativo (p99). Acima → fricção
//! inaceitável para LLM.
//!
//! Run:
//!   cargo bench --bench c2_query -p ph2d-spike

use criterion::{Criterion, criterion_group, criterion_main};
use mlua::{Function, Lua};
use std::hint::black_box;

const N: usize = 1000;

#[derive(Clone, Copy)]
struct Entity {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    hp: i32,
}

fn rust_native(entities: &mut [Entity]) {
    for e in entities {
        e.x += e.vx;
        e.y += e.vy;
        e.hp -= 1;
    }
}

const LUAU_SCRIPT: &str = r#"
--!strict
local entities = table.create(1000)
for i = 1, 1000 do
    entities[i] = { x = i, y = 0.0, vx = 1.0, vy = 1.0, hp = 100 }
end

function tick()
    for i = 1, 1000 do
        local e = entities[i]
        e.x = e.x + e.vx
        e.y = e.y + e.vy
        e.hp = e.hp - 1
    end
end
"#;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("c2_query");

    let mut buf: Vec<Entity> = (0..N)
        .map(|i| Entity { x: i as f32, y: 0.0, vx: 1.0, vy: 1.0, hp: 100 })
        .collect();
    group.bench_function("rust_native_1000", |b| {
        b.iter(|| {
            rust_native(black_box(&mut buf));
        });
    });

    let lua = Lua::new();
    lua.load(LUAU_SCRIPT).exec().unwrap();
    let tick: Function = lua.globals().get("tick").unwrap();
    group.bench_function("luau_table_1000", |b| {
        b.iter(|| {
            tick.call::<()>(()).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
