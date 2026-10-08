//! Gates do mundo da 1.ª cópia para `PH2D_PAN_DIAG` (doc 123 §8).

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};

fn linhas(p: &[[f32; 2]]) -> Stream {
    Stream::new(p.len())
        .with("P", Column::Vec2(p.to_vec()))
        .with("size", Column::Vec2(vec![[0.7, 0.3]; p.len()]))
        .with("rot", Column::Scalar(vec![31.0; p.len()]))
}

/// ⭐ O mundo lido é o `world_pos` que o lowering da CPU dá à linha `0` — o oráculo é a porta da
/// CPU (`lower_to_instances`), nunca uma segunda conta aqui.
#[test]
fn pan_diag_o_mundo_da_primeira_copia_e_o_da_cpu() {
    let (a, b) = (NodeId(7), NodeId(9));
    let s = linhas(&[[3.5, -1.25], [8.0, 2.0]]);
    let cpu = ph2d_eval_motion::lower_to_instances(&s)[0].world_pos;
    let lida = BTreeMap::from([(a, s.clone())]);
    assert_eq!(mundo_da_primeira_copia(&lida, &[a]), Some(cpu));
    assert_eq!(cpu, [3.5, -1.25], "CONTROLO: o oraculo leu a linha 0");

    // Sem `P`: a omissão da CPU, não `None` — a cópia existe.
    let sem_p = Stream::new(2).with("size", Column::Vec2(vec![[1.0, 1.0]; 2]));
    let cpu_sem_p = ph2d_eval_motion::lower_to_instances(&sem_p)[0].world_pos;
    let lida = BTreeMap::from([(a, sem_p)]);
    assert_eq!(mundo_da_primeira_copia(&lida, &[a]), Some(cpu_sem_p));

    // A 1.ª saída VAZIA não tem cópia: a 1.ª é a da seguinte, pela ordem de `sinks`.
    let lida = BTreeMap::from([(a, Stream::new(0)), (b, s)]);
    assert_eq!(mundo_da_primeira_copia(&lida, &[a, b]), Some(cpu));
    assert_eq!(mundo_da_primeira_copia(&lida, &[a]), None);
    assert_eq!(mundo_da_primeira_copia(&BTreeMap::new(), &[a, b]), None);
}

/// Sem leitura dos cartões (rota da CPU, 1.º quadro da placa) a porta do produto diz `None`.
#[test]
fn pan_diag_sem_leitura_dos_cartoes_nao_ha_mundo() {
    let mut m = MotionState::new();
    let out = m.doc.graph.add_node("motion.output");
    m.sinks = vec![out];
    assert_eq!(mundo_lido(&m), None);
    m.gpu_tap = Some(BTreeMap::from([(out, linhas(&[[1.0, 2.0]]))]));
    assert_eq!(
        mundo_lido(&m),
        Some([1.0, 2.0]),
        "CONTROLO: com leitura, ha"
    );
}

/// Linhas sem comentários `//`, aparadas.
fn sem_comentarios(src: &str) -> Vec<String> {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// O bloco `{ … }` que abre na linha `ini` (contagem de chavetas).
fn bloco(c: &[String], ini: usize) -> &[String] {
    let mut fundo = 0i32;
    for (k, l) in c.iter().enumerate().skip(ini) {
        fundo += l.matches('{').count() as i32 - l.matches('}').count() as i32;
        if fundo <= 0 {
            return &c[ini..=k];
        }
    }
    &c[ini..]
}

/// ⭐⭐ **Custo ZERO com a sonda desligada** (doc 123 §8, régua de texto como a do `tap_sem_espera`):
/// no ramo da placa do `coze_o_quadro` a leitura do mundo só corre dentro de `if
/// ph2d_pan_diag::on()`; o módulo não faz download nenhum (só lê `gpu_tap`); e a sonda nunca arma
/// uma tomada da bomba (recozeria na CPU, doc 120 §8.5).
#[test]
fn pan_diag_a_sonda_desligada_nao_le_nada_da_placa() {
    // (1) o ramo da placa.
    let q = sem_comentarios(include_str!("motion_bridge_quadro.rs"));
    let ini = q
        .iter()
        .position(|l| l.contains("gpu::GpuOutcome::Handled"))
        .expect("o ramo da placa");
    let fim = q[ini..]
        .iter()
        .position(|l| l == "return;")
        .map(|k| ini + k)
        .expect("o ramo da placa retorna");
    let ramo = &q[ini..=fim];
    // CONTROLO: a fatia achou as chamadas (uma fatia vazia passaria em tudo abaixo).
    assert!(ramo.iter().any(|l| l.contains("note_device(")), "{ramo:?}");
    assert!(
        ramo.iter().any(|l| l.contains("mundo_lido(")),
        "o ramo da placa le o mundo da 1.a copia: {ramo:?}"
    );
    let guarda = ramo
        .iter()
        .position(|l| l.contains("if ph2d_pan_diag::on() {"))
        .expect("a guarda da sonda no ramo da placa");
    let dentro = bloco(ramo, guarda);
    for chamada in ["mundo_lido(", "note_device_world("] {
        assert!(
            dentro.iter().any(|l| l.contains(chamada)),
            "`{chamada}` fora do `if ph2d_pan_diag::on()`: {ramo:?}"
        );
        let fora = ramo.len() - dentro.len();
        assert_eq!(
            ramo.iter().filter(|l| l.contains(chamada)).count(),
            dentro.iter().filter(|l| l.contains(chamada)).count(),
            "`{chamada}` tambem fora da guarda ({fora} linhas fora)"
        );
    }

    // (2) o módulo só lê a leitura que já existe.
    let m = include_str!("pan_diag_tap.rs");
    let m = sem_comentarios(m.split("#[cfg(test)]").next().unwrap_or(""));
    assert!(
        m.iter().any(|l| l.contains("gpu_tap")),
        "CONTROLO: le a leitura dos cartoes"
    );
    for proibido in [
        ".tap(",
        "tap_sem_espera",
        "read_instances",
        "debug_read",
        "set_taps",
    ] {
        assert!(
            !m.iter().any(|l| l.contains(proibido)),
            "`{proibido}` no modulo: download ou tomada nova (doc 123 §8)"
        );
    }

    // (3) a lista de tomadas da bomba não tem pedido da sonda.
    let b = sem_comentarios(include_str!("motion_bridge.rs"));
    let ini = b
        .iter()
        .position(|l| l.contains("let mut taps = motion.signal_taps.clone();"))
        .expect("a lista de tomadas");
    let fim = b[ini..]
        .iter()
        .position(|l| l.contains("motion.pump.set_taps(&taps);"))
        .map(|k| ini + k)
        .expect("as tomadas armam-se");
    let lista = &b[ini..=fim];
    assert!(
        lista.iter().any(|l| l.contains("pivot_gizmo::taps_for")),
        "CONTROLO: a fatia e' a lista de tomadas: {lista:?}"
    );
    assert!(
        !lista
            .iter()
            .any(|l| l.contains("pan_diag") || l.contains("ph2d_pan_diag::on()")),
        "a sonda arma uma tomada da bomba — na placa recoze na CPU (doc 120 §8.5)"
    );
}

/// ⭐⭐⭐ **O mundo da 1.ª cópia é IGUAL nas duas rotas** (doc 123 §8): a mesma cena cozida por
/// `coze_o_quadro` na placa (lida pela leitura dos cartões) e na CPU (a 1.ª cópia da bomba).
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib \
///     pan_diag_o_mundo_da_placa -- --ignored --nocapture`
#[test]
#[ignore = "placa: coze a cena na placa e compara com a CPU"]
fn pan_diag_o_mundo_da_placa_e_o_da_cpu() {
    use ph2d_nodegraph::graph::Edge;
    const DT: f64 = 1.0 / 60.0;
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        panic!("sem adapter — este gate mede a rota da placa");
    };
    // `grelha 4×3 → duplicador ← estrela → rotate → saída` (a cena do `formas_tests`, que a
    // placa toma); a grelha deslocada para o mundo da 1.ª cópia não ser a origem.
    let estado = |placa: bool| {
        let mut m = MotionState::new();
        m.gpu_enabled = placa;
        let g = &mut m.doc.graph;
        let grade = g.add_node("motion.grid");
        g.set_param(grade, "rows", 3.0);
        g.set_param(grade, "cols", 4.0);
        g.set_param(grade, "gap_x", 2.3);
        g.set_param(grade, "gap_y", 1.7);
        let forma = g.add_node("source.shape");
        let estrela = ph2d_node_motion_shape::ALL_KINDS
            .iter()
            .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
            .expect("a estrela");
        #[expect(clippy::cast_precision_loss, reason = "um indice pequeno")]
        g.set_param(forma, ph2d_node_motion_shape::param::KIND, estrela as f32);
        let dup = g.add_node("motion.duplicator");
        let roda = g.add_node("motion.rotate");
        g.set_param(roda, "angle", 23.0);
        let out = g.add_node("motion.output");
        for (de, para, porta) in [
            (forma, dup, 0u16),
            (grade, dup, 1),
            (dup, roda, 0),
            (roda, out, 0),
        ] {
            g.connect(Edge {
                from: (de, 0),
                to: (para, porta),
                delayed: false,
            })
            .expect("fio");
        }
        m.sinks = vec![out];
        m
    };
    let coze = |m: &mut MotionState| {
        let mut playhead = ph2d_core::Playhead::new(DT);
        for quadro in 1..=3u64 {
            playhead.seek(quadro as f64 * DT);
            crate::motion_bridge::quadro::coze_o_quadro(m, &gpu, &playhead, DT);
        }
    };

    let mut cpu = estado(false);
    coze(&mut cpu);
    assert!(!cpu.gpu_live, "a CPU: {:?}", cpu.route_said);
    let da_cpu = cpu
        .pump
        .instances
        .first()
        .map(|i| i.world_pos)
        .or_else(|| cpu.pump.vector_instances.first().map(|v| v.world_pos))
        .expect("a CPU baixou copias");

    let mut m = estado(true);
    coze(&mut m);
    assert!(
        m.gpu_live && m.route_said.unwrap_or("").starts_with("device"),
        "a cena tem de ir a placa: {:?}",
        m.route_said
    );
    // A leitura DESTE cozimento (o `tap` síncrono lê as MESMAS amostras que a sem espera).
    m.gpu_tap = m.gpu_cook.tap(&gpu, ph2d_gpu_cook::tap::TAP_SAMPLES);
    let da_placa = mundo_lido(&m).expect("a leitura dos cartoes traz a saida");
    eprintln!("[pan-diag] placa {da_placa:?} · cpu {da_cpu:?}");
    assert!(
        da_cpu != [0.0, 0.0],
        "CONTROLO: a 1.a copia nao e' a origem"
    );
    for k in 0..2 {
        assert!(
            (da_placa[k] - da_cpu[k]).abs() <= 1e-4,
            "eixo {k}: placa {da_placa:?} contra CPU {da_cpu:?}"
        );
    }
}
