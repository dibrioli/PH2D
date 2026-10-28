//! O ponto de simulação pelo quadro do PRODUTO (`o_quadro` arma as tomadas, [`publica`] devolve):
//! o `id` de cada partícula chega à pilha, igual a tocar e por scrub, e igual na placa e na CPU.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib -- --ignored --nocapture the_particle_ids_are_the_same
//! ```

use super::*;
use crate::motion_bridge::quadro::{MaoDoEditor, o_quadro};
use crate::motion_objeto::{Dono, IDENTIDADE};
use ph2d_ecs::StableId;
use ph2d_nodegraph::graph::{Edge, Graph};

const DT: f64 = 1.0 / 60.0;
const OBJETO: u64 = 5;

/// `emitter → integrate → output` do objecto `OBJETO` (vivo, visível), com vida curta: morrem
/// partículas.
fn cena(gpu: bool) -> (MotionState, World) {
    let mut m = MotionState::new();
    let mut g = Graph::new();
    let emissor = g.add_node("motion.emitter");
    g.set_param(emissor, "life", 0.25);
    let integra = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    for (de, para) in [(emissor, integra), (integra, out)] {
        g.connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("liga");
    }
    // O laço `pre` do integrador (sem forças): a velocidade do emissor acumula-se.
    g.connect(Edge {
        from: (integra, 0),
        to: (integra, 1),
        delayed: true,
    })
    .expect("liga o pre");
    m.doc.graph = g;
    for n in [emissor, integra, out] {
        m.doc.set_owner(n, OBJETO);
    }
    m.objeto.donos.insert(
        OBJETO,
        Dono {
            entity: 0,
            visible: true,
            afim: IDENTIDADE,
        },
    );
    m.gpu_enabled = gpu;
    // Sem a lei «só com forma» o gizmo de pontos não pede o sink: a tomada medida é a do pino.
    m.pump.define_a_lei(false);
    let mut w = World::new();
    w.insert_resource(ParticulasPedidas([StableId(OBJETO)].into()));
    // O 1.º quadro só lê o pedido: as tomadas armam-se no cozimento seguinte.
    publica(&mut m, &mut w);
    (m, w)
}

fn quadro(
    m: &mut MotionState,
    w: &mut World,
    gpu: Option<&ph2d_gpu::GpuContext>,
    p: &ph2d_core::Playhead,
) -> BTreeMap<u32, [f64; 2]> {
    let mao = MaoDoEditor {
        selecionado: None,
        arrasto: None,
    };
    o_quadro(m, gpu, p, DT, mao);
    if let Some(gpu) = gpu {
        crate::motion_bridge::gpu::espera_as_tomadas(m, gpu);
    }
    publica(m, w);
    w.get_resource::<Particulas>()
        .and_then(|p| p.0.get(&StableId(OBJETO)).cloned())
        .unwrap_or_default()
}

#[expect(clippy::cast_precision_loss, reason = "tiques")]
fn em(p: &mut ph2d_core::Playhead, t: u64) {
    p.seek(t as f64 * DT);
}

#[test]
fn a_particle_keeps_its_id_and_play_equals_scrub() {
    let (mut m, mut w) = cena(false);
    let mut p = ph2d_core::Playhead::new(DT);
    p.play();
    let mut tocado = Vec::new();
    for t in 0..=40 {
        em(&mut p, t);
        tocado.push(quadro(&mut m, &mut w, None, &p));
    }
    assert!(
        !tocado[1].is_empty(),
        "o pedido arma a tomada: no tique 1 já há partículas — {:?}",
        m.route_said
    );
    // A mesma partícula, de um tique para o seguinte: o id fica e ela ANDA.
    let id = *tocado[5].keys().next().expect("vivas no tique 5");
    assert!(tocado[6].contains_key(&id) && tocado[6][&id] != tocado[5][&id]);
    // Vida de 0,25 s = 15 tiques: no 40 ela morreu, e os ids de agora são outros.
    assert!(!tocado[40].contains_key(&id), "a partícula {id} já morreu");
    assert!(tocado[40].keys().all(|&i| i > id));

    // O scrub, por ordem salteada, numa bomba nova: o mesmo bit em cada tique.
    let (mut m, mut w) = cena(false);
    let mut p = ph2d_core::Playhead::new(DT);
    em(&mut p, 0);
    quadro(&mut m, &mut w, None, &p);
    for t in [40, 12, 30, 5, 6, 39, 1, 22] {
        em(&mut p, t);
        let lido = quadro(&mut m, &mut w, None, &p);
        assert_eq!(lido, tocado[t as usize], "tique {t}: scrub ≠ tocar");
    }
}

#[test]
fn a_pin_armed_while_paused_reads_without_moving_the_simulation() {
    let mut m = cena(false).0;
    let mut w = World::new();
    let mut p = ph2d_core::Playhead::new(DT);
    p.play();
    let mut referencia = cena(false);
    for t in 0..=10 {
        em(&mut p, t);
        quadro(&mut m, &mut w, None, &p);
        quadro(&mut referencia.0, &mut referencia.1, None, &p);
    }
    p.pause();
    assert!(w.get_resource::<Particulas>().is_none(), "sem pedido, nada");
    // O pedido chega com a cena parada: a tomada sai no quadro seguinte, no MESMO tique.
    w.insert_resource(ParticulasPedidas([StableId(OBJETO)].into()));
    quadro(&mut m, &mut w, None, &p);
    let lido = quadro(&mut m, &mut w, None, &p);
    let certo = quadro(&mut referencia.0, &mut referencia.1, None, &p);
    assert!(!lido.is_empty());
    assert_eq!(
        lido, certo,
        "parado, o tique repete-se pelo scrub: o mesmo bit"
    );
    // E tocar a seguir continua o mesmo filme (a simulação não andou um passo a mais).
    p.play();
    em(&mut p, 11);
    assert_eq!(
        quadro(&mut m, &mut w, None, &p),
        quadro(&mut referencia.0, &mut referencia.1, None, &p)
    );
}

#[test]
fn the_first_sink_with_an_id_column_answers() {
    use ph2d_nodegraph::attr::Column;
    let sem_id = Stream::new(1).with("P", Column::Vec2(vec![[9.0, 9.0]]));
    let a = Stream::new(2)
        .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
        .with("id", Column::Scalar(vec![7.0, 16_777_215.0]));
    let b = Stream::new(1)
        .with("P", Column::Vec2(vec![[5.0, 5.0]]))
        .with("id", Column::Scalar(vec![7.0]));
    let m = do_objeto([&sem_id, &a, &b]).unwrap();
    assert_eq!(
        m,
        BTreeMap::from([(7, [1.0, 2.0]), (16_777_215, [3.0, 4.0])])
    );
    assert!(do_objeto([&sem_id]).is_none());
}

fn adaptador() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

/// ⭐ A placa entrega os MESMOS ids que a CPU, no mesmo tique (a posição a menos do f32 da placa).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn the_particle_ids_are_the_same_on_the_device_and_the_cpu() {
    let Some(gpu) = adaptador() else {
        panic!("sem adapter — este gate mede a rota do device e não tem versão de CPU");
    };
    let mut cpu = cena(false);
    let mut placa = cena(true);
    let mut p = ph2d_core::Playhead::new(DT);
    p.play();
    for t in 0..=30 {
        em(&mut p, t);
        let c = quadro(&mut cpu.0, &mut cpu.1, None, &p);
        let g = quadro(&mut placa.0, &mut placa.1, Some(&gpu), &p);
        assert!(
            placa.0.gpu_live,
            "tique {t}: a placa cozeu — {:?}",
            placa.0.route_said
        );
        if t < 2 {
            continue;
        }
        assert!(!c.is_empty(), "tique {t}: há partículas");
        assert_eq!(
            c.keys().collect::<Vec<_>>(),
            g.keys().collect::<Vec<_>>(),
            "tique {t}: os ids da placa ({:?}) ≠ os da CPU",
            placa.0.route_said
        );
        for (id, q) in &c {
            let d = g[id];
            assert!(
                (q[0] - d[0]).abs() < 1e-3 && (q[1] - d[1]).abs() < 1e-3,
                "tique {t}, id {id}: {q:?} ≠ {d:?}"
            );
        }
    }
}
