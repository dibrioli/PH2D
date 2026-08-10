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
///
/// ⚠️ **Este gate cozinha de `t = 0` com os params FIXOS**, e é por isso que ele foi cego ao
/// report do Enio de 2026-08-10 (BUGS #1): EDITAR o campo no meio da corrida deixava o
/// retrato invertido, porque a máscara volta e a memória do `pulse.counter` não voltava.
/// Quem cobre o GESTO — e não o boot — é o
/// [`a_round_trip_of_the_field_leaves_the_scene_where_it_found_it`].
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

/// O id do nó de um tipo na cena.
fn node_of(doc: &MotionDoc, ty: &str) -> ph2d_nodegraph::graph::NodeId {
    doc.graph
        .nodes()
        .iter()
        .position(|n| n.type_name == ty)
        .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
        .unwrap_or_else(|| panic!("a cena tem um {ty}"))
}

/// **O CAMPO GATEIA O PULSO; ELE NÃO GATEIA A MEMÓRIA.**
///
/// O report do Enio (2026-08-10, cena `=23`): *"Nó Box inconsistente — ao checar Invert e
/// depois desmarcar, o resultado é diferente do inicial"*. Este gate separa as DUAS
/// perguntas que o olho não distingue na tela, e as duas metades são load-bearing:
///
/// - **O NÓ ESTÁ INOCENTE.** `field.box` é uma função PURA dos params: um ida-e-volta do
///   `invert` re-deriva a máscara **byte a byte** nas 262.144 linhas. A metade irmã
///   (`invert` MUDA todas elas) é o que impede o gate de ficar verde por vácuo — sem ela,
///   um memo que ignorasse o param passaria, porque *não mudar nada* também dá `diff == 0`.
/// - **A CENA NÃO VOLTA, e o retrato é o INVERSO EXATO.** O que muda de lugar não é a
///   máscara: é o `count_tick` do `pulse.counter`, que vive no `pre` self-loop. Enquanto o
///   campo está invertido, quem está FORA recebe as batidas e avança a paridade; quem está
///   dentro congela. Desmarcar devolve a máscara e **não** a memória.
///
/// ⚠️ **A informação que falta é destruída ANTES do contador:** o portão é um
/// `value.math(Multiply)`, e ele colapsa *"não há pulso agora"* e *"esta linha saiu do
/// campo"* no MESMO zero. O `pulse.counter` não tem como saber que uma linha saiu — e não
/// tem porta de RESET (`inputs` = `pulse` + `state`, o self-loop). É por isso que o
/// conserto NÃO é neste arquivo nem no `field.box`: as duas curas candidatas são os P1
/// abertos da folha 12 (a entrada de **reset** no contador · o **`pulse.adsr`**, um
/// envelope que volta ao repouso sozinho e tornaria a cena auto-curável).
///
/// ⚠️ **Este gate NASCEU pinando o defeito** (o report de 2026-08-10) e foi reescrito para a
/// lei nova quando a cura landou — que é o que o doc dele mandava fazer, em vez de afrouxá-lo.
/// O retrato invertido sobrevive aqui como **CONTROLE**: é o que acontece sem o fio do reset.
#[test]
fn a_round_trip_of_the_field_leaves_the_scene_where_it_found_it() {
    let (pure_moved, pure_diff, n) = invert_round_trip_on_the_node();
    assert_eq!(
        pure_diff, 0,
        "o `field.box` é PURO: o ida-e-volta do `invert` re-deriva a máscara byte a byte"
    );
    assert_eq!(
        pure_moved, n,
        "e o `invert` de fato MORDE todas as linhas — sem esta metade o gate ficaria verde \
         sobre um memo que ignorasse o param"
    );

    // ⚠️ DUAS janelas: uma com UMA batida dentro, outra com DUAS — e elas medem coisas
    // diferentes de propósito. Com UMA janela o retrato voltou IDÊNTICO (0 de 262.144
    // linhas diferentes) e eu quase escrevi isso como a lei; era **coincidência de
    // paridade** — perder 1 batida deixa o dentro em 3 contagens e o controle em 5, e
    // 3 e 5 são ambos ímpares. Com 2 batidas perdidas ele conta 2, e o quadro difere.
    for window in [TOGGLE_WINDOW, (20, 80)] {
        let (inside_big, inside_tot, outside_big, outside_tot) =
            classify(&scene_run(Some(window), Reset::Wired));
        // **A PROMESSA da cena, e é ela que o report cobrava:** fora do losango, repouso.
        assert_eq!(
            outside_big, 0,
            "com o `reset` ligado, quem SAI do campo é liberado (janela {window:?}): \
             {outside_big} de {outside_tot} ficaram acesos"
        );
        // **E o dentro fica COERENTE** — todas as linhas na mesma fase, nunca meio quadro
        // aceso. A fase em si pode diferir do controle (o dentro perdeu batidas enquanto
        // estava fora), e isso é honesto: o reset devolve o REPOUSO, não a história.
        assert!(
            inside_big == inside_tot || inside_big == 0,
            "o dentro pisca junto (janela {window:?}): {inside_big} de {inside_tot}"
        );
    }

    // O CONTROLE: sem o fio do reset, o mesmo gesto INVERTE o quadro. É o defeito que o
    // report descreveu, e é ele que prova que quem cura é a fiação e não o acaso.
    let (inside_big, _, outside_big, outside_tot) =
        classify(&scene_run(Some(TOGGLE_WINDOW), Reset::Unwired));
    assert_eq!(
        (inside_big, outside_big),
        (0, outside_tot),
        "sem o `reset` o retrato é o INVERSO EXATO — a máscara volta e a memória do \
         `pulse.counter` não"
    );
}

/// **SONDA do report do Enio** (*"marcar Invert e desmarcar dá resultado diferente"*):
/// separa DUAS perguntas que o olho não distingue na tela.
///
/// M1 — o `field.box` sozinho é PURO sob o ida-e-volta? (cozinha o nó, lê `falloff`.)
/// M2 — a CENA volta ao mesmo lugar? (cozinha os tiques, com o toggle no meio da corrida.)
#[test]
#[ignore = "sonda: cargo test -p ph2d-host-desktop --bins probe_invert_round_trip -- --ignored --nocapture"]
fn probe_invert_round_trip() {
    let (moved, diff, n) = invert_round_trip_on_the_node();
    eprintln!("M1  o NO: {n} linhas | invert MUDOU {moved} | ida-e-volta difere em {diff}");

    let plain = scene_run(None, Reset::Wired);
    let toggled = scene_run(Some(TOGGLE_WINDOW), Reset::Wired);
    let diff_m2 = plain
        .iter()
        .zip(&toggled)
        .filter(|(x, y)| (x[0] - y[0]).abs() > 1e-6)
        .count();
    eprintln!(
        "M2  a CENA no tique {RUN_TICKS}: {} linhas | difere em {diff_m2} ({:.1}%)",
        plain.len(),
        100.0 * diff_m2 as f32 / plain.len() as f32
    );
    for (nome, s) in [("sem toggle", &plain), ("apos ida-e-volta", &toggled)] {
        let (bi, ti, bo, to) = classify(s);
        let mut vals: Vec<String> = s
            .iter()
            .map(|v| format!("{:.4}", v[0]))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        vals.truncate(6);
        eprintln!(
            "M3  {nome:>18}: DENTRO {bi}/{ti} grandes | FORA {bo}/{to} grandes | tamanhos {vals:?}"
        );
    }
}

/// O fio do `reset` está ligado nesta corrida? A ablação que o gate usa como CONTROLE —
/// `Unwired` desconecta a aresta e reproduz o mundo do report.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Reset {
    Wired,
    Unwired,
}

/// Quantos tiques uma corrida da cena roda (2 s a 60 fps — cinco batidas de 0,5 s).
const RUN_TICKS: usize = 120;
/// O ida-e-volta do `invert`: liga no tique 20, desliga no 50. A janela contém **uma**
/// batida inteira (a de `t = 0,5`), que é o mínimo para a paridade se separar.
const TOGGLE_WINDOW: (usize, usize) = (20, 50);

/// **M1 — o NÓ.** Cozinha o `field.box` com `invert` 0 → 1 → 0 no MESMO `Cook` e devolve
/// `(linhas que o invert mudou, linhas que o ida-e-volta NÃO restaurou, total)`.
fn invert_round_trip_on_the_node() -> (usize, usize, usize) {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    build_gpu_pulse_gate_demo_document(&mut doc, &registry).expect("cena bem tipada");
    let bx = node_of(&doc, "field.box");
    let mut cook = Cook::new();
    let falloff = |doc: &MotionDoc, cook: &mut Cook| -> Vec<f32> {
        match cook.cook(&doc.graph, &registry, bx, 0.0).expect("cozinha")[0]
            .as_stream()
            .get("falloff")
        {
            Some(Column::Scalar(v)) => v.clone(),
            other => panic!("falloff Scalar, veio {other:?}"),
        }
    };
    let a = falloff(&doc, &mut cook);
    doc.graph.set_param(bx, "invert", 1.0);
    let inv = falloff(&doc, &mut cook);
    doc.graph.set_param(bx, "invert", 0.0);
    let b = falloff(&doc, &mut cook);
    let moved = a.iter().zip(&inv).filter(|(x, y)| x != y).count();
    let diff = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    (moved, diff, a.len())
}

/// **M2 — a CENA.** Cozinha [`RUN_TICKS`] tiques; com `flip`, liga e desliga o `invert` nos
/// tiques dados (o gesto do artista: um `set_param` no meio de uma corrida viva). Devolve a
/// coluna `size` do último quadro.
fn scene_run(flip: Option<(usize, usize)>, reset: Reset) -> Vec<[f32; 2]> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    let mut d = MotionDoc::new();
    let sinks = build_gpu_pulse_gate_demo_document(&mut d, &reg).expect("cena");
    let out = *sinks.first().expect("um sink");
    let bx = node_of(&d, "field.box");
    if reset == Reset::Unwired {
        let toggle = node_of(&d, "pulse.counter");
        assert!(
            d.graph.disconnect(toggle, 2).is_some(),
            "a cena TEM de trazer o fio do reset — sem ele o controle não é ablação de nada"
        );
    }
    let mut cook = Cook::new();
    let mut last = ph2d_nodegraph::attr::Stream::new(0);
    for k in 0..=RUN_TICKS {
        if let Some((on, off)) = flip {
            if k == on {
                d.graph.set_param(bx, "invert", 1.0);
            }
            if k == off {
                d.graph.set_param(bx, "invert", 0.0);
            }
        }
        let t = k as f64 / 60.0;
        last = cook.cook(&d.graph, &reg, out, t).expect("cozinha")[0]
            .as_stream()
            .clone();
        cook.advance_tick(&d.graph, &reg, t).expect("avanca");
    }
    match last.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        other => panic!("size Vec2, veio {other:?}"),
    }
}

/// **M3 — o QUE difere.** Classifica cada linha em dentro/fora pelo PRÓPRIO campo (cozido em
/// `invert = 0`, nunca por uma segunda conta de losango aqui) e conta as GRANDES de cada
/// lado: `(grandes dentro, total dentro, grandes fora, total fora)`.
fn classify(sizes: &[[f32; 2]]) -> (usize, usize, usize, usize) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    let mut d = MotionDoc::new();
    build_gpu_pulse_gate_demo_document(&mut d, &reg).expect("cena");
    let bx = node_of(&d, "field.box");
    let mut cook = Cook::new();
    let mask = match cook.cook(&d.graph, &reg, bx, 0.0).expect("cozinha")[0]
        .as_stream()
        .get("falloff")
    {
        Some(Column::Scalar(v)) => v.clone(),
        other => panic!("falloff, veio {other:?}"),
    };
    let (mut bi, mut ti, mut bo, mut to) = (0, 0, 0, 0);
    for (i, v) in sizes.iter().enumerate() {
        let big = v[0] > super::gpu_pulse_demo::DOT * 1.5;
        if mask[i] > 0.5 {
            ti += 1;
            bi += usize::from(big);
        } else {
            to += 1;
            bo += usize::from(big);
        }
    }
    (bi, ti, bo, to)
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
