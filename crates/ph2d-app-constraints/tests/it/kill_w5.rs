//! **Kill-criterion do resto da W5** (`docs/BlenderConstraints/06_W5_resto.md` §5): 0,5 ms por quadro
//! para 1 000 de cada peça, na MESMA rodada — variantes no mesmo processo, intercaladas em blocos,
//! resumo = o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`).
//!
//! `#[ignore]`: o relógio só vale sozinho e no perfil `smoke` —
//! `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-constraints --test it kill_w5 -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::relogio::Relogio;
use ph2d_app_constraints::valor::valores;
use ph2d_constraints::{Kind, Valor};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_timeline::TimelineDoc;

const N: u64 = 1_000;
const TECTO: Duration = Duration::from_micros(500);
const BLOCOS: usize = 7;
const QUADROS: u32 = 40;

/// Um alvo que sobe e desce e 1 000 donos, cada um com um Valor de mola sobre a altura dele.
fn cena_valor(sim: &mut SimWorld) -> Entity {
    let w = sim.world_mut();
    let alvo = w.spawn((StableId(1), Transform::IDENTITY)).id();
    for i in 0..N {
        let mut c = Constraints::default();
        c.push(Kind::Valor(Valor::default()), Some(StableId(1)));
        w.spawn((StableId(10 + i), Transform::IDENTITY, c));
    }
    alvo
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32"
)]
fn sobe(sim: &mut SimWorld, alvo: Entity, f: u32) {
    sim.world_mut()
        .get_mut::<Transform>(alvo)
        .unwrap()
        .translation
        .y = (f64::from(f % 48) / 47.0) as f32;
}

#[test]
#[ignore = "relógio: corre sozinho, --profile smoke"]
fn w5_pieces_under_half_a_millisecond_per_thousand() {
    let doc = TimelineDoc::new();
    let fps = doc.fps_display;
    let mut sv = SimWorld::new();
    let alvo = cena_valor(&mut sv);
    let mut valor = Duration::MAX;
    let mut f = 0;
    for _bloco in 0..BLOCOS {
        for _ in 0..QUADROS {
            f += 1;
            sobe(&mut sv, alvo, f);
            let r = Relogio {
                doc: &doc,
                t: f64::from(f) / fps,
                laco: None,
            };
            let t0 = Instant::now();
            valores(&mut sv, Some(r));
            valor = valor.min(t0.elapsed());
        }
    }
    println!("kill: valor {valor:?} / 1 000 itens com mola (mínimo de {BLOCOS}×{QUADROS})");
    assert!(valor <= TECTO, "valor {valor:?} > {TECTO:?}");
}
