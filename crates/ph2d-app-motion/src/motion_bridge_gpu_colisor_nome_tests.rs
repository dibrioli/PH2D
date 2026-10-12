//! ⭐ **O COLISOR PELO NOME NA PLACA** (doc 131 D.14) — uma grelha de discos `source.shape` com o
//! `Collide` DESLIGADO, e o raio de cada peça escrito pelo `Custom` do `motion.drive`
//! (`column = "collider"`, o valor `|P|·s`): a rota é a placa, e as formas caem onde a CPU as põe.

use super::super::passe_tests::{DT, TIQUE, formas_da_cpu, formas_da_placa};
use super::super::{GpuOutcome, cook_gpu};
use super::{RECUSA_COLISOR, RECUSA_PASSE, peca_declara_colisor};
use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_node_motion_shape::{ShapeKind, param};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::graph::{Edge, NodeId};

const CUSTOM: f32 = 9.0;
const VAO: f32 = 0.1;
/// A tolerância do doc 130 §4: relativa ao `alcance_max` (o maior raio de mundo).
const EPS_RELATIVO: f32 = 1e-5;
/// O controlo «o passe MEXE»: o mesmo documento a escrever outro nome desenha noutro sítio.
const MEXE: f32 = 1e-3;

/// `shape(Circle) + grid(lado × lado) → duplicator → drive → output(8 varreduras)`; o drive escreve
/// `nome` no `canal`, com o valor `|P| · s` de um `value.attribute`. Devolve o estado e o drive.
pub(crate) fn cena(lado: usize, canal: f32, nome: &str) -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let liga = |g: &mut ph2d_nodegraph::graph::Graph, de: NodeId, para: NodeId, porta: u16| {
        g.connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: false,
        })
        .expect("fio");
    };
    let forma = g.add_node("source.shape");
    g.set_param(forma, param::KIND, ShapeKind::Circle as i32 as f32);
    g.set_param(forma, param::SIZE, 0.08);
    g.set_param(forma, param::COLLIDE, 0.0);
    let grade = g.add_node("motion.grid");
    g.set_param(grade, "rows", lado as f32);
    g.set_param(grade, "cols", lado as f32);
    g.set_param(grade, "gap_x", VAO);
    g.set_param(grade, "gap_y", VAO);
    let dup = g.add_node("motion.duplicator");
    liga(g, forma, dup, 0);
    liga(g, grade, dup, 1);
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, "attr", "P");
    g.set_param(attr, "mode", 1.0);
    liga(g, dup, attr, 0);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", canal);
    g.set_param(drive, "mode", 1.0);
    // O raio (unidade da geometria, × `size`) cresce para fora: `≈ 1` na borda — o disco de mundo
    // passa o meio vão e sobrepõe-se aos vizinhos; o centro fica solto.
    g.set_param(drive, "scale", 14.0 / lado as f32);
    g.set_text_param(drive, "column", nome);
    liga(g, dup, drive, 0);
    liga(g, attr, drive, 1);
    let out = g.add_node("motion.output");
    g.set_param(out, ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_PARAM, 8.0);
    liga(g, drive, out, 0);
    m.sinks = vec![out];
    (m, drive)
}

/// O maior raio de MUNDO que a CPU declara, `collider · max(|sx|, |sy|)`, lido da tomada do drive.
fn alcance_max(cpu: &MotionState, drive: NodeId) -> f32 {
    let s = cpu
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == drive)
        .map(|(_, s)| s)
        .expect("a tomada do drive");
    let (Some(Column::Scalar(c)), Some(Column::Vec2(sz))) = (s.get("collider"), s.get("size"))
    else {
        panic!("a corrente do drive tem `collider` e `size`");
    };
    c.iter()
        .zip(sz)
        .map(|(r, z)| r * z[0].abs().max(z[1].abs()))
        .fold(0.0, f32::max)
}

/// O pior desvio de uma forma à forma MAIS PRÓXIMA do outro lado, nos dois sentidos — a ordenação
/// lexicográfica de [`pior`] troca pares cujo `x` difere num ulp (medido: `7e-1` sobre `3e-8`).
fn pior_sem_ordem(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    let um = |a: &[[f32; 2]], b: &[[f32; 2]]| {
        a.iter()
            .map(|p| {
                b.iter()
                    .map(|q| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
                    .fold(f32::INFINITY, f32::min)
            })
            .fold(0.0, f32::max)
    };
    um(a, b).max(um(b, a))
}

/// ⭐ **A escrita do raio pelo nome DECLARA** (arma a cerca do fim) — só o raio, só no Custom.
#[test]
fn a_escrita_do_raio_pelo_nome_declara() {
    for (canal, nome, esperado) in [
        (CUSTOM, "collider", true),
        (0.0, "collider", false),
        (CUSTOM, "collider_box", false),
        (CUSTOM, "heat", false),
    ] {
        let (mut m, _) = cena(4, canal, nome);
        assert_eq!(
            peca_declara_colisor(&mut m, 0.0),
            esperado,
            "canal {canal}, `{nome}`"
        );
    }
}

/// ⭐⭐ **A cena vai à placa e a placa separa onde a CPU separa** (doc 131 D.14).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_colisor_pelo_nome_separa_na_placa_como_na_cpu() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    for lado in [8, 32] {
        let (mut m, drive) = cena(lado, CUSTOM, "collider");
        let (mut cpu, _) = cena(lado, CUSTOM, "collider");
        let (mut sem, _) = cena(lado, CUSTOM, "heat");
        cpu.gpu_enabled = false;
        sem.gpu_enabled = false;
        cpu.pump.set_taps(&[drive]);
        crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
        assert!(
            matches!(saida, GpuOutcome::Handled),
            "{lado}²: a rota recusou — {:?}",
            m.route_said
        );
        assert_ne!(m.route_said, Some(RECUSA_COLISOR), "{lado}²");
        assert_ne!(m.route_said, Some(RECUSA_PASSE), "{lado}²");
        assert!(
            m.gpu_cook.passe_correu() >= 1,
            "{lado}²: a placa nao separou"
        );
        // O drive é um estágio da placa, e escreveu a coluna pelo nome do artista.
        let cols = m.gpu_cook.node_columns(drive).unwrap_or(&[]);
        assert!(
            cols.iter().any(|c| c == "collider") && !cols.iter().any(|c| c == "drive_named"),
            "{lado}²: o drive nao foi a' placa: {cols:?}"
        );
        let (fp, fc, fs) = (
            formas_da_placa(&gpu, &m),
            formas_da_cpu(&mut cpu),
            formas_da_cpu(&mut sem),
        );
        let alcance = alcance_max(&cpu, drive);
        let (d, controlo) = (pior_sem_ordem(&fp, &fc), pior_sem_ordem(&fc, &fs));
        eprintln!(
            "{lado}²: {} formas · alcance {alcance:.3e} · |placa − cpu| = {d:.2e} ({:.2e}·alcance) \
             · |com − sem| = {controlo:.2e} · {:?}",
            fp.len(),
            d / alcance,
            m.route_said
        );
        assert_eq!(fp.len(), lado * lado, "{lado}²: o numero de formas");
        assert_eq!(fp.len(), fc.len(), "{lado}²: as formas das duas rotas");
        assert!(
            controlo > MEXE,
            "{lado}²: o colisor nao mexeu na cena — gate vazio"
        );
        assert!(
            d <= EPS_RELATIVO * alcance,
            "{lado}²: a placa desenha a {d} da CPU"
        );
    }
}
