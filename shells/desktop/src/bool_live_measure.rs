//! **AS SONDAS DE CUSTO da booleana viva** — irmã dos gates pelo teto de 600 LOC do HR-18.
//!
//! ⚠️ O corte é por NATUREZA, e não por tamanho: um gate afirma e falha alto; uma sonda MEDE e
//! imprime, e corre `#[ignore]`, sob demanda, no perfil `--release`. Misturá-las faz a suíte de
//! toda a gente carregar medições que ninguém lê — e faz quem procura um número ter de o achar
//! entre asserções.

use super::*;

/// **Quanto custa um frame de booleana viva** — o número que decide se ela é animável.
///
/// Rodar: `cargo test -p ph2d-host-desktop --bins measure_a_live_boolean_frame --release
/// -- --ignored --nocapture`
///
/// ⚠️ O oráculo é o `recook` INTEIRO (a caminhada da árvore, o assamento em mundo, o motor e o
/// mapa), não o `pathfinder` isolado: o que o artista paga por frame é a porta do produto.
#[test]
#[ignore = "sonda de custo — roda sob demanda"]
fn measure_a_live_boolean_frame() {
    use std::time::Instant;
    println!("\n--- custo de UM frame de booleana viva (o `recook` inteiro) ---");
    for (name, op) in [("Union", 0u8), ("Subtract", 1), ("Intersect", 2)] {
        for (shape, n) in [("par simples", 2usize), ("dez operandos", 10)] {
            let mut sim = SimWorld::default();
            let mut scene = VecScene::new();
            let mut map = VecEntityMap::new();
            let mut ids = Vec::new();
            for i in 0..n {
                let x = i as f64 * 0.7;
                ids.push(scene.push_path(ph2d_vec_scene::ellipse([x, 0.0], 1.0, 1.0)));
            }
            crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
            let members: Vec<u64> = ids.iter().map(|i| map[i]).collect();
            let g = Entity::from_bits(
                crate::vec_entities::group_entities(&mut sim, &members, "B".into()).unwrap(),
            );
            sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });

            let mut bl = BoolLive::default();
            let xf = VecXforms::default();
            // Uma corrida a frio, e depois a MEDIÇÃO com o memo INVALIDADO a cada volta — é o
            // caso do arrasto, que é o único em que o custo importa. Um memo quente mede zero.
            let mut live = LiveGeometry::new();
            bl.recook(&scene, &sim, &map, &xf, &[], &mut live);
            let t = Instant::now();
            const N: u32 = 20;
            for k in 0..N {
                // Move um operando: invalida o memo, como um arrasto faz.
                let dx = f64::from(k) * 1e-4;
                for v in &mut scene.path_mut(ids[0]).unwrap().verts {
                    v.anchor[0] += dx;
                }
                let mut live = LiveGeometry::new();
                bl.recook(&scene, &sim, &map, &xf, &[], &mut live);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
            println!("  {op:>1} {name:<10} | {shape:<14} | {ms:>7.3} ms/frame");
        }
    }
    println!("\nOrcamento de um quadro a 60 fps: 16,6 ms.");
}

// ============================================================================
// **UM VERBO POR FORMA** (Enio, 2026-08-22) — os gates da cadeia dentro do grupo.
//
// O modelo: *as formas combinam-se na ordem da hierarquia, e cada uma traz o verbo com que dobra
// sobre o resultado das anteriores.* É o compound shape vivo do Illustrator.
//
// ⚠️ Estes gates vêm em pares CAPACIDADE/HERANÇA de propósito. A capacidade sozinha passaria com
// uma implementação que ignorasse o `op` do grupo; a herança sozinha passaria com uma que
// ignorasse o override. É a existência das duas que prende o desenho.
// ============================================================================
