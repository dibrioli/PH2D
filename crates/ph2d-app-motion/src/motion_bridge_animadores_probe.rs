//! **A MEDIÇÃO DO GRUPO «ANIMADORES»** — o passo 5 do ciclo 2
//! ([doc 105](../../../docs/Motion%20Nodes/105_ciclo_2_animadores.md)), e a lei §2.1 da
//! dinâmica: *todo nó de um ciclo diz onde corre e porquê*.
//!
//! ⚠️ **Um animador não POSITA objectos: ele mexe nos que chegam.** Por isso a régua deste
//! grupo é outra que a do ciclo 1 — ali media-se o nó sozinho, aqui mede-se
//! `grade → animador → output` **menos** `grade → output`. *Cronometrar a cadeia inteira
//! mediria a grade oito vezes.*
//!
//! `cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_animator_group`

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// A fila que todos recebem: `400 × 250` = **100 000** — a mesma contagem da medição da W3, para
/// as duas tabelas do ciclo se poderem ler juntas.
const LINHAS: f32 = 400.0;
const COLUNAS: f32 = 250.0;
/// Melhor de N cozimentos FRIOS. ⛔ **Um `Cook` novo por corrida** — o `motion.stagger` e o
/// `motion.delay` são `Effect::Pure`, e no mesmo `Cook` o memo responde: era assim que a
/// primeira tabela do ciclo 1 leu `0,00 ms` em dez linhas.
const CORRIDAS: usize = 5;
/// Quantos tiques se dão antes de cronometrar o último — o suficiente para o anel do
/// `motion.delay` estar cheio (ele guarda 32 fatias) e a mola ter estado.
const AQUECE: usize = 40;
const DT: f64 = 1.0 / 60.0;

struct Caso {
    node: &'static str,
    params: &'static [(&'static str, f32)],
    /// O `value.lfo` emite um **número**, não uma fila: para ele chegar à tela é preciso um
    /// `motion.drive`. ⚠️ Então a linha dele mede **as duas caixas** — que é exactamente a
    /// comparação da W5, e é por isso que ela existe.
    via_drive: bool,
    /// `out --pre--> state`: a realimentação que a shell arma sozinha ao largar o nó.
    realimenta: bool,
}

const GRUPO: [Caso; 8] = [
    Caso {
        node: "motion.oscillator",
        params: &[("channel", 1.0), ("frequency", 2.0), ("amplitude", 37.0)],
        via_drive: false,
        realimenta: false,
    },
    Caso {
        node: "value.lfo",
        params: &[("period", 0.5), ("amplitude", 37.0)],
        via_drive: true,
        realimenta: false,
    },
    Caso {
        node: "motion.wiggle",
        params: &[("channel", 1.0), ("amplitude", 37.0)],
        via_drive: false,
        realimenta: false,
    },
    Caso {
        node: "motion.noise",
        params: &[("channel", 1.0), ("amplitude", 37.0)],
        via_drive: false,
        realimenta: false,
    },
    Caso {
        node: "motion.stagger",
        params: &[("channel", 1.0), ("max", 37.0)],
        via_drive: false,
        realimenta: false,
    },
    Caso {
        node: "motion.orbit",
        params: &[("speed", 90.0)],
        via_drive: false,
        realimenta: false,
    },
    Caso {
        node: "motion.spring",
        params: &[("channel", 1.0)],
        via_drive: false,
        realimenta: true,
    },
    Caso {
        node: "motion.delay",
        params: &[("channel", 1.0), ("ticks", 8.0)],
        via_drive: false,
        realimenta: true,
    },
];

fn liga(m: &mut MotionState, de: NodeId, dp: u16, para: NodeId, pp: u16, atrasado: bool) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, dp),
            to: (para, pp),
            delayed: atrasado,
        })
        .expect("as portas encaixam");
}

/// Monta a cadeia e devolve o sink. `caso = None` ⇒ a **linha de base**: só a grade.
fn monta(m: &mut MotionState, caso: Option<&Caso>) -> NodeId {
    let feed = m.doc.graph.add_node("motion.grid".to_string());
    m.doc.graph.set_param(feed, "rows", LINHAS);
    m.doc.graph.set_param(feed, "cols", COLUNAS);
    let out = m.doc.graph.add_node("motion.output".to_string());
    let Some(c) = caso else {
        liga(m, feed, 0, out, 0, false);
        return out;
    };
    let src = m.doc.graph.add_node(c.node.to_string());
    for (p, v) in c.params {
        m.doc.graph.set_param(src, *p, *v);
    }
    liga(m, feed, 0, src, 0, false);
    if c.realimenta {
        liga(m, src, 0, src, 1, true);
    }
    if c.via_drive {
        let drive = m.doc.graph.add_node("motion.drive".to_string());
        m.doc.graph.set_param(drive, "channel", 1.0);
        m.doc.graph.set_param(drive, "mode", 0.0);
        liga(m, feed, 0, drive, 0, false);
        liga(m, src, 0, drive, 1, false);
        liga(m, drive, 0, out, 0, false);
    } else {
        liga(m, src, 0, out, 0, false);
    }
    out
}

/// `(ms a FRIO, ms em REGIME, elementos, no device?, passes)`.
///
/// ⛔⛔ **UM NÚMERO SÓ MENTE AQUI, e a 1.ª versão desta sonda leu `0,00 ms` na grade e no
/// `motion.stagger`.** Não era ruído de relógio: os dois são `Effect::Pure`, e num tique seguinte
/// **o memo responde** — a régua estava a medir a tabela de hash, que é exactamente o que o
/// ciclo 1 já tinha pago uma vez.
///
/// ⚠️ **E a cura não é «cozer sempre a frio», porque a `motion.spring` e o `motion.delay` pedem o
/// contrário:** eles carregam estado por uma aresta `delayed`, e o **primeiro** tique deles é a
/// identidade por desenho — a frio mede-se o nó desligado. *As duas metades do grupo precisam de
/// medições opostas*, então a tabela traz as duas:
///
/// - **frio** — `Cook` novo, primeiro cozimento: o que o nó custa **quando tem de correr** (um
///   arrasto de param, uma cena a abrir).
/// - **regime** — o 40.º tique do mesmo `Cook`: o que ele custa **por quadro** numa cena a andar.
///   Aqui um nó `Pure` cujo resultado não muda com o tempo custa ~`0`, e isso não é um erro de
///   medição: é a resposta.
fn mede(caso: Option<&Caso>) -> (f64, f64, usize, bool, usize) {
    let (mut frio, mut regime) = (f64::MAX, f64::MAX);
    let (mut n, mut gpu, mut passes) = (0usize, false, 0usize);
    for _ in 0..CORRIDAS {
        let mut m = MotionState::new();
        let sink = monta(&mut m, caso);
        let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink);
        gpu = plano.is_fully_gpu();
        passes = plano.dispatching_stages(&m.registry);
        let mut cook = Cook::new();
        for k in 0..AQUECE {
            let t = 0.5 + k as f64 * DT;
            let relogio = std::time::Instant::now();
            let conta = cook
                .cook(&m.doc.graph, &m.registry, sink, t)
                .map_or(0, |v| v.first().map_or(0, |c| c.as_stream().count()));
            let ms = relogio.elapsed().as_secs_f64() * 1000.0;
            n = conta;
            if k == 0 {
                frio = frio.min(ms);
            } else if k + 1 == AQUECE {
                regime = regime.min(ms);
            }
            let _ = cook.advance_tick(&m.doc.graph, &m.registry, t);
        }
    }
    (frio, regime, n, gpu, passes)
}

#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_animator_group() {
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let (base_frio, base_reg, n_base, gpu_base, _) = mede(None);
    eprintln!(
        "  linha de base: a grade sozinha, {n_base} elementos, {base_frio:.2} ms a frio e \
         {base_reg:.2} em regime, device = {}",
        if gpu_base { "SIM" } else { "nao" }
    );
    eprintln!(
        "\n  {:<20} │ {:>9} │ {:>6} │ {:>6} │ {:>9} │ {:>9} │ {:>11}",
        "nó", "elementos", "device", "passes", "frio ms", "regime ms", "objectos/ms"
    );
    eprintln!(
        "  ---------------------|-----------|--------|--------|-----------|-----------|------------"
    );
    for c in &GRUPO {
        let (frio, regime, n, gpu, passes) = mede(Some(c));
        assert!(n > 0, "`{}` nao emitiu elemento nenhum", c.node);
        let proprio = (frio - base_frio).max(0.0);
        eprintln!(
            "  {:<20} │ {n:>9} │ {:>6} │ {passes:>6} │ {:>9.2} │ {:>9.2} │ {:>11.0}",
            c.node,
            if gpu { "SIM" } else { "NAO" },
            proprio,
            (regime - base_reg).max(0.0),
            n as f64 / proprio.max(1e-9),
        );
    }
    eprintln!(
        "\n  (device = o planeador reivindica `grade -> no' -> output` INTEIRA para a placa.
   frio = `Cook` novo, o custo de CORRER; regime = o 40.o tique, o custo POR QUADRO — um no'
   `Pure` cujo resultado nao muda no tempo custa ~0 ali porque o memo responde, e isso e' a
   resposta, nao um erro de medicao.
   O `value.lfo` mede-se com o `motion.drive` a seguir: sozinho ele emite um numero, nao uma fila.
   Um quadro de 60 fps tem 16,67 ms; o tecto medido da placa e' 4,19 M objectos em 3,85 ms.)\n"
    );
}
