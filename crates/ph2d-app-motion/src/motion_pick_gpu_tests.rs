//! ⭐ GEF 19 — **o clique nas cópias cozidas NA PLACA**: [`super::do_dispositivo`] lê as cópias do
//! cozimento do dispositivo e acha o objecto dono, igual à junta da CPU para o mesmo grafo.
//!
//! ```text
//! PH2D_GPU=1 cargo test -p ph2d-app-motion --profile smoke --lib -- --ignored --nocapture motion_pick::gpu_tests
//! ```

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

use super::{CacheDoClique, Copia, da_cpu, do_dispositivo, donos_em, pose};
use crate::motion_bridge::quadro::{MaoDoEditor, o_quadro};
use crate::motion_objeto::{Dono, IDENTIDADE};
use crate::motion_state::MotionState;

const DT: f64 = 1.0 / 60.0;
const OBJETO: u64 = 41;

fn liga(g: &mut Graph, a: NodeId, porta: u16, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, porta),
        delayed: false,
    })
    .expect("ligar");
}

/// `source.object` (um ladrilho do átlas) carimbado numa grelha `lado × lado`, do objecto `OBJETO`.
fn cena(lado: f32, placa: bool) -> MotionState {
    let mut m = MotionState::new();
    m.pump.cook.set_external(
        "Particle",
        Stream::new(1)
            .with("texture_id", Column::Scalar(vec![0.0]))
            .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 1.0, 1.0]]))
            .with("size", Column::Vec2(vec![[0.08, 0.08]])),
    );
    let g = &mut m.doc.graph;
    let obj = g.add_node("source.object");
    g.set_text_param(obj, "object", "Particle");
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", lado);
    g.set_param(grid, "cols", lado);
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    liga(g, obj, 0, dup);
    liga(g, grid, 1, dup);
    liga(g, dup, 0, out);
    for (i, n) in [obj, grid, dup, out].into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "quatro nos")]
        g.set_pos(
            n,
            Pos {
                x: i as f32 * 200.0,
                y: 0.0,
            },
        );
    }
    for n in [obj, grid, dup, out] {
        m.doc.set_owner(n, OBJETO);
    }
    m.doc.graph.validate(&m.registry).expect("bem tipado");
    m.objeto.donos.insert(
        OBJETO,
        Dono {
            entity: OBJETO,
            visible: true,
            afim: IDENTIDADE,
        },
    );
    m.gpu_enabled = placa;
    m
}

fn quadro(m: &mut MotionState, gpu: Option<&ph2d_gpu::GpuContext>, tique: u64) {
    let mut p = ph2d_core::Playhead::new(DT);
    #[expect(clippy::cast_precision_loss, reason = "tiques")]
    p.seek(tique as f64 * DT);
    o_quadro(m, gpu, &p, DT, MaoDoEditor::default());
}

fn adaptador() -> ph2d_gpu::GpuContext {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter — este gate mede a rota do dispositivo e não tem versão de CPU")
}

/// As duas cenas cozidas: a da placa (CONTROLO: `gpu_live`) e a da CPU.
fn as_duas(gpu: &ph2d_gpu::GpuContext, lado: f32) -> (MotionState, MotionState) {
    let (mut placa, mut cpu) = (cena(lado, true), cena(lado, false));
    for t in 0..2 {
        quadro(&mut placa, Some(gpu), t);
        quadro(&mut cpu, None, t);
    }
    assert!(
        placa.gpu_live,
        "CONTROLO: a cena tem de cozer na PLACA, senão o gate mede a CPU — {:?}",
        placa.route_said
    );
    assert!(!cpu.gpu_live, "CONTROLO: a outra cena coze na CPU");
    (placa, cpu)
}

fn centro(c: &Copia) -> [f32; 2] {
    let p = pose(c) * ph2d_vector::Point::new(0.0, 0.0);
    #[expect(clippy::cast_possible_truncation, reason = "o mundo é f32")]
    let c = [p.x as f32, p.y as f32];
    c
}

/// ⭐ **O clique numa cópia cozida na placa acha o dono**, igual à rota da CPU; o vazio, nada.
/// (Mutação: `do_dispositivo` a devolver `Vec::new()` ⇒ RED na 1.ª afirmação.)
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_clique_na_copia_cozida_na_placa_acha_o_dono() {
    let gpu = adaptador();
    let (placa, cpu) = as_duas(&gpu, 8.0);
    let lidas = do_dispositivo(&placa, &gpu);
    let juntas = da_cpu(&cpu);
    eprintln!(
        "  placa: {} cópias · cpu: {} cópias",
        lidas.len(),
        juntas.len()
    );
    assert_eq!(lidas.len(), 64, "a placa leu as 64 cópias da grelha");
    assert_eq!(lidas.len(), juntas.len(), "as duas rotas, as mesmas cópias");
    assert!(lidas.iter().all(|c| c.dono == OBJETO));
    for k in [0, 27, 63] {
        let ponto = centro(&lidas[k]);
        let na_placa = donos_em(&placa, &gpu, &mut CacheDoClique::default(), ponto);
        let na_cpu = donos_em(&cpu, &gpu, &mut CacheDoClique::default(), ponto);
        assert_eq!(na_placa, vec![OBJETO], "cópia {k} em {ponto:?}");
        assert_eq!(na_placa, na_cpu, "cópia {k}: placa × CPU");
    }
    let vazio = [1.0e4, -1.0e4];
    assert!(donos_em(&placa, &gpu, &mut CacheDoClique::default(), vazio).is_empty());
    assert!(donos_em(&cpu, &gpu, &mut CacheDoClique::default(), vazio).is_empty());
}

fn mediana(v: &mut [u128]) -> u128 {
    v.sort_unstable();
    v[v.len() / 2]
}

/// Sonda (MEDIR_VELOCIDADE): a leitura do realce na placa × a junta da CPU, cada leitura depois de
/// um cozimento novo na placa (a leitura espera a fila). 7 rodadas × 20, blocos rodados.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn sonda_a_leitura_do_realce_na_placa() {
    let gpu = adaptador();
    for lado in [8.0f32, 64.0] {
        let (mut placa, cpu) = as_duas(&gpu, lado);
        let (mut ns_placa, mut ns_cpu) = (Vec::new(), Vec::new());
        let mut tique = 2;
        for rodada in 0..7 {
            for bloco in 0..2 {
                let pela_placa = (rodada + bloco) % 2 == 0;
                for _ in 0..20 {
                    quadro(&mut placa, Some(&gpu), tique);
                    tique += 1;
                    let t = std::time::Instant::now();
                    let n = if pela_placa {
                        do_dispositivo(&placa, &gpu).len()
                    } else {
                        da_cpu(&cpu).len()
                    };
                    let dt = t.elapsed().as_nanos();
                    std::hint::black_box(n);
                    if pela_placa {
                        ns_placa.push(dt);
                    } else {
                        ns_cpu.push(dt);
                    }
                }
            }
        }
        assert!(placa.gpu_live, "CONTROLO: a sonda mediu a placa");
        let (min_p, min_c) = (ns_placa.iter().min().copied(), ns_cpu.iter().min().copied());
        let (med_p, med_c) = (mediana(&mut ns_placa), mediana(&mut ns_cpu));
        eprintln!("| cópias | variante | mínimo (µs) | mediana (µs) |");
        eprintln!("|---:|---|---:|---:|");
        for (nome, min, med) in [("do_dispositivo", min_p, med_p), ("da_cpu", min_c, med_c)] {
            let min = min.unwrap_or_default();
            eprintln!(
                "| {} | {nome} | {:.1} | {:.1} |",
                lado * lado,
                min as f64 / 1e3,
                med as f64 / 1e3
            );
        }
    }
}
