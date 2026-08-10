//! Gates da cena do **PORTÃO ESPACIAL** (`PH2D_GPU_COOK_DEMO=23`).
//!
//! ⚠️ **Estes cozinham, não planejam.** As cenas vizinhas pinam o PLANO (fully-GPU, quantos
//! dispatches) porque a afirmação delas é *"isto roda no device"*; a afirmação desta é *"o
//! artista vê X"*, e a cadeia é CPU-only (os seis `pulse.*`/`value.*` não têm kernel). Um
//! gate de plano aqui ficaria verde sobre uma cena que não pisca.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Cozinha a cena de `t = 0` até o tique `until` e devolve a coluna `size` do último quadro,
/// junto com as posições (para classificar dentro/fora sem perguntar ao campo).
fn sizes_at(until: usize) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_pulse_gate_demo_document(&mut doc, &registry).expect("cena bem tipada");
    let out = *sinks.first().expect("um sink");

    let mut cook = Cook::new();
    let mut last = ph2d_nodegraph::attr::Stream::new(0);
    for k in 0..=until {
        let t = k as f64 / 60.0;
        last = cook.cook(&doc.graph, &registry, out, t).expect("cozinha")[0]
            .as_stream()
            .clone();
        cook.advance_tick(&doc.graph, &registry, t).expect("avança");
    }
    let size = match last.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        other => panic!("a cena tem de emitir `size` como Vec2, veio {other:?}"),
    };
    let p = match last.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a cena tem de emitir `P`"),
    };
    (size, p)
}

/// O índice da linha mais próxima de `(x, y)`.
fn nearest(p: &[[f32; 2]], x: f32, y: f32) -> usize {
    let mut best = (f32::INFINITY, 0);
    for (i, q) in p.iter().enumerate() {
        let d = (q[0] - x).powi(2) + (q[1] - y).powi(2);
        if d < best.0 {
            best = (d, i);
        }
    }
    best.1
}

/// **A CENA MOSTRA O QUE PROMETE: só quem está dentro do losango pisca.**
///
/// O metrônomo bate a cada 0,5 s e o toggle inverte a cada batida recebida, então em
/// `t = 0,25` (depois da batida do início) os pontos de dentro estão GRANDES e em `t = 0,75`
/// (depois da segunda) voltaram ao repouso. Os de fora não recebem batida nenhuma e ficam no
/// tamanho de repouso nos dois instantes — é essa metade que torna o gate um portão e não um
/// pisca-pisca global.
#[test]
fn the_scene_blinks_only_inside_the_box() {
    let (on, p) = sizes_at(15); // t = 0.25 — depois da batida de t = 0
    let (off, _) = sizes_at(45); // t = 0.75 — depois da batida de t = 0.5
    let inside = nearest(&p, 0.0, 0.0);
    let outside = nearest(&p, 5.5, 5.5); // uma quina, fora do losango por larga margem

    assert!(
        on[inside][0] > super::gpu_pulse_demo::DOT * 1.5,
        "dentro da caixa o ponto CRESCE no compasso: {:?}",
        on[inside]
    );
    assert!(
        (off[inside][0] - super::gpu_pulse_demo::DOT).abs() < 1e-5,
        "e volta ao repouso na batida seguinte (o toggle): {:?}",
        off[inside]
    );
    assert!(
        (on[outside][0] - super::gpu_pulse_demo::DOT).abs() < 1e-5
            && (off[outside][0] - super::gpu_pulse_demo::DOT).abs() < 1e-5,
        "fora dela NADA acontece, nos dois instantes: {:?} / {:?}",
        on[outside],
        off[outside]
    );
}

/// **E quem confina o pisca-pisca é o PORTÃO, não a máscara do próprio `motion.drive`.**
///
/// ⚠️ Este gate existe por causa de um confound que a cena quase teve: o `motion.drive` LÊ a
/// coluna `falloff` como máscara de força (o fallback é 1.0 quando ela está ausente), então
/// pôr o `field.box` no caminho de INSTÂNCIAS faria a cena mostrar o quadro certo **pelo
/// motivo errado** — o crescimento ficaria confinado à caixa mesmo com o `value.math` do
/// portão removido, e o gate acima ficaria VERDE sobre a feature deletada.
///
/// ⚠️ **E isso está MEDIDO, não temido:** com as duas mutações juntas — o `value.math` do
/// portão fora da cadeia **e** o campo no caminho de instâncias — o gate acima fica
/// **VERDE** e só este falha. A cena continuaria bonita com a feature da wave deletada.
///
/// A prova é sobre o DADO: o stream que chega ao drive não carrega `falloff`.
#[test]
fn the_gate_is_the_pulse_not_the_drives_own_mask() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    build_gpu_pulse_gate_demo_document(&mut doc, &registry).expect("cena bem tipada");
    let drive = doc
        .graph
        .nodes()
        .iter()
        .position(|n| n.type_name == "motion.drive")
        .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
        .expect("a cena tem um motion.drive");

    let mut cook = Cook::new();
    let art = cook
        .cook(&doc.graph, &registry, drive, 0.0)
        .expect("cozinha")[0]
        .as_stream()
        .clone();
    assert!(
        art.get("falloff").is_none(),
        "o drive recebe a arte SEM máscara — senão a caixa confinaria o crescimento sozinha, \
         e o gate do pisca-pisca passaria com o portão de pulso deletado"
    );
}

/// Quanto custa um tique desta cena, e é dele que o `SIDE` sai (§0: meça antes de limitar).
#[test]
#[ignore = "sonda: cargo test -p ph2d-host-desktop --bins measure_the_gate_scene_tick -- --ignored --nocapture"]
fn measure_the_gate_scene_tick() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_pulse_gate_demo_document(&mut doc, &registry).expect("cena bem tipada");
    let out = *sinks.first().expect("um sink");
    let mut cook = Cook::new();
    // Aquecimento: o 1º cozimento de qualquer nó é um miss do memo.
    for k in 0..10 {
        let t = k as f64 / 60.0;
        cook.cook(&doc.graph, &registry, out, t).expect("cozinha");
        cook.advance_tick(&doc.graph, &registry, t).expect("avança");
    }
    let t0 = std::time::Instant::now();
    const N: usize = 60;
    for k in 10..10 + N {
        let t = k as f64 / 60.0;
        cook.cook(&doc.graph, &registry, out, t).expect("cozinha");
        cook.advance_tick(&doc.graph, &registry, t).expect("avança");
    }
    let ms = t0.elapsed().as_secs_f64() * 1000.0 / N as f64;
    let n = super::gpu_pulse_demo::SIDE * super::gpu_pulse_demo::SIDE;
    eprintln!(
        "  cena =23: {n:.0} pontos, {ms:.3} ms/tique ({:.1} ns/ponto)",
        ms * 1e6 / n as f64
    );
}
