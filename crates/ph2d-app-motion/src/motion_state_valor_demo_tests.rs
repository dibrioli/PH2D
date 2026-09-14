//! Os gates da cena `=117` — a do ciclo 6 (doc 110 §12).
//!
//! ⚠️⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente**
//! (`CLAUDE.md` §5.0). O anúncio manda o dono ver **quatro coisas distintas**, e cada uma delas é
//! um gate aqui: os dois panos de cima respiram DIFERENTE, os dois de baixo piscam com ritmos
//! DIFERENTES, e a diferença de cada par é **um nó**, não um número.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// Monta a cena e devolve `(estado, os quatro sinks)`.
fn cena() -> (MotionState, Vec<NodeId>) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    (m, sinks)
}

/// O `size.x` de cada peça do sink, no instante `t`.
fn tamanhos(m: &mut MotionState, sink: NodeId, t: f64) -> Vec<f32> {
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, t)
        .expect("coze");
    match saida[0].as_stream().get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|s| s[0]).collect(),
        outra => panic!("a cena escreve `size`: {outra:?}"),
    }
}

/// ⭐ **SÃO QUATRO PANOS, e cada um tem as 36 peças que o anúncio nomeia.**
#[test]
fn the_scene_is_four_cloths_of_thirty_six() {
    let (mut m, sinks) = cena();
    assert_eq!(sinks.len(), 4, "quatro quadrantes");
    for (k, s) in sinks.iter().enumerate() {
        let saida = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, *s, 0.0)
            .expect("coze");
        assert_eq!(
            saida[0].as_stream().count(),
            (LADO * LADO) as usize,
            "quadrante {k}"
        );
    }
}

/// ⭐⭐⭐ **EM CIMA: os dois panos respiram DIFERENTE, e a diferença é o DEGRAU.**
///
/// ⚠️ **A régua não é «os dois mexem»** — é o pano da direita tomar **menos valores distintos** ao
/// longo do ciclo. Uma grelha é exactamente isso: um contradomínio mais pequeno. *Comparar dois
/// instantes diria que os dois respiram e não diria qual é que salta.*
#[test]
fn the_top_pair_breathes_smooth_and_stepped() {
    let (mut m, sinks) = cena();
    let distintos = |m: &mut MotionState, sink: NodeId| {
        let mut vistos: Vec<i64> = Vec::new();
        for k in 0..60 {
            let t = f64::from(k) * f64::from(PERIODO) / 60.0;
            // A 1e-4 de resolução: abaixo disso contaríamos ruído de `f32` como degraus.
            let v = (f64::from(tamanhos(m, sink, t)[0]) * 10_000.0).round() as i64;
            if !vistos.contains(&v) {
                vistos.push(v);
            }
        }
        vistos.len()
    };
    let liso = distintos(&mut m, sinks[0]);
    let degraus = distintos(&mut m, sinks[1]);
    assert!(
        liso > degraus * 3,
        "o pano liso toma {liso} tamanhos distintos e o dos degraus {degraus} -- se estes \
         numeros forem parecidos o `Quantize` nao esta' a chegar, e a cena ensina o contrario \
         do que o anuncio diz"
    );
    assert!(
        degraus >= 2,
        "o pano dos degraus toma {degraus} tamanho(s) -- ele TEM de respirar, so' que aos saltos"
    );
}

/// ⭐⭐⭐ **EM BAIXO: os dois panos piscam com ritmos DIFERENTES, e a razão é o CONTADOR.**
///
/// ⚠️ A régua conta **quantas vezes o pano acende** numa janela de tempo igual para os dois.
#[test]
fn the_bottom_pair_flashes_at_different_rates() {
    let (mut m, sinks) = cena();
    let acendeu = |m: &mut MotionState, sink: NodeId| {
        // Uma janela de `A_CADA` batidas mais um pouco: a da esquerda tem de acender várias
        // vezes e a da direita uma só.
        let passos = 120;
        let fim = f64::from(BATIDA) * f64::from(A_CADA) * 1.1;
        let mut n = 0usize;
        let mut antes = 0.0f32;
        for k in 0..=passos {
            let t = f64::from(k) * fim / f64::from(passos);
            let agora = tamanhos(m, sink, t)[0];
            // ⚠️ **Conta a SUBIDA, não o valor**: o clarão desvanece, então um limiar sobre o
            // tamanho contaria o mesmo flash muitas vezes.
            if agora > antes + 0.05 {
                n += 1;
            }
            antes = agora;
            m.pump
                .cook
                .advance_tick(&m.doc.graph, &m.registry, t)
                .expect("tique");
        }
        n
    };
    let cada = acendeu(&mut m, sinks[2]);
    let quatro = acendeu(&mut m, sinks[3]);
    assert!(
        cada > quatro,
        "a esquerda acendeu {cada} vez(es) e a direita {quatro} -- o `Counter` nao esta' a \
         desbastar nada, e a cena ensina o contrario do que o anuncio diz"
    );
    assert!(
        quatro >= 1,
        "a direita nunca acendeu em {:.2} s -- ela TEM de piscar, so' que a cada {A_CADA}",
        f64::from(BATIDA) * f64::from(A_CADA) * 1.1
    );
}

/// ⚠️⚠️ **A DIFERENÇA DE CADA PAR É UM NÓ, e não um número.**
///
/// ⛔ Sem isto alguém «afina» a cena mudando um param de um lado, os gates acima continuam verdes,
/// e o anúncio — que diz *«muda UM CARTÃO»* — passa a mentir.
#[test]
fn each_pair_differs_by_exactly_one_node() {
    let (m, _) = cena();
    let tipos: Vec<&str> = m
        .doc
        .graph
        .nodes()
        .iter()
        .map(|i| i.type_name.as_str())
        .collect();
    let conta = |t: &str| tipos.iter().filter(|x| **x == t).count();
    // Os quatro panos: quatro grelhas e quatro deslocamentos.
    assert_eq!(conta("motion.grid"), 4, "{tipos:?}");
    assert_eq!(conta("motion.move"), 4, "{tipos:?}");
    // Em cima: DOIS osciladores (um por metade) e UM quantizador — o nó que faz a diferença.
    assert_eq!(conta("value.lfo"), 2, "{tipos:?}");
    assert_eq!(conta("value.quantize"), 1, "{tipos:?}");
    // Em baixo: DOIS metrónomos, DOIS flashes e UM contador.
    assert_eq!(conta("pulse.beat"), 2, "{tipos:?}");
    assert_eq!(conta("motion.strobe"), 2, "{tipos:?}");
    assert_eq!(conta("pulse.counter"), 1, "{tipos:?}");
}

/// ⚠️ **O CONTADOR é ligado pela porta do CARRY**, e não pela saída de valor.
///
/// ⛔ A porta `0` dele é a CONTAGEM (um número que sobe), e ligá-la a um consumidor de pulso seria
/// um erro de tipo — mas a porta `1` é um pulso, e as duas são alcançáveis do mesmo cartão. *Um
/// gate que só contasse os nós não veria a diferença.*
#[test]
fn the_counter_feeds_the_flash_through_its_carry_port() {
    let (m, _) = cena();
    let contador = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|i| i.type_name == "pulse.counter")
        .map(|i| i.id)
        .expect("o contador");
    let saidas: Vec<u16> = m
        .doc
        .graph
        .edges()
        .iter()
        .filter(|e| e.from.0 == contador && !e.delayed)
        .map(|e| e.from.1)
        .collect();
    assert_eq!(
        saidas,
        vec![1],
        "o contador tem de alimentar o flash pela porta 1 (o carry): {saidas:?}"
    );
}

/// **SONDA — os cartões que a cena `=117` pinta, com as linhas de cada um.**
///
/// ⚠️ É o que o tutorial e o anúncio podem nomear: um passo que manda clicar numa linha AFIRMA
/// que ela está na lista.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn probe_the_cards_of_this_scene() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    eprintln!("\n  cartão             | linhas");
    eprintln!("  -------------------|-------");
    for v in &snap.nodes {
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        eprintln!("  {:<18} | {}", v.display_name, rows.join(" · "));
    }
    eprintln!();
}

/// A FONTE do tutorial deste ciclo — lida para que os nomes do gate e os do texto não possam
/// divergir em silêncio.
const TUTORIAL: &str =
    include_str!("../../../docs/Motion Nodes/tutoriais/src/06_valor_e_pulso.html");

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (ciclo 6, passo 7 — doc 103 §1).
///
/// ⛔⛔ **Um passo que manda arrastar uma linha AFIRMA que ela está no cartão**, e a casa já pagou
/// por escrever um passo impossível. O irmão deste gate (`every_card_an_announcement_...`) defende
/// o ANÚNCIO do terminal; este defende o **PDF**, que é o que o dono lê com o app aberto ao lado.
#[test]
fn every_row_the_value_tutorial_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    // O que o tutorial manda procurar: (título do cartão, linhas dentro dele).
    let pedidos: &[(&str, &[&str])] = &[
        ("LFO", &["Amplitude", "Period"]),
        ("Quantize", &["Step"]),
        ("Beat", &["Period"]),
        ("Strobe", &["Decay"]),
        ("Counter", &["Count"]),
        // ⚠️ O fio do oscilador aterra NESTA linha, e o tutorial aponta-lhe o dedo.
        ("Scale", &["Scale"]),
    ];
    for (titulo, linhas) in pedidos {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == *titulo)
            .unwrap_or_else(|| {
                let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
                panic!("o tutorial nomeia o cartao `{titulo}` e a cena tem {havia:?}")
            });
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        for l in *linhas {
            assert!(
                rows.contains(l),
                "o tutorial manda arrastar `{l}` no cartao `{titulo}`, e o cartao mostra {rows:?}"
            );
        }
        // ⚠️ **E o título tem de estar de facto NO TUTORIAL** — senão esta lista é uma promessa
        // sobre um texto que não a faz, e ela envelhece calada.
        assert!(
            TUTORIAL.contains(titulo),
            "o gate defende o cartao `{titulo}` e o tutorial nunca o nomeia"
        );
    }

    // ⛔⛔ **O comando do tutorial é `--profile smoke`, NUNCA `--release`** — o `release` optimiza a
    // shell num só thread e cada correcção pós-smoke custava `161 s` contra `3 s` (auditoria de
    // 2026-09-10 §3.9). *Um tutorial que manda o dono esperar dois minutos por engano é um custo
    // que ninguém volta a medir.*
    assert!(
        !TUTORIAL.contains("--release"),
        "o tutorial manda correr com `--release`"
    );
    // ⚠️ E as DUAS cenas que ele abre têm de ser as que existem.
    for cena in ["PH2D_GPU_COOK_DEMO=117", "PH2D_GPU_COOK_DEMO=116"] {
        assert!(TUTORIAL.contains(cena), "o tutorial tem de abrir `{cena}`");
    }
}

/// ⚠️ **AS FIGURAS que o tutorial mostra existem, e são QUATRO DIFERENTES.**
///
/// ⛔ A lei que o ciclo 4 pagou: uma corrida vermelha do gerador deixa duas figuras idênticas no
/// disco, e o tutorial passa a mostrar a mesma imagem debaixo de duas legendas diferentes.
#[test]
fn the_four_figures_the_tutorial_shows_exist_and_differ() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    let nomes = [
        "valor_liso",
        "valor_degraus",
        "valor_cada_batida",
        "valor_a_cada_quatro",
    ];
    let mut corpos: Vec<String> = Vec::new();
    for n in nomes {
        assert!(
            TUTORIAL.contains(&format!("{n}.svg")),
            "o gate defende a figura `{n}` e o tutorial nao a mostra"
        );
        let p = dir.join(format!("{n}.svg"));
        let s = std::fs::read_to_string(&p).unwrap_or_else(|e| {
            panic!(
                "a figura `{n}` nao esta' no disco ({e}) -- corra o \
                 `write_the_value_figures`"
            )
        });
        assert!(s.len() > 500, "a figura `{n}` esta' vazia");
        corpos.push(s);
    }
    for i in 0..corpos.len() {
        for j in (i + 1)..corpos.len() {
            assert_ne!(
                corpos[i], corpos[j],
                "as figuras `{}` e `{}` sao IDENTICAS -- o tutorial mostraria a mesma imagem \
                 debaixo de duas legendas diferentes",
                nomes[i], nomes[j]
            );
        }
    }
}
