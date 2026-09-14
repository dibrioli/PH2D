//! Os gates da cena `=116` — a que mostra a W1a do ciclo 6 (doc 110 §6).
//!
//! ⚠️⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente**
//! (`CLAUDE.md` §5.0), e esta promete uma coisa que não se vê na imagem: **onde ela corre**. Se o
//! anúncio manda o dono ler `device: o plano inteiro` no terminal, é este ficheiro que tem de
//! garantir que é isso que lá vai aparecer — e que com o interruptor desligado aparece o contrário.

use super::*;
use crate::motion_state::MotionState;

/// Monta a cena e devolve `(estado, sink)`.
fn cena() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    let sink = *sinks.first().expect("um sink");
    (m, sink)
}

/// ⭐⭐⭐ **A CENA CORRE NO DISPOSITIVO — que é a coisa inteira que ela existe para mostrar.**
#[test]
fn the_wire_scene_is_claimed_by_the_device() {
    let (mut m, sink) = cena();
    let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
    let plano =
        ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, sink, &dirigidos);
    assert!(
        plano.is_fully_gpu(),
        "a `=116` tem de ser reivindicada de ponta a ponta -- e' o assunto dela: {:?}",
        plano.boundaries
    );
}

/// ⛔⛔ **E COM O INTERRUPTOR DESLIGADO ELA CAI** — sem esta metade, o passo 3 do anúncio manda o
/// dono comparar duas corridas que dizem a mesma coisa, e a cena ensina que a mudança não fez nada.
///
/// ⚠️ A régua não lê a variável de ambiente (um teste que a pusesse mexeria no processo inteiro, e
/// a suíte corre em paralelo): ela usa o **mapa vazio**, que é literalmente o que o interruptor
/// devolve.
#[test]
fn with_the_switch_off_the_same_scene_falls_to_the_cpu() {
    let (m, sink) = cena();
    let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink);
    assert!(
        !plano.is_fully_gpu(),
        "sem os valores a cena TEM de cair -- senao as duas corridas do anuncio sao iguais"
    );
}

/// ⚠️ **O FIO EXISTE, e é UM.** Uma cena que perdesse o `drive_param` continuaria a desenhar um
/// pano a respirar (o `amount` cairia no default) e os dois gates acima continuariam a passar —
/// *o sujeito da cena tem de ser afirmado, não suposto*.
#[test]
fn the_scene_has_exactly_one_wire_and_it_drives_the_scale() {
    let (m, _) = cena();
    let fios = m.doc.graph.all_param_sources();
    assert_eq!(fios.len(), 1, "um no' dirigido: {fios:?}");
    let (no, params) = fios.iter().next().expect("o no'");
    assert_eq!(
        m.doc.graph.node(*no).map(|i| i.type_name.as_str()),
        Some("motion.scale"),
        "o fio chega ao `motion.scale`"
    );
    assert_eq!(
        params.keys().collect::<Vec<_>>(),
        vec!["amount"],
        "e comanda o `amount`"
    );
}

/// ⚠️ **AS 102 400 PEÇAS SÃO O SUJEITO, não o cenário** — um custo de `50×` sobre dez peças não se
/// vê, e uma cena que encolhesse deixaria de mostrar o que anuncia.
#[test]
fn the_cloth_is_as_big_as_the_announcement_says() {
    let (mut m, sink) = cena();
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("coze");
    assert_eq!(
        saida[0].as_stream().count(),
        102_400,
        "320 x 320 -- a mesma contagem das tabelas do doc 98"
    );
}

/// ⭐ **O PANO RESPIRA** — o fio tem de MOVER alguma coisa, senão a cena é um pano parado com uma
/// linha de terminal ao lado.
#[test]
fn the_cloth_actually_breathes() {
    let (mut m, sink) = cena();
    let tamanho = |m: &mut MotionState, t: f64| -> f32 {
        let saida = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, t)
            .expect("coze");
        match saida[0].as_stream().get("size") {
            Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.first().map_or(0.0, |s| s[0]),
            _ => panic!("a cena escreve `size` -- e' o que o fio comanda"),
        }
    };
    // Um quarto de período e meio período: a onda tem de ter ido a sítios diferentes.
    let (a, b) = (
        tamanho(&mut m, 0.0),
        tamanho(&mut m, f64::from(PERIODO) / 4.0),
    );
    assert!(
        (a - b).abs() > 1e-3,
        "o tamanho nao mexeu entre dois instantes ({a} e {b}) -- o fio nao esta' a comandar nada"
    );
}

/// **SONDA — o que a FORMA custa na cena do dono** (report de 2026-09-14: *«usando shape (exemplo:
/// star) fps cai para 27»*).
///
/// ⚠️ A pergunta que ela responde e que o relógio do app não separa: **quantos elementos a cena
/// passa a ter**. O carimbo emite `formas × pontos`, e uma contagem que decuplica explica um custo
/// que *«estrelas são caras»* não explica.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib --release probe_what_the_shape_costs -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao — corra em RELEASE"]
fn probe_what_the_shape_costs() {
    eprintln!("\n  caso                   | elementos | cook      | rota");
    eprintln!("  -----------------------|-----------|-----------|------------");
    for (rotulo, forma) in [("quads (o de omissão)", None), ("estrelas", Some(5.0))] {
        let mut m = MotionState::new();
        let sink = *build_com(&mut m.doc, &m.registry, forma)
            .expect("a cena monta")
            .first()
            .expect("um sink");
        crate::motion_shape_gen::publish(&mut m, 0.0);
        let t = std::time::Instant::now();
        let saida = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, 0.0)
            .expect("coze");
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let n = saida[0].as_stream().count();
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let no_device =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, sink, &dirigidos)
                .is_fully_gpu();
        eprintln!(
            "  {rotulo:<22} | {n:>9} | {ms:>6.2} ms | {}",
            if no_device { "dispositivo" } else { "⛔ CPU" }
        );
    }
}
